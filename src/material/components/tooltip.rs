//! M3 plain tooltip。
//!
//! 来源:
//! - [`DTooltipHost.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DTooltipHost.qml): hover 延迟、focus 立即显示。
//! - [`DTooltipV2.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DTooltipV2.qml): 定位、夹紧、配色、进出场透明度。
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): tooltip 的触发与保留规则。
//!
//! 这是 plain tooltip：只显示文本，不接受交互，不抢 hover。`DTooltipV2.qml` 里
//! 的自动选边只有在 `side` 为空时才会触发；上层 `DTooltipHost.qml` 默认传
//! `"bottom"`，所以实际行为是固定方位 + 窗口内夹紧，这里保持一致。
//!
//! 每帧都需要调用 [`Tooltip::show_for`]。状态按 `Response::id` 存在 egui 的
//! temp data 里，因此 hover 结束后仍能画完 100ms 的退场淡出。

use std::sync::Arc;

use egui::text::Galley;
use egui::{
    Align, Area, Color32, CornerRadius, Id, Order, Pos2, Rect, Response, Ui, Vec2, pos2, vec2,
};

use crate::material::effects::motion::{self, Motion, MotionPrefs};
use crate::material::m3_ui::M3Ui;
use crate::material::text::{self, TextLayout, TextStyle};
use crate::material::theme::Theme;
use crate::material::tokens;

/// tooltip 相对目标控件的方位。
///
/// 来源: `DTooltipHost.qml` 的 `side`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

/// tooltip builder。
///
/// `text` 为空时不显示。`show_for` 每个控件每帧调用一次：
///
/// ```ignore
/// let response = ui.button("保存");
/// m.tooltip("写入磁盘").side(TooltipSide::Top).show_for(&response, m);
/// ```
pub struct Tooltip<'a> {
    text: &'a str,
    side: TooltipSide,
    multiline: bool,
    motion: Option<MotionPrefs>,
}

impl<'a> Tooltip<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            side: TooltipSide::Bottom,
            multiline: false,
            motion: None,
        }
    }

    /// 显示方位。来源: `DTooltipHost.qml` 的 `side`。
    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    /// 多行模式，按任意字符换行。来源: `DTooltipV2.qml` 的 `multiline`。
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// 覆盖 [`M3Ui`] 提供的动效偏好。
    pub fn motion(mut self, motion: MotionPrefs) -> Self {
        self.motion = Some(motion);
        self
    }

    /// 为控件显示 tooltip。
    ///
    /// 必须在控件存在的每一帧调用，否则退场动画无法完成。控件消失时
    /// tooltip 也随之消失。
    pub fn show_for(self, response: &Response, m: &mut M3Ui<'_>) {
        let theme = m.theme();
        let prefs = self.motion.unwrap_or_else(|| m.motion());
        let text = if self.text.is_empty() {
            None
        } else {
            Some(self.text)
        };
        show_for_response(
            m.ui(),
            theme,
            prefs,
            response,
            text,
            self.side,
            self.multiline,
        );
    }
}

/// 组件内部的显示入口。`text = None` 表示目标不再需要 tooltip，会走退场。
pub(crate) fn show_for_response(
    ui: &Ui,
    theme: &Theme,
    prefs: MotionPrefs,
    response: &Response,
    text: Option<&str>,
    side: TooltipSide,
    multiline: bool,
) {
    let ctx = ui.ctx();
    let now = ui.input(|input| input.time);
    let state_id = response.id.with("m3_tooltip");
    let anim_id = state_id.with("tooltip_opacity");
    let area_id = state_id.with("tooltip_area");

    let has_text = text.is_some_and(|value| !value.is_empty());
    // `Response::hovered()` 用的 `interact_pos` 会在 `PointerGone` 后再保留一帧，
    // 指针离开窗口时还要看 `has_pointer`，否则 tooltip 会晚一帧才退场。
    let hovered = response.hovered() && ui.input(|input| input.pointer.has_pointer());
    // 禁用控件不显示 tooltip，和 `DTooltipHost.qml` 的 `enabled` 一致。
    let active = response.enabled() && has_text && (hovered || response.has_focus());
    let focused = active && response.has_focus();

    let mut state = match ctx.data(|data| data.get_temp::<TooltipState>(state_id)) {
        Some(state) => state,
        None => {
            if !active {
                return;
            }
            // egui 的第一次 animate 调用会直接落在目标值上，先钉住 0 才能淡入。
            motion::set_animation_start(ui, anim_id, 0.0);
            TooltipState {
                phase: if focused {
                    Phase::Visible
                } else {
                    Phase::Waiting
                },
                hover_started_at: now,
                target: response.rect,
                text: String::new(),
                side,
                multiline,
            }
        }
    };

    state.target = response.rect;
    if let Some(value) = text {
        state.set_text(value);
    }
    state.side = side;
    state.multiline = multiline;

    match state.advance(now, active, focused) {
        Step::Drop => {
            // 延迟期间就移开了，没有显示过，不需要退场。
            ctx.data_mut(|data| data.remove::<TooltipState>(state_id));
            return;
        }
        Step::Wait(seconds) => {
            ctx.request_repaint_after_secs(seconds);
            ctx.data_mut(|data| data.insert_temp(state_id, state));
            return;
        }
        Step::Paint { reset_opacity } => {
            if reset_opacity {
                motion::set_animation_start(ui, anim_id, 0.0);
            }
        }
    }

    let (opacity, finished) = match state.phase {
        Phase::Visible => {
            let animated =
                motion::animate_toward(ui, anim_id, 1.0, Motion::EXPRESSIVE_EFFECTS, prefs);
            (animated.value, false)
        }
        Phase::Hiding => {
            let animated = motion::animate_toward(ui, anim_id, 0.0, Motion::TOOLTIP_HIDE, prefs);
            (animated.value, animated.finished)
        }
        Phase::Waiting => unreachable!("waiting returns before painting"),
    };

    if finished {
        ctx.data_mut(|data| data.remove::<TooltipState>(state_id));
        return;
    }

    paint(ui, theme, area_id, &state, opacity);
    ctx.data_mut(|data| data.insert_temp(state_id, state));
}

/// tooltip 的显示阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// 等 400ms 延迟。
    Waiting,
    Visible,
    /// 100ms 退场淡出。
    Hiding,
}

/// 状态机推进一步后的动作。
enum Step {
    /// 丢弃状态，不绘制。
    Drop,
    /// 保持当前状态，等待 `seconds` 后重绘。
    Wait(f32),
    /// 推进透明度并绘制。
    Paint {
        /// 是否把透明度重新钉到 0（从“未显示”重新淡入）。
        reset_opacity: bool,
    },
}

/// 每个控件一份的 tooltip 状态。
///
/// 存在 `ctx.data` 里而不是调用方结构体里，和 egui 的立即模式一致；
/// `Response::id` 是 key，调用方不需要额外管理生命周期。
#[derive(Debug, Clone)]
struct TooltipState {
    phase: Phase,
    hover_started_at: f64,
    target: Rect,
    text: String,
    side: TooltipSide,
    multiline: bool,
}

impl TooltipState {
    fn set_text(&mut self, value: &str) {
        if self.text != value {
            self.text.clear();
            self.text.push_str(value);
        }
    }

    /// 根据 hover / focus 推进阶段。
    ///
    /// 规则来自 `DTooltipHost.qml` 与 `StateLayer.qml`：hover 等 400ms，
    /// focus 立即显示；移出后 100ms 退场；重新进入仍从 400ms 延迟开始。
    fn advance(&mut self, now: f64, active: bool, focused: bool) -> Step {
        match self.phase {
            Phase::Waiting if !active => Step::Drop,
            Phase::Waiting if focused => {
                self.phase = Phase::Visible;
                Step::Paint {
                    reset_opacity: true,
                }
            }
            Phase::Waiting => {
                let elapsed = (now - self.hover_started_at) as f32;
                if elapsed >= tokens::TOOLTIP_DELAY {
                    self.phase = Phase::Visible;
                    Step::Paint {
                        reset_opacity: true,
                    }
                } else {
                    Step::Wait(tokens::TOOLTIP_DELAY - elapsed)
                }
            }
            Phase::Visible => {
                if !active {
                    self.phase = Phase::Hiding;
                }
                Step::Paint {
                    reset_opacity: false,
                }
            }
            Phase::Hiding if active && focused => {
                self.phase = Phase::Visible;
                Step::Paint {
                    reset_opacity: false,
                }
            }
            Phase::Hiding if active => {
                self.phase = Phase::Waiting;
                self.hover_started_at = now;
                Step::Wait(tokens::TOOLTIP_DELAY)
            }
            Phase::Hiding => Step::Paint {
                reset_opacity: false,
            },
        }
    }
}

/// 绘制 tooltip。
fn paint(ui: &Ui, theme: &Theme, area_id: Id, state: &TooltipState, opacity: f32) {
    if opacity <= 0.0 || state.text.is_empty() {
        return;
    }

    let (size, galley) = measure(ui, &state.text, state.multiline);
    let pos = place(state.target, size, ui.ctx().content_rect(), state.side);
    let rect = Rect::from_min_size(pos, size);
    let scheme = theme.scheme();
    let background: Color32 = scheme.inverse_surface.into();
    let foreground: Color32 = scheme.inverse_on_surface.into();

    Area::new(area_id)
        .order(Order::Tooltip)
        .fixed_pos(pos)
        .interactable(false)
        .movable(false)
        // 透明度由 tooltip 自己控制；Area 自带的淡入会和退场时长冲突。
        .fade_in(false)
        .constrain(false)
        .show(ui.ctx(), |area_ui| {
            area_ui.set_min_size(size);
            area_ui.multiply_opacity(opacity);
            let painter = area_ui.painter();
            painter.rect_filled(rect, CornerRadius::same(tokens::CORNER_XS), background);
            painter.galley(text_origin(rect), galley, foreground);
        });
}

/// 文本的绘制原点。
///
/// epaint 的 `halign = Center` 以绘制点为水平中心（`galley.rect.center().x == 0`），
/// 所以 x 用整个 tooltip 的中心；y 保持 `rect.top() == 0` 的左上语义。
fn text_origin(rect: Rect) -> Pos2 {
    Pos2::new(rect.center().x, rect.top() + tokens::SPACING_S)
}

/// 先量文本，再算出带 padding 的最终尺寸。
///
/// 宽度上限是 `tooltipMaxWidth`，单行超出时省略；多行在宽度上限内换行。
fn measure(ui: &Ui, text: &str, multiline: bool) -> (Vec2, Arc<Galley>) {
    let natural = if multiline {
        text::layout_text(
            ui,
            text,
            TextLayout::multi_line(TextStyle::TOOLTIP, f32::INFINITY),
        )
    } else {
        text::layout_text(
            ui,
            text,
            TextLayout::single_line(TextStyle::TOOLTIP, f32::INFINITY),
        )
    };
    let content_width = natural.size().x.clamp(0.0, tokens::TOOLTIP_MAX_WIDTH);

    let layout = if multiline {
        TextLayout::multi_line_anywhere(TextStyle::TOOLTIP, content_width)
    } else {
        TextLayout::single_line(TextStyle::TOOLTIP, content_width)
    }
    .halign(Align::Center);
    let galley = text::layout_text(ui, text, layout);

    let size = vec2(
        content_width + tokens::SPACING_M * 2.0,
        galley.size().y + tokens::SPACING_S * 2.0,
    );
    (size, galley)
}

/// 计算 tooltip 左上角位置。
///
/// 来源: `DTooltipV2.qml` 的 `show()`：先按 side 摆放，再夹紧到窗口内，
/// 边距为 `spacingXS`。夹紧顺序和 QML 的
/// `max(margin, min(parent - size - margin, value))` 一致。
fn place(target: Rect, size: Vec2, screen: Rect, side: TooltipSide) -> Pos2 {
    let gap = tokens::SPACING_S;
    let margin = tokens::SPACING_XS;

    let raw = match side {
        TooltipSide::Top => pos2(
            target.center().x - size.x * 0.5,
            target.top() - size.y - gap,
        ),
        TooltipSide::Bottom => pos2(target.center().x - size.x * 0.5, target.bottom() + gap),
        TooltipSide::Left => pos2(
            target.left() - size.x - gap,
            target.center().y - size.y * 0.5,
        ),
        TooltipSide::Right => pos2(target.right() + gap, target.center().y - size.y * 0.5),
    };

    pos2(
        raw.x
            .min(screen.right() - size.x - margin)
            .max(screen.left() + margin),
        raw.y
            .min(screen.bottom() - size.y - margin)
            .max(screen.top() + margin),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    fn target() -> Rect {
        Rect::from_min_size(pos2(100.0, 100.0), vec2(40.0, 20.0))
    }

    fn screen() -> Rect {
        Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))
    }

    #[test]
    fn bottom_is_centered_horizontally_below_target() {
        let size = vec2(100.0, 30.0);
        let pos = place(target(), size, screen(), TooltipSide::Bottom);
        assert_eq!(pos, pos2(70.0, 128.0));
    }

    #[test]
    fn top_is_centered_horizontally_above_target() {
        let size = vec2(100.0, 30.0);
        let pos = place(target(), size, screen(), TooltipSide::Top);
        assert_eq!(pos, pos2(70.0, 62.0));
    }

    #[test]
    fn left_and_right_are_centered_vertically() {
        let size = vec2(100.0, 30.0);
        // 左侧空间不足，x 被夹紧到窗口边距。
        assert_eq!(
            place(target(), size, screen(), TooltipSide::Left),
            pos2(tokens::SPACING_XS, 95.0)
        );
        assert_eq!(
            place(target(), size, screen(), TooltipSide::Right),
            pos2(148.0, 95.0)
        );
    }

    #[test]
    fn placement_is_clamped_to_screen_margin() {
        let size = vec2(100.0, 30.0);
        let near_edge = Rect::from_min_size(pos2(780.0, 590.0), vec2(10.0, 10.0));
        let pos = place(near_edge, size, screen(), TooltipSide::Bottom);
        assert_eq!(pos.x, 800.0 - 100.0 - tokens::SPACING_XS);
        assert_eq!(pos.y, 600.0 - 30.0 - tokens::SPACING_XS);
    }

    #[test]
    fn placement_falls_back_to_margin_when_too_large() {
        let size = vec2(900.0, 700.0);
        let pos = place(target(), size, screen(), TooltipSide::Bottom);
        assert_eq!(pos.x, tokens::SPACING_XS);
        assert_eq!(pos.y, tokens::SPACING_XS);
    }

    /// 模拟 hover -> 延迟 -> 显示 -> 移出 -> 退场 的完整状态机。
    #[test]
    fn hover_delay_and_fade_out_state_machine() {
        use egui::{Event, RawInput, Sense};

        let full =
            crate::material::color::generate_theme(crate::material::color::ThemeVariant::Cmf {
                primary: 0xff769CDF,
                secondary: None,
            });
        let theme = Theme::from_color_theme(&full, crate::material::color::ThemeMode::Dark);

        let context = egui::Context::default();
        let button_id = Id::unique("tooltip_state_machine");
        let target_rect = Rect::from_min_size(pos2(40.0, 40.0), vec2(80.0, 30.0));
        let state_id = button_id.with("m3_tooltip");
        let area_id = state_id.with("tooltip_area");

        let run = |time: f64, event: Event| {
            let mut input = RawInput {
                time: Some(time),
                screen_rect: Some(screen()),
                ..Default::default()
            };
            input.events.push(event);
            let mut output = context.run_ui(input, |ui| {
                let response = ui.interact(target_rect, button_id, Sense::click());
                show_for_response(
                    ui,
                    &theme,
                    MotionPrefs::DEFAULT,
                    &response,
                    Some("tooltip"),
                    TooltipSide::Bottom,
                    false,
                );
            });
            output.textures_delta.clear();
            output
        };

        let phase = || {
            context
                .data(|data| data.get_temp::<TooltipState>(state_id))
                .map(|state| state.phase)
        };

        // 第一帧只负责让 egui 记录控件位置，hover 从下一帧开始生效。
        run(0.0, Event::PointerMoved(target_rect.center()));
        run(0.0, Event::PointerMoved(target_rect.center()));
        assert_eq!(phase(), Some(Phase::Waiting));

        run(0.2, Event::PointerMoved(target_rect.center()));
        assert_eq!(phase(), Some(Phase::Waiting));

        run(0.45, Event::PointerMoved(target_rect.center()));
        assert_eq!(phase(), Some(Phase::Visible));

        // egui 在目标切换的那一帧返回旧值；Area 第一次出现时还会先跑一遍
        // invisible 的 sizing pass，所以再推进两帧才能拿到真实绘制。
        run(0.46, Event::PointerMoved(target_rect.center()));
        let output = run(0.47, Event::PointerMoved(target_rect.center()));
        let area = egui::AreaState::load(&context, area_id).expect("tooltip area is painted");
        assert!(area.rect().top() > target_rect.bottom());

        // halign = Center 的 galley 以绘制点为水平中心，文字必须落在 tooltip 中心。
        let text = output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) if text.galley.job.text == "tooltip" => Some(text),
                _ => None,
            })
            .expect("tooltip text shape");
        let text_center_x = text.pos.x + text.galley.rect.center().x;
        assert!((text_center_x - area.rect().center().x).abs() < 0.5);

        run(0.5, Event::PointerGone);
        assert_eq!(phase(), Some(Phase::Hiding));

        run(0.7, Event::PointerGone);
        assert_eq!(phase(), None);
    }
}

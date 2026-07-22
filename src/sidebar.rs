//! 二级 List 响应式状态机。
//!
//! 基于 feedback.md 的 Pinned/Flyout/Modal 三态模型：
//! - Pinned(rail)  : 内联面板，扁平无阴影、无圆角，占布局。
//! - Flyout(rail)  : Order::Foreground 悬浮层，阴影+大圆角，无 scrim。
//! - Modal(rail)   : 悬浮层 + scrim 全屏半透明遮罩，Esc/点 scrim 关闭。
//! - Hidden        : 二级 List 完全不渲染。
//!
//! 响应式三阶梯自动选默认：
//! - Wide(>=1000)        : Pinned{pinned_rail}
//! - Medium(600..1000)   : Hidden，Hover 触发 Flyout
//! - Narrow(<600)        : Hidden，Rail 点击触发 Modal
//! 用户 Pin 按钮可覆盖自动默认；跨阈值仅在 auto 状态 (!is_user_pinned) 下改模式。
//!
//! Hover 防抖 180ms。动画 200ms quadratic_out。
//! Preview 淡入 150ms。
//!
//! 本文件目前为骨架（拆分期），具体 P1/P2 实施时填充渲染与状态转换逻辑。

use std::time::Instant;

use crate::material::ListItem;
use egui::Widget;

/// 应用中的板块 id（demo 板块）。
/// 现有 NavRail 测试态（theme_toggle/theme_switch/opt_* 等）保留不动，
/// 此处仅描述 4 个新的二级 List demo 板块。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailId {
    Files,
    Projects,
    Settings,
    Help,
}

impl RailId {
    /// 板块标题（用于 sidebar_body Header 文本 / Breadcrumb）。
    pub fn title(self) -> &'static str {
        match self {
            RailId::Files => "Files",
            RailId::Projects => "Projects",
            RailId::Settings => "Settings",
            RailId::Help => "Help",
        }
    }
}

/// 二级 List 当前渲染形态。
///
/// 设计上区分「模式」与「Pinned 目标」：
/// 模式决定渲染层（Panel / Area / Area+scrim / 不渲染），
/// `pinned_rail` 决定 Pinned 模式回退目标，且 Hover Preview 切换时仅改 pinned_rail。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    /// 不渲染二级 List。
    Hidden,
    /// 悬浮层，无 scrim；只能由 Hover(180ms) 触发。
    Flyout(RailId),
    /// 内联 Panel，占布局、扁平。
    Pinned(RailId),
    /// 悬浮层 + 全屏 scrim；由 Narrow 屏幕 Rail 点击触发。
    Modal(RailId),
}

impl SidebarMode {
    /// 当前激活的 rail（若不处于 Hidden）。
    pub fn rail(self) -> Option<RailId> {
        match self {
            SidebarMode::Hidden => None,
            SidebarMode::Flyout(r) | SidebarMode::Pinned(r) | SidebarMode::Modal(r) => Some(r),
        }
    }

    /// 是否属于悬浮层渲染（Flyout / Modal）。
    pub fn is_overlay(self) -> bool {
        matches!(self, SidebarMode::Flyout(_) | SidebarMode::Modal(_))
    }

    /// 是否为 Modal（需要 scrim + Esc 关闭）。
    pub fn is_modal(self) -> bool {
        matches!(self, SidebarMode::Modal(_))
    }
}

/// 响应式阈值（screen_rect().width()）。
const WIDE_MIN: f32 = 1000.0;
#[allow(dead_code)]
const MEDIUM_MIN: f32 = 600.0;

/// Hover 防抖延迟（ms）。
pub const HOVER_DELAY_MS: u64 = 180;

/// Hover 防抖判定：是否触发 Flyout / Preview。
#[allow(dead_code)]
fn hover_exceeded(since: Option<Instant>) -> bool {
    match since {
        Some(t) => t.elapsed().as_millis() as u64 >= HOVER_DELAY_MS,
        None => false,
    }
}

#[derive(Debug, Clone)]
pub struct SidebarState {
    /// 当前渲染模式。
    pub mode: SidebarMode,
    /// 自动选默认时的初始 rail（Wide 默认 Pinned 用的目标）。
    pub default_rail: RailId,
    /// 当前被 Pinned 的板块（Hover Preview 切换时改这个）。
    pub pinned_rail: RailId,
    /// 用户是否手动 Pin/Unpin 过；为 true 时跨响应式阈值不再自动切换模式。
    pub is_user_pinned: bool,
    /// 当前悬停的 rail（用于防抖计数）。
    pub hover_rail: Option<RailId>,
    /// 悬停开始时刻。
    pub hover_since: Option<Instant>,
    /// Pinned 模式下 Hover Preview 浮层的淡入 alpha（0..1）。
    pub preview_fade: f32,
}

impl SidebarState {
    pub fn new(default_rail: RailId) -> Self {
        Self {
            mode: SidebarMode::Hidden,
            default_rail,
            pinned_rail: default_rail,
            is_user_pinned: false,
            hover_rail: None,
            hover_since: None,
            preview_fade: 0.0,
        }
    }
}

/// 根据屏幕宽度给出响应式默认模式（未涉及用户手动 Pin）。
pub fn responsive_default(rail: RailId, screen_width: f32) -> SidebarMode {
    if screen_width >= WIDE_MIN {
        SidebarMode::Pinned(rail)
    } else {
        SidebarMode::Hidden
    }
}

/// 由 Rail hover 触发的状态更新（180ms 防抖）。
/// 返回是否需要切换模式（由调用者根据响应式默认再决定 Flyout / Modal）。
pub fn update_hover(
    state: &mut SidebarState,
    hovered_rail: Option<RailId>,
    now: Instant,
    screen_width: f32,
) {
    // 基础版本：暂不实现防抖，直接更新hover状态
    // 后续步骤5会实现完整的180ms防抖逻辑
    state.hover_rail = hovered_rail;
    state.hover_since = if hovered_rail.is_some() {
        Some(now)
    } else {
        None
    };

    // 基础版本：hover时直接切换到Flyout模式（仅在Medium屏幕）
    // 后续会完善为防抖后的逻辑
    if let Some(rail) = hovered_rail {
        if screen_width < WIDE_MIN && screen_width >= MEDIUM_MIN {
            // Medium屏幕：hover触发Flyout
            state.mode = SidebarMode::Flyout(rail);
        }
    } else if state.mode.is_overlay() && !state.mode.is_modal() {
        // 鼠标离开且当前是Flyout模式，隐藏
        state.mode = SidebarMode::Hidden;
    }
}

/// Rail 点击触发的模式切换。
pub fn rail_click(state: &mut SidebarState, rail: RailId, screen_width: f32) {
    match state.mode {
        SidebarMode::Hidden => {
            // Hidden状态：根据屏幕宽度选择模式
            if screen_width >= WIDE_MIN {
                // Wide屏幕：切换到Pinned模式
                state.mode = SidebarMode::Pinned(rail);
                state.pinned_rail = rail;
            } else if screen_width >= MEDIUM_MIN {
                // Medium屏幕：切换到Flyout模式
                state.mode = SidebarMode::Flyout(rail);
                state.pinned_rail = rail;
            } else {
                // Narrow屏幕：切换到Modal模式
                state.mode = SidebarMode::Modal(rail);
                state.pinned_rail = rail;
            }
        }
        SidebarMode::Pinned(current_rail) => {
            if current_rail == rail {
                // 点击当前rail：隐藏
                state.mode = SidebarMode::Hidden;
            } else {
                // 点击其他rail：切换pinned_rail
                state.pinned_rail = rail;
                state.mode = SidebarMode::Pinned(rail);
            }
        }
        SidebarMode::Flyout(current_rail) => {
            if current_rail == rail {
                // 点击当前rail：隐藏Flyout
                state.mode = SidebarMode::Hidden;
            } else {
                // 点击其他rail：切换到该rail的Flyout
                state.pinned_rail = rail;
                state.mode = SidebarMode::Flyout(rail);
            }
        }
        SidebarMode::Modal(current_rail) => {
            if current_rail == rail {
                // 点击当前rail：隐藏Modal
                state.mode = SidebarMode::Hidden;
            } else {
                // 点击其他rail：切换到该rail的Modal
                state.pinned_rail = rail;
                state.mode = SidebarMode::Modal(rail);
            }
        }
    }
}

/// 渲染分发：根据 state.mode 调度 Pinned Panel / Flyout Area / Modal Area+scrim。

/// 渲染覆盖层（Flyout / Modal）。每帧调用，不需 Panel。
pub fn render_overlays(
    ctx: &egui::Context,
    state: &mut SidebarState,
    surface_color: egui::Color32,
    content_rect: egui::Rect,
    screen_width: f32,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    match state.mode {
        SidebarMode::Flyout(rail) => {
            render_flyout(
                ctx,
                rail,
                surface_color,
                content_rect,
                list_sel_std,
                list_sel_seg_0,
                list_sel_seg_1,
                list_sel_seg_2,
            );
        }
        SidebarMode::Modal(rail) => {
            let scrim_color: egui::Color32 =
                crate::material::color::access(|_p, s| s.scrim).into();

            egui::Area::new(egui::Id::new("modal_scrim"))
                .fixed_pos(egui::pos2(96.0, content_rect.top()))
                .constrain(false)
                .order(egui::Order::Foreground)
                .interactable(true)
                .fade_in(false)
                .show(ctx, |ui| {
                    let scrim_size =
                        egui::vec2(screen_width - 96.0, content_rect.height());
                    ui.allocate_ui(scrim_size, |ui| {
                        ui.painter()
                            .rect_filled(ui.max_rect(), 0.0, scrim_color.gamma_multiply(0.5));
                        if ui
                            .interact(
                                ui.max_rect(),
                                egui::Id::new("scrim_click"),
                                egui::Sense::click(),
                            )
                            .clicked()
                        {
                            state.mode = SidebarMode::Hidden;
                        }
                    });
                });

            egui::Area::new(egui::Id::new("modal_content"))
                .fixed_pos(egui::pos2(96.0, content_rect.top()))
                .constrain(false)
                .order(egui::Order::Foreground)
                .interactable(true)
                .fade_in(false)
                .show(ctx, |ui| {
                    ui.allocate_ui(egui::vec2(300.0, content_rect.height()), |ui| {
                        render_overlay_content(
                            ui,
                            rail,
                            surface_color,
                            list_sel_std,
                            list_sel_seg_0,
                            list_sel_seg_1,
                            list_sel_seg_2,
                        );
                    });
                });
        }
        _ => {}
    }
}

fn render_overlay_content(
    ui: &mut egui::Ui,
    rail: RailId,
    surface_color: egui::Color32,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    let frame = egui::containers::Frame {
        inner_margin: egui::Margin::symmetric(0, 4),
        outer_margin: egui::Margin::ZERO,
        corner_radius: egui::CornerRadius {
            nw: 0,
            ne: 12,
            sw: 0,
            se: 12,
        },
        shadow: egui::Shadow {
            offset: [8, 0],
            blur: 24,
            spread: 0,
            color: egui::Color32::from_black_alpha(60),
        },
        fill: surface_color,
        stroke: egui::Stroke::NONE,
    };

    frame.show(ui, |ui| {
        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0, 0.0);
        ui.heading(rail.title());
        ui.add_space(4.);
        render_sidebar_content(
            ui,
            list_sel_std,
            list_sel_seg_0,
            list_sel_seg_1,
            list_sel_seg_2,
        );
        ui.add_space(ui.available_height());
    });
}

fn render_flyout(
    ctx: &egui::Context,
    rail: RailId,
    surface_color: egui::Color32,
    content_rect: egui::Rect,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    egui::Area::new(egui::Id::new("sidebar_flyout"))
        .fixed_pos(egui::pos2(96.0, content_rect.top()))
        .constrain(false)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(300.0, content_rect.height()), |ui| {
                render_overlay_content(
                    ui,
                    rail,
                    surface_color,
                    list_sel_std,
                    list_sel_seg_0,
                    list_sel_seg_1,
                    list_sel_seg_2,
                );
            });
        });
}

/// 渲染面板内容（Pinned 模式）。在 Panel::left 内部调用。
pub fn render_pinned(
    ui: &mut egui::Ui,
    state: &SidebarState,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    if let SidebarMode::Pinned(rail) = state.mode {
        ui.set_width(300.0);
        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0, 0.0);
        ui.heading(rail.title());
        ui.add_space(4.);
        render_sidebar_content(
            ui,
            list_sel_std,
            list_sel_seg_0,
            list_sel_seg_1,
            list_sel_seg_2,
        );
    }
}

/// 渲染sidebar内容（可复用）
fn render_sidebar_content(
    ui: &mut egui::Ui,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    ui.heading("Standard (单选)");
    ui.add_space(4.);
    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0., 2.);
        if ListItem::new(
            "std_0",
            "我的世界",
            None,
            None,
            *list_sel_std,
            false,
            false,
            false,
        )
        .ui(ui)
        .clicked()
        {
            *list_sel_std = true;
        }
        if ListItem::new(
            "std_1",
            "进入1qjkl异世界",
            Some("qqqqqqq1111"),
            None,
            !*list_sel_std,
            false,
            false,
            false,
        )
        .ui(ui)
        .clicked()
        {
            *list_sel_std = false;
        }
    });

    ui.add_space(16.);
    ui.heading("Segmented (多选)");
    ui.add_space(4.);

    let seg0 = *list_sel_seg_0;
    let seg1 = *list_sel_seg_1;
    let seg2 = *list_sel_seg_2;

    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0., 2.);
        if ListItem::new("seg_0", "叫我起床", None, None, seg0, true, false, !seg1)
            .ui(ui)
            .clicked()
        {
            *list_sel_seg_0 = !*list_sel_seg_0;
        }
        if ListItem::new(
            "seg_1",
            "别叫我起床",
            Some("因为我想多睡点觉"),
            None,
            seg1,
            true,
            !seg0,
            !seg2,
        )
        .ui(ui)
        .clicked()
        {
            *list_sel_seg_1 = !*list_sel_seg_1;
        }
        if ListItem::new(
            "seg_2",
            "在半夜叫我",
            Some("喵喵11111111111111111122222211111111111111111"),
            Some("嗯111111"),
            seg2,
            true,
            !seg1,
            false,
        )
        .ui(ui)
        .clicked()
        {
            *list_sel_seg_2 = !*list_sel_seg_2;
        }
    });
}

/// 自动应用响应式默认（每帧调用，跨阈值仅在 auto 状态下改模式）。
pub fn apply_responsive_default(state: &mut SidebarState, screen_width: f32) {
    // 用户手动Pin过，不自动切换
    if state.is_user_pinned {
        return;
    }

    // 获取当前屏幕宽度对应的默认模式
    let new_default = responsive_default(state.pinned_rail, screen_width);

    // 只在auto模式下重置（用户未手动Pin）
    // 检查当前模式是否需要更新
    let should_update = match (state.mode, new_default) {
        // 当前Hidden，但屏幕宽度应该显示Pinned
        (SidebarMode::Hidden, SidebarMode::Pinned(_)) => true,
        // 当前Pinned，但屏幕宽度应该Hidden
        (SidebarMode::Pinned(_), SidebarMode::Hidden) => true,
        // 其他情况不更新
        _ => false,
    };

    if should_update {
        state.mode = new_default;
    }
}

/// 输入 Event 处理（Modal 的 Esc 关闭）。
pub fn handle_input(ctx: &egui::Context, state: &mut SidebarState) {
    if state.mode.is_modal() {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            state.mode = SidebarMode::Hidden;
        }
    }
}

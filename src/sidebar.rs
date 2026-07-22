//! 二级 List 响应式状态机。
//!
//! 基于 feedback.md 的 Pinned/Flyout 二态模型：
//! - Pinned  : 内联面板，扁平无阴影、无圆角，占布局。
//! - Flyout  : 悬浮层 + scrim + 阴影+大圆角，不占布局。
//! - Hidden  : 二级 List 完全不渲染。
//!
//! active_rail 独立于 mode，点击切换 rail 不受模式切换影响。
//!
//! 响应式三阶梯自动选默认：
//! - Wide(>=1000)        : Pinned
//! - Medium(600..1000)   : Hidden，Hover 触发 Flyout
//! - Narrow(<600)        : Hidden，Rail 点击触发 Flyout

use std::time::Instant;

use crate::material::ListItem;
use egui::Widget;

/// Flyout 触发方式：Click 触发不自动关，Hover 触发离开即关。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlyoutTrigger {
    Click,
    Hover,
}

/// 应用中的板块 id（demo 板块）。
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

/// 渲染模式（纯模式，不含 rail 数据）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Hidden,
    Flyout,
    Pinned,
}

impl SidebarMode {
    pub fn is_overlay(self) -> bool {
        matches!(self, SidebarMode::Flyout)
    }
}

const WIDE_MIN: f32 = 1000.0;
const MEDIUM_MIN: f32 = 600.0;

pub const HOVER_DELAY_MS: u64 = 180;

fn hover_exceeded(since: Option<Instant>, delay_ms: u64) -> bool {
    match since {
        Some(t) => t.elapsed().as_millis() as u64 >= delay_ms,
        None => false,
    }
}

/// Flyout 渲染时使用的 rail：hover 触发的用 hover_rail，其它情况用 active_rail。
fn flyout_rail(state: &SidebarState) -> Option<RailId> {
    match state.flyout_trigger {
        Some(FlyoutTrigger::Hover) => state.hover_rail.or(state.active_rail),
        _ => state.active_rail,
    }
}

#[derive(Debug, Clone)]
pub struct SidebarState {
    pub mode: SidebarMode,
    /// 用户选中的 panel（点击切换时更新，不受模式影响）。
    pub active_rail: Option<RailId>,
    pub is_user_pinned: bool,
    pub hover_rail: Option<RailId>,
    pub hover_since: Option<Instant>,
    pub preview_fade: f32,
    pub close_requested: bool,
    pub flyout_trigger: Option<FlyoutTrigger>,
}

impl SidebarState {
    pub fn new(active_rail: Option<RailId>) -> Self {
        Self {
            mode: SidebarMode::Hidden,
            active_rail,
            is_user_pinned: false,
            hover_rail: None,
            hover_since: None,
            preview_fade: 0.0,
            close_requested: false,
            flyout_trigger: None,
        }
    }
}

pub fn responsive_default(screen_width: f32) -> SidebarMode {
    if screen_width >= WIDE_MIN {
        SidebarMode::Pinned
    } else {
        SidebarMode::Hidden
    }
}

/// Hover 防抖状态更新。返回 None 无需重绘，Some(dur) 在 dur 后重绘。
pub fn update_hover(
    state: &mut SidebarState,
    hovered_rail: Option<RailId>,
    now: Instant,
    screen_width: f32,
) -> Option<std::time::Duration> {
    if screen_width >= WIDE_MIN {
        return None;
    }

    let is_click_flyout = state.mode == SidebarMode::Flyout
        && state.flyout_trigger == Some(FlyoutTrigger::Click);
    if is_click_flyout {
        return None;
    }

    let delay_ms = if screen_width < MEDIUM_MIN {
        300
    } else {
        HOVER_DELAY_MS
    };

    if hovered_rail != state.hover_rail {
        state.hover_rail = hovered_rail;
        state.hover_since = hovered_rail.map(|_| now);
    }

    if hovered_rail.is_some() {
        if hover_exceeded(state.hover_since, delay_ms) {
            state.mode = SidebarMode::Flyout;
            state.flyout_trigger = Some(FlyoutTrigger::Hover);
        } else if let Some(since) = state.hover_since {
            let elapsed = since.elapsed().as_millis() as u64;
            if elapsed < delay_ms {
                return Some(std::time::Duration::from_millis(delay_ms - elapsed));
            }
        }
    }

    None
}

pub fn rail_click(state: &mut SidebarState, rail: RailId, screen_width: f32) {
    let same_rail = state.active_rail == Some(rail);
    state.active_rail = Some(rail);

    match state.mode {
        SidebarMode::Hidden => {
            if screen_width >= WIDE_MIN {
                state.mode = SidebarMode::Pinned;
            } else {
                state.mode = SidebarMode::Flyout;
                state.flyout_trigger = Some(FlyoutTrigger::Click);
            }
        }
        SidebarMode::Pinned => {
            if same_rail {
                state.mode = SidebarMode::Hidden;
            }
        }
        SidebarMode::Flyout => {
            if same_rail {
                state.mode = SidebarMode::Hidden;
                state.flyout_trigger = None;
            } else {
                state.flyout_trigger = Some(FlyoutTrigger::Click);
            }
        }
    }
}

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
    if state.mode != SidebarMode::Flyout {
        return;
    }
    let rail = match flyout_rail(state) {
        Some(r) => r,
        None => return,
    };

    let scrim_color: egui::Color32 =
        crate::material::color::access(|_p, s| s.scrim).into();

    egui::Area::new(egui::Id::new("sidebar_scrim"))
        .fixed_pos(egui::pos2(96.0, content_rect.top()))
        .constrain(false)
        .order(egui::Order::Foreground)
        .interactable(true)
        .fade_in(false)
        .show(ctx, |ui| {
            let scrim_size = egui::vec2(screen_width - 96.0, content_rect.height());
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
                    state.flyout_trigger = None;
                }
            });
        });

    egui::Area::new(egui::Id::new("sidebar_flyout"))
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

/// 渲染面板内容（Pinned 模式）。
pub fn render_pinned(
    ui: &mut egui::Ui,
    state: &SidebarState,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    if state.mode != SidebarMode::Pinned {
        return;
    }
    let rail = match state.active_rail {
        Some(r) => r,
        None => return,
    };

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

pub fn apply_responsive_default(state: &mut SidebarState, screen_width: f32) {
    if state.is_user_pinned {
        return;
    }
    let new_default = responsive_default(screen_width);
    if state.mode != new_default
        && !matches!(
            (state.mode, new_default),
            (SidebarMode::Flyout, _) | (_, SidebarMode::Flyout)
        )
    {
        state.mode = new_default;
    }
}

pub fn handle_input(ctx: &egui::Context, state: &mut SidebarState) {
    if state.mode.is_overlay() {
        if state.close_requested || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            state.mode = SidebarMode::Hidden;
            state.close_requested = false;
            state.flyout_trigger = None;
        }
    }
}

/// Hover 触发的 Flyout 离开联合区域（rail + flyout）时自动关闭。
/// Click 触发的 Flyout 不在此处关闭（由 Esc/scrim/close_requested 关闭）。
pub fn check_flyout_leave(ctx: &egui::Context, state: &mut SidebarState, content_rect: egui::Rect) {
    if !state.mode.is_overlay() || state.flyout_trigger != Some(FlyoutTrigger::Hover) {
        return;
    }
    if let Some(pos) = ctx.pointer_latest_pos() {
        let rail_rect = egui::Rect::from_min_max(
            egui::pos2(0.0, content_rect.top()),
            egui::pos2(96.0, content_rect.bottom()),
        );
        let flyout_rect = egui::Rect::from_min_max(
            egui::pos2(96.0, content_rect.top()),
            egui::pos2(396.0, content_rect.bottom()),
        );
        if !rail_rect.contains(pos) && !flyout_rect.contains(pos) {
            state.mode = SidebarMode::Hidden;
            state.flyout_trigger = None;
        }
    }
}

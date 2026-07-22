//! 二级 List 响应式状态机。
//!
//! 基于显式状态枚举（SidebarInner），消除所有非法状态组合。
//!
//! 变体:
//! - Hidden: 不渲染
//! - Pinned: 内嵌面板（width >= 1000 默认）
//! - FlyoutHoverPending: Hover 延迟等待中
//! - FlyoutHover: Hover 触发的 overlay（离开即关）
//! - FlyoutClick: Click 触发的 overlay（不自动关）
//!
//! 公开 API:
//! - refresh(): 响应式默认、键盘、Hover 计时/追踪
//! - click_rail(): 点击 rail 触发状态转换
//! - check_flyout_leave(): Hover Flyout 离开关闭
//! - render_mode(): 渲染时判断画什么

use std::time::{Duration, Instant};

use crate::material::ListItem;
use egui::Widget;

const WIDE_MIN: f32 = 1000.0;
const MEDIUM_MIN: f32 = 600.0;
const HOVER_DELAY_MS: u64 = 180;
const NARROW_HOVER_DELAY_MS: u64 = 300;

/// 应用中的板块 id（demo 板块）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailId {
    Files,
    Projects,
    Settings,
    Help,
}

impl RailId {
    pub fn title(self) -> &'static str {
        match self {
            RailId::Files => "Files",
            RailId::Projects => "Projects",
            RailId::Settings => "Settings",
            RailId::Help => "Help",
        }
    }
}

/// 渲染意图：渲染函数匹配此枚举，无需关心内部状态细节。
#[derive(Debug, Clone, Copy)]
pub enum RenderMode {
    Hidden,
    Pinned(RailId),
    Flyout(RailId),
}

/// 显式状态枚举：每个变体完整描述当前状态，无歧义字段组合。
#[derive(Debug, Clone)]
enum SidebarInner {
    /// 侧边栏不渲染
    Hidden {
        active_rail: Option<RailId>,
        /// Click 关闭后阻止 hover 立即重新打开
        hover_blocked: bool,
    },
    /// 内嵌面板（width >= 1000 时默认或用户 Pin）
    Pinned {
        active_rail: Option<RailId>,
        /// true = 用户手动 Pin，跨阈值不自动切换
        is_user_pinned: bool,
    },
    /// Hover 延迟等待中（定时器未到期，不可见）
    FlyoutHoverPending {
        active_rail: Option<RailId>,
        hover_rail: RailId,
        hover_since: Instant,
    },
    /// Hover 触发后的 overlay（可见，离开即关）
    FlyoutHover {
        active_rail: Option<RailId>,
        hover_rail: RailId,
    },
    /// Click 触发后的 overlay（可见，不自动关）
    FlyoutClick {
        active_rail: RailId,
    },
}

impl SidebarInner {
    fn active_rail(&self) -> Option<RailId> {
        match self {
            SidebarInner::Hidden { active_rail, .. }
            | SidebarInner::Pinned { active_rail, .. }
            | SidebarInner::FlyoutHoverPending { active_rail, .. }
            | SidebarInner::FlyoutHover { active_rail, .. } => *active_rail,
            SidebarInner::FlyoutClick { active_rail, .. } => Some(*active_rail),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SidebarState {
    inner: SidebarInner,
    /// 外部关闭请求（List Item 点击等预留）
    pub close_requested: bool,
    /// ------ demo list 选中状态 ------
    pub list_sel_std: bool,
    pub list_sel_seg_0: bool,
    pub list_sel_seg_1: bool,
    pub list_sel_seg_2: bool,
}

impl SidebarState {
    pub fn new(active_rail: Option<RailId>) -> Self {
        Self {
            inner: SidebarInner::Hidden {
                active_rail,
                hover_blocked: false,
            },
            close_requested: false,
            list_sel_std: false,
            list_sel_seg_0: false,
            list_sel_seg_1: false,
            list_sel_seg_2: false,
        }
    }

    /// 用户选中的 panel（用于 rail 按钮高亮）。
    pub fn active_rail(&self) -> Option<RailId> {
        self.inner.active_rail()
    }

    /// 渲染意图，调用方按此分支渲染。
    pub fn render_mode(&self) -> RenderMode {
        match &self.inner {
            SidebarInner::Hidden { .. } | SidebarInner::FlyoutHoverPending { .. } => {
                RenderMode::Hidden
            }
            SidebarInner::Pinned { active_rail, .. } => match active_rail {
                Some(rail) => RenderMode::Pinned(*rail),
                None => RenderMode::Hidden,
            },
            SidebarInner::FlyoutHover { hover_rail, .. } => RenderMode::Flyout(*hover_rail),
            SidebarInner::FlyoutClick { active_rail, .. } => RenderMode::Flyout(*active_rail),
        }
    }

    pub fn debug_mode_str(&self) -> &'static str {
        match &self.inner {
            SidebarInner::Hidden { hover_blocked, .. } => {
                if *hover_blocked {
                    "Hidden(B)"
                } else {
                    "Hidden"
                }
            }
            SidebarInner::Pinned { is_user_pinned, .. } => {
                if *is_user_pinned {
                    "Pinned(UP)"
                } else {
                    "Pinned"
                }
            }
            SidebarInner::FlyoutHoverPending { .. } => "Pending",
            SidebarInner::FlyoutHover { .. } => "Flyout(Hover)",
            SidebarInner::FlyoutClick { .. } => "Flyout(Click)",
        }
    }

    pub fn debug_trigger_str(&self) -> &'static str {
        match &self.inner {
            SidebarInner::FlyoutHover { .. } | SidebarInner::FlyoutHoverPending { .. } => "Hover",
            SidebarInner::FlyoutClick { .. } => "Click",
            _ => "-",
        }
    }

    pub fn is_overlay(&self) -> bool {
        matches!(
            self.inner,
            SidebarInner::FlyoutHover { .. }
                | SidebarInner::FlyoutHoverPending { .. }
                | SidebarInner::FlyoutClick { .. }
        )
    }

    // ---------------------------------------------------------------------------
    // 状态转换
    // ---------------------------------------------------------------------------

    /// 每帧调用，处理所有时间和输入驱动的自动转换。
    ///
    /// 返回 `Some(Duration)` 时需要 `request_repaint_after`。
    pub fn refresh(
        &mut self,
        ctx: &egui::Context,
        screen_width: f32,
        now: Instant,
        hovered_rail: Option<RailId>,
    ) -> Option<Duration> {
        // 1. 响应式默认
        self.apply_responsive(screen_width);

        // 2. close_requested
        if self.close_requested {
            self.close_requested = false;
            if self.is_overlay() {
                self.close();
                return None;
            }
        }

        // 3. Escape
        if self.is_overlay() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close();
            return None;
        }

        // 4. Hover 计时 + 追踪
        self.apply_hover(screen_width, now, hovered_rail)
    }

    /// 用户点击 rail 按钮。
    pub fn click_rail(&mut self, rail: RailId, screen_width: f32) {
        let same_rail = self.inner.active_rail() == Some(rail);
        let wide = screen_width >= WIDE_MIN;

        self.inner = match &self.inner {
            SidebarInner::Hidden { .. } => {
                let active_rail = Some(rail);
                if wide {
                    SidebarInner::Pinned {
                        active_rail,
                        is_user_pinned: false,
                    }
                } else {
                    SidebarInner::FlyoutClick { active_rail: rail }
                }
            }
            SidebarInner::Pinned { is_user_pinned, .. } => {
                if same_rail {
                    return;
                }
                SidebarInner::Pinned {
                    active_rail: Some(rail),
                    is_user_pinned: *is_user_pinned,
                }
            }
            SidebarInner::FlyoutHover { .. }
            | SidebarInner::FlyoutHoverPending { .. } => {
                SidebarInner::FlyoutClick { active_rail: rail }
            }
            SidebarInner::FlyoutClick { .. } => {
                if same_rail {
                    SidebarInner::Hidden {
                        active_rail: Some(rail),
                        hover_blocked: true,
                    }
                } else {
                    SidebarInner::FlyoutClick { active_rail: rail }
                }
            }
        };
    }

    /// Hover 触发的 Flyout 离开联合区域（rail + flyout）时自动关闭。
    pub fn check_flyout_leave(
        &mut self,
        ctx: &egui::Context,
        content_rect: egui::Rect,
    ) {
        let active_rail = match &self.inner {
            SidebarInner::FlyoutHover { active_rail, .. } => *active_rail,
            _ => return,
        };
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
                self.inner = SidebarInner::Hidden {
                    active_rail,
                    hover_blocked: false,
                };
            }
        }
    }

    // ---------------------------------------------------------------------------
    // 内部辅助
    // ---------------------------------------------------------------------------

    fn is_user_pinned(&self) -> bool {
        matches!(
            &self.inner,
            SidebarInner::Pinned {
                is_user_pinned: true,
                ..
            }
        )
    }

    fn close(&mut self) {
        let active_rail = self.inner.active_rail();
        self.inner = SidebarInner::Hidden {
            active_rail,
            hover_blocked: false,
        };
        self.close_requested = false;
    }

    /// 响应式默认：根据 screen_width 在 Pinned / Hidden 间自动切换。
    /// Flyout* 在 grow 时也切到 Pinned。
    fn apply_responsive(&mut self, screen_width: f32) {
        if self.is_user_pinned() {
            return;
        }
        let wide = screen_width >= WIDE_MIN;
        let new_inner = match (wide, &self.inner) {
            // 缩小: Pinned → Hidden
            (false, SidebarInner::Pinned { active_rail, .. }) => {
                Some(SidebarInner::Hidden {
                    active_rail: *active_rail,
                    hover_blocked: false,
                })
            }
            // 放大: Hidden → Pinned
            (true, SidebarInner::Hidden { active_rail, .. }) => {
                Some(SidebarInner::Pinned {
                    active_rail: *active_rail,
                    is_user_pinned: false,
                })
            }
            // 放大: Flyout* → Pinned
            (true, SidebarInner::FlyoutHoverPending { active_rail, .. }) => {
                Some(SidebarInner::Pinned {
                    active_rail: *active_rail,
                    is_user_pinned: false,
                })
            }
            (true, SidebarInner::FlyoutHover { active_rail, .. }) => {
                Some(SidebarInner::Pinned {
                    active_rail: *active_rail,
                    is_user_pinned: false,
                })
            }
            (true, SidebarInner::FlyoutClick { active_rail }) => {
                Some(SidebarInner::Pinned {
                    active_rail: Some(*active_rail),
                    is_user_pinned: false,
                })
            }
            _ => None,
        };
        if let Some(inner) = new_inner {
            self.inner = inner;
        }
    }

    /// Hover 计时和追踪逻辑。
    fn apply_hover(
        &mut self,
        screen_width: f32,
        now: Instant,
        hovered_rail: Option<RailId>,
    ) -> Option<Duration> {
        if screen_width >= WIDE_MIN {
            return None;
        }

        let delay_ms = if screen_width < MEDIUM_MIN {
            NARROW_HOVER_DELAY_MS
        } else {
            HOVER_DELAY_MS
        };

        match (&self.inner, hovered_rail) {
            // Click flyout 活跃时，忽略 hover
            (SidebarInner::FlyoutClick { .. }, _) => None,

            // Hover block 中，等待鼠标先离开 rail 再启用
            (SidebarInner::Hidden { hover_blocked: true, .. }, _) => {
                if hovered_rail.is_none() {
                    let active_rail = self.inner.active_rail();
                    self.inner = SidebarInner::Hidden {
                        active_rail,
                        hover_blocked: false,
                    };
                }
                None
            }

            // Hidden → 鼠标移到 rail 上 → 开始计时
            (SidebarInner::Hidden { active_rail, .. }, Some(rail)) => {
                self.inner = SidebarInner::FlyoutHoverPending {
                    active_rail: *active_rail,
                    hover_rail: rail,
                    hover_since: now,
                };
                Some(Duration::from_millis(delay_ms))
            }

            // Pending → 鼠标离开了 rail
            (
                SidebarInner::FlyoutHoverPending {
                    active_rail, ..
                },
                None,
            ) => {
                self.inner = SidebarInner::Hidden {
                    active_rail: *active_rail,
                    hover_blocked: false,
                };
                None
            }

            // Pending → 同一 rail，检查计时
            (
                SidebarInner::FlyoutHoverPending {
                    active_rail,
                    hover_rail,
                    hover_since,
                },
                Some(rail),
            ) if *hover_rail == rail => {
                let elapsed = hover_since.elapsed();
                if elapsed >= Duration::from_millis(delay_ms) {
                    self.inner = SidebarInner::FlyoutHover {
                        active_rail: *active_rail,
                        hover_rail: rail,
                    };
                    None
                } else {
                    Some(Duration::from_millis(delay_ms) - elapsed)
                }
            }

            // Pending → 不同 rail，重启计时
            (
                SidebarInner::FlyoutHoverPending {
                    active_rail, ..
                },
                Some(rail),
            ) => {
                self.inner = SidebarInner::FlyoutHoverPending {
                    active_rail: *active_rail,
                    hover_rail: rail,
                    hover_since: now,
                };
                Some(Duration::from_millis(delay_ms))
            }

            // Hover 已可见 → 同一 rail，保持
            (
                SidebarInner::FlyoutHover { .. },
                Some(rail),
            ) if Some(rail) == self.inner.active_rail() => {
                // Already showing, do nothing. Leave is handled by check_flyout_leave.
                None
            }

            // Hover 已可见 → 不同 rail，立即切换
            (
                SidebarInner::FlyoutHover {
                    active_rail, ..
                },
                Some(rail),
            ) => {
                self.inner = SidebarInner::FlyoutHover {
                    active_rail: *active_rail,
                    hover_rail: rail,
                };
                None
            }

            // Hover 已可见 → 鼠标不在 rail 上，不急于关闭（check_flyout_leave 处理）
            (SidebarInner::FlyoutHover { .. }, None) => None,

            // 其余组合无操作
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 渲染函数
// ---------------------------------------------------------------------------

/// Flyout 叠加层（scrim + flyout content）。
pub fn render_overlays(
    ctx: &egui::Context,
    state: &mut SidebarState,
    surface_color: egui::Color32,
    content_rect: egui::Rect,
    screen_width: f32,
) {
    let rail = match state.render_mode() {
        RenderMode::Flyout(r) => r,
        _ => return,
    };

    let scrim_color: egui::Color32 =
        crate::material::color::access(|_p, s| s.scrim).into();

    // scrim
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
                    state.close();
                }
            });
        });

    // flyout content
    egui::Area::new(egui::Id::new("sidebar_flyout"))
        .fixed_pos(egui::pos2(96.0, content_rect.top()))
        .constrain(false)
        .order(egui::Order::Foreground)
        .interactable(true)
        .fade_in(false)
        .show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(300.0, content_rect.height()), |ui| {
                render_overlay_content(ui, rail, surface_color, state);
            });
        });
}

fn render_overlay_content(
    ui: &mut egui::Ui,
    rail: RailId,
    surface_color: egui::Color32,
    state: &mut SidebarState,
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
        render_sidebar_content(ui, state);
        ui.add_space(ui.available_height());
    });
}

/// Pinned 内嵌面板。
pub fn render_pinned(ui: &mut egui::Ui, state: &mut SidebarState) {
    let rail = match state.render_mode() {
        RenderMode::Pinned(r) => r,
        _ => return,
    };

    ui.set_width(300.0);
    ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0, 0.0);
    ui.heading(rail.title());
    ui.add_space(4.);
    render_sidebar_content(ui, state);
}

fn render_sidebar_content(ui: &mut egui::Ui, state: &mut SidebarState) {
    ui.heading("Standard (单选)");
    ui.add_space(4.);
    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0., 2.);
        if ListItem::new(
            "std_0",
            "我的世界",
            None,
            None,
            state.list_sel_std,
            false,
            false,
            false,
        )
        .ui(ui)
        .clicked()
        {
            state.list_sel_std = true;
        }
        if ListItem::new(
            "std_1",
            "进入1qjkl异世界",
            Some("qqqqqqq1111"),
            None,
            !state.list_sel_std,
            false,
            false,
            false,
        )
        .ui(ui)
        .clicked()
        {
            state.list_sel_std = false;
        }
    });

    ui.add_space(16.);
    ui.heading("Segmented (多选)");
    ui.add_space(4.);

    let seg0 = state.list_sel_seg_0;
    let seg1 = state.list_sel_seg_1;
    let seg2 = state.list_sel_seg_2;

    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0., 2.);
        if ListItem::new("seg_0", "叫我起床", None, None, seg0, true, false, !seg1)
            .ui(ui)
            .clicked()
        {
            state.list_sel_seg_0 = !state.list_sel_seg_0;
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
            state.list_sel_seg_1 = !state.list_sel_seg_1;
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
            state.list_sel_seg_2 = !state.list_sel_seg_2;
        }
    });
}

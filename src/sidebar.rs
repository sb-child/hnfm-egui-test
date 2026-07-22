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
#[allow(unused_variables)]
pub fn update_hover(
    state: &mut SidebarState,
    hovered_rail: Option<RailId>,
    now: Instant,
    screen_width: f32,
) {
    todo!(
        "P1: 校验 hover_rail 切换时刻清零 hover_since；超过 180ms 后切换 mode 到 Flyout 或生成 Preview"
    )
}

/// Rail 点击触发的模式切换。
#[allow(unused_variables)]
pub fn rail_click(state: &mut SidebarState, rail: RailId, screen_width: f32) {
    todo!(
        "P1: Hidden 点击 -> Pinned(Wide) / Flyout(Medium) / Modal(Narrow)；Pinned(r) 点击 r -> Hidden；Pinned(a) 点击 b -> 切 pinned_rail=b（覆盖预览态）"
    )
}

/// 渲染分发：根据 state.mode 调度 Pinned Panel / Flyout Area / Modal Area+scrim。
#[allow(unused_variables)]
pub fn render(
    ui: &mut egui::Ui,
    state: &mut SidebarState,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    todo!(
        "P1+P2: 根据 mode 分发到 render_pinned_panel / render_flyout_area / render_modal / 不渲染"
    )
}

/// 自动应用响应式默认（每帧调用，跨阈值仅在 auto 状态下改模式）。
#[allow(unused_variables)]
pub fn apply_responsive_default(state: &mut SidebarState, screen_width: f32) {
    if state.is_user_pinned {
        return;
    }
    let new_default = responsive_default(state.pinned_rail, screen_width);
    let _ = new_default;
    todo!("P2: 检测当前 mode 与 new_default 是否冲突；只在 auto 模式下重置")
}

/// 输入 Event 处理（Modal 的 Esc / scrim 点击关闭）。
#[allow(unused_variables)]
pub fn handle_input(ctx: &egui::Context, state: &mut SidebarState) {
    todo!("P2: Esc -> Hidden；scrim 区域 interact click -> Hidden")
}

//! 通用工具函数：颜色/圆角插值、Hover 防抖等。
//!
//! 本模块集中放置 M3 组件之间共享的数学/动画工具，方便移植到 HoshinekoFM。

use egui::CornerRadius;

/// 在两个 CornerRadius 之间做线性插值（逐分量 round）。
pub fn lerp_corner_radius(a: CornerRadius, b: CornerRadius, t: f32) -> CornerRadius {
    fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
        (a as f32 + (b as f32 - a as f32) * t).round() as u8
    }
    CornerRadius {
        nw: lerp_u8(a.nw, b.nw, t),
        ne: lerp_u8(a.ne, b.ne, t),
        sw: lerp_u8(a.sw, b.sw, t),
        se: lerp_u8(a.se, b.se, t),
    }
}

/// f32 线性插值。
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
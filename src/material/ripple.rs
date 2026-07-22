//! M3 涟漪（Ripple）水波纹回调占位。
//!
//! P3+ 阶段补齐 per-pointer 涟漪动画，当前为 todo 占位。

use egui::{CornerRadius, Vec2};
use egui_wgpu::CallbackTrait;

/// P3+ 启用：per-pointer 涟漪 wgpu 渲染。
#[allow(dead_code)]
pub struct RippleCallback {
    size: Vec2,
    radius: CornerRadius,
    pointers: Vec<(Vec2, f32)>,
}

pub struct Ripple {}

impl RippleCallback {
    #[allow(unused)]
    pub fn new(size: Vec2, radius: CornerRadius, pointers: Vec<(Vec2, f32)>) -> Self {
        Self {
            size,
            radius,
            pointers,
        }
    }
}

impl CallbackTrait for RippleCallback {
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut eframe::wgpu::RenderPass<'static>,
        _callback_resources: &egui_wgpu::CallbackResources,
    ) {
        todo!("P3+: 涟漪 wgpu 渲染，按 pointers 展开水波")
    }
}

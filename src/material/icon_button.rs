//! M3 IconButton 组件（toggle 变体）。
//!
//! 规范：container 圆形、state layer hover/press 反馈、toggle 状态颜色切换。
//! 图标统一用 circle_stroke / circle_filled 圆圈占位（与 NavRailItem/ListItem 风格一致）。
//! 参考 https://m3.material.io/components/icon-button/specs

use egui::{Color32, Id, Sense, Stroke, Vec2, emath::easing};

use crate::material;

/// M3 IconButton（toggle 型）。
///
/// 绘制半径 = self.container_size / 2 的圆形容器，hover/press 时叠加 state layer，
/// selected 时填充 secondary_container。居中圆圈占位图标：selected 实心，否则空心。
///
/// 用法：
/// ```ignore
/// if IconButton::new("pin", pinned).size(24.0).ui(ui).clicked() { pinned = !pinned; }
/// ```
pub struct IconButton<'a> {
    key: &'a str,
    selected: bool,
    container_size: f32,
}

impl<'a> IconButton<'a> {
    pub fn new(key: &'a str, selected: bool) -> Self {
        Self {
            key,
            selected,
            container_size: 40.0,
        }
    }

    /// 覆盖容器尺寸（单位 dp，`Painter::circle_*` 尺度适配）。
    pub fn size(mut self, s: f32) -> Self {
        self.container_size = s;
        self
    }

    /// 渲染 IconButton 并返回点击响应。
    ///
    /// 占用 parent_width × container_size 的最小矩形区域，
    /// 居中绘制圆形控件。
    pub fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let parent_width = ui.available_width();
        let desired = Vec2::new(parent_width, self.container_size);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
        let center = rect.center();
        let radius = self.container_size / 2.0;

        let active_anim = {
            let anim_id = Id::new(self.key).with("icon-btn-active");
            ui.animate_bool_with_time_and_easing(anim_id, self.selected, 0.2, easing::quadratic_out)
        };

        let hov = response.hovered();
        let hod = response.is_pointer_button_down_on();
        let hover_anim = {
            let anim_id = Id::new(self.key).with("icon-btn-hover");
            ui.animate_bool_with_time_and_easing(anim_id, hov, 0.2, easing::quadratic_out)
        };

        let layer_alpha = 0.08;

        let (on_surface, on_surface_variant, secondary_container) =
            material::color::access(|_p, s| {
                (s.on_surface, s.on_surface_variant, s.secondary_container)
            });

        let container_fill: Color32 = secondary_container.with_alpha_f32(active_anim);

        let painter = ui.painter();

        if container_fill.a() > 0 {
            painter.circle_filled(center, radius, container_fill);
        }

        let state_alpha = hover_anim * layer_alpha + if hod { layer_alpha } else { 0.0 };
        if state_alpha > 0.0 {
            painter.circle_filled(center, radius, on_surface.with_alpha_f32(state_alpha));
        }

        let icon_color: Color32 = {
            let c: Color32 = on_surface_variant.into();
            let target: Color32 = on_surface.into();
            c.lerp_to_gamma(target, active_anim.max(hover_anim))
        };
        let icon_radius = radius * 0.4;

        if self.selected {
            painter.circle_filled(center, icon_radius, icon_color);
        } else {
            painter.circle_stroke(center, icon_radius, Stroke::new(1.0, icon_color));
        }

        response
    }
}

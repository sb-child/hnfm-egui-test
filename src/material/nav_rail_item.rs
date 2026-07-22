//! M3 NavigationRail 单项组件。
//!
//! 参考 https://m3.material.io/components/navigation-rail/specs 与 devlog
//! 「NavigationRail 对齐 Compose M3 标准」，移植 HoshinekoFM 时整文件可用。

use egui::{
    Color32, CornerRadius, FontId, Id, Pos2, Rect, Sense, Stroke, TextFormat, Vec2,
    emath::easing,
    epaint::text::VariationCoords,
    text::{LayoutJob, TextWrapping},
};

use crate::fonts::ff_sans;
use crate::material;

pub struct NavRailItem<'a> {
    key: &'a str,
    label: &'a str,
    active: bool,
}

impl<'a> NavRailItem<'a> {
    pub fn new(key: &'a str, label: &'a str, active: bool) -> Self {
        Self { key, label, active }
    }
}

impl<'a> egui::Widget for NavRailItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let parent_width = ui.available_width();

        let active_anim = {
            let anim_id = Id::new(self.key).with("active");
            ui.animate_bool_with_time_and_easing(anim_id, self.active, 0.2, easing::quadratic_out)
        };

        let font_weight = 400. + 100. * active_anim;
        let label_font_id = FontId::new(12.0, ff_sans());

        let label_galley = {
            let mut job = LayoutJob::default();
            job.append(
                self.label,
                0.0,
                TextFormat {
                    font_id: label_font_id.clone(),
                    color: Color32::PLACEHOLDER,
                    line_height: Some(16.),
                    coords: VariationCoords::new([(b"wght", font_weight)]),
                    ..Default::default()
                },
            );
            job.halign = egui::Align::Center;
            job.wrap = TextWrapping {
                max_width: parent_width,
                max_rows: 2,
                overflow_character: None,
                break_anywhere: false,
            };
            ui.fonts_mut(|f: &mut egui::epaint::FontsView<'_>| f.layout_job(job))
        };

        let num_rows = label_galley.rows.len();
        let container_height = if num_rows <= 1 { 56. } else { 72. };

        let desired_size = Vec2::new(parent_width, container_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        let (
            on_surface_variant_color,
            secondary_container_color,
            on_surface_color,
            on_secondary_container_color,
        ) = material::color::access(|_p, s| {
            (
                s.on_surface_variant,
                s.secondary_container,
                s.on_surface,
                s.on_secondary_container,
            )
        });

        let layer_alpha = 0.08;

        let hov = response.hovered();
        let hod = response.is_pointer_button_down_on();

        let hover_anim = {
            let anim_id = Id::new(self.key).with("hover");
            ui.animate_bool_with_time_and_easing(anim_id, hov, 0.2, easing::quadratic_out)
        };

        let calculated_indicator_color = secondary_container_color.with_alpha_f32(active_anim);

        let calculated_indicator_overlay_color = {
            let overlay_alpha = hover_anim * layer_alpha + if hod { layer_alpha } else { 0. };
            on_surface_color.with_alpha_f32(overlay_alpha)
        };

        let calculated_label_color = {
            let t = active_anim.max(hover_anim);
            let c: Color32 = on_surface_variant_color.into();
            let target: Color32 = on_surface_color.into();
            c.lerp_to_gamma(target, t)
        };

        let calculated_icon_color = {
            let c: Color32 = on_surface_color.into();
            let target: Color32 = on_secondary_container_color.into();
            c.lerp_to_gamma(target, active_anim)
        };

        let label_text_anchor = Pos2::new(rect.center().x, rect.top() + 40.);
        let painter = ui.painter();
        let icon_center = Pos2::new(rect.center().x, rect.top() + 20.);
        let indicator_width = {
            let piece = 56. / 3.;
            let anim_part = piece * active_anim;
            anim_part + piece * 2.
        };
        let indicator_rect = Rect::from_center_size(icon_center, Vec2::new(indicator_width, 32.));
        let indicator_overlay_rect = Rect::from_center_size(icon_center, Vec2::new(56., 32.));
        let indicator_radius: CornerRadius = CornerRadius::same(32 / 2);

        painter.rect_filled(indicator_rect, indicator_radius, calculated_indicator_color);

        painter.rect_filled(
            indicator_overlay_rect,
            indicator_radius,
            calculated_indicator_overlay_color,
        );
        painter.circle_stroke(
            icon_center,
            24. / 2.,
            Stroke::new(1., calculated_icon_color),
        );
        painter.galley(label_text_anchor, label_galley, calculated_label_color);
        response
    }
}

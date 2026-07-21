//! M3 ListItem 组件：standard / segmented 两种模式，支持 overline/headline/supporting
//! 三行文本、颜色动画、圆角动画。
//!
//! 参考实现见 devlog「ListItem 组件实现」。移植 HoshinekoFM 时整文件可用。

use egui::{
    Color32, CornerRadius, FontId, Id, Pos2, Sense, Stroke, TextFormat, Vec2,
    emath::easing, epaint::text::VariationCoords, text::{LayoutJob, TextWrapping},
};

use crate::material;
use crate::fonts::ff_sans;

pub struct ListItem<'a> {
    key: &'a str,
    headline: &'a str,
    supporting: Option<&'a str>,
    overline: Option<&'a str>,
    active: bool,
    segmented: bool,
    above: bool,
    below: bool,
}

impl<'a> ListItem<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: &'a str,
        headline: &'a str,
        supporting: Option<&'a str>,
        overline: Option<&'a str>,
        active: bool,
        segmented: bool,
        above: bool,
        below: bool,
    ) -> Self {
        Self {
            key,
            headline,
            supporting,
            overline,
            active,
            segmented,
            above,
            below,
        }
    }
}

impl<'a> egui::Widget for ListItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let parent_width = ui.available_width();

        let active_anim = {
            let anim_id = Id::new(self.key).with("active");
            ui.animate_bool_with_time_and_easing(anim_id, self.active, 0.2, easing::quadratic_out)
        };

        let text_max_width = parent_width - 16.0 - 24.0 - 16.0 - 16.0;

        let layout_text = |ui: &egui::Ui,
                           text: &str,
                           font_size: f32,
                           weight: f32,
                           line_height: f32,
                           max_rows: usize| {
            let mut job = LayoutJob::default();
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: FontId::new(font_size, ff_sans()),
                    color: Color32::PLACEHOLDER,
                    line_height: Some(line_height),
                    coords: VariationCoords::new([(b"wght", weight)]),
                    ..Default::default()
                },
            );
            job.wrap = TextWrapping {
                max_width: text_max_width,
                max_rows,
                overflow_character: None,
                break_anywhere: false,
            };
            ui.fonts_mut(|f| f.layout_job(job))
        };

        let headline_galley = layout_text(ui, self.headline, 16.0, 400.0, 24.0, 1);
        let supporting_galley = self
            .supporting
            .map(|t| layout_text(ui, t, 14.0, 400.0, 20.0, 2));
        let overline_galley = self
            .overline
            .map(|t| layout_text(ui, t, 11.0, 500.0, 16.0, 1));

        let has_overline = self.overline.is_some();
        let has_supporting = self.supporting.is_some();

        let headline_h = headline_galley.size().y;
        let supporting_h = supporting_galley.as_ref().map_or(0.0, |g| g.size().y);
        let overline_h = overline_galley.as_ref().map_or(0.0, |g| g.size().y);
        let text_block_height = overline_h + headline_h + supporting_h;

        let container_height = (text_block_height + 20.0).max(56.0);

        let desired_size = Vec2::new(parent_width, container_height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        let (
            surface,
            surface_container,
            secondary_container,
            on_surface,
            on_surface_variant,
            on_secondary_container,
        ) = material::color::access(|_p, s| {
            (
                s.surface,
                s.surface_container,
                s.secondary_container,
                s.on_surface,
                s.on_surface_variant,
                s.on_secondary_container,
            )
        });

        let hov = response.hovered();
        let hod = response.is_pointer_button_down_on();

        let hover_anim = {
            let anim_id = Id::new(self.key).with("hover");
            ui.animate_bool_with_time_and_easing(anim_id, hov, 0.2, easing::quadratic_out)
        };

        let container_fill: Color32 = {
            let default_fill: Color32 = if self.segmented {
                surface_container.into()
            } else {
                surface.into()
            };
            let target: Color32 = secondary_container.into();
            default_fill.lerp_to_gamma(target, active_anim)
        };

        let state_layer = {
            let layer_alpha = 0.08;
            let alpha = hover_anim * layer_alpha + if hod { layer_alpha } else { 0.0 };
            on_surface.with_alpha_f32(alpha)
        };

        let headline_color = {
            let c: Color32 = on_surface.into();
            let target: Color32 = on_secondary_container.into();
            c.lerp_to_gamma(target, active_anim.max(hover_anim))
        };

        let sub_text_color = {
            let c: Color32 = on_surface_variant.into();
            let target: Color32 = on_secondary_container.into();
            c.lerp_to_gamma(target, active_anim)
        };

        let icon_color = {
            let c: Color32 = on_surface_variant.into();
            let target: Color32 = on_secondary_container.into();
            c.lerp_to_gamma(target, active_anim)
        };

        let interaction_r =
            (4.0 + 8.0 * hover_anim.max(active_anim) + 4.0 * active_anim).round() as u8;

        let top_big = !self.above;
        let bot_big = !self.below;

        let top_anim = ui.animate_bool_with_time_and_easing(
            Id::new(self.key).with("top-big"),
            top_big,
            0.2,
            easing::quadratic_out,
        );
        let bot_anim = ui.animate_bool_with_time_and_easing(
            Id::new(self.key).with("bot-big"),
            bot_big,
            0.2,
            easing::quadratic_out,
        );

        let top_r = (4.0 + 8.0 * top_anim).round() as u8;
        let bot_r = (4.0 + 8.0 * bot_anim).round() as u8;
        let seg_corner = CornerRadius {
            nw: top_r,
            ne: top_r,
            sw: bot_r,
            se: bot_r,
        };

        let use_segmented = self.segmented && !self.active && !hov;
        let seg_mode_anim = ui.animate_bool_with_time_and_easing(
            Id::new(self.key).with("seg-shape"),
            use_segmented,
            0.2,
            easing::quadratic_out,
        );

        let interaction_corner = CornerRadius::same(interaction_r);
        let corner_radius =
            material::util::lerp_corner_radius(interaction_corner, seg_corner, seg_mode_anim);

        let painter = ui.painter();
        painter.rect_filled(rect, corner_radius, container_fill);
        if state_layer.a() > 0 {
            painter.rect_filled(rect, corner_radius, state_layer);
        }

        let icon_center = Pos2::new(rect.left() + 16.0 + 12.0, rect.center().y);
        painter.circle_stroke(icon_center, 24.0 / 2.0, Stroke::new(1.0, icon_color));

        let text_left = rect.left() + 16.0 + 24.0 + 16.0;
        let headline_h = headline_galley.size().y;
        let overline_h = overline_galley.as_ref().map_or(0.0, |g| g.size().y);
        let supporting_h = supporting_galley.as_ref().map_or(0.0, |g| g.size().y);
        let text_block_height = overline_h + headline_h + supporting_h;

        let text_top = if has_supporting || has_overline {
            rect.top() + 10.0
        } else {
            rect.center().y - text_block_height / 2.0
        };

        let mut y = text_top;
        if let Some(ref galley) = overline_galley {
            painter.galley(Pos2::new(text_left, y), galley.clone(), sub_text_color);
            y += galley.size().y;
        }
        painter.galley(
            Pos2::new(text_left, y),
            headline_galley.clone(),
            headline_color,
        );
        y += headline_galley.size().y;
        if let Some(ref galley) = supporting_galley {
            painter.galley(Pos2::new(text_left, y), galley.clone(), sub_text_color);
        }

        response
    }
}
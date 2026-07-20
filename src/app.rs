use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontFamily, FontId, Id, Mesh, PaintCallback, Pos2, Rect, RichText,
    Sense, Shape, Stroke, TextFormat, Vec2, Widget,
    emath::easing,
    epaint::text::VariationCoords,
    text::{LayoutJob, TextWrapping},
};
use egui_wgpu::CallbackTrait;

use crate::{fonts::ff_serif, material::color::ThemeVariant};
use crate::{
    fonts::{self, ff_sans},
    material,
};

pub struct AppLayout {
    terminal_expanded: bool,
    active_1: bool,
    active_1_before: bool,
    active_2: bool,
    active_2_before: bool,
    active_3: bool,
    active_opt: u8,
    list_sel_std: bool,
    list_sel_seg_0: bool,
    list_sel_seg_1: bool,
    list_sel_seg_2: bool,
}

impl AppLayout {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::add_fonts(&cc.egui_ctx);
        fonts::configure_text_styles(&cc.egui_ctx, fonts::TextStyleOpt::Sans);
        let theme = material::color::generate_theme(ThemeVariant::Cmf {
            primary: 0xff769CDF,
            secondary: Some(0xff8991A2),
        });
        material::color::set_global_scheme(theme);
        material::color::set_global_theme_mode(material::color::ThemeMode::Dark);
        Self {
            terminal_expanded: true,
            active_1: false,
            active_1_before: false,
            active_2: false,
            active_2_before: false,
            active_3: false,
            active_opt: 0,
            list_sel_std: false,
            list_sel_seg_0: false,
            list_sel_seg_1: false,
            list_sel_seg_2: false,
        }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let surface_color = material::color::access(|_p, s| s.surface).into();
        egui::Panel::bottom("bottom-statusbar")
            .resizable(false)
            .show(ui, bottom_statusbar);
        let surface_frame = egui::containers::Frame {
            inner_margin: egui::epaint::Margin::symmetric(0, 4),
            outer_margin: egui::epaint::Margin::same(0),
            corner_radius: egui::CornerRadius::ZERO,
            shadow: eframe::epaint::Shadow::NONE,
            fill: surface_color,
            stroke: Stroke::NONE,
        };
        egui::Panel::left("navigation-rail")
            .frame(surface_frame)
            .resizable(false)
            .show_separator_line(false)
            .show(ui, |ui| {
                nav_rail(
                    ui,
                    &mut self.active_1,
                    &mut self.active_2,
                    &mut self.active_3,
                    &mut self.active_opt,
                )
            });
        egui::Panel::left("sidebar")
            .frame(surface_frame)
            .resizable(false)
            .show_separator_line(false)
            .show(ui, |ui| {
                sidebar(
                    ui,
                    &mut self.list_sel_std,
                    &mut self.list_sel_seg_0,
                    &mut self.list_sel_seg_1,
                    &mut self.list_sel_seg_2,
                )
            });
        egui::Panel::top("tabs").resizable(false).show(ui, tabs);
        egui::Panel::bottom("terminal-tab")
            .resizable(true)
            .default_size(200.0)
            .size_range(60.0..=600.0)
            .show_collapsible(ui, &mut self.terminal_expanded, terminal);
        egui::CentralPanel::default().show(ui, content);
        if self.active_1 != self.active_1_before {
            let new_theme_mode = if self.active_1 {
                material::color::ThemeMode::Light
            } else {
                material::color::ThemeMode::Dark
            };
            material::color::set_global_theme_mode(new_theme_mode);
            self.active_1_before = self.active_1;
        }
        if self.active_2 != self.active_2_before {
            let new_theme = if self.active_2 {
                material::color::generate_theme(ThemeVariant::Cmf {
                    primary: 0xffB33B15,
                    secondary: Some(0xffB88576),
                })
            } else {
                material::color::generate_theme(ThemeVariant::Cmf {
                    primary: 0xff769CDF,
                    secondary: Some(0xff8991A2),
                })
            };
            material::color::set_global_scheme(new_theme);
            self.active_2_before = self.active_2;
        }
    }
}

fn bottom_statusbar(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("bottom_statusbar");
        if ui.button("test").clicked() {}
    });
}

fn nav_rail(
    ui: &mut egui::Ui,
    active_1: &mut bool,
    active_2: &mut bool,
    active_3: &mut bool,
    active_opt: &mut u8,
) {
    // https://m3.material.io/components/navigation-rail/specs
    ui.set_width(96.); // Nav rail collapsed container width = 96 dp

    ui.vertical(|ui| {
        ui.add_space(44.);
        // ui.style_mut().spacing.item_spacing = Vec2::new(0., 0.);
        ui.style_mut().spacing.item_spacing = Vec2::new(0., 4.);
        // ui.style_mut().spacing.indent = 0.;
        // ui.button("1111");
        if NavRailItem::new("theme_toggle", "白天/晚上模式", *active_1)
            .ui(ui)
            .clicked()
        {
            *active_1 = !*active_1;
        };
        if NavRailItem::new("theme_switch", "主题，切换", *active_2)
            .ui(ui)
            .clicked()
        {
            *active_2 = !*active_2;
        };
        if NavRailItem::new("toggle_test", "toggle 啊啊啊啊啊啊啊啊啊啊", *active_3)
            .ui(ui)
            .clicked()
        {
            *active_3 = !*active_3;
        };
        if NavRailItem::new("opt_0", &format!("opt={}", active_opt), *active_opt == 0)
            .ui(ui)
            .clicked()
        {
            *active_opt = 0;
        };
        if NavRailItem::new("opt_1", "opt2", *active_opt == 1)
            .ui(ui)
            .clicked()
        {
            *active_opt = 1;
        };
        if NavRailItem::new("opt_2", "opt3", *active_opt == 2)
            .ui(ui)
            .clicked()
        {
            *active_opt = 2;
        };
    });
}

struct NavRailItem<'a> {
    key: &'a str,
    label: &'a str,
    active: bool,
}

impl<'a> NavRailItem<'a> {
    fn new(key: &'a str, label: &'a str, active: bool) -> Self {
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
        let pos = response.interact_pointer_pos();

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
        // painter.rect_stroke(
        //     rect,
        //     CornerRadius::ZERO,
        //     Stroke::new(1., calculated_icon_color),
        //     egui::StrokeKind::Inside,
        // ); // debug
        painter.circle_stroke(
            icon_center,
            24. / 2.,
            Stroke::new(1., calculated_icon_color),
        );
        painter.galley(label_text_anchor, label_galley, calculated_label_color);
        response
    }
}

// pub struct SurfaceCallback {
//     size: Vec2,
//     radius: CornerRadius,
//     pointers: Vec<(Vec2, f32)>,
// }

// impl SurfaceCallback {
//     pub fn new(rect: Rect, radius: CornerRadius, pointers: Vec<(Vec2, f32)>) -> Self {
//         Self {
//             size,
//             radius,
//             pointers,
//         }
//     }
// }

// impl CallbackTrait for SurfaceCallback {
//     fn paint(
//         &self,
//         info: egui::PaintCallbackInfo,
//         render_pass: &mut eframe::wgpu::RenderPass<'static>,
//         callback_resources: &egui_wgpu::CallbackResources,
//     ) {
//         todo!()
//     }
// }

pub struct RippleCallback {
    size: Vec2,
    radius: CornerRadius,
    pointers: Vec<(Vec2, f32)>,
}

pub struct Ripple {}

impl RippleCallback {
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
        info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
        todo!()
    }
}

fn lerp_corner_radius(a: CornerRadius, b: CornerRadius, t: f32) -> CornerRadius {
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

struct ListItem<'a> {
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
    fn new(
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
        let corner_radius = lerp_corner_radius(interaction_corner, seg_corner, seg_mode_anim);

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

fn sidebar(
    ui: &mut egui::Ui,
    list_sel_std: &mut bool,
    list_sel_seg_0: &mut bool,
    list_sel_seg_1: &mut bool,
    list_sel_seg_2: &mut bool,
) {
    ui.set_width(300.);
    ui.style_mut().spacing.item_spacing = Vec2::new(0.0, 0.0);

    ui.heading("Standard (单选)");
    ui.add_space(4.);
    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = Vec2::new(0., 2.);
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

    let sego_above_0 = false;
    let sego_below_0 = !seg1;
    let sego_above_1 = !seg0;
    let sego_below_1 = !seg2;
    let sego_above_2 = !seg1;
    let sego_below_2 = false;
    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = Vec2::new(0., 2.);
        if ListItem::new(
            "seg_0",
            "叫我起床",
            None,
            None,
            seg0,
            true,
            sego_above_0,
            sego_below_0,
        )
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
            sego_above_1,
            sego_below_1,
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
            sego_above_2,
            sego_below_2,
        )
        .ui(ui)
        .clicked()
        {
            *list_sel_seg_2 = !*list_sel_seg_2;
        }
    });
}

fn tabs(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("标签页");
        if ui.button("test").clicked() {}
    });
}

fn terminal(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("终端 terminal");
        if ui.button("test").clicked() {}
    });
}

fn content(ui: &mut egui::Ui) {
    let t = "egui 支持可变字体，可以通过 VariationCoords 控制粗细等属性。查看示例 font_variations";
    ui.heading("Top Heading");

    ui.add_space(15.);
    ui.label(t);

    ui.add_space(15.);
    ui.label(RichText::new(t).text_style(fonts::ts_body_sans()));
    ui.label(RichText::new(t).text_style(fonts::ts_body_sans()).strong());
    for i in [300., 500., 800., 1000.] {
        ui.label(
            RichText::new(t)
                .text_style(fonts::ts_body_sans())
                .variation("wght", i),
        );
    }

    ui.label(RichText::new(t).text_style(fonts::ts_body_sans()).italics());

    ui.add_space(15.);
    ui.monospace(t);

    ui.add_space(15.);
    ui.label(RichText::new(t).text_style(fonts::ts_body_serif()).strong());
    for i in [300., 500., 800., 1000.] {
        ui.label(
            RichText::new(t)
                .text_style(fonts::ts_body_serif())
                .variation("wght", i),
        );
    }

    // ui.add_space(15.);
    // ui.label(t);
}

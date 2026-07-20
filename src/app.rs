use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontId, Id, Mesh, PaintCallback, Pos2, Rect, RichText, Sense, Shape,
    Stroke, TextFormat, Vec2, Widget,
    emath::easing,
    epaint::text::VariationCoords,
    text::{LayoutJob, TextWrapping},
};
use egui_wgpu::CallbackTrait;

use crate::{fonts, material};

pub struct AppLayout {
    terminal_expanded: bool,
    active_1: bool,
    active_1_before: bool,
    active_2: bool,
    active_2_before: bool,
    active_3: bool,
    active_opt: u8,
}

impl AppLayout {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::add_fonts(&cc.egui_ctx);
        fonts::configure_text_styles(&cc.egui_ctx, fonts::TextStyleOpt::Sans);
        let theme = material::color::generate_theme(0xff769CDF, Some(0xff8991A2));
        material::color::set_global_scheme(theme);
        material::color::set_global_theme_mode(material::color::ThemeMode::DarkMediumContrast);
        Self {
            terminal_expanded: true,
            active_1: false,
            active_1_before: false,
            active_2: false,
            active_2_before: false,
            active_3: false,
            active_opt: 0,
        }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let surface_color = material::color::access(|_p, s| s.surface).into();
        egui::Panel::bottom("bottom-statusbar")
            .resizable(false)
            .show(ui, bottom_statusbar);
        let my_frame = egui::containers::Frame {
            inner_margin: egui::epaint::Margin::symmetric(0, 4),
            outer_margin: egui::epaint::Margin::same(0),
            corner_radius: egui::CornerRadius::ZERO,
            shadow: eframe::epaint::Shadow::NONE,
            fill: surface_color,
            stroke: Stroke::NONE,
        };
        egui::Panel::left("navigation-rail")
            .frame(my_frame)
            .resizable(false)
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
            .resizable(false)
            .show(ui, sidebar);
        egui::Panel::top("tabs").resizable(false).show(ui, tabs);
        egui::Panel::bottom("terminal-tab")
            .resizable(true)
            .default_size(200.0)
            .size_range(60.0..=600.0)
            .show_collapsible(ui, &mut self.terminal_expanded, terminal);
        egui::CentralPanel::default().show(ui, content);
        if self.active_1 != self.active_1_before {
            let new_theme_mode = if self.active_1 {
                material::color::ThemeMode::LightMediumContrast
            } else {
                material::color::ThemeMode::DarkMediumContrast
            };
            material::color::set_global_theme_mode(new_theme_mode);
            self.active_1_before = self.active_1;
        }
        if self.active_2 != self.active_2_before {
            let new_theme = if self.active_2 {
                material::color::generate_theme(0xffB33B15, Some(0xffB88576))
            } else {
                material::color::generate_theme(0xff769CDF, Some(0xff8991A2))
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
        Self {
            key,
            label,
            active,
        }
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
        let label_font_id = FontId::proportional(12.0);

        let layout_label = |ui: &mut egui::Ui, color: Color32| {
            let mut job = LayoutJob::default();
            job.append(
                self.label,
                0.0,
                TextFormat {
                    font_id: label_font_id.clone(),
                    color,
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

        let label_galley_pre = layout_label(ui, Color32::BLACK);
        let num_rows = label_galley_pre.rows.len();
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

        let layer_alpha = 20u8;

        let hov = response.hovered();
        let hod = response.is_pointer_button_down_on();
        let pos = response.interact_pointer_pos();

        let calculated_indicator_color = secondary_container_color.with_alpha_f32(active_anim);

        let calculated_indicator_overlay_color = {
            let base_color = Color32::TRANSPARENT;
            let mix_hover = if hov {
                let layer = on_surface_color.with_alpha_u8(layer_alpha);
                base_color.blend(layer)
            } else {
                base_color
            };
            let mix_hold = if hod {
                let layer = on_surface_color.with_alpha_u8(layer_alpha);
                mix_hover.blend(layer)
            } else {
                mix_hover
            };
            mix_hold
        };

        let calculated_label_color = {
            let c: Color32 = if self.active || hov {
                on_surface_color
            } else {
                on_surface_variant_color
            }
            .into();
            c
        };

        let calculated_icon_color = {
            let c: Color32 = if self.active {
                on_secondary_container_color
            } else {
                on_surface_color
            }
            .into();
            c
        };

        let label_galley = layout_label(ui, calculated_label_color);
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

fn sidebar(ui: &mut egui::Ui) {
    ui.set_width(200.);
    ui.vertical(|ui| {
        ui.heading("侧边栏");
        if ui.button("test").clicked() {}
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

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontId, Mesh, PaintCallback, Pos2, Rect, RichText, Sense, Shape, Stroke, TextFormat, Vec2, Widget, text::{LayoutJob, TextWrapping},
};
use egui_wgpu::CallbackTrait;

use crate::{fonts, material};

pub struct AppLayout {
    terminal_expanded: bool,
    active_1: bool,
    active_2: bool,
    active_3: bool,
    active_4: bool,
    active_5: bool,
    active_6: bool,
}

impl AppLayout {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::add_fonts(&cc.egui_ctx);
        fonts::configure_text_styles(&cc.egui_ctx, fonts::TextStyleOpt::Sans);
        material::color::set_global_scheme();
        material::color::set_global_theme_mode(material::color::ThemeMode::DarkMediumContrast);
        Self {
            terminal_expanded: true,
            active_1: false,
            active_2: false,
            active_3: false,
            active_4: false,
            active_5: false,
            active_6: false,
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
            inner_margin: egui::epaint::Margin::same(0),
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
                    &mut self.active_4,
                    &mut self.active_5,
                    &mut self.active_6,
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
    active_4: &mut bool,
    active_5: &mut bool,
    active_6: &mut bool,
) {
    // https://m3.material.io/components/navigation-rail/specs
    ui.set_width(96.); // Nav rail collapsed container width = 96 dp

    ui.vertical(|ui| {
        ui.add_space(40.);
        // ui.style_mut().spacing.item_spacing = Vec2::new(0., 0.);
        ui.style_mut().spacing.item_spacing = Vec2::new(0., 12.);
        // ui.style_mut().spacing.indent = 0.;
        // ui.button("1111");
        if NavRailItem::new("你的家", *active_1).ui(ui).clicked() {
            *active_1 = !*active_1;
        };
        if NavRailItem::new("你的家", *active_2).ui(ui).clicked() {
            *active_2 = !*active_2;
        };
        if NavRailItem::new("啊啊啊啊啊啊啊啊啊啊", *active_3)
            .ui(ui)
            .clicked()
        {
            *active_3 = !*active_3;
        };
        if NavRailItem::new("你的家", *active_4).ui(ui).clicked() {
            *active_4 = !*active_4;
        };
        if NavRailItem::new("111111111111111111", *active_5)
            .ui(ui)
            .clicked()
        {
            *active_5 = !*active_5;
        };
        if NavRailItem::new("你的家", *active_6).ui(ui).clicked() {
            *active_6 = !*active_6;
        };
    });
}

struct NavRailItem<'a> {
    label: &'a str,
    active: bool,
}

impl<'a> NavRailItem<'a> {
    fn new(label: &'a str, active: bool) -> Self {
        Self { label, active }
    }
}

impl<'a> egui::Widget for NavRailItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Nav rail item icon size = 24dp
        // Nav rail item active indicator leading/trailing space = 16dp
        // Nav rail item vertical icon label space = 4dp
        // Nav rail item vertical active indicator width = 56dp
        // Nav rail item vertical active indicator height = 32dp
        // |4dp
        // (---16dp [24dp icon] 16dp---) (|32dp)
        // |4dp
        // text ... (|16dp)
        // Nav rail item vertical label text = weight 500, 12pt, line height 16pt
        // Nav rail item container height = 64dp
        // Nav rail item short container height = 56dp (4+32+4+16)
        let parent_width = ui.available_width();
        let desired_size = Vec2::new(parent_width, 56.);
        // Sense clicks and hover
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        // let bg_color = Color32::from_rgb(29, 78, 216);

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

        // md.sys.state.hover.state-layer-opacity = 0.08
        let layer_alpha = 20u8;

        let hov = response.hovered();
        let hod = response.is_pointer_button_down_on();
        let pos = response.interact_pointer_pos();
        // let clk = response.contains_pointer();

        let calculated_indicator_color = {
            // 没激活就是透明的
            let base_color = if self.active {
                secondary_container_color.into()
            } else {
                Color32::TRANSPARENT
            };
            // hover 叠加层
            let mix_hover = if hov {
                let layer: Color32 = on_surface_color.into();
                let layer = layer.gamma_multiply_u8(layer_alpha);
                base_color.blend(layer)
            } else {
                base_color
            };
            // hold 叠加层
            let mix_hold = if hod {
                let layer: Color32 = on_surface_color.into();
                let layer = layer.gamma_multiply_u8(layer_alpha);
                mix_hover.blend(layer)
            } else {
                mix_hover
            };
            mix_hold
        };

        let calculated_label_color = {
            // 激活和hov是同一个颜色
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

        let mut label_job = LayoutJob::default();
        let label_font_id = FontId::proportional(12.0);
        label_job.append(
            self.label,
            0.0,
            TextFormat {
                font_id: label_font_id.clone(),
                color: calculated_label_color,
                line_height: Some(16.),
                ..Default::default()
            },
        );
        label_job.halign = egui::Align::Center;
        label_job.wrap = TextWrapping {
            max_width: parent_width,
            max_rows: 1,
            overflow_character: Some('…'),
            break_anywhere: true,
        };
        let label_galley =
            ui.fonts_mut(|f: &mut egui::epaint::FontsView<'_>| f.layout_job(label_job));
        let label_text_anchor = Pos2::new(rect.center().x, rect.bottom() - 16.);
        let painter = ui.painter();
        let indicator_start = Pos2::new(rect.center().x - (24. / 2.) - 16., rect.top() + 4.);
        // let indicator_start = rect.left_top() + Vec2::new(0., 4.);
        let indicator_end = Pos2::new(rect.center().x + (24. / 2.) + 16., rect.top() + 4. + 32.);
        // let indicator_end = rect.right_top() + Vec2::new(0., 4. + 32.);
        let indicator_rect = Rect::from_two_pos(indicator_start, indicator_end);
        let indicator_radius = CornerRadius::same(32 / 2);
        painter.rect_filled(indicator_rect, indicator_radius, calculated_indicator_color);
        let icon_center = Pos2::new(rect.center().x, rect.top() + 4. + (32. / 2.));

        //    let ripple =  RippleCallback::new(indicator_rect.size(), indicator_radius, vec![

        //    ]);
        //     let ripple_cb = egui_wgpu::Callback::new_paint_callback(indicator_rect, ());\
        // painter.add(Shape::Callback());

        painter.circle_stroke(
            icon_center,
            24. / 2.,
            Stroke::new(1., calculated_icon_color),
        );
        painter.galley(label_text_anchor, label_galley, calculated_label_color);
        // debug
        // painter.rect_stroke(
        //     rect,
        //     CornerRadius::ZERO,
        //     Stroke::new(1., Color32::RED),
        //     egui::StrokeKind::Middle,
        // );
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

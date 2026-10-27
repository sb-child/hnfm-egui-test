use egui::Stroke;

use crate::material::color::ThemeVariant;
use crate::{fonts, material};

pub struct AppLayout {
    demo_clicks: u32,
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
        Self { demo_clicks: 0 }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let surface_color = material::color::access(|_p, s| s.surface).into();
        let content_frame = egui::containers::Frame {
            inner_margin: egui::epaint::Margin::symmetric(
                material::tokens::SPACING_L as i8,
                material::tokens::SPACING_S as i8,
            ),
            outer_margin: egui::epaint::Margin::same(0),
            corner_radius: egui::CornerRadius::ZERO,
            shadow: eframe::epaint::Shadow::NONE,
            fill: surface_color,
            stroke: Stroke::NONE,
        };

        egui::CentralPanel::default()
            .frame(content_frame)
            .show(ui, |ui| content(ui, &mut self.demo_clicks));
    }
}

fn content(ui: &mut egui::Ui, demo_clicks: &mut u32) {
    material::Card::new()
        .title("M3 Button")
        .width(material::CardWidth::Fill)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if material::Button::new("btn_filled", "填充按钮")
                    .variant(material::ButtonVariant::Filled)
                    .icon("+")
                    .ui(ui)
                    .clicked()
                {
                    *demo_clicks += 1;
                }
                material::Button::new("btn_tonal", "色调按钮")
                    .variant(material::ButtonVariant::Tonal)
                    .ui(ui);
                material::Button::new("btn_outlined", "描边按钮")
                    .variant(material::ButtonVariant::Outlined)
                    .ui(ui);
                material::Button::new("btn_text", "文本按钮")
                    .variant(material::ButtonVariant::Text)
                    .ui(ui);
                material::Button::new("btn_disabled", "禁用")
                    .variant(material::ButtonVariant::Filled)
                    .enabled(false)
                    .ui(ui);
            });

            ui.horizontal_wrapped(|ui| {
                material::Button::new("btn_xs", "XS")
                    .size(material::ButtonSize::Xs)
                    .ui(ui);
                material::Button::new("btn_s", "S")
                    .size(material::ButtonSize::S)
                    .ui(ui);
                material::Button::new("btn_m", "M")
                    .size(material::ButtonSize::M)
                    .ui(ui);
                material::Button::new("btn_square", "方角")
                    .shape(material::ButtonShape::Square)
                    .ui(ui);
            });

            material::Button::new("btn_fill", "Fill 宽度")
                .variant(material::ButtonVariant::Tonal)
                .width(material::ButtonWidth::Fill)
                .ui(ui);

            ui.label(format!("Button clicks: {}", *demo_clicks));
        });
}

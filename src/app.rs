use egui::Stroke;

use crate::material::color::ThemeVariant;
use crate::material::components::{ButtonShape, ButtonSize, ButtonVariant, ButtonWidth, CardWidth};
use crate::{fonts, material};

pub struct AppLayout {
    theme: material::Theme,
    demo_clicks: u32,
}

impl AppLayout {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::add_fonts(&cc.egui_ctx);
        fonts::configure_text_styles(&cc.egui_ctx, fonts::TextStyleOpt::Sans);
        let color_theme = material::color::generate_theme(ThemeVariant::Cmf {
            primary: 0xff769CDF,
            secondary: Some(0xff8991A2),
        });
        let theme =
            material::Theme::from_color_theme(&color_theme, material::color::ThemeMode::Dark);
        Self {
            theme,
            demo_clicks: 0,
        }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let surface_color = self.theme.scheme().surface.into();
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

        let theme = &self.theme;
        let demo_clicks = &mut self.demo_clicks;
        egui::CentralPanel::default()
            .frame(content_frame)
            .show(ui, |ui| {
                let mut m = material::M3Ui::new(ui, theme, material::MotionPrefs::DEFAULT);
                content(&mut m, demo_clicks);
            });
    }
}

fn content(m: &mut material::M3Ui<'_>, demo_clicks: &mut u32) {
    m.card()
        .title("M3 Button")
        .width(CardWidth::Fill)
        .show(m, |m| {
            m.horizontal_wrapped(|m| {
                if m.button("btn_filled", "填充按钮")
                    .variant(ButtonVariant::Filled)
                    .icon("+")
                    .show(m)
                    .clicked()
                {
                    *demo_clicks += 1;
                }
                m.button("btn_tonal", "色调按钮")
                    .variant(ButtonVariant::Tonal)
                    .show(m);
                m.button("btn_outlined", "描边按钮")
                    .variant(ButtonVariant::Outlined)
                    .show(m);
                m.button("btn_text", "文本按钮")
                    .variant(ButtonVariant::Text)
                    .show(m);
                m.button("btn_disabled", "禁用")
                    .variant(ButtonVariant::Filled)
                    .enabled(false)
                    .show(m);
            });

            m.horizontal_wrapped(|m| {
                m.button("btn_xs", "XS").size(ButtonSize::Xs).show(m);
                m.button("btn_s", "S").size(ButtonSize::S).show(m);
                m.button("btn_m", "M").size(ButtonSize::M).show(m);
                m.button("btn_square", "方角")
                    .shape(ButtonShape::Square)
                    .show(m);
            });

            m.button("btn_fill", "Fill 宽度")
                .variant(ButtonVariant::Tonal)
                .width(ButtonWidth::Fill)
                .show(m);

            m.ui().label(format!("Button clicks: {}", *demo_clicks));
        });
}

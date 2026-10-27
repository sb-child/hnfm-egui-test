use egui::Stroke;

use crate::material::color::ThemeVariant;
use crate::material::components::{
    ButtonShape, ButtonSize, ButtonVariant, ButtonWidth, CardWidth, Icon, TextWrap, TooltipSide,
};
use crate::{fonts, icons, material};

pub struct AppLayout {
    theme: material::Theme,
    demo_clicks: u32,
}

impl AppLayout {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::add_fonts(&cc.egui_ctx);
        icons::add_icon_font(&cc.egui_ctx);
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
                let primary: egui::Color32 = m.theme().scheme().primary.into();
                let settings = m.icon(Icon::SETTINGS).show(m);
                m.tooltip("设置").show_for(&settings, m);
                m.icon(Icon::FAVORITE).filled(true).color(primary).show(m);
                m.icon(Icon::DELETE).size(16.0).show(m);
                m.icon(Icon::SAVE).size(32.0).weight(500.0).show(m);

                if m.button("btn_filled", "填充按钮")
                    .variant(ButtonVariant::Filled)
                    .icon(Icon::ADD)
                    .enable_scale_animation(true)
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
                m.button("btn_color", "自定义颜色")
                    .variant(ButtonVariant::Tonal)
                    .background_color(egui::Color32::from_rgb(98, 0, 238))
                    .text_color(egui::Color32::WHITE)
                    .tooltip("覆盖背景色与文字色")
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

            m.horizontal_wrapped(|m| {
                m.button("btn_wrap", "多行按钮文本，用来验证 wrapText 的高度")
                    .width(ButtonWidth::Fixed(160.0))
                    .text_wrap(TextWrap::MultiLine)
                    .enable_scale_animation(true)
                    .show(m);
                m.button("btn_reserve", "保存")
                    .reserve_text("保存中...")
                    .show(m);
                m.button("btn_busy", "保存中")
                    .variant(ButtonVariant::Tonal)
                    .busy(true)
                    .tooltip("正在写入磁盘，这一条 tooltip 显示在上方")
                    .tooltip_side(TooltipSide::Top)
                    .show(m);
                m.button("btn_max", "Hug 最大宽度限制")
                    .maximum_width(120.0)
                    .show(m);
            });

            m.button("btn_fill", "Fill 宽度")
                .variant(ButtonVariant::Tonal)
                .width(ButtonWidth::Fill)
                .show(m);

            m.horizontal_wrapped(|m| {
                m.spinner("spinner_24").size(24.0).show(m);
                m.spinner("spinner_40").size(40.0).show(m);
                m.spinner("spinner_static")
                    .size(40.0)
                    .running(false)
                    .show(m);
            });

            let clicks = m.ui().label(format!("Button clicks: {}", *demo_clicks));
            m.tooltip("这个计数会在点击按钮后增加").show_for(&clicks, m);
        });
}

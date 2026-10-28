use egui::Stroke;

use crate::material::color::{ThemeMode, ThemeVariant};
use crate::material::components::{
    ButtonShape, ButtonSize, ButtonVariant, ButtonWidth, CardWidth, Icon, TextWrap, TooltipSide,
};
use crate::{fonts, icons, material};

/// 两个 Cmf 主题色轮换。
const COLOR_VARIANTS: [(&str, ThemeVariant); 2] = [
    (
        "蓝紫",
        ThemeVariant::Cmf {
            primary: 0xff769cdf,
            secondary: Some(0xff8991a2),
        },
    ),
    (
        "青绿",
        ThemeVariant::Cmf {
            primary: 0xff4c9a7c,
            secondary: Some(0xff7fa593),
        },
    ),
];

/// `ThemeMode` 轮换顺序：浅色在前，对比度递增。
const THEME_MODES: [(&str, ThemeMode); 6] = [
    ("浅色", ThemeMode::Light),
    ("浅色中对比", ThemeMode::LightMediumContrast),
    ("浅色高对比", ThemeMode::LightHighContrast),
    ("深色", ThemeMode::Dark),
    ("深色中对比", ThemeMode::DarkMediumContrast),
    ("深色高对比", ThemeMode::DarkHighContrast),
];

pub struct AppLayout {
    /// 按 [`COLOR_VARIANTS`] 预生成，切换时只重建 [`material::Theme`]。
    color_themes: [material::MaterialColorTheme; 2],
    variant_index: usize,
    mode_index: usize,
    demo_clicks: u32,
}

impl AppLayout {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::add_fonts(&cc.egui_ctx);
        icons::add_icon_font(&cc.egui_ctx);
        fonts::configure_text_styles(&cc.egui_ctx, fonts::TextStyleOpt::Sans);
        Self {
            color_themes: COLOR_VARIANTS
                .map(|(_, variant)| material::color::generate_theme(variant)),
            variant_index: 0,
            // 默认深色，和之前的 demo 一致。
            mode_index: 3,
            demo_clicks: 0,
        }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mode = THEME_MODES[self.mode_index].1;
        let theme = material::Theme::from_color_theme(&self.color_themes[self.variant_index], mode);
        let surface_color = theme.scheme().surface.into();
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

        let theme = &theme;
        let demo_clicks = &mut self.demo_clicks;
        let variant_index = &mut self.variant_index;
        let mode_index = &mut self.mode_index;
        egui::CentralPanel::default()
            .frame(content_frame)
            .show(ui, |ui| {
                let mut m = material::M3Ui::new(ui, theme, material::MotionPrefs::DEFAULT);
                content(&mut m, demo_clicks, variant_index, mode_index);
            });
    }
}

/// 轮换主题色和明暗模式。
///
/// 点击后只改索引，[`AppLayout::ui`] 在下一帧重建主题，所以背景色会晚一帧。
fn theme_controls(m: &mut material::M3Ui<'_>, variant_index: &mut usize, mode_index: &mut usize) {
    m.horizontal_wrapped(|m| {
        let color_label = format!("主题色: {}", COLOR_VARIANTS[*variant_index].0);
        if m.button("btn_theme_color", &color_label)
            .variant(ButtonVariant::Tonal)
            .icon(Icon::PALETTE)
            .show(m)
            .clicked()
        {
            *variant_index = (*variant_index + 1) % COLOR_VARIANTS.len();
        }

        let mode_label = format!("模式: {}", THEME_MODES[*mode_index].0);
        if m.button("btn_theme_mode", &mode_label)
            .variant(ButtonVariant::Tonal)
            .icon(Icon::CONTRAST)
            .show(m)
            .clicked()
        {
            *mode_index = (*mode_index + 1) % THEME_MODES.len();
        }
    });
}

fn content(
    m: &mut material::M3Ui<'_>,
    demo_clicks: &mut u32,
    variant_index: &mut usize,
    mode_index: &mut usize,
) {
    theme_controls(m, variant_index, mode_index);

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

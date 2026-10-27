use egui::{RichText, Stroke, Vec2, Widget};

use crate::material::color::ThemeVariant;
use crate::{
    fonts, material,
    sidebar::{self, RailId, SidebarState},
};

pub struct AppLayout {
    terminal_expanded: bool,
    active_1: bool,
    active_1_before: bool,
    active_2: bool,
    active_2_before: bool,
    sidebar_state: SidebarState,
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
        Self {
            terminal_expanded: true,
            active_1: false,
            active_1_before: false,
            active_2: false,
            active_2_before: false,
            sidebar_state: SidebarState::new(Some(RailId::Files)),
            demo_clicks: 0,
        }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let surface_color = material::color::access(|_p, s| s.surface).into();
        let surface_frame = egui::containers::Frame {
            inner_margin: egui::epaint::Margin::symmetric(0, 4),
            outer_margin: egui::epaint::Margin::same(0),
            corner_radius: egui::CornerRadius::ZERO,
            shadow: eframe::epaint::Shadow::NONE,
            fill: surface_color,
            stroke: Stroke::NONE,
        };
        let screen_width = ui
            .ctx()
            .input(|i| i.raw.screen_rect.map(|r| r.width()).unwrap_or(800.0));

        // 状态栏
        egui::Panel::bottom("bottom-statusbar")
            .resizable(false)
            .show(ui, |ui| {
                bottom_statusbar(ui, &self.sidebar_state, screen_width)
            });

        // 导航栏
        let mut hovered_rail = None;
        egui::Panel::left("navigation-rail")
            .frame(surface_frame)
            .resizable(false)
            .show_separator_line(false)
            .show(ui, |ui| {
                hovered_rail = nav_rail(
                    ui,
                    &mut self.active_1,
                    &mut self.active_2,
                    &mut self.sidebar_state,
                    screen_width,
                );
            });

        // 统一状态机更新
        let now = std::time::Instant::now();
        if let Some(delay) = self
            .sidebar_state
            .refresh(ui.ctx(), screen_width, now, hovered_rail)
        {
            ui.ctx().request_repaint_after(delay);
        }

        // 渲染 Pinned 面板
        if matches!(
            self.sidebar_state.render_mode(),
            sidebar::RenderMode::Pinned(_)
        ) {
            egui::Panel::left("sidebar")
                .frame(surface_frame)
                .resizable(false)
                .show_separator_line(false)
                .show(ui, |ui| {
                    sidebar::render_pinned(ui, &mut self.sidebar_state);
                });
        }

        // 覆盖层（Flyout）
        let content_rect = ui.available_rect_before_wrap();
        sidebar::render_overlays(
            ui.ctx(),
            &mut self.sidebar_state,
            surface_color,
            content_rect,
            screen_width,
        );
        self.sidebar_state
            .check_flyout_leave(ui.ctx(), content_rect);

        // 标签页栏
        egui::Panel::top("tabs").resizable(false).show(ui, tabs);

        // 终端
        egui::Panel::bottom("terminal-tab")
            .resizable(true)
            .default_size(200.0)
            .size_range(60.0..=600.0)
            .show_collapsible(ui, &mut self.terminal_expanded, terminal);

        // 内容区
        let mut content_frame = surface_frame;
        content_frame.inner_margin = egui::epaint::Margin::symmetric(
            material::tokens::SPACING_L as i8,
            material::tokens::SPACING_S as i8,
        );
        egui::CentralPanel::default()
            .frame(content_frame)
            .show(ui, |ui| content(ui, &mut self.demo_clicks));

        // 主题
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

fn bottom_statusbar(ui: &mut egui::Ui, state: &SidebarState, screen_width: f32) {
    let mode_str = state.debug_mode_str();
    let active_str = state
        .active_rail()
        .map_or("None".to_string(), |r| format!("{:?}", r));
    let trigger_str = state.debug_trigger_str();
    let overlay_str = if state.is_overlay() { "O" } else { "-" };
    ui.horizontal(|ui| {
        ui.label(format!(
            "M:{mode_str} A:{active_str} T:{trigger_str} {overlay_str}  W:{screen_width:.0}",
        ));
        if ui.button("test").clicked() {}
    });
}

fn nav_rail(
    ui: &mut egui::Ui,
    active_1: &mut bool,
    active_2: &mut bool,
    sidebar_state: &mut SidebarState,
    screen_width: f32,
) -> Option<RailId> {
    ui.set_width(96.);

    ui.vertical(|ui| {
        ui.add_space(44.);
        ui.style_mut().spacing.item_spacing = Vec2::new(0., 4.);
        if rail_button(ui, "白天/晚上模式", *active_1).clicked() {
            *active_1 = !*active_1;
        };
        if rail_button(ui, "主题，切换", *active_2).clicked() {
            *active_2 = !*active_2;
        };

        ui.separator();

        ui.style_mut().spacing.item_spacing = Vec2::new(0., 4.);
        let mut hovered_rail = None;
        if sidebar_button(ui, sidebar_state, RailId::Files, screen_width) {
            hovered_rail = Some(RailId::Files);
        }
        if sidebar_button(ui, sidebar_state, RailId::Projects, screen_width) {
            hovered_rail = Some(RailId::Projects);
        }
        if sidebar_button(ui, sidebar_state, RailId::Settings, screen_width) {
            hovered_rail = Some(RailId::Settings);
        }
        if sidebar_button(ui, sidebar_state, RailId::Help, screen_width) {
            hovered_rail = Some(RailId::Help);
        }
        hovered_rail
    })
    .inner
}

fn rail_button(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    ui.add_sized(
        Vec2::new(ui.available_width(), 56.0),
        egui::Button::new(label).selected(active),
    )
}

fn sidebar_button(
    ui: &mut egui::Ui,
    state: &mut SidebarState,
    rail: RailId,
    screen_width: f32,
) -> bool {
    let active = state.active_rail() == Some(rail);
    let resp = rail_button(ui, rail.title(), active);
    if resp.clicked() {
        state.click_rail(rail, screen_width);
    }
    resp.hovered()
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

    ui.add_space(15.0);
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
}

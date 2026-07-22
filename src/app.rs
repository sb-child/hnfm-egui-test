use egui::{RichText, Stroke, Vec2, Widget};

use crate::material::color::ThemeVariant;
use crate::{
    fonts, material,
    material::NavRailItem,
    sidebar::{self, RailId, SidebarState},
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
    sidebar_state: SidebarState,
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
            sidebar_state: SidebarState::new(RailId::Files),
        }
    }
}

impl eframe::App for AppLayout {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let surface_color = material::color::access(|_p, s| s.surface).into();
        // surface背景
        let surface_frame = egui::containers::Frame {
            inner_margin: egui::epaint::Margin::symmetric(0, 4),
            outer_margin: egui::epaint::Margin::same(0),
            corner_radius: egui::CornerRadius::ZERO,
            shadow: eframe::epaint::Shadow::NONE,
            fill: surface_color,
            stroke: Stroke::NONE,
        };
        // 状态栏
        egui::Panel::bottom("bottom-statusbar")
            .resizable(false)
            .show(ui, bottom_statusbar);
        // 导航栏
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
        // 二级列表（集成sidebar状态机）
        let screen_width = ui.ctx().input(|i| {
            i.raw.screen_rect
                .map(|r| r.width())
                .unwrap_or(800.0)
        });
        sidebar::apply_responsive_default(&mut self.sidebar_state, screen_width);
        sidebar::handle_input(ui.ctx(), &mut self.sidebar_state);

        let is_pinned = matches!(self.sidebar_state.mode, sidebar::SidebarMode::Pinned(_));
        if is_pinned {
            egui::Panel::left("sidebar")
                .frame(surface_frame)
                .resizable(false)
                .show_separator_line(false)
                .show(ui, |ui| {
                    sidebar::render(
                        ui,
                        &mut self.sidebar_state,
                        &mut self.list_sel_std,
                        &mut self.list_sel_seg_0,
                        &mut self.list_sel_seg_1,
                        &mut self.list_sel_seg_2,
                    );
                });
        }
        // 标签页栏
        egui::Panel::top("tabs").resizable(false).show(ui, tabs);
        // 终端
        egui::Panel::bottom("terminal-tab")
            .resizable(true)
            .default_size(200.0)
            .size_range(60.0..=600.0)
            .show_collapsible(ui, &mut self.terminal_expanded, terminal);
        // 内容区
        egui::CentralPanel::default().show(ui, content);
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
        ui.style_mut().spacing.item_spacing = Vec2::new(0., 4.);
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

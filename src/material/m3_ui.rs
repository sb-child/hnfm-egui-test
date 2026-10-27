//! 每帧显式构造的 Material 上下文。
//!
//! 主题和动效偏好只在这里传入一次。组件通过 `M3Ui` 读取它们，
//! 不再依赖全局单例。`ui()` 保留为原生 egui 的 escape hatch。

use egui::{InnerResponse, Ui};

use crate::material::components::{Button, Card};
use crate::material::motion::MotionPrefs;
use crate::material::theme::Theme;

pub struct M3Ui<'a> {
    ui: &'a mut Ui,
    theme: &'a Theme,
    motion: MotionPrefs,
}

impl<'a> M3Ui<'a> {
    pub fn new(ui: &'a mut Ui, theme: &'a Theme, motion: MotionPrefs) -> Self {
        Self { ui, theme, motion }
    }

    /// 原生 egui 的 escape hatch。
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// 当前帧的颜色主题。
    pub fn theme(&self) -> &'a Theme {
        self.theme
    }

    /// 当前帧的动效偏好。组件可以用自己的 `.motion()` 覆盖。
    pub fn motion(&self) -> MotionPrefs {
        self.motion
    }

    /// 创建一个按钮 builder。
    pub fn button<'b>(&self, key: &'b str, label: &'b str) -> Button<'b> {
        Button::new(key, label)
    }

    /// 创建一个卡片 builder。
    pub fn card<'b>(&self) -> Card<'b> {
        Card::new()
    }

    pub fn horizontal_wrapped<R>(
        &mut self,
        add_contents: impl FnOnce(&mut M3Ui<'_>) -> R,
    ) -> InnerResponse<R> {
        let theme = self.theme;
        let motion = self.motion;
        self.ui.horizontal_wrapped(|ui| {
            let mut child = M3Ui::new(ui, theme, motion);
            add_contents(&mut child)
        })
    }

    pub fn vertical<R>(
        &mut self,
        add_contents: impl FnOnce(&mut M3Ui<'_>) -> R,
    ) -> InnerResponse<R> {
        let theme = self.theme;
        let motion = self.motion;
        self.ui.vertical(|ui| {
            let mut child = M3Ui::new(ui, theme, motion);
            add_contents(&mut child)
        })
    }
}

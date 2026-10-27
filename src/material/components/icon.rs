//! M3 图标。
//!
//! 来源:
//! - [`DIcon.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DIcon.qml): 尺寸、`filled`、`weight`、`grade`。
//! - [`Fonts.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Fonts.qml): 图标字体族。
//!
//! 图标名是 [`Icon`] 常量，调用方不接触字符串。尺寸固定、无交互、只绘制。
//!
//! 契约:
//! - 尺寸: 固定方形，默认 [`tokens::ICON_SIZE`]。
//! - 动效: 无。
//! - 命中: `Sense::hover()`，不抢点击。

use egui::{Color32, Painter, Rect, Response, Sense, Vec2};

use crate::icons::{self, IconStyle};
use crate::material::m3_ui::M3Ui;
use crate::material::theme::Theme;
use crate::material::tokens;

/// 图标名。定义在 [`crate::icons`]，这里重新导出，调用方只需要引入组件模块。
pub use crate::icons::Icon;

/// 图标 builder。
///
/// ```ignore
/// m.icon(Icon::SETTINGS).size(24.0).show(m);
/// ```
pub struct IconWidget {
    icon: Icon,
    size: Option<f32>,
    color: Option<Color32>,
    filled: bool,
    weight: Option<f32>,
}

impl IconWidget {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: None,
            color: None,
            filled: false,
            weight: None,
        }
    }

    /// 方形边长。默认 [`tokens::ICON_SIZE`]。
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size.max(0.0));
        self
    }

    /// 覆盖颜色。默认 [`crate::material::Scheme::on_surface`]。
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// `FILL` 轴。来源: `DIcon.qml` 的 `filled`。
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// 覆盖 `wght` 轴。来源: `DIcon.qml` 的 `weight`。
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = Some(weight);
        self
    }

    /// 按固定尺寸占位并绘制。
    pub fn show(self, m: &mut M3Ui<'_>) -> Response {
        let theme = m.theme();
        let size = self.size.unwrap_or(tokens::ICON_SIZE);
        let color = self
            .color
            .unwrap_or_else(|| theme.scheme().on_surface.into());
        let grade = grade_for(theme);
        let weight = self.weight.unwrap_or(tokens::FONT_WEIGHT_NORMAL);

        let (rect, response) = m
            .ui()
            .allocate_exact_size(Vec2::splat(size), Sense::hover());
        paint(
            m.ui().painter(),
            rect,
            self.icon,
            color,
            grade,
            weight,
            self.filled,
        );
        response
    }
}

/// `GRAD` 轴。来源: `DIcon.qml` 的 `grade`：浅色模式 0，暗色模式 -25。
pub(crate) fn grade_for(theme: &Theme) -> f32 {
    if theme.mode().is_light() {
        tokens::ICON_GRADE_LIGHT
    } else {
        tokens::ICON_GRADE_DARK
    }
}

/// 在 `rect` 内居中绘制图标。
///
/// 图标按 `rect` 的短边排版，因此调用方可以传任意矩形，不需要自己算方形。
pub(crate) fn paint(
    painter: &Painter,
    rect: Rect,
    icon: Icon,
    color: Color32,
    grade: f32,
    weight: f32,
    filled: bool,
) {
    let size = rect.width().min(rect.height());
    if size <= 0.0 {
        return;
    }

    let style = IconStyle {
        size,
        fill: if filled { 1.0 } else { 0.0 },
        weight,
        grade,
    };
    let galley = icons::layout_icon(painter.ctx(), icon, style);
    // galley 的坐标以绘制原点为 0，先把字形中心对到 rect 中心。
    let pos = rect.center() - galley.rect.center().to_vec2();
    painter.galley(pos, galley, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Pos2, RawInput};

    /// 图标必须是单个字形，并且绘制中心落在给定矩形中心。
    #[test]
    fn icon_is_centered_in_its_rect() {
        let context = Context::default();
        crate::icons::add_icon_font(&context);
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::splat(24.0));

        let mut output = context.run_ui(RawInput::default(), |ui| {
            paint(
                ui.painter(),
                rect,
                Icon::SETTINGS,
                Color32::WHITE,
                0.0,
                tokens::FONT_WEIGHT_NORMAL,
                false,
            );
        });
        output.textures_delta.clear();

        let text = output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::epaint::Shape::Text(text) => Some(text),
                _ => None,
            })
            .expect("图标绘制成了文本");
        let visible = text
            .galley
            .rows
            .iter()
            .flat_map(|row| &row.glyphs)
            .filter(|glyph| !glyph.uv_rect.is_nothing())
            .count();
        assert_eq!(visible, 1);

        let center = text.pos + text.galley.rect.center().to_vec2();
        assert!((center - rect.center()).length() < 0.5);
    }
}

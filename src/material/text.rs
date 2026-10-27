//! 文本测量与排版。
//!
//! 来源:
//! - [`StyledText.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StyledText.qml): 字体、换行和省略。
//! - [`StyledTextMetrics.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StyledTextMetrics.qml): 文本测量。
//! - [`DButton.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DButton.qml): 按钮 label 的字号和字重。
//!
//! 颜色固定为 `Color32::PLACEHOLDER`，绘制时由 `Painter::galley` 的 fallback
//! color 提供。这样同一份 galley 可以复用给不同颜色。

use std::sync::Arc;

use egui::epaint::text::VariationCoords;
use egui::text::{Galley, LayoutJob, TextFormat, TextWrapping};
use egui::{Align, Color32, FontFamily, FontId, Ui, Vec2};

use crate::fonts::{ff_mono, ff_sans};
use crate::material::tokens;

/// 字体族。
///
/// 来源: `StyledText.qml` 的 `fontToken`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFont {
    Sans,
    Mono,
}

impl TextFont {
    fn family(self) -> FontFamily {
        match self {
            Self::Sans => ff_sans(),
            Self::Mono => ff_mono(),
        }
    }
}

/// 一段文本的排版样式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub font: TextFont,
    pub size: f32,
    pub weight: f32,
    /// `None` 表示使用字体默认行高，对应 QML `Text` 未设置 `lineHeight` 的情况。
    pub line_height: Option<f32>,
}

impl TextStyle {
    pub const fn new(font: TextFont, size: f32, weight: f32, line_height: f32) -> Self {
        Self {
            font,
            size,
            weight,
            line_height: Some(line_height),
        }
    }

    /// 使用字体默认行高。
    pub const fn font_line_height(font: TextFont, size: f32, weight: f32) -> Self {
        Self {
            font,
            size,
            weight,
            line_height: None,
        }
    }

    /// 按钮 label。来源: `DButton.qml` 的 `font.pixelSize` / `font.weight`。
    pub const BUTTON: Self = Self::new(
        TextFont::Sans,
        tokens::FONT_SIZE_MEDIUM,
        tokens::FONT_WEIGHT_MEDIUM,
        tokens::LINE_HEIGHT_MEDIUM,
    );

    /// Tooltip 文本。
    ///
    /// 来源: `DTooltipV2.qml` 的 `contentItem`：
    /// - `fontSizeSmall` (12)
    /// - 未设置 `font.weight`，使用默认字重
    /// - 未设置 `lineHeight`，使用字体默认行高
    pub const TOOLTIP: Self = Self::font_line_height(
        TextFont::Sans,
        tokens::FONT_SIZE_SMALL,
        tokens::FONT_WEIGHT_NORMAL,
    );
}

/// 换行策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    /// 单行，超出时显示省略号。
    SingleLine,
    /// 多行，按词边界换行，不显示省略号。
    MultiLine,
    /// 多行，任意字符处都可换行，不显示省略号。
    ///
    /// 来源: `DTooltipV2.qml` 多行模式使用的 `Text.WrapAnywhere`。
    MultiLineAnywhere,
}

/// 一次文本排版的完整参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextLayout {
    pub style: TextStyle,
    pub wrap: TextWrap,
    pub max_width: f32,
    /// 水平对齐。默认左对齐。
    pub halign: Align,
}

impl TextLayout {
    pub const fn single_line(style: TextStyle, max_width: f32) -> Self {
        Self {
            style,
            wrap: TextWrap::SingleLine,
            max_width,
            halign: Align::LEFT,
        }
    }

    pub const fn multi_line(style: TextStyle, max_width: f32) -> Self {
        Self {
            style,
            wrap: TextWrap::MultiLine,
            max_width,
            halign: Align::LEFT,
        }
    }

    pub const fn multi_line_anywhere(style: TextStyle, max_width: f32) -> Self {
        Self {
            style,
            wrap: TextWrap::MultiLineAnywhere,
            max_width,
            halign: Align::LEFT,
        }
    }

    /// 设置水平对齐。
    pub fn halign(mut self, align: Align) -> Self {
        self.halign = align;
        self
    }

    fn wrapping(self) -> TextWrapping {
        match self.wrap {
            TextWrap::SingleLine => TextWrapping {
                max_width: self.max_width,
                max_rows: 1,
                overflow_character: Some('…'),
                break_anywhere: false,
            },
            TextWrap::MultiLine => TextWrapping {
                max_width: self.max_width,
                max_rows: usize::MAX,
                overflow_character: None,
                break_anywhere: false,
            },
            TextWrap::MultiLineAnywhere => TextWrapping {
                max_width: self.max_width,
                max_rows: usize::MAX,
                overflow_character: None,
                break_anywhere: true,
            },
        }
    }
}

/// 排版文本并返回 galley。`max_width = f32::INFINITY` 时用于 Hug 测量。
pub fn layout_text(ui: &Ui, text: &str, layout: TextLayout) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(layout.style.size, layout.style.font.family()),
            color: Color32::PLACEHOLDER,
            line_height: layout.style.line_height,
            coords: VariationCoords::new([(b"wght", layout.style.weight)]),
            ..Default::default()
        },
    );
    job.wrap = layout.wrapping();
    job.halign = layout.halign;
    ui.fonts_mut(|f| f.layout_job(job))
}

/// 只测量文本尺寸，不保留 galley。
pub fn measure_text(ui: &Ui, text: &str, layout: TextLayout) -> Vec2 {
    layout_text(ui, text, layout).size()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_style_matches_tokens() {
        assert_eq!(TextStyle::BUTTON.font, TextFont::Sans);
        assert_eq!(TextStyle::BUTTON.size, tokens::FONT_SIZE_MEDIUM);
        assert_eq!(TextStyle::BUTTON.weight, tokens::FONT_WEIGHT_MEDIUM);
        assert_eq!(
            TextStyle::BUTTON.line_height,
            Some(tokens::LINE_HEIGHT_MEDIUM)
        );
    }

    #[test]
    fn tooltip_style_uses_font_line_height() {
        assert_eq!(TextStyle::TOOLTIP.size, tokens::FONT_SIZE_SMALL);
        assert_eq!(TextStyle::TOOLTIP.weight, tokens::FONT_WEIGHT_NORMAL);
        assert_eq!(TextStyle::TOOLTIP.line_height, None);
    }

    #[test]
    fn single_line_uses_ellipsis() {
        let wrapping = TextLayout::single_line(TextStyle::BUTTON, 100.0).wrapping();
        assert_eq!(wrapping.max_rows, 1);
        assert_eq!(wrapping.overflow_character, Some('…'));
    }

    #[test]
    fn multi_line_does_not_elide() {
        let wrapping = TextLayout::multi_line(TextStyle::BUTTON, 100.0).wrapping();
        assert_eq!(wrapping.max_rows, usize::MAX);
        assert_eq!(wrapping.overflow_character, None);
        assert!(!wrapping.break_anywhere);
    }

    #[test]
    fn multi_line_anywhere_breaks_anywhere() {
        let wrapping = TextLayout::multi_line_anywhere(TextStyle::BUTTON, 100.0).wrapping();
        assert_eq!(wrapping.max_rows, usize::MAX);
        assert_eq!(wrapping.overflow_character, None);
        assert!(wrapping.break_anywhere);
    }

    #[test]
    fn halign_is_forwarded_to_layout() {
        let layout = TextLayout::multi_line(TextStyle::BUTTON, 100.0).halign(Align::Center);
        assert_eq!(layout.halign, Align::Center);
    }

    #[test]
    fn measure_text_returns_positive_size() {
        let ctx = egui::Context::default();
        let mut size = Vec2::ZERO;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            size = measure_text(
                ui,
                "Save",
                TextLayout::single_line(TextStyle::BUTTON, f32::INFINITY),
            );
        });
        output.textures_delta.clear();
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
    }
}

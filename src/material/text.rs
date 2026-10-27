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
use egui::{Color32, FontFamily, FontId, Ui, Vec2};

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
    pub line_height: f32,
}

impl TextStyle {
    pub const fn new(font: TextFont, size: f32, weight: f32, line_height: f32) -> Self {
        Self {
            font,
            size,
            weight,
            line_height,
        }
    }

    /// 按钮 label。来源: `DButton.qml` 的 `font.pixelSize` / `font.weight`。
    pub const BUTTON: Self = Self::new(
        TextFont::Sans,
        tokens::FONT_SIZE_MEDIUM,
        tokens::FONT_WEIGHT_MEDIUM,
        tokens::LINE_HEIGHT_MEDIUM,
    );
}

/// 换行策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    /// 单行，超出时显示省略号。
    SingleLine,
    /// 多行，按词边界换行，不显示省略号。
    MultiLine,
}

/// 一次文本排版的完整参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextLayout {
    pub style: TextStyle,
    pub wrap: TextWrap,
    pub max_width: f32,
}

impl TextLayout {
    pub const fn single_line(style: TextStyle, max_width: f32) -> Self {
        Self {
            style,
            wrap: TextWrap::SingleLine,
            max_width,
        }
    }

    pub const fn multi_line(style: TextStyle, max_width: f32) -> Self {
        Self {
            style,
            wrap: TextWrap::MultiLine,
            max_width,
        }
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
            line_height: Some(layout.style.line_height),
            coords: VariationCoords::new([(b"wght", layout.style.weight)]),
            ..Default::default()
        },
    );
    job.wrap = layout.wrapping();
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
        assert_eq!(TextStyle::BUTTON.line_height, tokens::LINE_HEIGHT_MEDIUM);
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

//! M3 卡片容器。
//!
//! 来源:
//! - [`DCard.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DCard.qml): 内边距、圆角、标题与 tone。
//! - [`DTone.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DTone.qml): tone 对应的容器与内容颜色。
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): `spacingM` / `cornerRadiusM` / `cardSurface`。
//! - [`cards/specs`](https://m3.material.io/components/cards/specs): 卡片形态。

use egui::{Color32, CornerRadius, InnerResponse, RichText, Stroke, Ui};

use crate::material::Scheme;
use crate::material::m3_ui::M3Ui;
use crate::material::tokens;

/// 卡片色调。
///
/// 来源: `DTone.qml` 的 `tone`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardTone {
    #[default]
    Surface,
    Primary,
    Secondary,
    Tertiary,
}

/// 卡片宽度策略。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardWidth {
    /// 内容宽度加两侧 padding。
    Hug,
    /// 占满当前可用宽度。
    Fill,
    /// 固定总宽度。内部内容宽度为 `Fixed - padding * 2`。
    Fixed(f32),
}

/// M3 卡片容器。
///
/// ```ignore
/// let mut m = M3Ui::new(ui, &theme, MotionPrefs::DEFAULT);
/// m.card()
///     .title("Buttons")
///     .width(CardWidth::Fill)
///     .show(&mut m, |m| {
///         m.button("save", "保存").show(m);
///     });
/// ```
pub struct Card<'a> {
    title: Option<&'a str>,
    tone: CardTone,
    padding: f32,
    radius: CornerRadius,
    width: CardWidth,
    border: bool,
}

impl<'a> Default for Card<'a> {
    fn default() -> Self {
        Self {
            title: None,
            tone: CardTone::Surface,
            padding: tokens::SPACING_M,
            radius: CornerRadius::same(tokens::CORNER_M),
            width: CardWidth::Hug,
            border: false,
        }
    }
}

impl<'a> Card<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// 可选的单行标题。来源: `DCard.qml` 的 `title`。
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn tone(mut self, tone: CardTone) -> Self {
        self.tone = tone;
        self
    }

    /// 内边距，默认 `spacingM`。来源: `DCard.qml` 的 `pad`。
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// 圆角，默认 `cornerRadiusM`。来源: `DCard.qml` 的 `restRadius`。
    pub fn radius(mut self, radius: CornerRadius) -> Self {
        self.radius = radius;
        self
    }

    pub fn width(mut self, width: CardWidth) -> Self {
        self.width = width;
        self
    }

    /// 是否绘制 `outlineVariant` 描边。来源: `DCard.qml` 的 `border`。
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// 绘制卡片背景，然后执行内容闭包。
    pub fn show<R>(
        self,
        m: &mut M3Ui<'_>,
        add_contents: impl FnOnce(&mut M3Ui<'_>) -> R,
    ) -> InnerResponse<R> {
        let theme = m.theme();
        let motion = m.motion();
        let colors = card_colors(theme.scheme(), self.tone);
        let padding = self.padding.round().clamp(0.0, i8::MAX as f32) as i8;
        let stroke = if self.border {
            Stroke::new(tokens::OUTLINE_WIDTH, colors.border)
        } else {
            Stroke::NONE
        };

        let frame = egui::Frame {
            inner_margin: egui::epaint::Margin::same(padding),
            outer_margin: egui::epaint::Margin::ZERO,
            corner_radius: self.radius,
            shadow: egui::epaint::Shadow::NONE,
            fill: colors.fill,
            stroke,
        };

        let ui = m.ui();
        frame.show(ui, |ui| {
            self.apply_width(ui);

            if let Some(title) = self.title {
                ui.add(
                    egui::Label::new(
                        RichText::new(title)
                            .size(tokens::FONT_SIZE_MEDIUM)
                            .variation("wght", tokens::FONT_WEIGHT_MEDIUM)
                            .color(colors.accent),
                    )
                    .truncate(),
                );
                // 标题后已有 item_spacing，这里补足到 spacingXS。
                let gap = (tokens::SPACING_XS - ui.spacing().item_spacing.y).max(0.0);
                ui.add_space(gap);
            }

            let mut child = M3Ui::new(ui, theme, motion);
            add_contents(&mut child)
        })
    }

    fn apply_width(&self, ui: &mut Ui) {
        match self.width {
            CardWidth::Hug => {}
            CardWidth::Fill => {
                let width = ui.available_width();
                ui.set_width(width);
            }
            CardWidth::Fixed(width) => {
                let inner = (width - self.padding * 2.0).max(0.0);
                ui.set_width(inner);
            }
        }
    }
}

struct CardColors {
    fill: Color32,
    accent: Color32,
    border: Color32,
}

/// 来源: `DTone.qml` 的 `containerColor` / `contentColor` / `accentColor`。
fn card_colors(scheme: &Scheme, tone: CardTone) -> CardColors {
    CardColors {
        fill: match tone {
            CardTone::Surface => scheme.surface_container.into(),
            CardTone::Primary => scheme.primary_container.into(),
            CardTone::Secondary => scheme.secondary_container.into(),
            CardTone::Tertiary => scheme.tertiary_container.into(),
        },
        accent: match tone {
            CardTone::Surface => scheme.primary.into(),
            CardTone::Primary => scheme.on_primary_container.into(),
            CardTone::Secondary => scheme.on_secondary_container.into(),
            CardTone::Tertiary => scheme.on_tertiary_container.into(),
        },
        border: scheme.outline_variant.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_card_tokens() {
        let card = Card::new();
        assert_eq!(card.padding, tokens::SPACING_M);
        assert_eq!(card.radius, CornerRadius::same(tokens::CORNER_M));
        assert_eq!(card.width, CardWidth::Hug);
        assert!(!card.border);
    }
}

//! M3 按钮。
//!
//! 来源:
//! - [`DButton.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DButton.qml): 尺寸、内边距、图标尺寸。
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): 按钮高度、最小宽度、state layer。
//! - [`Shape.js`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Shape.js): 按钮圆角计算。
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): state color。
//! - [`SHAPES.md`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/SHAPES.md): 按钮圆角基线。
//! - [`buttons/specs`](https://m3.material.io/components/buttons/specs): 变体颜色映射。
//!
//! `.ui()` 返回 [`Hit`]；图标目前使用文本字形。

use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Painter, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2,
    text::Galley,
};

use crate::fonts::ff_sans;
use crate::material::color;
use crate::material::interact::{Hit, hit_rounded};
use crate::material::motion::{self, Motion, MotionPrefs};
use crate::material::state_layer;
use crate::material::text::{self, TextLayout, TextStyle};
use crate::material::tokens;
use crate::material::util;

/// 按钮的视觉变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Filled,
    Tonal,
    Outlined,
    Text,
}

/// 按钮高度档位。
///
/// - 来源: `Style.qml`: `buttonHeightXXS` 到 `buttonHeightM`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Xxs,
    Xs,
    S,
    M,
}

impl ButtonSize {
    fn height(self) -> f32 {
        match self {
            Self::Xxs => tokens::BUTTON_HEIGHT_XXS,
            Self::Xs => tokens::BUTTON_HEIGHT_XS,
            Self::S => tokens::BUTTON_HEIGHT_S,
            Self::M => tokens::BUTTON_HEIGHT_M,
        }
    }

    /// 来源: `DButton.qml` 的 `horizontalPadding`。
    fn horizontal_padding(self) -> f32 {
        match self {
            Self::Xxs | Self::Xs => tokens::SPACING_M,
            Self::S => tokens::SPACING_L,
            Self::M => tokens::SPACING_XL,
        }
    }

    /// 来源: `DButton.qml` 的 `contentRow.spacing`。
    fn content_spacing(self) -> f32 {
        match self {
            Self::Xxs | Self::Xs => tokens::SPACING_XS,
            Self::S | Self::M => tokens::SPACING_S,
        }
    }

    /// 来源: `DButton.qml` 的 `iconSize`。
    fn icon_size(self) -> f32 {
        if self.height() <= tokens::BUTTON_HEIGHT_S {
            tokens::ICON_SIZE_MEDIUM
        } else {
            tokens::ICON_SIZE
        }
    }
}

/// 圆角形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonShape {
    /// 胶囊。
    ///
    /// - 来源: `Shape.js`: `fullRadius(width, height)`。
    Round,
    /// 方角。
    ///
    /// - 来源: `Shape.js`: `buttonRadius(height, pressed, false)`。
    Square,
}

/// 宽度策略。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonWidth {
    /// 内容宽度加两侧 padding，并保证不小于 `BUTTON_MIN_WIDTH`。
    Hug,
    /// 占满当前可用宽度。
    Fill,
    /// 固定宽度。允许小于最小宽度，由调用方负责。
    Fixed(f32),
}

/// M3 按钮。
///
/// `key` 必须是全局唯一且稳定的。建议用 `模块_控件` 前缀，例如
/// `"toolbar_save"`；它会作为 egui 的 `Id` 和动画 ID 的根。
///
/// ```ignore
/// let hit = Button::new("save", "保存")
///     .variant(ButtonVariant::Filled)
///     .ui(ui);
/// if hit.clicked() {
///     save();
/// }
/// ```
pub struct Button<'a> {
    key: &'a str,
    label: &'a str,
    icon: Option<&'a str>,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    width: ButtonWidth,
    enabled: bool,
    motion: MotionPrefs,
}

impl<'a> Button<'a> {
    pub fn new(key: &'a str, label: &'a str) -> Self {
        Self {
            key,
            label,
            icon: None,
            variant: ButtonVariant::Filled,
            size: ButtonSize::S,
            shape: ButtonShape::Round,
            width: ButtonWidth::Hug,
            enabled: true,
            motion: MotionPrefs::DEFAULT,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn width(mut self, width: ButtonWidth) -> Self {
        self.width = width;
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 动效偏好。默认 [`MotionPrefs::DEFAULT`]。
    pub fn motion(mut self, motion: MotionPrefs) -> Self {
        self.motion = motion;
        self
    }

    pub fn ui(self, ui: &mut Ui) -> Hit {
        let enabled = self.enabled && ui.is_enabled();
        let metrics = self.measure(ui);
        let width = self.resolve_width(ui, &metrics);
        let (_, rect) = ui.allocate_space(Vec2::new(width, metrics.height));

        // 命中用 rest 圆角；按压时视觉圆角变化不影响命中区域。
        let radius_rest = match self.shape {
            ButtonShape::Round => tokens::full_radius(width, metrics.height),
            ButtonShape::Square => tokens::button_radius(metrics.height, false),
        };
        let radius_pressed = tokens::button_radius(metrics.height, true);

        let id = Id::unique(self.key);
        let hit = hit_rounded(ui, id, rect, radius_rest, enabled);

        let colors = button_colors(self.variant, enabled);
        let anim = transitions(ui, id, &hit, enabled, self.motion);
        let radius = util::lerp_corner_radius(radius_rest, radius_pressed, anim.shape);
        let layer_alpha = state_layer::alpha(anim.hover, anim.press, anim.focus);

        let painter = ui.painter();
        paint_surface(painter, rect, radius, &colors, layer_alpha);
        self.paint_content(ui, painter, rect, width, &metrics, &colors);
        if hit.focused() {
            paint_focus_ring(painter, rect, radius, &colors);
        }

        hit
    }

    /// 测量 Hug 宽度。返回的 galley 在绘制阶段可复用。
    fn measure(&self, ui: &Ui) -> ButtonMetrics {
        let height = self.size.height();
        let padding = self.size.horizontal_padding();
        let spacing = self.size.content_spacing();
        let icon_size = self.size.icon_size();

        let label = text::layout_text(
            ui,
            self.label,
            TextLayout::single_line(TextStyle::BUTTON, f32::INFINITY),
        );
        let icon_width = if self.icon.is_some() {
            icon_size + spacing
        } else {
            0.0
        };
        let content_width = label.size().x + icon_width;

        ButtonMetrics {
            height,
            padding,
            spacing,
            icon_size,
            icon_width,
            content_width,
            label,
        }
    }

    fn resolve_width(&self, ui: &Ui, metrics: &ButtonMetrics) -> f32 {
        match self.width {
            ButtonWidth::Hug => {
                (metrics.content_width + metrics.padding * 2.0).max(tokens::BUTTON_MIN_WIDTH)
            }
            ButtonWidth::Fill => ui.available_width().max(0.0),
            ButtonWidth::Fixed(width) => width.max(0.0),
        }
    }

    fn paint_content(
        &self,
        ui: &Ui,
        painter: &Painter,
        rect: Rect,
        width: f32,
        metrics: &ButtonMetrics,
        colors: &ButtonColors,
    ) {
        let label_max_width = match self.width {
            ButtonWidth::Hug => f32::INFINITY,
            ButtonWidth::Fill | ButtonWidth::Fixed(_) => {
                (width - metrics.padding * 2.0 - metrics.icon_width).max(0.0)
            }
        };
        // 文本放得下时复用测量阶段的 galley；只有需要省略号时重新排版。
        let label = if metrics.label.size().x <= label_max_width {
            metrics.label.clone()
        } else {
            text::layout_text(
                ui,
                self.label,
                TextLayout::single_line(TextStyle::BUTTON, label_max_width),
            )
        };

        let total_width = metrics.icon_width + label.size().x;
        let mut x = rect.center().x - total_width * 0.5;

        if let Some(icon) = self.icon {
            painter.text(
                Pos2::new(x + metrics.icon_size * 0.5, rect.center().y),
                Align2::CENTER_CENTER,
                icon,
                FontId::new(metrics.icon_size, ff_sans()),
                colors.text,
            );
            x += metrics.icon_size + metrics.spacing;
        }

        painter.galley(
            Pos2::new(x, rect.center().y - label.size().y * 0.5),
            label,
            colors.text,
        );
    }
}

struct ButtonMetrics {
    height: f32,
    padding: f32,
    spacing: f32,
    icon_size: f32,
    icon_width: f32,
    content_width: f32,
    /// 无换行的测量结果。颜色是 PLACEHOLDER，绘制时由 `painter.galley` 的
    /// fallback color 替换。
    label: Arc<Galley>,
}

struct ButtonAnim {
    hover: f32,
    press: f32,
    focus: f32,
    shape: f32,
}

fn transitions(ui: &Ui, id: Id, hit: &Hit, enabled: bool, prefs: MotionPrefs) -> ButtonAnim {
    if !enabled {
        return ButtonAnim {
            hover: 0.0,
            press: 0.0,
            focus: 0.0,
            shape: 0.0,
        };
    }
    ButtonAnim {
        hover: motion::animate01(
            ui,
            id.with("hover"),
            hit.hovered(),
            Motion::EXPRESSIVE_EFFECTS,
            prefs,
        ),
        press: motion::animate01(
            ui,
            id.with("press"),
            hit.pressed(),
            Motion::EXPRESSIVE_EFFECTS,
            prefs,
        ),
        focus: motion::animate01(
            ui,
            id.with("focus"),
            hit.focused(),
            Motion::EXPRESSIVE_EFFECTS,
            prefs,
        ),
        shape: motion::animate01(
            ui,
            id.with("shape"),
            hit.pressed(),
            Motion::SHAPE_CHANGE,
            prefs,
        ),
    }
}

fn paint_surface(
    painter: &Painter,
    rect: Rect,
    radius: CornerRadius,
    colors: &ButtonColors,
    layer_alpha: f32,
) {
    if colors.background != Color32::TRANSPARENT {
        painter.rect_filled(rect, radius, colors.background);
    }
    if colors.border != Color32::TRANSPARENT {
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(tokens::OUTLINE_WIDTH, colors.border),
            StrokeKind::Inside,
        );
    }
    state_layer::paint(painter, rect, radius, colors.state, layer_alpha);
}

fn paint_focus_ring(painter: &Painter, rect: Rect, radius: CornerRadius, colors: &ButtonColors) {
    painter.rect_stroke(
        rect.expand(tokens::FOCUS_RING_OFFSET),
        tokens::focus_ring_radius(radius),
        Stroke::new(tokens::FOCUS_RING_WIDTH, colors.focus_ring),
        StrokeKind::Middle,
    );
}

struct ButtonColors {
    background: Color32,
    text: Color32,
    border: Color32,
    state: Color32,
    focus_ring: Color32,
}

/// M3 按钮颜色。
///
/// 来源:
/// - `buttons/specs`: 变体颜色映射。
/// - `DButton.qml`: `backgroundColor` / `textColor`。
/// - `StateLayer.qml`: state color。
fn button_colors(variant: ButtonVariant, enabled: bool) -> ButtonColors {
    let (primary, on_primary, secondary_container, on_secondary_container, outline, on_surface): (
        Color32,
        Color32,
        Color32,
        Color32,
        Color32,
        Color32,
    ) = color::access(|_p, s| {
        (
            s.primary.into(),
            s.on_primary.into(),
            s.secondary_container.into(),
            s.on_secondary_container.into(),
            s.outline.into(),
            s.on_surface.into(),
        )
    });

    if !enabled {
        let disabled_text = with_alpha(on_surface, 0.38);
        let disabled_container = with_alpha(on_surface, 0.12);
        return match variant {
            ButtonVariant::Filled | ButtonVariant::Tonal => ButtonColors {
                background: disabled_container,
                text: disabled_text,
                border: Color32::TRANSPARENT,
                state: Color32::TRANSPARENT,
                focus_ring: Color32::TRANSPARENT,
            },
            ButtonVariant::Outlined => ButtonColors {
                background: Color32::TRANSPARENT,
                text: disabled_text,
                border: disabled_container,
                state: Color32::TRANSPARENT,
                focus_ring: Color32::TRANSPARENT,
            },
            ButtonVariant::Text => ButtonColors {
                background: Color32::TRANSPARENT,
                text: disabled_text,
                border: Color32::TRANSPARENT,
                state: Color32::TRANSPARENT,
                focus_ring: Color32::TRANSPARENT,
            },
        };
    }

    let (background, text, border) = match variant {
        ButtonVariant::Filled => (primary, on_primary, Color32::TRANSPARENT),
        ButtonVariant::Tonal => (
            secondary_container,
            on_secondary_container,
            Color32::TRANSPARENT,
        ),
        ButtonVariant::Outlined => (Color32::TRANSPARENT, primary, outline),
        ButtonVariant::Text => (Color32::TRANSPARENT, primary, Color32::TRANSPARENT),
    };

    ButtonColors {
        background,
        text,
        border,
        state: text,
        focus_ring: primary,
    }
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_tokens_match_source() {
        assert_eq!(ButtonSize::Xxs.height(), 28.0);
        assert_eq!(ButtonSize::Xs.height(), 32.0);
        assert_eq!(ButtonSize::S.height(), 40.0);
        assert_eq!(ButtonSize::M.height(), 56.0);

        assert_eq!(ButtonSize::Xs.horizontal_padding(), 12.0);
        assert_eq!(ButtonSize::S.horizontal_padding(), 16.0);
        assert_eq!(ButtonSize::M.horizontal_padding(), 24.0);

        assert_eq!(ButtonSize::Xs.content_spacing(), 4.0);
        assert_eq!(ButtonSize::S.content_spacing(), 8.0);
        assert_eq!(ButtonSize::M.content_spacing(), 8.0);

        assert_eq!(ButtonSize::S.icon_size(), 20.0);
        assert_eq!(ButtonSize::M.icon_size(), 24.0);
    }
}

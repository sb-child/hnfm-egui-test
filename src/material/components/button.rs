//! M3 按钮。
//!
//! 来源:
//! - [`DButton.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DButton.qml): 尺寸、内边距、最大宽、换行、预留文本。
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): 按钮高度、最小宽度、state layer。
//! - [`Shape.js`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Shape.js): 按钮圆角计算。
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): state color。
//! - [`SHAPES.md`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/SHAPES.md): 按钮圆角基线。
//! - [`buttons/specs`](https://m3.material.io/components/buttons/specs): 变体颜色映射。
//!
//! `.show()` 返回 [`Hit`]；图标目前使用文本字形。

use std::sync::Arc;

use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Painter, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2,
    text::Galley,
};

use crate::fonts::ff_sans;
use crate::material::Scheme;
use crate::material::effects::interpolate;
use crate::material::effects::motion::{self, Motion, MotionPrefs};
use crate::material::effects::state_layer;
use crate::material::interact::{Hit, hit_rounded};
use crate::material::m3_ui::M3Ui;
pub use crate::material::text::TextWrap;
use crate::material::text::{self, TextLayout, TextStyle};
use crate::material::theme::Theme;
use crate::material::tokens;

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
}

/// 来源: `DButton.qml` 的 `horizontalPadding`。
fn horizontal_padding_for_height(height: f32) -> f32 {
    if height <= tokens::BUTTON_HEIGHT_XS {
        tokens::SPACING_M
    } else if height <= tokens::BUTTON_HEIGHT_S {
        tokens::SPACING_L
    } else {
        tokens::SPACING_XL
    }
}

/// 来源: `DButton.qml` 的 `contentRow.spacing`。
fn content_spacing_for_height(height: f32) -> f32 {
    if height <= tokens::BUTTON_HEIGHT_XS {
        tokens::SPACING_XS
    } else if height <= tokens::BUTTON_HEIGHT_M {
        tokens::SPACING_S
    } else {
        tokens::SPACING_M
    }
}

/// 来源: `DButton.qml` 的 `iconSize`。
fn icon_size_for_height(height: f32) -> f32 {
    if height <= tokens::BUTTON_HEIGHT_S {
        tokens::ICON_SIZE_MEDIUM
    } else {
        tokens::ICON_SIZE
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
    /// 内容宽度加两侧 padding，并且不小于 `minimum_width`、不大于 `maximum_width`。
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
/// let mut m = M3Ui::new(ui, &theme, MotionPrefs::DEFAULT);
/// let hit = m
///     .button("save", "保存")
///     .variant(ButtonVariant::Filled)
///     .show(&mut m);
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
    height: Option<f32>,
    minimum_width: f32,
    horizontal_padding: Option<f32>,
    icon_size: Option<f32>,
    shape: ButtonShape,
    width: ButtonWidth,
    maximum_width: f32,
    reserve_text: Option<&'a str>,
    text_wrap: TextWrap,
    enabled: bool,
    motion: Option<MotionPrefs>,
}

impl<'a> Button<'a> {
    pub fn new(key: &'a str, label: &'a str) -> Self {
        Self {
            key,
            label,
            icon: None,
            variant: ButtonVariant::Filled,
            size: ButtonSize::S,
            height: None,
            minimum_width: tokens::BUTTON_MIN_WIDTH,
            horizontal_padding: None,
            icon_size: None,
            shape: ButtonShape::Round,
            width: ButtonWidth::Hug,
            maximum_width: f32::INFINITY,
            reserve_text: None,
            text_wrap: TextWrap::SingleLine,
            enabled: true,
            motion: None,
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

    /// 覆盖高度。padding、spacing、icon size 的默认值会按新高度重算。
    /// 来源: `DButton.qml` 的 `buttonHeight`。
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height.max(0.0));
        self
    }

    /// Hug 宽度的最小宽度。来源: `DButton.qml` 的 `minimumWidth`。
    pub fn minimum_width(mut self, width: f32) -> Self {
        self.minimum_width = width.max(0.0);
        self
    }

    /// 覆盖水平内边距。来源: `DButton.qml` 的 `horizontalPadding`。
    pub fn horizontal_padding(mut self, padding: f32) -> Self {
        self.horizontal_padding = Some(padding.max(0.0));
        self
    }

    /// 覆盖图标尺寸。来源: `DButton.qml` 的 `iconSize`。
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = Some(size.max(0.0));
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

    /// Hug 宽度的上限。Fill / Fixed 不受影响。
    /// 来源: `DButton.qml` 的 `maximumWidth`。
    pub fn maximum_width(mut self, width: f32) -> Self {
        self.maximum_width = width.max(0.0);
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 预留文本宽度，避免 label 变化时按钮宽度跳动。
    /// 来源: `DButton.qml` 的 `reserveText`。
    pub fn reserve_text(mut self, text: &'a str) -> Self {
        self.reserve_text = Some(text);
        self
    }

    /// 文本换行策略。来源: `DButton.qml` 的 `wrapText`。
    pub fn text_wrap(mut self, wrap: TextWrap) -> Self {
        self.text_wrap = wrap;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 覆盖 [`M3Ui`] 提供的动效偏好。
    pub fn motion(mut self, motion: MotionPrefs) -> Self {
        self.motion = Some(motion);
        self
    }

    pub fn show(self, m: &mut M3Ui<'_>) -> Hit {
        let prefs = self.motion.unwrap_or_else(|| m.motion());
        let theme = m.theme();
        let ui = m.ui();
        self.show_in(ui, theme, prefs)
    }

    fn show_in(self, ui: &mut Ui, theme: &Theme, prefs: MotionPrefs) -> Hit {
        let enabled = self.enabled && ui.is_enabled();
        let metrics = self.measure(ui);
        let (_, rect) = ui.allocate_space(Vec2::new(metrics.width, metrics.height));

        // 命中用 rest 圆角；按压时视觉圆角变化不影响命中区域。
        let radius_rest = match self.shape {
            ButtonShape::Round => tokens::full_radius(metrics.width, metrics.height),
            ButtonShape::Square => tokens::button_radius(metrics.button_height, false),
        };
        let radius_pressed = tokens::button_radius(metrics.button_height, true);

        let id = Id::unique(self.key);
        let hit = hit_rounded(ui, id, rect, radius_rest, enabled);

        let colors = button_colors(theme.scheme(), self.variant, enabled);
        let anim = transitions(ui, id, &hit, enabled, prefs);
        let radius = interpolate::lerp_corner_radius(radius_rest, radius_pressed, anim.shape);
        let layer_alpha = state_layer::alpha(anim.hover, anim.press, anim.focus);

        let painter = ui.painter();
        paint_surface(painter, rect, radius, &colors, layer_alpha);
        self.paint_content(painter, rect, &metrics, &colors);
        if hit.focused() {
            paint_focus_ring(painter, rect, radius, &colors);
        }

        hit
    }

    fn spec(&self) -> ButtonSpec {
        let height = self.height.unwrap_or_else(|| self.size.height());
        ButtonSpec {
            height,
            padding: self
                .horizontal_padding
                .unwrap_or_else(|| horizontal_padding_for_height(height)),
            spacing: content_spacing_for_height(height),
            icon_size: self
                .icon_size
                .unwrap_or_else(|| icon_size_for_height(height)),
            min_width: self.minimum_width,
        }
    }

    /// 先按自然文本算宽度，再用最终宽度排版 label。
    fn measure(&self, ui: &Ui) -> ButtonMetrics {
        let spec = self.spec();
        let natural_label = text::layout_text(
            ui,
            self.label,
            TextLayout::single_line(TextStyle::BUTTON, f32::INFINITY),
        );
        let reserved_width = self.reserve_text.map_or(0.0, |text| {
            text::layout_text(
                ui,
                text,
                TextLayout::single_line(TextStyle::BUTTON, f32::INFINITY),
            )
            .size()
            .x
        });
        let icon_width = if self.icon.is_some() {
            spec.icon_size + spec.spacing
        } else {
            0.0
        };
        let desired_label_width = natural_label.size().x.max(reserved_width);

        let width = self.resolve_width(ui, desired_label_width + icon_width, &spec);
        let label_max_width = (width - spec.padding * 2.0 - icon_width).max(0.0);
        let label_box_width = desired_label_width.min(label_max_width);

        let label = match self.text_wrap {
            TextWrap::SingleLine => {
                if natural_label.size().x <= label_box_width {
                    natural_label
                } else {
                    text::layout_text(
                        ui,
                        self.label,
                        TextLayout::single_line(TextStyle::BUTTON, label_box_width),
                    )
                }
            }
            TextWrap::MultiLine => text::layout_text(
                ui,
                self.label,
                TextLayout::multi_line(TextStyle::BUTTON, label_box_width),
            ),
        };

        let content_height = if self.icon.is_some() {
            spec.icon_size
        } else {
            0.0
        }
        .max(label.size().y);
        let height = match self.text_wrap {
            TextWrap::SingleLine => spec.height,
            TextWrap::MultiLine => spec.height.max(content_height + tokens::SPACING_S * 2.0),
        };

        ButtonMetrics {
            width,
            height,
            button_height: spec.height,
            spacing: spec.spacing,
            icon_size: spec.icon_size,
            icon_width,
            label_width: label_box_width,
            label,
        }
    }

    fn resolve_width(&self, ui: &Ui, content_width: f32, spec: &ButtonSpec) -> f32 {
        match self.width {
            ButtonWidth::Hug => {
                let width = (content_width + spec.padding * 2.0).max(spec.min_width);
                width.min(self.maximum_width)
            }
            ButtonWidth::Fill => ui.available_width().max(0.0),
            ButtonWidth::Fixed(width) => width.max(0.0),
        }
    }

    fn paint_content(
        &self,
        painter: &Painter,
        rect: Rect,
        metrics: &ButtonMetrics,
        colors: &ButtonColors,
    ) {
        let total_width = metrics.icon_width + metrics.label_width;
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
            Pos2::new(x, rect.center().y - metrics.label.size().y * 0.5),
            metrics.label.clone(),
            colors.text,
        );
    }
}

struct ButtonSpec {
    height: f32,
    padding: f32,
    spacing: f32,
    icon_size: f32,
    min_width: f32,
}

struct ButtonMetrics {
    width: f32,
    height: f32,
    button_height: f32,
    spacing: f32,
    icon_size: f32,
    icon_width: f32,
    label_width: f32,
    /// 最终排版结果。颜色是 PLACEHOLDER，绘制时由 `painter.galley` 的
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
fn button_colors(scheme: &Scheme, variant: ButtonVariant, enabled: bool) -> ButtonColors {
    let primary: Color32 = scheme.primary.into();
    let on_primary: Color32 = scheme.on_primary.into();
    let secondary_container: Color32 = scheme.secondary_container.into();
    let on_secondary_container: Color32 = scheme.on_secondary_container.into();
    let outline: Color32 = scheme.outline.into();
    let on_surface: Color32 = scheme.on_surface.into();

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

        assert_eq!(horizontal_padding_for_height(ButtonSize::Xs.height()), 12.0);
        assert_eq!(horizontal_padding_for_height(ButtonSize::S.height()), 16.0);
        assert_eq!(horizontal_padding_for_height(ButtonSize::M.height()), 24.0);

        assert_eq!(content_spacing_for_height(ButtonSize::Xs.height()), 4.0);
        assert_eq!(content_spacing_for_height(ButtonSize::S.height()), 8.0);
        assert_eq!(content_spacing_for_height(ButtonSize::M.height()), 8.0);
        assert_eq!(content_spacing_for_height(60.0), 12.0);

        assert_eq!(icon_size_for_height(ButtonSize::S.height()), 20.0);
        assert_eq!(icon_size_for_height(ButtonSize::M.height()), 24.0);
    }

    #[test]
    fn custom_height_recomputes_defaults() {
        let spec = Button::new("key", "label").height(60.0).spec();
        assert_eq!(spec.height, 60.0);
        assert_eq!(spec.padding, 24.0);
        assert_eq!(spec.spacing, 12.0);
        assert_eq!(spec.icon_size, 24.0);
        assert_eq!(spec.min_width, tokens::BUTTON_MIN_WIDTH);
    }

    #[test]
    fn builder_overrides_are_kept() {
        let button = Button::new("key", "label")
            .minimum_width(100.0)
            .horizontal_padding(10.0)
            .icon_size(18.0)
            .maximum_width(200.0)
            .reserve_text("reserved")
            .text_wrap(TextWrap::MultiLine);

        assert_eq!(button.minimum_width, 100.0);
        assert_eq!(button.horizontal_padding, Some(10.0));
        assert_eq!(button.icon_size, Some(18.0));
        assert_eq!(button.maximum_width, 200.0);
        assert_eq!(button.reserve_text, Some("reserved"));
        assert_eq!(button.text_wrap, TextWrap::MultiLine);
    }

    fn measure(button: &Button<'_>) -> ButtonMetrics {
        let ctx = egui::Context::default();
        let mut metrics = None;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            metrics = Some(button.measure(ui));
        });
        output.textures_delta.clear();
        metrics.unwrap()
    }

    #[test]
    fn maximum_width_caps_hug_width() {
        let button = Button::new("key", "Save").maximum_width(20.0);
        assert_eq!(measure(&button).width, 20.0);
    }

    #[test]
    fn reserve_text_widens_hug_width() {
        let plain = measure(&Button::new("key", "Save")).width;
        let reserved =
            measure(&Button::new("key", "Save").reserve_text("A much longer reserved label")).width;
        assert!(reserved > plain);
    }

    #[test]
    fn wrap_text_grows_height() {
        let metrics = measure(
            &Button::new("key", "A long label that needs more than one line")
                .width(ButtonWidth::Fixed(100.0))
                .text_wrap(TextWrap::MultiLine),
        );
        assert!(metrics.height > ButtonSize::S.height());
    }
}

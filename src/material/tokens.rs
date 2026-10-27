//! Material 3 组件共享的尺寸与状态 token。
//!
//! 来源:
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): 间距、字号、图标尺寸、按钮高度、state layer。
//! - [`Shape.js`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Shape.js): 圆角阶梯与圆角计算。
//! - [`SHAPES.md`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/SHAPES.md): 圆角基线。
//! - [`Theme.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/Common/Theme.qml): 应用主题覆盖的焦点环偏移。
//! - [`StyledText.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StyledText.qml): 行高实现。
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): state layer 的合成。
//! - [`DSpinner.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DSpinner.qml): spinner 线宽下限。
//! - [`type-scale-tokens`](https://m3.material.io/styles/typography/type-scale-tokens): BodyMedium 行高。
//!
//! 这里的单位是 egui 的 point（逻辑像素）。非整数缩放下由 egui 统一乘
//! `pixels_per_point`，组件不要自己再乘一遍。

use egui::CornerRadius;

/// 间距阶梯。来源: `Style.qml` 的 `spacingXXS` 到 `spacingXL`。
pub const SPACING_XXS: f32 = 2.0;
pub const SPACING_XS: f32 = 4.0;
pub const SPACING_S: f32 = 8.0;
pub const SPACING_M: f32 = 12.0;
pub const SPACING_L: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;

/// 字号。来源: `Style.qml` 的 `fontSizeSmall` 到 `fontSizeLarge`。
pub const FONT_SIZE_SMALL: f32 = 12.0;
pub const FONT_SIZE_MEDIUM: f32 = 14.0;
pub const FONT_SIZE_LARGE: f32 = 16.0;

/// M3 BodyMedium 行高。
///
/// 来源:
/// - `type-scale-tokens`: BodyMedium 行高。
/// - `StyledText.qml`: 行高实现。
pub const LINE_HEIGHT_MEDIUM: f32 = 20.0;

/// M3 的 Medium 字重。通过可变字体轴 `wght` 使用。
pub const FONT_WEIGHT_MEDIUM: f32 = 500.0;

/// 图标尺寸。来源: `Style.qml` 的 `iconSizeSmall` 到 `iconSizeLarge`。
pub const ICON_SIZE_SMALL: f32 = 16.0;
pub const ICON_SIZE_MEDIUM: f32 = 20.0;
pub const ICON_SIZE: f32 = 24.0;
pub const ICON_SIZE_LARGE: f32 = 32.0;

/// 按钮高度。来源: `Style.qml` 的 `buttonHeightXXS` 到 `buttonHeightM`。
pub const BUTTON_HEIGHT_XXS: f32 = 28.0;
pub const BUTTON_HEIGHT_XS: f32 = 32.0;
pub const BUTTON_HEIGHT_S: f32 = 40.0;
pub const BUTTON_HEIGHT_M: f32 = 56.0;
pub const BUTTON_MIN_WIDTH: f32 = 58.0;

/// 按钮按压缩放。来源: `Theme.qml` 的 `pressScale`。
pub const PRESS_SCALE: f32 = 0.98;

/// State layer 透明度。
///
/// 来源:
/// - `Style.qml`: `stateLayerHover` / `stateLayerFocus` / `stateLayerPressed`。
/// - `StateLayer.qml`: state layer 的合成。
pub const STATE_HOVER: f32 = 0.08;
pub const STATE_FOCUS: f32 = 0.12;
pub const STATE_PRESSED: f32 = 0.12;

/// 描边与焦点环。
///
/// 来源:
/// - `Theme.qml`: `focusRingWidth` 1.5，`focusRingOffset` 3。
/// - `Style.qml`: fallback width 2，offset 4。
pub const OUTLINE_WIDTH: f32 = 1.0;
pub const FOCUS_RING_WIDTH: f32 = 1.5;
pub const FOCUS_RING_OFFSET: f32 = 3.0;

/// Spinner 最小线宽。来源: `DSpinner.qml` 的 `strokeWidth` 默认值。
pub const SPINNER_STROKE_WIDTH: f32 = 2.0;

/// 圆角阶梯。来源: `Shape.js` 的 `corners`。
pub const CORNER_XXS: u8 = 2;
pub const CORNER_XS: u8 = 4;
pub const CORNER_S: u8 = 8;
pub const CORNER_M: u8 = 12;
pub const CORNER_L: u8 = 16;
pub const CORNER_L_INCREASED: u8 = 20;
pub const CORNER_XL: u8 = 28;
pub const CORNER_XL_INCREASED: u8 = 32;
pub const CORNER_XXL: u8 = 48;

/// 胶囊/圆形控件的圆角：短边的一半，并限制在 `CornerRadius` 的 u8 范围。
///
/// 来源: `Shape.js` 的 `fullRadius(width, height, scale, fixed)`。
pub fn full_radius(width: f32, height: f32) -> CornerRadius {
    let r = (width.min(height) * 0.5).round().clamp(0.0, u8::MAX as f32) as u8;
    CornerRadius::same(r)
}

/// 按钮方角：按高度选择 rest / pressed 圆角。
///
/// 来源: `Shape.js` 的 `buttonCorner(sizeHeight, pressed)`。
///
/// - `<= 40`: rest `m` (12), pressed `s` (8)
/// - `<= 56`: rest `l` (16), pressed `m` (12)
/// - `> 56`: rest `xl` (28), pressed `l` (16)
pub fn button_radius(height: f32, pressed: bool) -> CornerRadius {
    let r = if height <= BUTTON_HEIGHT_S {
        if pressed { CORNER_S } else { CORNER_M }
    } else if height <= BUTTON_HEIGHT_M {
        if pressed { CORNER_M } else { CORNER_L }
    } else if pressed {
        CORNER_L
    } else {
        CORNER_XL
    };
    CornerRadius::same(r)
}

/// 焦点环圆角：在组件圆角基础上外扩 `FOCUS_RING_OFFSET`。
pub fn focus_ring_radius(radius: CornerRadius) -> CornerRadius {
    let offset = FOCUS_RING_OFFSET.round() as u8;
    CornerRadius {
        nw: radius.nw.saturating_add(offset),
        ne: radius.ne.saturating_add(offset),
        sw: radius.sw.saturating_add(offset),
        se: radius.se.saturating_add(offset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_radius_is_half_of_shorter_side() {
        assert_eq!(full_radius(100.0, 40.0), CornerRadius::same(20));
        assert_eq!(full_radius(40.0, 40.0), CornerRadius::same(20));
    }

    #[test]
    fn button_radius_follows_material_steps() {
        assert_eq!(
            button_radius(BUTTON_HEIGHT_S, false),
            CornerRadius::same(CORNER_M)
        );
        assert_eq!(
            button_radius(BUTTON_HEIGHT_S, true),
            CornerRadius::same(CORNER_S)
        );
        assert_eq!(
            button_radius(BUTTON_HEIGHT_M, false),
            CornerRadius::same(CORNER_L)
        );
        assert_eq!(
            button_radius(BUTTON_HEIGHT_M, true),
            CornerRadius::same(CORNER_M)
        );
    }
}

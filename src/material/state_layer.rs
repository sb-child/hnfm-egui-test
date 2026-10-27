//! M3 state layer：交互反馈的半透明叠加层。
//!
//! 来源:
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): state layer 的合成与绘制。
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): `stateLayerHover` / `stateLayerFocus` / `stateLayerPressed`。

use egui::{Color32, CornerRadius, Painter, Rect};

use crate::material::tokens::{STATE_HOVER, STATE_PRESSED};

/// 由 hover / press / focus 的动画进度合成最终透明度。
///
/// 来源: `StateLayer.qml`。
/// 优先级：pressed 和 focus 都是 0.12，高于 hover 的 0.08。`press_t` 和 `focus_t` 取较大值。
pub fn alpha(hover_t: f32, press_t: f32, focus_t: f32) -> f32 {
    let press_focus = press_t.max(focus_t).clamp(0.0, 1.0);
    let hover = STATE_HOVER * hover_t.clamp(0.0, 1.0);
    (hover * (1.0 - press_focus) + STATE_PRESSED * press_focus).clamp(0.0, STATE_PRESSED)
}

/// 在控件上叠加 state layer。
pub fn paint(painter: &Painter, rect: Rect, radius: CornerRadius, color: Color32, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let a = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    painter.rect_filled(
        rect,
        radius,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::tokens::STATE_FOCUS;

    #[test]
    fn nothing_active_is_transparent() {
        assert_eq!(alpha(0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn hover_is_weaker_than_press_and_focus() {
        assert!((alpha(1.0, 0.0, 0.0) - STATE_HOVER).abs() < 1e-6);
        assert!((alpha(0.0, 1.0, 0.0) - STATE_PRESSED).abs() < 1e-6);
        assert!((alpha(0.0, 0.0, 1.0) - STATE_FOCUS).abs() < 1e-6);
    }

    #[test]
    fn hovered_and_pressed_is_capped() {
        assert!((alpha(1.0, 1.0, 1.0) - STATE_PRESSED).abs() < 1e-6);
    }
}

//! 形状命中与交互状态。
//!
//! egui 0.37 的 `Response` 只提供矩形命中，且布尔状态没有 setter。
//! 这里在按下时根据圆角形状选择 `Sense`，避免事后修改 `Response`。
//!
//! 参考:
//! - `StateLayer.qml`:
//!   https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml
//! - `StyledButton.qml`:
//!   https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StyledButton.qml

use egui::{CornerRadius, CursorIcon, Id, PointerButton, Pos2, Rect, Response, Sense, Ui};

/// 一个组件的有效交互状态。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Interaction {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

impl Interaction {
    pub fn any(self) -> bool {
        self.hovered || self.pressed || self.focused
    }
}

/// `hit_rounded` 的结果。命中判断使用 `clicked()` / `hovered()` /
/// `pressed()`；`response()` 返回未过滤的 egui 响应。
#[derive(Debug)]
pub struct Hit {
    response: Response,
    hovered: bool,
    pressed: bool,
    focused: bool,
    clicked: bool,
}

impl Hit {
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn hovered(&self) -> bool {
        self.hovered
    }

    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn interaction(&self) -> Interaction {
        Interaction {
            hovered: self.hovered,
            pressed: self.pressed,
            focused: self.focused,
        }
    }

    /// 原始 egui 响应。注意它的 `hovered()` / `clicked()` 仍是矩形语义，
    /// 不要直接拿来判断圆角形状。
    pub fn response(&self) -> &Response {
        &self.response
    }
}

/// 在 `rect` 上做圆角形状命中。
///
/// `radius` 应使用控件 rest 状态的圆角，而不是按压动画中的圆角；
/// 否则按压时命中形状会跟着变化。
pub fn hit_rounded(ui: &mut Ui, id: Id, rect: Rect, radius: CornerRadius, enabled: bool) -> Hit {
    let enabled = enabled && ui.is_enabled();
    let pointer_pos = ui.ctx().input(|i| i.pointer.interact_pos());
    let inside_now = pointer_pos.is_some_and(|p| rounded_rect_contains(rect, radius, p));

    // 记录“这次按下是否从形状内部开始”。释放时只看当前点不够，
    // 否则“角上按下、拖到中心释放”会误触发。
    let press_key = id.with("press_inside");
    if ui.ctx().input(|i| i.pointer.any_pressed()) {
        ui.ctx()
            .data_mut(|d| d.insert_temp(press_key, enabled && inside_now));
    }
    let press_inside = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(press_key))
        .unwrap_or(false);

    // 键盘焦点时也必须保持 click sense，否则 Enter/Space 的 fake click 会失效。
    let focused_before = ui.ctx().memory(|m| m.has_focus(id));
    let sense = if enabled && (inside_now || press_inside || focused_before) {
        Sense::click()
    } else {
        Sense::hover()
    };

    let response = ui.interact(rect, id, sense);

    let hovered = enabled && inside_now && response.hovered();
    let pressed = enabled && press_inside && response.is_pointer_button_down_on();
    let focused = enabled && response.has_focus();

    let mouse_clicked =
        enabled && response.clicked_by(PointerButton::Primary) && press_inside && inside_now;
    // fake click 来自键盘 Enter/Space 或 accesskit。
    let keyboard_clicked =
        enabled && response.clicked() && !response.clicked_by(PointerButton::Primary);

    if hovered {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    Hit {
        response,
        hovered,
        pressed,
        focused,
        clicked: mouse_clicked || keyboard_clicked,
    }
}

/// 点是否落在圆角矩形内部。
///
/// `CornerRadius` 是每个角的圆角半径。判定使用几何边界，
/// 不处理绘制时的抗锯齿边缘。
pub fn rounded_rect_contains(rect: Rect, radius: CornerRadius, p: Pos2) -> bool {
    if !rect.contains(p) {
        return false;
    }

    let max_r = 0.5 * rect.width().min(rect.height());
    let nw = (radius.nw as f32).min(max_r);
    let ne = (radius.ne as f32).min(max_r);
    let sw = (radius.sw as f32).min(max_r);
    let se = (radius.se as f32).min(max_r);

    if p.x < rect.left() + nw && p.y < rect.top() + nw {
        let center = Pos2::new(rect.left() + nw, rect.top() + nw);
        return (p - center).length_sq() <= nw * nw;
    }
    if p.x > rect.right() - ne && p.y < rect.top() + ne {
        let center = Pos2::new(rect.right() - ne, rect.top() + ne);
        return (p - center).length_sq() <= ne * ne;
    }
    if p.x < rect.left() + sw && p.y > rect.bottom() - sw {
        let center = Pos2::new(rect.left() + sw, rect.bottom() - sw);
        return (p - center).length_sq() <= sw * sw;
    }
    if p.x > rect.right() - se && p.y > rect.bottom() - se {
        let center = Pos2::new(rect.right() - se, rect.bottom() - se);
        return (p - center).length_sq() <= se * se;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn pill() -> (Rect, CornerRadius) {
        (
            Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 40.0)),
            CornerRadius::same(20),
        )
    }

    #[test]
    fn center_is_inside() {
        let (rect, radius) = pill();
        assert!(rounded_rect_contains(rect, radius, rect.center()));
    }

    #[test]
    fn box_corner_is_outside_pill() {
        let (rect, radius) = pill();
        assert!(!rounded_rect_contains(rect, radius, pos2(0.0, 0.0)));
        assert!(!rounded_rect_contains(rect, radius, pos2(99.0, 39.0)));
    }

    #[test]
    fn middle_edges_are_inside() {
        let (rect, radius) = pill();
        assert!(rounded_rect_contains(rect, radius, pos2(1.0, 20.0)));
        assert!(rounded_rect_contains(rect, radius, pos2(99.0, 20.0)));
        assert!(rounded_rect_contains(rect, radius, pos2(50.0, 1.0)));
    }

    #[test]
    fn zero_radius_is_plain_rect() {
        let (rect, _) = pill();
        assert!(rounded_rect_contains(
            rect,
            CornerRadius::ZERO,
            pos2(0.0, 0.0)
        ));
        assert!(rounded_rect_contains(
            rect,
            CornerRadius::ZERO,
            pos2(99.0, 39.0)
        ));
    }
}

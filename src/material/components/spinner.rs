//! M3 圆形 spinner。
//!
//! 来源:
//! - [`DSpinner.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DSpinner.qml): 尺寸、线宽、旋转、扫角与循环周期。
//! - [`Anims.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Anims.qml): `standard` 曲线。
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): 默认颜色与 spinner 线宽。
//!
//! `key` 必须是全局唯一且稳定的，动画状态从它派生。

use egui::{Color32, Id, Painter, Pos2, Rect, Response, Sense, Stroke, Vec2};

use crate::material::effects::motion::{self, Curve, MotionPrefs};
use crate::material::m3_ui::M3Ui;
use crate::material::tokens;

const ROTATION_SECONDS: f32 = 1.568;
const ARC_SECONDS: f32 = 0.666;
const CYCLE_SECONDS: f32 = ARC_SECONDS * 2.0;
const MIN_SWEEP_DEGREES: f32 = 16.0;
const MAX_SWEEP_DEGREES: f32 = 270.0;
const START_SHIFT_DEGREES: f32 = 254.0;

/// 圆形 spinner。
///
/// `.show()` 只分配尺寸并绘制，不处理点击。
///
/// ```ignore
/// let mut m = M3Ui::new(ui, &theme, MotionPrefs::DEFAULT);
/// m.spinner("loading").size(24.0).show(&mut m);
/// ```
pub struct Spinner<'a> {
    key: &'a str,
    size: f32,
    stroke_width: Option<f32>,
    color: Option<Color32>,
    running: bool,
    motion: Option<MotionPrefs>,
}

impl<'a> Spinner<'a> {
    /// 创建一个 spinner builder。
    pub fn new(key: &'a str) -> Self {
        Self {
            key,
            size: 48.0,
            stroke_width: None,
            color: None,
            running: true,
            motion: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(0.0);
        self
    }

    /// 覆盖线宽。默认是 `max(2, size / 12)`。
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width.max(0.0));
        self
    }

    /// 覆盖颜色。默认使用当前主题的 `primary`。
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// 是否运行。停止时显示 16 度静态弧。
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// 覆盖 [`M3Ui`] 提供的动效偏好。
    pub fn motion(mut self, motion: MotionPrefs) -> Self {
        self.motion = Some(motion);
        self
    }

    /// 绘制 spinner，返回布局响应。
    pub fn show(self, m: &mut M3Ui<'_>) -> Response {
        let prefs = self.motion.unwrap_or_else(|| m.motion());
        let color = self
            .color
            .unwrap_or_else(|| m.theme().scheme().primary.into());
        let id = Id::unique(self.key);
        let ui = m.ui();
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        let elapsed = motion::looping_seconds(ui, id, self.running && self.size > 0.0, prefs);
        paint(
            ui.painter(),
            rect,
            color,
            self.stroke_width
                .unwrap_or_else(|| default_stroke_width(self.size)),
            elapsed,
        );
        response
    }
}

fn default_stroke_width(size: f32) -> f32 {
    (size / 12.0).max(tokens::SPINNER_STROKE_WIDTH)
}

/// 在 `rect` 内绘制 spinner。供 [`crate::material::components::Button`] 的 icon 槽复用。
pub(super) fn paint(
    painter: &Painter,
    rect: Rect,
    color: Color32,
    stroke_width: f32,
    elapsed: Option<f32>,
) {
    let size = rect.width().min(rect.height());
    if size <= 0.0 || stroke_width <= 0.0 {
        return;
    }

    let radius = ((size - stroke_width).max(0.0) * 0.5).max(1.0);
    let (start_degrees, sweep_degrees) = arc_angles(elapsed.unwrap_or(0.0));
    paint_arc(
        painter,
        rect.center(),
        radius,
        stroke_width,
        start_degrees,
        sweep_degrees,
        color,
    );
}

/// 返回当前弧的起始角和扫角。
///
/// 来源: `DSpinner.qml` 的 1568ms 旋转、两段 666ms 扫角与
/// `ScriptAction` 的 254 度折返。
fn arc_angles(elapsed: f32) -> (f32, f32) {
    let rotation = (elapsed / ROTATION_SECONDS).rem_euclid(1.0) * 360.0;
    let cycles = (elapsed / CYCLE_SECONDS).floor();
    let phase = elapsed.rem_euclid(CYCLE_SECONDS) / CYCLE_SECONDS;

    let (start, sweep) = if phase < 0.5 {
        let progress = Curve::STANDARD.eval(phase * 2.0);
        (0.0, lerp(MIN_SWEEP_DEGREES, MAX_SWEEP_DEGREES, progress))
    } else {
        let progress = Curve::STANDARD.eval((phase - 0.5) * 2.0);
        (
            lerp(0.0, START_SHIFT_DEGREES, progress),
            lerp(MAX_SWEEP_DEGREES, MIN_SWEEP_DEGREES, progress),
        )
    };

    (
        (rotation + cycles * START_SHIFT_DEGREES + start).rem_euclid(360.0),
        sweep,
    )
}

/// 用折线绘制圆弧，并在两端补圆点模拟 RoundCap。
fn paint_arc(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    stroke_width: f32,
    start_degrees: f32,
    sweep_degrees: f32,
    color: Color32,
) {
    let sweep = sweep_degrees.to_radians();
    if sweep.abs() <= f32::EPSILON {
        return;
    }

    let arc_length = sweep.abs() * radius;
    let segments = (arc_length / 2.0).ceil().max(1.0) as usize;
    let start = start_degrees.to_radians();
    let mut points = Vec::with_capacity(segments + 1);
    for index in 0..=segments {
        let angle = start + sweep * (index as f32 / segments as f32);
        points.push(center + Vec2::angled(angle) * radius);
    }

    let first = points[0];
    let last = points[segments];
    painter.line(points, Stroke::new(stroke_width, color));
    painter.circle_filled(first, stroke_width * 0.5, color);
    painter.circle_filled(last, stroke_width * 0.5, color);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stroke_width_matches_source() {
        assert_eq!(default_stroke_width(24.0), 2.0);
        assert_eq!(default_stroke_width(48.0), 4.0);
        assert_eq!(default_stroke_width(0.0), 2.0);
    }

    #[test]
    fn stopped_spinner_shows_short_arc() {
        let (start, sweep) = arc_angles(0.0);
        assert_eq!(start, 0.0);
        assert_eq!(sweep, MIN_SWEEP_DEGREES);
    }

    #[test]
    fn half_cycle_reaches_max_sweep() {
        let (_, sweep) = arc_angles(ARC_SECONDS);
        assert!((sweep - MAX_SWEEP_DEGREES).abs() < 0.01);
    }

    #[test]
    fn cycle_boundary_keeps_start_angle_continuous() {
        let before = arc_angles(CYCLE_SECONDS - 0.0001).0;
        let after = arc_angles(CYCLE_SECONDS + 0.0001).0;
        let delta = (after - before + 180.0).rem_euclid(360.0) - 180.0;
        assert!(delta.abs() < 0.1);
    }
}

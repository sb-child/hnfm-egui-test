//! Material 3 / Expressive 动效。
//!
//! `Curve` 是可采样的 cubic bezier；`animate01` 推进 0..1 进度并应用曲线，
//! `looping_seconds` 提供 spinner 一类无限循环动画的时钟。
//!
//! 来源（dux-qml-common commit `c6e4a6b`）:
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): `expressiveCurves` / `expressiveDurations`。
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): 默认 `shorterDuration` + `standardDecel`。
//! - [`Theme.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/Common/Theme.qml): `shorterDuration` 实际值 100ms。
//! - [`DButton.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DButton.qml): 200ms `standard` 与 350ms `expressiveFastSpatial`。
//! - [`DRipple.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DRipple.qml): 500ms。

use egui::{Id, Ui};

/// 三次贝塞尔曲线，端点固定为 `(0,0)` 和 `(1,1)`。
///
/// 与 CSS `cubic-bezier(x1, y1, x2, y2)` 相同。`y` 可以大于 1，
/// 用于 Expressive 曲线的轻微过冲。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Curve {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Curve {
    pub const LINEAR: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    /// 来源: `Style.qml` 的 `expressiveCurves.standard`。
    pub const STANDARD: Self = Self::new(0.2, 0.0, 0.0, 1.0);

    /// 来源: `Style.qml` 的 `expressiveCurves.standardDecel`。
    pub const STANDARD_DECEL: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// 来源: `Style.qml` 的 `expressiveCurves.expressiveEffects`。
    pub const EXPRESSIVE_EFFECTS: Self = Self::new(0.34, 0.8, 0.34, 1.0);

    /// 来源: `Style.qml` 的 `expressiveCurves.expressiveFastSpatial`。
    pub const EXPRESSIVE_FAST_SPATIAL: Self = Self::new(0.42, 1.67, 0.21, 0.9);

    /// 来源: `Style.qml` 的 `expressiveCurves.expressiveDefaultSpatial`。
    pub const EXPRESSIVE_DEFAULT_SPATIAL: Self = Self::new(0.38, 1.21, 0.22, 1.0);

    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// 求 `x(t) = progress` 时的 `y`。
    ///
    /// 用二分法求参数 `s`。这些曲线的 `x` 在 `[0,1]` 内单调，
    /// 16 次迭代足够 UI 使用，不需要 Newton 迭代。
    pub fn eval(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        if progress <= 0.0 {
            return 0.0;
        }
        if progress >= 1.0 {
            return 1.0;
        }

        let mut lo = 0.0;
        let mut hi = 1.0;
        for _ in 0..16 {
            let mid = (lo + hi) * 0.5;
            if self.x(mid) < progress {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.y((lo + hi) * 0.5)
    }

    #[inline]
    fn x(self, s: f32) -> f32 {
        let inv = 1.0 - s;
        3.0 * inv * inv * s * self.x1 + 3.0 * inv * s * s * self.x2 + s * s * s
    }

    #[inline]
    fn y(self, s: f32) -> f32 {
        let inv = 1.0 - s;
        3.0 * inv * inv * s * self.y1 + 3.0 * inv * s * s * self.y2 + s * s * s
    }
}

/// 动效速度档位。
///
/// 来源: `Style.qml` 的 `AnimationSpeed` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationSpeed {
    None,
    #[default]
    Short,
    Medium,
    Long,
    Custom,
}

impl AnimationSpeed {
    fn enabled(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// 动效偏好。每帧由调用方显式传入。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionPrefs {
    pub reduce_motion: bool,
    pub speed: AnimationSpeed,
}

impl MotionPrefs {
    /// 正常动效。来源: `Style.qml` 的 `currentAnimationSpeed` 默认值。
    pub const DEFAULT: Self = Self {
        reduce_motion: false,
        speed: AnimationSpeed::Short,
    };

    /// 关闭所有动效。
    pub const NONE: Self = Self {
        reduce_motion: true,
        speed: AnimationSpeed::None,
    };

    /// 当前偏好是否允许动效。
    pub fn animations_enabled(self) -> bool {
        !self.reduce_motion && self.speed.enabled()
    }

    /// 按偏好返回实际使用的动效；关闭时返回 [`Motion::NONE`]。
    pub fn effective(self, motion: Motion) -> Motion {
        if self.animations_enabled() {
            motion
        } else {
            Motion::NONE
        }
    }
}

/// 把源文件里的毫秒值转换成秒。只在编译期使用。
const fn ms(milliseconds: f32) -> f32 {
    milliseconds / 1000.0
}

/// 一个动效 token：时长 + 曲线。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    /// 时长，单位秒。
    pub duration: f32,
    pub curve: Curve,
}

impl Motion {
    /// 无动画。测试、reduce motion 或“立即跳到目标”时使用。
    pub const NONE: Self = Self {
        duration: 0.0,
        curve: Curve::LINEAR,
    };

    /// State layer 的透明度过渡。来源: `Theme.qml` 的 `shorterDuration`。
    pub const SHORTER: Self = Self {
        duration: ms(100.0),
        curve: Curve::STANDARD_DECEL,
    };

    /// 颜色、展开等通用效果。来源: `Style.qml` 的 `expressiveDurations.expressiveEffects`。
    pub const EXPRESSIVE_EFFECTS: Self = Self {
        duration: ms(200.0),
        curve: Curve::EXPRESSIVE_EFFECTS,
    };

    /// 形状变化，例如按钮圆角在 rest 和 pressed 之间过渡。来源: `DButton.qml` 的 `Behavior on pressProgress`：200ms，standard 曲线。
    pub const SHAPE_CHANGE: Self = Self {
        duration: ms(200.0),
        curve: Curve::STANDARD,
    };

    /// 按钮按压缩放。来源: `Style.qml` 的 `expressiveDurations.expressiveFastSpatial`。
    pub const EXPRESSIVE_FAST_SPATIAL: Self = Self {
        duration: ms(350.0),
        curve: Curve::EXPRESSIVE_FAST_SPATIAL,
    };

    /// 较慢的空间过渡，例如 dialog / ripple。来源: `Style.qml` 的 `expressiveDurations.expressiveDefaultSpatial`。
    pub const EXPRESSIVE_DEFAULT_SPATIAL: Self = Self {
        duration: ms(500.0),
        curve: Curve::EXPRESSIVE_DEFAULT_SPATIAL,
    };
}

/// 把 0 或 1 的目标值推进成带曲线的当前值。
///
/// 只适合二值过渡。需要按进度插值像素值或缩放值时，先用
/// [`animate01`] 得到 0..1 进度再插值。`id` 必须稳定。
pub fn animate(ui: &Ui, id: Id, target: f32, motion: Motion, prefs: MotionPrefs) -> f32 {
    let motion = prefs.effective(motion);
    if motion.duration <= 0.0 {
        return target;
    }

    let linear = ui
        .ctx()
        .animate_value_with_time(id, target, motion.duration);
    motion.curve.eval(linear)
}

/// 把布尔目标推进成带曲线的 0..1 进度。
///
/// `id` 必须稳定，例如从组件 key 派生 `id.with("hover")`。
/// 反向过渡使用反向曲线，和 QML `Behavior` 的缓动方向一致。
pub fn animate01(ui: &Ui, id: Id, active: bool, motion: Motion, prefs: MotionPrefs) -> f32 {
    let motion = prefs.effective(motion);
    if motion.duration <= 0.0 {
        return if active { 1.0 } else { 0.0 };
    }

    let linear =
        ui.ctx()
            .animate_value_with_time(id, if active { 1.0 } else { 0.0 }, motion.duration);
    ease01(linear, active, motion.curve)
}

fn ease01(linear: f32, active: bool, curve: Curve) -> f32 {
    if active {
        curve.eval(linear)
    } else {
        1.0 - curve.eval(1.0 - linear)
    }
}

/// 返回无限循环动画已经运行的秒数。
///
/// `id` 必须稳定。`running` 变为 false 或动效关闭时会清零，
/// 再次运行从 0 开始。运行期间每帧请求重绘。
pub fn looping_seconds(ui: &Ui, id: Id, running: bool, prefs: MotionPrefs) -> Option<f32> {
    let start_id = id.with("loop_start");
    if !running || !prefs.animations_enabled() {
        ui.ctx().data_mut(|data| data.remove_temp::<f64>(start_id));
        return None;
    }

    let now = ui.input(|input| input.time);
    let start = ui
        .ctx()
        .data(|data| data.get_temp::<f64>(start_id))
        .unwrap_or_else(|| {
            ui.ctx().data_mut(|data| data.insert_temp(start_id, now));
            now
        });

    ui.ctx().request_repaint();
    Some((now - start).max(0.0) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_hits_endpoints() {
        for curve in [
            Curve::LINEAR,
            Curve::STANDARD,
            Curve::STANDARD_DECEL,
            Curve::EXPRESSIVE_EFFECTS,
            Curve::EXPRESSIVE_FAST_SPATIAL,
        ] {
            assert_eq!(curve.eval(0.0), 0.0);
            assert_eq!(curve.eval(1.0), 1.0);
        }
    }

    #[test]
    fn standard_curve_is_monotonic() {
        let mut previous = 0.0;
        for step in 0..=100 {
            let value = Curve::STANDARD.eval(step as f32 / 100.0);
            assert!(value >= previous - 1e-6);
            previous = value;
        }
    }

    #[test]
    fn durations_are_stored_in_seconds() {
        assert_eq!(Motion::NONE.duration, 0.0);
        assert!((Motion::SHORTER.duration - 0.1).abs() < 1e-6);
        assert!((Motion::EXPRESSIVE_EFFECTS.duration - 0.2).abs() < 1e-6);
        assert!((Motion::SHAPE_CHANGE.duration - 0.2).abs() < 1e-6);
        assert!((Motion::EXPRESSIVE_FAST_SPATIAL.duration - 0.35).abs() < 1e-6);
        assert!((Motion::EXPRESSIVE_DEFAULT_SPATIAL.duration - 0.5).abs() < 1e-6);
    }

    #[test]
    fn expressive_fast_spatial_can_overshoot() {
        let peak = (0..=100)
            .map(|step| Curve::EXPRESSIVE_FAST_SPATIAL.eval(step as f32 / 100.0))
            .fold(f32::MIN, f32::max);
        assert!(peak > 1.0);
    }

    #[test]
    fn reverse_transition_uses_reverse_curve() {
        assert_eq!(ease01(1.0, false, Curve::STANDARD), 1.0);
        assert_eq!(ease01(0.0, false, Curve::STANDARD), 0.0);
        assert!((ease01(0.5, false, Curve::LINEAR) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn looping_seconds_advances_and_resets() {
        let context = egui::Context::default();
        let id = Id::unique("spinner");

        let mut output = context.run_ui(
            egui::RawInput {
                time: Some(10.0),
                ..Default::default()
            },
            |ui| {
                assert_eq!(
                    looping_seconds(ui, id, true, MotionPrefs::DEFAULT),
                    Some(0.0)
                );
            },
        );
        output.textures_delta.clear();

        let mut output = context.run_ui(
            egui::RawInput {
                time: Some(10.5),
                ..Default::default()
            },
            |ui| {
                let elapsed = looping_seconds(ui, id, true, MotionPrefs::DEFAULT).unwrap();
                assert!((elapsed - 0.5).abs() < 1e-6);
            },
        );
        output.textures_delta.clear();

        let mut output = context.run_ui(
            egui::RawInput {
                time: Some(11.0),
                ..Default::default()
            },
            |ui| {
                assert_eq!(looping_seconds(ui, id, false, MotionPrefs::DEFAULT), None);
            },
        );
        output.textures_delta.clear();

        let mut output = context.run_ui(
            egui::RawInput {
                time: Some(12.0),
                ..Default::default()
            },
            |ui| {
                assert_eq!(
                    looping_seconds(ui, id, true, MotionPrefs::DEFAULT),
                    Some(0.0)
                );
            },
        );
        output.textures_delta.clear();
    }

    #[test]
    fn looping_seconds_respects_reduced_motion() {
        let context = egui::Context::default();
        let mut output = context.run_ui(Default::default(), |ui| {
            assert_eq!(
                looping_seconds(ui, Id::unique("spinner"), true, MotionPrefs::NONE),
                None
            );
        });
        output.textures_delta.clear();
    }

    #[test]
    fn default_prefs_keep_motion() {
        assert_eq!(
            MotionPrefs::DEFAULT.effective(Motion::SHAPE_CHANGE),
            Motion::SHAPE_CHANGE
        );
    }

    #[test]
    fn reduced_motion_disables_animation() {
        assert_eq!(
            MotionPrefs::NONE.effective(Motion::SHAPE_CHANGE),
            Motion::NONE
        );
        let prefs = MotionPrefs {
            reduce_motion: true,
            speed: AnimationSpeed::Short,
        };
        assert_eq!(prefs.effective(Motion::SHAPE_CHANGE), Motion::NONE);
    }

    #[test]
    fn speed_none_disables_animation() {
        let prefs = MotionPrefs {
            reduce_motion: false,
            speed: AnimationSpeed::None,
        };
        assert_eq!(prefs.effective(Motion::SHAPE_CHANGE), Motion::NONE);
    }
}

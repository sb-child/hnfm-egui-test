//! Material 3 / Expressive 动效。
//!
//! `Curve` 是可采样的 cubic bezier；`animate01` 推进 0..1 进度并应用曲线。
//!
//! 来源（dux-qml-common commit `c6e4a6b`）:
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): `expressiveCurves` / `expressiveDurations`。
//! - [`StateLayer.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/StateLayer.qml): 默认 50ms `standardDecel`。
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

    /// State layer 的透明度过渡。来源: `StateLayer.qml` 默认值。
    pub const SHORTER: Self = Self {
        duration: ms(50.0),
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

/// 把布尔目标推进成带曲线的 0..1 进度。
///
/// `id` 必须稳定，例如从组件 key 派生 `id.with("hover")`。
pub fn animate01(ui: &Ui, id: Id, active: bool, motion: Motion) -> f32 {
    let target = if active { 1.0 } else { 0.0 };
    if motion.duration <= 0.0 {
        return target;
    }

    let linear = ui
        .ctx()
        .animate_value_with_time(id, target, motion.duration);
    motion.curve.eval(linear)
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
        assert!((Motion::SHORTER.duration - 0.05).abs() < 1e-6);
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
}

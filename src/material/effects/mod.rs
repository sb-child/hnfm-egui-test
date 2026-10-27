//! 动效与绘制效果。
//!
//! 这里放可复用的效果机制；组件自己的动效策略留在 `components/`。

pub mod interpolate;
pub mod motion;
pub mod state_layer;

pub use motion::{AnimationSpeed, Curve, Motion, MotionPrefs};

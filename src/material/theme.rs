//! 显式主题上下文。
//!
//! 来源:
//! - [`Style.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Style.qml): 主题色 token。
//! - [`Theme.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/Common/Theme.qml): 应用主题覆盖。
//!
//! 组件从 [`M3Ui`](crate::material::M3Ui) 读取主题，不再查全局单例。

use crate::material::color::ThemeMode;
use crate::material::{MaterialColorTheme, Scheme};

/// 一帧内使用的颜色主题。
#[derive(Debug, Clone)]
pub struct Theme {
    scheme: Scheme,
}

impl Theme {
    pub fn new(scheme: Scheme) -> Self {
        Self { scheme }
    }

    /// 从完整主题里取出指定模式。
    pub fn from_color_theme(theme: &MaterialColorTheme, mode: ThemeMode) -> Self {
        Self {
            scheme: mode.scheme(theme).clone(),
        }
    }

    /// 当前颜色方案。
    pub fn scheme(&self) -> &Scheme {
        &self.scheme
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::color::{ThemeVariant, generate_theme};

    #[test]
    fn from_color_theme_matches_mode() {
        let full = generate_theme(ThemeVariant::Cmf {
            primary: 0xff769CDF,
            secondary: Some(0xff8991A2),
        });
        let theme = Theme::from_color_theme(&full, ThemeMode::Dark);
        assert_eq!(theme.scheme().surface, full.schemes.dark.surface);
    }
}

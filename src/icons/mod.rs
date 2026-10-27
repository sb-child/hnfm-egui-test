//! Material Symbols 图标字体与名字表。
//!
//! 来源:
//! - [`DIcon.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Widgets/DIcon.qml): `FILL` / `GRAD` / `opsz` / `wght` 轴，明暗模式的 grade。
//! - [`Fonts.qml`](https://github.com/duxlabs-dev/dux-qml-common/blob/c6e4a6bce24b804688225b0889410d2ec0ae06d7/DCommon/Common/Fonts.qml): 图标字体文件。
//!
//! 图标用 ligature 名作为文本排版，和 QML 里 `font.family: Fonts.icons` 的行为
//! 一致。图标字体是独立 family，不参与普通文本的 fallback，否则 `settings`
//! 这样的单词会被合成一个图标。
//!
//! 名字表由 [`download_icons.py`](../../download_icons.py) 生成，调用方只会
//! 接触到 [`Icon`] 常量，不接触字符串。
//!
//! 字体: google/material-design-icons 的 Material Symbols Rounded 可变字体，
//! Apache-2.0，见同目录 `LICENSE`。

use std::sync::Arc;

use egui::epaint::text::{FontInsert, FontPriority, FontTweak, InsertFontFamily, VariationCoords};
use egui::text::{Galley, LayoutJob, TextFormat};
use egui::{Color32, Context, FontData, FontFamily, FontId};

use crate::material::tokens;

mod generated;

use generated::ICON_NAMES;

/// 图标字体族名。和 sans / monospace 完全隔离。
const FAMILY: &str = "material-symbols";

/// 图标字体族。只有 [`layout_icon`] 会用到，调用方不需要关心。
pub fn ff_icons() -> FontFamily {
    FontFamily::Name(FAMILY.into())
}

/// 注册图标字体。在 [`crate::fonts::add_fonts`] 之后调用一次即可。
pub fn add_icon_font(ctx: &Context) {
    ctx.add_font(FontInsert::new(
        "MaterialSymbolsRounded",
        FontData::from_static(include_bytes!("MaterialSymbolsRounded.ttf")).tweak(FontTweak {
            // 字体级默认值只影响行高和未覆盖轴的排版；绘制时每个 run 都会
            // 重新给出完整坐标。
            coords: VariationCoords::new([
                (b"FILL", 0.0),
                (b"GRAD", 0.0),
                (b"opsz", tokens::ICON_OPSZ),
                (b"wght", tokens::FONT_WEIGHT_NORMAL),
            ]),
            ..Default::default()
        }),
        vec![InsertFontFamily {
            family: ff_icons(),
            priority: FontPriority::Highest,
        }],
    ));
}

/// 语义化图标名。
///
/// 只能由生成器给出的常量（例如 [`Icon::SETTINGS`]）或校验过的
/// [`Icon::from_name`] 创建，因此不会出现字体里不存在的名字。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Icon(&'static str);

impl Icon {
    /// 字体里的 ligature 名。
    pub const fn name(self) -> &'static str {
        self.0
    }

    /// 从配置、CLI 等动态来源构造。名字不存在时返回 `None`。
    pub fn from_name(name: &str) -> Option<Self> {
        ICON_NAMES
            .binary_search(&name)
            .ok()
            .map(|index| Self(ICON_NAMES[index]))
    }

    /// 全部图标名，按字典序排列。
    pub const ALL: &'static [&'static str] = generated::ICON_NAMES;
}

/// 图标的可变字体参数。
///
/// 对应 `DIcon.qml` 的 `filled` / `weight` / `grade`。`size` 是字号，也就是
/// 图标占据的方形边长。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconStyle {
    pub size: f32,
    /// `FILL` 轴：0 为描边，1 为实心。
    pub fill: f32,
    /// `wght` 轴。
    pub weight: f32,
    /// `GRAD` 轴：暗色模式用 -25 提升可读性。
    pub grade: f32,
}

impl IconStyle {
    /// 默认样式：`FILL = 0`，正常字重，`GRAD = 0`。
    pub fn new(size: f32) -> Self {
        Self {
            size,
            fill: 0.0,
            weight: tokens::FONT_WEIGHT_NORMAL,
            grade: 0.0,
        }
    }
}

/// 把图标名排版成一个字形。
///
/// 使用 `Color32::PLACEHOLDER`，实际颜色由绘制时的 fallback color 决定，
/// 这样同一份 galley 可以在不同颜色之间复用。
pub fn layout_icon(ctx: &Context, icon: Icon, style: IconStyle) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    job.append(
        icon.name(),
        0.0,
        TextFormat {
            font_id: FontId::new(style.size, ff_icons()),
            color: Color32::PLACEHOLDER,
            coords: VariationCoords::new([
                (b"FILL", style.fill),
                (b"GRAD", style.grade),
                (b"opsz", tokens::ICON_OPSZ),
                (b"wght", style.weight),
            ]),
            ..Default::default()
        },
    );
    job.wrap.max_width = f32::INFINITY;
    ctx.fonts_mut(|fonts| fonts.layout_job(job))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_sorted_and_unique() {
        assert!(Icon::ALL.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn from_name_round_trips_for_every_icon() {
        for name in Icon::ALL {
            assert_eq!(Icon::from_name(name).map(Icon::name), Some(*name));
        }
        assert_eq!(Icon::from_name("not_an_icon"), None);
    }

    /// 生成器用名字表而不是字体本身做校验，这里用排版兜底：每个名字都必须
    /// 被合成为单个字形。拼错或字体版本不匹配时会变成多个普通字母。
    ///
    /// epaint 会为 ligature 省掉的字符补零宽续接字形，让 glyph 数等于字符数，
    /// 所以这里只数真正有图集的字形。
    #[test]
    fn every_icon_renders_one_glyph() {
        let context = Context::default();
        add_icon_font(&context);
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            for name in Icon::ALL {
                let icon = Icon::from_name(name).expect("名字来自生成的表");
                let galley = layout_icon(ui.ctx(), icon, IconStyle::new(tokens::ICON_SIZE));
                let glyphs = galley
                    .rows
                    .iter()
                    .flat_map(|row| &row.glyphs)
                    .filter(|glyph| !glyph.uv_rect.is_nothing())
                    .count();
                assert_eq!(glyphs, 1, "图标 {name} 没有合成为单个字形");
            }
        });
        output.textures_delta.clear();
    }
}

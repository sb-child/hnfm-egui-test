use lazy_static::lazy_static;
use std::collections::BTreeMap;

use egui::epaint::text::{FontInsert, InsertFontFamily};
use egui::{FontFamily, FontId, TextStyle};

lazy_static! {
    static ref FF_SERIF: std::sync::Arc<str> = "serif".into();
    static ref TS_BODY_SANS: std::sync::Arc<str> = "style-body-sans".into();
    static ref TS_BODY_SERIF: std::sync::Arc<str> = "style-body-serif".into();
}

#[inline]
pub fn ts_body_sans() -> TextStyle {
    TextStyle::Name(TS_BODY_SANS.clone())
}

#[inline]
pub fn ts_body_serif() -> TextStyle {
    TextStyle::Name(TS_BODY_SERIF.clone())
}

#[inline]
pub fn ff_sans() -> FontFamily {
    FontFamily::Proportional
}

#[inline]
pub fn ff_serif() -> FontFamily {
    FontFamily::Name(FF_SERIF.clone())
}

#[inline]
pub fn ff_mono() -> FontFamily {
    FontFamily::Monospace
}

pub fn add_fonts(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "SourceHanSansSC",
        egui::FontData::from_static(include_bytes!("SourceHanSansSC-VF.otf")),
        vec![
            InsertFontFamily {
                family: ff_sans(),
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: ff_mono(),
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
    ctx.add_font(FontInsert::new(
        "SourceHanSerifSC",
        egui::FontData::from_static(include_bytes!("SourceHanSerifSC-VF.otf")),
        vec![
            InsertFontFamily {
                family: ff_serif(),
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: ff_mono(),
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
    ctx.add_font(FontInsert::new(
        "MapleMonoNL-NF-CN-Regular-unhinted",
        egui::FontData::from_static(include_bytes!("MapleMonoNL-NF-CN-Regular-unhinted.ttf")),
        vec![
            InsertFontFamily {
                family: ff_sans(),
                priority: egui::epaint::text::FontPriority::Lowest,
            },
            InsertFontFamily {
                family: ff_mono(),
                priority: egui::epaint::text::FontPriority::Highest,
            },
        ],
    ));
}

pub enum TextStyleOpt {
    Sans,
    Serif,
}

pub fn configure_text_styles(ctx: &egui::Context, opt: TextStyleOpt) {
    let main_ff = match opt {
        TextStyleOpt::Sans => ff_sans(),
        TextStyleOpt::Serif => ff_serif(),
    };
    let text_styles: BTreeMap<TextStyle, FontId> = [
        (TextStyle::Heading, FontId::new(25.0, main_ff.clone())),
        (TextStyle::Body, FontId::new(16.0, main_ff.clone())),
        (ts_body_sans(), FontId::new(16.0, ff_sans())),
        (ts_body_serif(), FontId::new(16.0, ff_serif())),
        (TextStyle::Monospace, FontId::new(12.0, ff_mono())),
        (TextStyle::Button, FontId::new(12.0, main_ff.clone())),
        (TextStyle::Small, FontId::new(8.0, main_ff.clone())),
    ]
    .into();
    ctx.all_styles_mut(move |style| style.text_styles = text_styles.clone());
}

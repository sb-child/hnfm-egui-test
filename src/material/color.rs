use lazy_static::lazy_static;
use material_color_utilities_rs::{Hct, Platform, SchemeCmf, SpecVersion};

use std::collections::BTreeMap;

use crate::material::{Color24, CoreColors, MaterialColorTheme, Palettes, Scheme, Schemes, Tone};

lazy_static! {
    static ref GLOBAL_SCHEME: std::sync::Arc<std::sync::RwLock<Option<crate::material::MaterialColorTheme>>> =
        std::sync::Arc::new(std::sync::RwLock::new(None));
    static ref GLOBAL_THEME_MODE: std::sync::Arc<std::sync::RwLock<ThemeMode>> =
        std::sync::Arc::new(std::sync::RwLock::new(ThemeMode::Light));
}

#[derive(Clone, Debug)]
pub enum ThemeMode {
    Light,
    LightMediumContrast,
    LightHighContrast,
    Dark,
    DarkMediumContrast,
    DarkHighContrast,
}

const ALL_TONES: [Tone; 18] = [
    Tone::T0,
    Tone::T5,
    Tone::T10,
    Tone::T15,
    Tone::T20,
    Tone::T25,
    Tone::T30,
    Tone::T35,
    Tone::T40,
    Tone::T50,
    Tone::T60,
    Tone::T70,
    Tone::T80,
    Tone::T90,
    Tone::T95,
    Tone::T98,
    Tone::T99,
    Tone::T100,
];

fn argb_to_color24(argb: u32) -> Color24 {
    let a = ((argb >> 24) & 0xFF) as u8;
    if a != 255 {
        eprintln!("WARNING: alpha={a} != 255 in argb={argb:#010X}");
    }
    Color24 {
        r: ((argb >> 16) & 0xFF) as u8,
        g: ((argb >> 8) & 0xFF) as u8,
        b: (argb & 0xFF) as u8,
    }
}

macro_rules! build_scheme {
    ($s:expr) => {
        Scheme {
            primary: argb_to_color24($s.primary()),
            surface_tint: argb_to_color24($s.surface_tint()),
            on_primary: argb_to_color24($s.on_primary()),
            primary_container: argb_to_color24($s.primary_container()),
            on_primary_container: argb_to_color24($s.on_primary_container()),
            secondary: argb_to_color24($s.secondary()),
            on_secondary: argb_to_color24($s.on_secondary()),
            secondary_container: argb_to_color24($s.secondary_container()),
            on_secondary_container: argb_to_color24($s.on_secondary_container()),
            tertiary: argb_to_color24($s.tertiary()),
            on_tertiary: argb_to_color24($s.on_tertiary()),
            tertiary_container: argb_to_color24($s.tertiary_container()),
            on_tertiary_container: argb_to_color24($s.on_tertiary_container()),
            error: argb_to_color24($s.error()),
            on_error: argb_to_color24($s.on_error()),
            error_container: argb_to_color24($s.error_container()),
            on_error_container: argb_to_color24($s.on_error_container()),
            background: argb_to_color24($s.background()),
            on_background: argb_to_color24($s.on_background()),
            surface: argb_to_color24($s.surface()),
            on_surface: argb_to_color24($s.on_surface()),
            surface_variant: argb_to_color24($s.surface_variant()),
            on_surface_variant: argb_to_color24($s.on_surface_variant()),
            outline: argb_to_color24($s.outline()),
            outline_variant: argb_to_color24($s.outline_variant()),
            shadow: argb_to_color24($s.shadow()),
            scrim: argb_to_color24($s.scrim()),
            inverse_surface: argb_to_color24($s.inverse_surface()),
            inverse_on_surface: argb_to_color24($s.inverse_on_surface()),
            inverse_primary: argb_to_color24($s.inverse_primary()),
            primary_fixed: argb_to_color24($s.primary_fixed()),
            on_primary_fixed: argb_to_color24($s.on_primary_fixed()),
            primary_fixed_dim: argb_to_color24($s.primary_fixed_dim()),
            on_primary_fixed_variant: argb_to_color24($s.on_primary_fixed_variant()),
            secondary_fixed: argb_to_color24($s.secondary_fixed()),
            on_secondary_fixed: argb_to_color24($s.on_secondary_fixed()),
            secondary_fixed_dim: argb_to_color24($s.secondary_fixed_dim()),
            on_secondary_fixed_variant: argb_to_color24($s.on_secondary_fixed_variant()),
            tertiary_fixed: argb_to_color24($s.tertiary_fixed()),
            on_tertiary_fixed: argb_to_color24($s.on_tertiary_fixed()),
            tertiary_fixed_dim: argb_to_color24($s.tertiary_fixed_dim()),
            on_tertiary_fixed_variant: argb_to_color24($s.on_tertiary_fixed_variant()),
            surface_dim: argb_to_color24($s.surface_dim()),
            surface_bright: argb_to_color24($s.surface_bright()),
            surface_container_lowest: argb_to_color24($s.surface_container_lowest()),
            surface_container_low: argb_to_color24($s.surface_container_low()),
            surface_container: argb_to_color24($s.surface_container()),
            surface_container_high: argb_to_color24($s.surface_container_high()),
            surface_container_highest: argb_to_color24($s.surface_container_highest()),
        }
    };
}

macro_rules! build_tonal_map {
    ($pal:expr) => {{
        let mut map = BTreeMap::new();
        for tone in &ALL_TONES {
            let hct = $pal.get_hct(*tone as u32 as i32);
            map.insert(*tone, argb_to_color24(hct.to_int()));
        }
        map
    }};
}

macro_rules! build_palettes {
    ($s:expr) => {
        Palettes {
            primary: build_tonal_map!($s.primary_palette),
            secondary: build_tonal_map!($s.secondary_palette),
            tertiary: build_tonal_map!($s.tertiary_palette),
            neutral: build_tonal_map!($s.neutral_palette),
            neutral_variant: build_tonal_map!($s.neutral_variant_palette),
        }
    };
}

pub fn generate_theme(primary_argb: u32, secondary_argb: Option<u32>) -> MaterialColorTheme {
    let primary = Hct::from_int(primary_argb);
    let secondary = secondary_argb.map_or(primary, Hct::from_int);

    let scheme_light = SchemeCmf::from_hcts_with_options(
        vec![primary, secondary],
        false,
        -1.0,
        Some(SpecVersion::Spec2026),
        Some(Platform::Phone),
    )
    .unwrap();
    let scheme_light_mc = SchemeCmf::from_hcts_with_options(
        vec![primary, secondary],
        false,
        0.0,
        Some(SpecVersion::Spec2026),
        Some(Platform::Phone),
    )
    .unwrap();
    let scheme_light_hc = SchemeCmf::from_hcts_with_options(
        vec![primary, secondary],
        false,
        1.0,
        Some(SpecVersion::Spec2026),
        Some(Platform::Phone),
    )
    .unwrap();
    let scheme_dark = SchemeCmf::from_hcts_with_options(
        vec![primary, secondary],
        true,
        -1.0,
        Some(SpecVersion::Spec2026),
        Some(Platform::Phone),
    )
    .unwrap();
    let scheme_dark_mc = SchemeCmf::from_hcts_with_options(
        vec![primary, secondary],
        true,
        0.0,
        Some(SpecVersion::Spec2026),
        Some(Platform::Phone),
    )
    .unwrap();
    let scheme_dark_hc = SchemeCmf::from_hcts_with_options(
        vec![primary, secondary],
        true,
        1.0,
        Some(SpecVersion::Spec2026),
        Some(Platform::Phone),
    )
    .unwrap();

    let theme = MaterialColorTheme {
        description: "".into(),
        seed: argb_to_color24(primary_argb),
        core_colors: CoreColors {
            primary: argb_to_color24(primary_argb),
        },
        extended_colors: vec![],
        schemes: Schemes {
            light: build_scheme!(scheme_light),
            light_medium_contrast: build_scheme!(scheme_light_mc),
            light_high_contrast: build_scheme!(scheme_light_hc),
            dark: build_scheme!(scheme_dark),
            dark_medium_contrast: build_scheme!(scheme_dark_mc),
            dark_high_contrast: build_scheme!(scheme_dark_hc),
        },
        palettes: build_palettes!(scheme_light),
    };
    theme
}

pub fn set_global_scheme(theme: MaterialColorTheme) {
    let mut t = GLOBAL_SCHEME.write().unwrap();
    t.replace(theme);
    drop(t);
}

pub fn set_global_theme_mode(m: ThemeMode) {
    let mut t = GLOBAL_THEME_MODE.write().unwrap();
    *t = m;
    drop(t);
}

pub fn access<F, R>(f: F) -> R
where
    F: FnOnce(&Palettes, &Scheme) -> R,
{
    let tm = GLOBAL_THEME_MODE.read().unwrap();
    let tm_copy = tm.clone();
    drop(tm);
    let r = GLOBAL_SCHEME.read().unwrap();
    let w = r.as_ref().unwrap();
    let p = &w.palettes;
    let s = match tm_copy {
        ThemeMode::Light => &w.schemes.light,
        ThemeMode::LightMediumContrast => &w.schemes.light_medium_contrast,
        ThemeMode::LightHighContrast => &w.schemes.light_high_contrast,
        ThemeMode::Dark => &w.schemes.dark,
        ThemeMode::DarkMediumContrast => &w.schemes.dark_medium_contrast,
        ThemeMode::DarkHighContrast => &w.schemes.dark_high_contrast,
    };
    let ret = f(p, s);
    drop(r);
    ret
}

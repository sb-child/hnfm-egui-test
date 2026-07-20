use lazy_static::lazy_static;
use material_color_utilities_rs::{
    DynamicScheme, Hct, SpecVersion, TonalPalette, Variant,
    dynamiccolor::dynamic_scheme::DynamicSchemeOptions,
};

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

pub enum ThemeVariant {
    Monochrome {
        seed: u32,
    },
    Neutral {
        seed: u32,
    },
    TonalSpot {
        seed: u32,
    },
    Vibrant {
        seed: u32,
    },
    Expressive {
        seed: u32,
    },
    Fidelity {
        seed: u32,
    },
    Content {
        seed: u32,
    },
    Rainbow {
        seed: u32,
    },
    FruitSalad {
        seed: u32,
    },
    Cmf {
        primary: u32,
        secondary: Option<u32>,
    },
}

impl ThemeVariant {
    fn to_variant(&self) -> Variant {
        match self {
            ThemeVariant::Monochrome { .. } => Variant::Monochrome,
            ThemeVariant::Neutral { .. } => Variant::Neutral,
            ThemeVariant::TonalSpot { .. } => Variant::TonalSpot,
            ThemeVariant::Vibrant { .. } => Variant::Vibrant,
            ThemeVariant::Expressive { .. } => Variant::Expressive,
            ThemeVariant::Fidelity { .. } => Variant::Fidelity,
            ThemeVariant::Content { .. } => Variant::Content,
            ThemeVariant::Rainbow { .. } => Variant::Rainbow,
            ThemeVariant::FruitSalad { .. } => Variant::FruitSalad,
            ThemeVariant::Cmf { .. } => Variant::Cmf,
        }
    }

    fn seed_argb(&self) -> u32 {
        match self {
            ThemeVariant::Cmf { primary, .. } => *primary,
            ThemeVariant::Monochrome { seed }
            | ThemeVariant::Neutral { seed }
            | ThemeVariant::TonalSpot { seed }
            | ThemeVariant::Vibrant { seed }
            | ThemeVariant::Expressive { seed }
            | ThemeVariant::Fidelity { seed }
            | ThemeVariant::Content { seed }
            | ThemeVariant::Rainbow { seed }
            | ThemeVariant::FruitSalad { seed } => *seed,
        }
    }

    fn source_hcts(&self) -> Vec<Hct> {
        match self {
            ThemeVariant::Cmf { primary, secondary } => {
                let p = Hct::from_int(*primary);
                let s = secondary.map_or(p, Hct::from_int);
                vec![p, s]
            }
            _ => vec![Hct::from_int(self.seed_argb())],
        }
    }
}

fn cmf_error_hue(primary_hue: f64, tertiary_hue: f64) -> f64 {
    if primary_hue <= 8.0 {
        if tertiary_hue <= 24.0 {
            28.0
        } else if tertiary_hue <= 32.0 {
            16.0
        } else {
            20.0
        }
    } else if primary_hue <= 16.0 {
        if tertiary_hue <= 24.0 {
            32.0
        } else if tertiary_hue <= 32.0 {
            20.0
        } else {
            24.0
        }
    } else if primary_hue <= 20.0 {
        if tertiary_hue <= 28.0 {
            32.0
        } else if tertiary_hue <= 32.0 {
            24.0
        } else {
            28.0
        }
    } else if primary_hue <= 28.0 {
        if tertiary_hue <= 24.0 { 32.0 } else { 16.0 }
    } else if primary_hue <= 32.0 {
        if tertiary_hue <= 20.0 {
            24.0
        } else if tertiary_hue <= 28.0 {
            16.0
        } else {
            20.0
        }
    } else if primary_hue <= 40.0 {
        if tertiary_hue > 20.0 && tertiary_hue <= 28.0 {
            16.0
        } else {
            24.0
        }
    } else if primary_hue <= 152.0 {
        if tertiary_hue > 24.0 && tertiary_hue <= 36.0 {
            20.0
        } else {
            32.0
        }
    } else if primary_hue <= 272.0 {
        if tertiary_hue > 20.0 && tertiary_hue <= 28.0 {
            16.0
        } else {
            24.0
        }
    } else if tertiary_hue > 12.0 && tertiary_hue <= 28.0 {
        32.0
    } else {
        16.0
    }
}

fn cmf_palettes(
    source_hcts: &[Hct],
) -> (
    TonalPalette,
    TonalPalette,
    TonalPalette,
    TonalPalette,
    TonalPalette,
    TonalPalette,
) {
    let source = source_hcts[0];
    let secondary_source = *source_hcts.get(1).unwrap_or(&source);

    let primary_palette = TonalPalette::from_hue_and_chroma(source.hue(), source.chroma());
    let secondary_palette = TonalPalette::from_hue_and_chroma(source.hue(), source.chroma() * 0.5);
    let tertiary_palette = if source.to_int() == secondary_source.to_int() {
        TonalPalette::from_hue_and_chroma(source.hue(), source.chroma() * 0.75)
    } else {
        TonalPalette::from_hue_and_chroma(secondary_source.hue(), secondary_source.chroma())
    };
    let neutral_palette = TonalPalette::from_hue_and_chroma(source.hue(), source.chroma() * 0.2);
    let neutral_variant_palette =
        TonalPalette::from_hue_and_chroma(source.hue(), source.chroma() * 0.2);
    let error_palette = TonalPalette::from_hue_and_chroma(
        cmf_error_hue(source.hue(), secondary_source.hue()),
        source.chroma().max(50.0),
    );

    (
        primary_palette,
        secondary_palette,
        tertiary_palette,
        neutral_palette,
        neutral_variant_palette,
        error_palette,
    )
}

pub fn generate_theme(tv: ThemeVariant) -> MaterialColorTheme {
    let variant_crate = tv.to_variant();
    let source_hcts = tv.source_hcts();
    let seed_argb = tv.seed_argb();

    let is_cmf = matches!(tv, ThemeVariant::Cmf { .. });

    let make_scheme = |is_dark: bool, contrast_level: f64| -> DynamicScheme {
        let (pp, sp, tp, np, nvp, ep) = if is_cmf {
            let pals = cmf_palettes(&source_hcts);
            (
                Some(pals.0),
                Some(pals.1),
                Some(pals.2),
                Some(pals.3),
                Some(pals.4),
                Some(pals.5),
            )
        } else {
            (None, None, None, None, None, None)
        };

        DynamicScheme::new(DynamicSchemeOptions {
            source_color_hct: None,
            source_color_hcts: Some(source_hcts.clone()),
            variant: variant_crate,
            contrast_level,
            is_dark,
            platform: None,
            spec_version: Some(SpecVersion::Spec2026),
            primary_palette: pp,
            secondary_palette: sp,
            tertiary_palette: tp,
            neutral_palette: np,
            neutral_variant_palette: nvp,
            error_palette: ep,
        })
        .unwrap()
    };

    let scheme_light = make_scheme(false, 0.0);
    let scheme_light_mc = make_scheme(false, 0.5);
    let scheme_light_hc = make_scheme(false, 1.0);
    let scheme_dark = make_scheme(true, 0.0);
    let scheme_dark_mc = make_scheme(true, 0.5);
    let scheme_dark_hc = make_scheme(true, 1.0);

    MaterialColorTheme {
        description: "".into(),
        seed: argb_to_color24(seed_argb),
        core_colors: CoreColors {
            primary: argb_to_color24(seed_argb),
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
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use material_color_utilities_rs::{
        SchemeCmf, SchemeContent, SchemeExpressive, SchemeFidelity, SchemeFruitSalad,
        SchemeMonochrome, SchemeNeutral, SchemeRainbow, SchemeTonalSpot, SchemeVibrant,
    };

    const TEST_SEED: u32 = 0xff769CDF;
    const TEST_SECONDARY: u32 = 0xff8991A2;

    fn get_scheme(theme: &MaterialColorTheme, is_dark: bool, contrast_level: f64) -> &Scheme {
        match (is_dark, contrast_level) {
            (false, x) if (x - 0.0).abs() < f64::EPSILON => &theme.schemes.light,
            (false, x) if (x - 0.5).abs() < f64::EPSILON => &theme.schemes.light_medium_contrast,
            (false, x) if (x - 1.0).abs() < f64::EPSILON => &theme.schemes.light_high_contrast,
            (true, x) if (x - 0.0).abs() < f64::EPSILON => &theme.schemes.dark,
            (true, x) if (x - 0.5).abs() < f64::EPSILON => &theme.schemes.dark_medium_contrast,
            (true, x) if (x - 1.0).abs() < f64::EPSILON => &theme.schemes.dark_high_contrast,
            _ => panic!("unexpected combination: is_dark={is_dark}, cl={contrast_level}"),
        }
    }

    macro_rules! assert_scheme_eq {
        ($our:expr, $ref:expr) => {
            assert_eq!(
                $our.primary,
                super::argb_to_color24($ref.primary()),
                "primary"
            );
            assert_eq!(
                $our.surface_tint,
                super::argb_to_color24($ref.surface_tint()),
                "surface_tint"
            );
            assert_eq!(
                $our.on_primary,
                super::argb_to_color24($ref.on_primary()),
                "on_primary"
            );
            assert_eq!(
                $our.primary_container,
                super::argb_to_color24($ref.primary_container()),
                "primary_container"
            );
            assert_eq!(
                $our.on_primary_container,
                super::argb_to_color24($ref.on_primary_container()),
                "on_primary_container"
            );
            assert_eq!(
                $our.secondary,
                super::argb_to_color24($ref.secondary()),
                "secondary"
            );
            assert_eq!(
                $our.on_secondary,
                super::argb_to_color24($ref.on_secondary()),
                "on_secondary"
            );
            assert_eq!(
                $our.secondary_container,
                super::argb_to_color24($ref.secondary_container()),
                "secondary_container"
            );
            assert_eq!(
                $our.on_secondary_container,
                super::argb_to_color24($ref.on_secondary_container()),
                "on_secondary_container"
            );
            assert_eq!(
                $our.tertiary,
                super::argb_to_color24($ref.tertiary()),
                "tertiary"
            );
            assert_eq!(
                $our.on_tertiary,
                super::argb_to_color24($ref.on_tertiary()),
                "on_tertiary"
            );
            assert_eq!(
                $our.tertiary_container,
                super::argb_to_color24($ref.tertiary_container()),
                "tertiary_container"
            );
            assert_eq!(
                $our.on_tertiary_container,
                super::argb_to_color24($ref.on_tertiary_container()),
                "on_tertiary_container"
            );
            assert_eq!($our.error, super::argb_to_color24($ref.error()), "error");
            assert_eq!(
                $our.on_error,
                super::argb_to_color24($ref.on_error()),
                "on_error"
            );
            assert_eq!(
                $our.error_container,
                super::argb_to_color24($ref.error_container()),
                "error_container"
            );
            assert_eq!(
                $our.on_error_container,
                super::argb_to_color24($ref.on_error_container()),
                "on_error_container"
            );
            assert_eq!(
                $our.background,
                super::argb_to_color24($ref.background()),
                "background"
            );
            assert_eq!(
                $our.on_background,
                super::argb_to_color24($ref.on_background()),
                "on_background"
            );
            assert_eq!(
                $our.surface,
                super::argb_to_color24($ref.surface()),
                "surface"
            );
            assert_eq!(
                $our.on_surface,
                super::argb_to_color24($ref.on_surface()),
                "on_surface"
            );
            assert_eq!(
                $our.surface_variant,
                super::argb_to_color24($ref.surface_variant()),
                "surface_variant"
            );
            assert_eq!(
                $our.on_surface_variant,
                super::argb_to_color24($ref.on_surface_variant()),
                "on_surface_variant"
            );
            assert_eq!(
                $our.outline,
                super::argb_to_color24($ref.outline()),
                "outline"
            );
            assert_eq!(
                $our.outline_variant,
                super::argb_to_color24($ref.outline_variant()),
                "outline_variant"
            );
            assert_eq!($our.shadow, super::argb_to_color24($ref.shadow()), "shadow");
            assert_eq!($our.scrim, super::argb_to_color24($ref.scrim()), "scrim");
            assert_eq!(
                $our.inverse_surface,
                super::argb_to_color24($ref.inverse_surface()),
                "inverse_surface"
            );
            assert_eq!(
                $our.inverse_on_surface,
                super::argb_to_color24($ref.inverse_on_surface()),
                "inverse_on_surface"
            );
            assert_eq!(
                $our.inverse_primary,
                super::argb_to_color24($ref.inverse_primary()),
                "inverse_primary"
            );
            assert_eq!(
                $our.primary_fixed,
                super::argb_to_color24($ref.primary_fixed()),
                "primary_fixed"
            );
            assert_eq!(
                $our.on_primary_fixed,
                super::argb_to_color24($ref.on_primary_fixed()),
                "on_primary_fixed"
            );
            assert_eq!(
                $our.primary_fixed_dim,
                super::argb_to_color24($ref.primary_fixed_dim()),
                "primary_fixed_dim"
            );
            assert_eq!(
                $our.on_primary_fixed_variant,
                super::argb_to_color24($ref.on_primary_fixed_variant()),
                "on_primary_fixed_variant"
            );
            assert_eq!(
                $our.secondary_fixed,
                super::argb_to_color24($ref.secondary_fixed()),
                "secondary_fixed"
            );
            assert_eq!(
                $our.on_secondary_fixed,
                super::argb_to_color24($ref.on_secondary_fixed()),
                "on_secondary_fixed"
            );
            assert_eq!(
                $our.secondary_fixed_dim,
                super::argb_to_color24($ref.secondary_fixed_dim()),
                "secondary_fixed_dim"
            );
            assert_eq!(
                $our.on_secondary_fixed_variant,
                super::argb_to_color24($ref.on_secondary_fixed_variant()),
                "on_secondary_fixed_variant"
            );
            assert_eq!(
                $our.tertiary_fixed,
                super::argb_to_color24($ref.tertiary_fixed()),
                "tertiary_fixed"
            );
            assert_eq!(
                $our.on_tertiary_fixed,
                super::argb_to_color24($ref.on_tertiary_fixed()),
                "on_tertiary_fixed"
            );
            assert_eq!(
                $our.tertiary_fixed_dim,
                super::argb_to_color24($ref.tertiary_fixed_dim()),
                "tertiary_fixed_dim"
            );
            assert_eq!(
                $our.on_tertiary_fixed_variant,
                super::argb_to_color24($ref.on_tertiary_fixed_variant()),
                "on_tertiary_fixed_variant"
            );
            assert_eq!(
                $our.surface_dim,
                super::argb_to_color24($ref.surface_dim()),
                "surface_dim"
            );
            assert_eq!(
                $our.surface_bright,
                super::argb_to_color24($ref.surface_bright()),
                "surface_bright"
            );
            assert_eq!(
                $our.surface_container_lowest,
                super::argb_to_color24($ref.surface_container_lowest()),
                "surface_container_lowest"
            );
            assert_eq!(
                $our.surface_container_low,
                super::argb_to_color24($ref.surface_container_low()),
                "surface_container_low"
            );
            assert_eq!(
                $our.surface_container,
                super::argb_to_color24($ref.surface_container()),
                "surface_container"
            );
            assert_eq!(
                $our.surface_container_high,
                super::argb_to_color24($ref.surface_container_high()),
                "surface_container_high"
            );
            assert_eq!(
                $our.surface_container_highest,
                super::argb_to_color24($ref.surface_container_highest()),
                "surface_container_highest"
            );
        };
    }

    const ALL_CONTRASTS: [(bool, f64); 6] = [
        (false, 0.0),
        (false, 0.5),
        (false, 1.0),
        (true, 0.0),
        (true, 0.5),
        (true, 1.0),
    ];

    macro_rules! test_variant {
        ($test_name:ident, $variant:ident, $scheme_type:ty) => {
            #[test]
            fn $test_name() {
                let tv = ThemeVariant::$variant { seed: TEST_SEED };
                let theme = generate_theme(tv);
                for &(is_dark, cl) in &ALL_CONTRASTS {
                    let ref_s = <$scheme_type>::from_hcts_with_options(
                        vec![Hct::from_int(TEST_SEED)],
                        is_dark,
                        cl,
                        Some(SpecVersion::Spec2026),
                        None,
                    )
                    .unwrap();
                    let our = get_scheme(&theme, is_dark, cl);
                    assert_scheme_eq!(our, &ref_s);
                }
            }
        };
    }

    test_variant!(monochrome_matches_reference, Monochrome, SchemeMonochrome);
    test_variant!(neutral_matches_reference, Neutral, SchemeNeutral);
    test_variant!(tonal_spot_matches_reference, TonalSpot, SchemeTonalSpot);
    test_variant!(vibrant_matches_reference, Vibrant, SchemeVibrant);
    test_variant!(expressive_matches_reference, Expressive, SchemeExpressive);
    test_variant!(fidelity_matches_reference, Fidelity, SchemeFidelity);
    test_variant!(content_matches_reference, Content, SchemeContent);
    test_variant!(rainbow_matches_reference, Rainbow, SchemeRainbow);
    test_variant!(fruit_salad_matches_reference, FruitSalad, SchemeFruitSalad);

    #[test]
    fn cmf_matches_reference() {
        let tv = ThemeVariant::Cmf {
            primary: TEST_SEED,
            secondary: Some(TEST_SECONDARY),
        };
        let theme = generate_theme(tv);
        for &(is_dark, cl) in &ALL_CONTRASTS {
            let ref_s = SchemeCmf::from_hcts_with_options(
                vec![Hct::from_int(TEST_SEED), Hct::from_int(TEST_SECONDARY)],
                is_dark,
                cl,
                Some(SpecVersion::Spec2026),
                None,
            )
            .unwrap();
            let our = get_scheme(&theme, is_dark, cl);
            assert_scheme_eq!(our, &ref_s);
        }
    }
}

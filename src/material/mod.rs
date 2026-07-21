pub mod color;
pub mod icon_button;
pub mod list_item;
pub mod nav_rail_item;
pub mod ripple;
pub mod util;

pub use icon_button::IconButton;
pub use list_item::ListItem;
pub use nav_rail_item::NavRailItem;

use egui::Color32;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color24 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Into<Color32> for Color24 {
    fn into(self) -> Color32 {
        Color32::from_rgb(self.r, self.g, self.b)
    }
}

impl Color24 {
    pub fn with_alpha_u8(&self, a: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(self.r, self.g, self.b, a)
    }

    pub fn with_alpha_f32(&self, a: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(self.r, self.g, self.b, (a * 255.) as u8)
    }
}

impl<'de> Deserialize<'de> for Color24 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if !s.starts_with('#') || s.len() != 7 {
            return Err(serde::de::Error::custom(format!(
                "Invalid color format: '{}'. Expected '#RRGGBB'",
                s
            )));
        }

        let r = u8::from_str_radix(&s[1..3], 16).map_err(serde::de::Error::custom)?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(serde::de::Error::custom)?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(serde::de::Error::custom)?;

        Ok(Color24 { r, g, b })
    }
}

impl Serialize for Color24 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b);
        serializer.serialize_str(&s)
    }
}

/// Material Design 色阶 (Tonal Palette) 标准阶度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tone {
    T0 = 0,
    T5 = 5,
    T10 = 10,
    T15 = 15,
    T20 = 20,
    T25 = 25,
    T30 = 30,
    T35 = 35,
    T40 = 40,
    T50 = 50,
    T60 = 60,
    T70 = 70,
    T80 = 80,
    T90 = 90,
    T95 = 95,
    T98 = 98,
    T99 = 99,
    T100 = 100,
}

impl<'de> Deserialize<'de> for Tone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(Tone::T0),
            "5" => Ok(Tone::T5),
            "10" => Ok(Tone::T10),
            "15" => Ok(Tone::T15),
            "20" => Ok(Tone::T20),
            "25" => Ok(Tone::T25),
            "30" => Ok(Tone::T30),
            "35" => Ok(Tone::T35),
            "40" => Ok(Tone::T40),
            "50" => Ok(Tone::T50),
            "60" => Ok(Tone::T60),
            "70" => Ok(Tone::T70),
            "80" => Ok(Tone::T80),
            "90" => Ok(Tone::T90),
            "95" => Ok(Tone::T95),
            "98" => Ok(Tone::T98),
            "99" => Ok(Tone::T99),
            "100" => Ok(Tone::T100),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown tone value: {}",
                s
            ))),
        }
    }
}

impl Serialize for Tone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = *self as u32;
        serializer.serialize_str(&val.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CoreColors {
    pub primary: Color24,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Scheme {
    pub primary: Color24,
    pub surface_tint: Color24,
    pub on_primary: Color24,
    pub primary_container: Color24,
    pub on_primary_container: Color24,
    pub secondary: Color24,
    pub on_secondary: Color24,
    pub secondary_container: Color24,
    pub on_secondary_container: Color24,
    pub tertiary: Color24,
    pub on_tertiary: Color24,
    pub tertiary_container: Color24,
    pub on_tertiary_container: Color24,
    pub error: Color24,
    pub on_error: Color24,
    pub error_container: Color24,
    pub on_error_container: Color24,
    pub background: Color24,
    pub on_background: Color24,
    pub surface: Color24,
    pub on_surface: Color24,
    pub surface_variant: Color24,
    pub on_surface_variant: Color24,
    pub outline: Color24,
    pub outline_variant: Color24,
    pub shadow: Color24,
    pub scrim: Color24,
    pub inverse_surface: Color24,
    pub inverse_on_surface: Color24,
    pub inverse_primary: Color24,
    pub primary_fixed: Color24,
    pub on_primary_fixed: Color24,
    pub primary_fixed_dim: Color24,
    pub on_primary_fixed_variant: Color24,
    pub secondary_fixed: Color24,
    pub on_secondary_fixed: Color24,
    pub secondary_fixed_dim: Color24,
    pub on_secondary_fixed_variant: Color24,
    pub tertiary_fixed: Color24,
    pub on_tertiary_fixed: Color24,
    pub tertiary_fixed_dim: Color24,
    pub on_tertiary_fixed_variant: Color24,
    pub surface_dim: Color24,
    pub surface_bright: Color24,
    pub surface_container_lowest: Color24,
    pub surface_container_low: Color24,
    pub surface_container: Color24,
    pub surface_container_high: Color24,
    pub surface_container_highest: Color24,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schemes {
    pub light: Scheme,
    pub light_medium_contrast: Scheme,
    pub light_high_contrast: Scheme,
    pub dark: Scheme,
    pub dark_medium_contrast: Scheme,
    pub dark_high_contrast: Scheme,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Palettes {
    pub primary: BTreeMap<Tone, Color24>,
    pub secondary: BTreeMap<Tone, Color24>,
    pub tertiary: BTreeMap<Tone, Color24>,
    pub neutral: BTreeMap<Tone, Color24>,
    pub neutral_variant: BTreeMap<Tone, Color24>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaterialColorTheme {
    pub description: String,
    pub seed: Color24,
    pub core_colors: CoreColors,
    pub extended_colors: Vec<serde_json::Value>,
    pub schemes: Schemes,
    pub palettes: Palettes,
}

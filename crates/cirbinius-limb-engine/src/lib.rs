pub mod engines;
pub mod limb_ops;
pub mod modular;

use std::fmt::Debug;

pub trait LimbEngine: Debug + Clone + Send + Sync {
    type Limb: Copy + Debug + PartialEq + Eq;
    fn bit_width() -> u32;
    fn name() -> &'static str;
    fn zero() -> Self::Limb;
    fn one() -> Self::Limb;
    fn add(a: Self::Limb, b: Self::Limb) -> (Self::Limb, bool);
    fn sub(a: Self::Limb, b: Self::Limb) -> (Self::Limb, bool);
    fn widemul(a: Self::Limb, b: Self::Limb) -> (Self::Limb, Self::Limb);
    fn from_u64(v: u64) -> Self::Limb;
    fn to_u64(v: Self::Limb) -> u64;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LimbWidth {
    U8,
    U16,
    U32,
    U64,
    #[serde(rename = "auto")]
    #[default]
    Auto,
}

impl LimbWidth {
    pub fn bit_width(&self) -> u32 {
        match self {
            LimbWidth::U8 => 8,
            LimbWidth::U16 => 16,
            LimbWidth::U32 => 32,
            LimbWidth::U64 => 64,
            LimbWidth::Auto => 32,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LimbWidth::U8 => "u8",
            LimbWidth::U16 => "u16",
            LimbWidth::U32 => "u32",
            LimbWidth::U64 => "u64",
            LimbWidth::Auto => "auto",
        }
    }
}

impl std::str::FromStr for LimbWidth {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "u8" => Ok(LimbWidth::U8),
            "u16" => Ok(LimbWidth::U16),
            "u32" => Ok(LimbWidth::U32),
            "u64" => Ok(LimbWidth::U64),
            "auto" => Ok(LimbWidth::Auto),
            _ => Err(format!(
                "unknown LimbWidth: {s} (expected u8/u16/u32/u64/auto)"
            )),
        }
    }
}

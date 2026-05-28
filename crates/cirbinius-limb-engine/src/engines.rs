use crate::LimbEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct U32Engine;

impl LimbEngine for U32Engine {
    type Limb = u32;
    fn bit_width() -> u32 {
        32
    }
    fn name() -> &'static str {
        "u32"
    }
    fn zero() -> u32 {
        0
    }
    fn one() -> u32 {
        1
    }
    fn add(a: u32, b: u32) -> (u32, bool) {
        let (sum, carry) = a.overflowing_add(b);
        (sum, carry)
    }
    fn sub(a: u32, b: u32) -> (u32, bool) {
        let (diff, borrow) = a.overflowing_sub(b);
        (diff, borrow)
    }
    fn widemul(a: u32, b: u32) -> (u32, u32) {
        let result = (a as u64) * (b as u64);
        ((result & 0xFFFF_FFFF) as u32, (result >> 32) as u32)
    }
    fn from_u64(v: u64) -> u32 {
        v as u32
    }
    fn to_u64(v: u32) -> u64 {
        v as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct U64Engine;

impl LimbEngine for U64Engine {
    type Limb = u64;
    fn bit_width() -> u32 {
        64
    }
    fn name() -> &'static str {
        "u64"
    }
    fn zero() -> u64 {
        0
    }
    fn one() -> u64 {
        1
    }
    fn add(a: u64, b: u64) -> (u64, bool) {
        a.overflowing_add(b)
    }
    fn sub(a: u64, b: u64) -> (u64, bool) {
        a.overflowing_sub(b)
    }
    fn widemul(a: u64, b: u64) -> (u64, u64) {
        let result = (a as u128) * (b as u128);
        (
            (result & 0xFFFF_FFFF_FFFF_FFFF) as u64,
            (result >> 64) as u64,
        )
    }
    fn from_u64(v: u64) -> u64 {
        v
    }
    fn to_u64(v: u64) -> u64 {
        v
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct U16Engine;

impl LimbEngine for U16Engine {
    type Limb = u16;
    fn bit_width() -> u32 {
        16
    }
    fn name() -> &'static str {
        "u16"
    }
    fn zero() -> u16 {
        0
    }
    fn one() -> u16 {
        1
    }
    fn add(a: u16, b: u16) -> (u16, bool) {
        a.overflowing_add(b)
    }
    fn sub(a: u16, b: u16) -> (u16, bool) {
        a.overflowing_sub(b)
    }
    fn widemul(a: u16, b: u16) -> (u16, u16) {
        let result = (a as u32) * (b as u32);
        ((result & 0xFFFF) as u16, (result >> 16) as u16)
    }
    fn from_u64(v: u64) -> u16 {
        v as u16
    }
    fn to_u64(v: u16) -> u64 {
        v as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct U8Engine;

impl LimbEngine for U8Engine {
    type Limb = u8;
    fn bit_width() -> u32 {
        8
    }
    fn name() -> &'static str {
        "u8"
    }
    fn zero() -> u8 {
        0
    }
    fn one() -> u8 {
        1
    }
    fn add(a: u8, b: u8) -> (u8, bool) {
        a.overflowing_add(b)
    }
    fn sub(a: u8, b: u8) -> (u8, bool) {
        a.overflowing_sub(b)
    }
    fn widemul(a: u8, b: u8) -> (u8, u8) {
        let result = (a as u16) * (b as u16);
        ((result & 0xFF) as u8, (result >> 8) as u8)
    }
    fn from_u64(v: u64) -> u8 {
        v as u8
    }
    fn to_u64(v: u8) -> u64 {
        v as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_add_no_carry() {
        let (sum, carry) = U32Engine::add(5, 3);
        assert_eq!(sum, 8);
        assert!(!carry);
    }

    #[test]
    fn u32_add_with_carry() {
        let (sum, carry) = U32Engine::add(u32::MAX, 1);
        assert_eq!(sum, 0);
        assert!(carry);
    }

    #[test]
    fn u32_widemul_basic() {
        let (lo, hi) = U32Engine::widemul(100_000, 200_000);
        let reconstructed = (hi as u64) << 32 | lo as u64;
        assert_eq!(reconstructed, 20_000_000_000u64);
    }

    #[test]
    fn u64_widemul_basic() {
        let a: u64 = 1_000_000_000;
        let b: u64 = 2_000_000_000;
        let (lo, hi) = U64Engine::widemul(a, b);
        let reconstructed = (hi as u128) << 64 | lo as u128;
        assert_eq!(reconstructed, 2_000_000_000_000_000_000u128);
    }

    #[test]
    fn u16_roundtrip() {
        let a: u16 = 255;
        let b: u16 = 1;
        let (sum, carry) = U16Engine::add(a, b);
        assert_eq!(sum, 256);
        assert!(!carry);
    }

    #[test]
    fn u8_roundtrip() {
        let a: u8 = 200;
        let b: u8 = 100;
        let (sum, carry) = U8Engine::add(a, b);
        assert_eq!(sum, 44);
        assert!(carry);
    }
}

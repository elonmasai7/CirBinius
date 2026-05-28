use anyhow::Result;
use num_bigint::BigUint;
use num_traits::Zero;

use crate::LimbEngine;
use crate::limb_ops::{add_limbs, mul_limbs};

pub fn biguint_to_limbs<E: LimbEngine>(value: &BigUint, num_limbs: usize) -> Vec<E::Limb> {
    let limb_mask = if E::bit_width() < 64 {
        BigUint::from((1u64 << E::bit_width()) - 1)
    } else {
        BigUint::from(u64::MAX)
    };
    let mut limbs = Vec::with_capacity(num_limbs);
    let mut remaining = value.clone();
    for _ in 0..num_limbs {
        let chunk = &remaining & &limb_mask;
        let chunk_u64 = chunk.to_u64_digits().first().copied().unwrap_or(0);
        limbs.push(E::from_u64(chunk_u64));
        remaining >>= E::bit_width();
    }
    limbs
}

pub fn limbs_to_biguint<E: LimbEngine>(limbs: &[E::Limb]) -> BigUint {
    let mut result = BigUint::zero();
    for (i, &limb) in limbs.iter().enumerate() {
        let val = BigUint::from(E::to_u64(limb));
        result += val << (i * E::bit_width() as usize);
    }
    result
}

pub fn fm_add<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb], modulus: &[E::Limb]) -> Vec<E::Limb> {
    let sum = add_limbs::<E>(a, b);
    let mod_biguint = limbs_to_biguint::<E>(modulus);
    let sum_biguint = limbs_to_biguint::<E>(&sum);
    let result = if sum_biguint >= mod_biguint {
        sum_biguint - mod_biguint
    } else {
        sum_biguint
    };
    biguint_to_limbs::<E>(&result, modulus.len())
}

pub fn fm_sub<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb], modulus: &[E::Limb]) -> Vec<E::Limb> {
    let a_biguint = limbs_to_biguint::<E>(a);
    let b_biguint = limbs_to_biguint::<E>(b);
    let mod_biguint = limbs_to_biguint::<E>(modulus);
    let result = if a_biguint >= b_biguint {
        a_biguint - b_biguint
    } else {
        mod_biguint.clone() - (b_biguint - a_biguint) % &mod_biguint
    };
    biguint_to_limbs::<E>(&result, modulus.len())
}

pub fn fm_mul<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb], modulus: &[E::Limb]) -> Vec<E::Limb> {
    let prod = mul_limbs::<E>(a, b);
    let prod_biguint = limbs_to_biguint::<E>(&prod);
    let mod_biguint = limbs_to_biguint::<E>(modulus);
    let result = prod_biguint % mod_biguint;
    biguint_to_limbs::<E>(&result, modulus.len())
}

pub fn fm_neg<E: LimbEngine>(a: &[E::Limb], modulus: &[E::Limb]) -> Vec<E::Limb> {
    let a_biguint = limbs_to_biguint::<E>(a);
    let mod_biguint = limbs_to_biguint::<E>(modulus);
    // neg(a) = (modulus - a) % modulus
    if a_biguint.is_zero() {
        biguint_to_limbs::<E>(&BigUint::zero(), modulus.len())
    } else {
        let result = &mod_biguint - a_biguint;
        biguint_to_limbs::<E>(&result, modulus.len())
    }
}

pub fn fm_is_zero<E: LimbEngine>(a: &[E::Limb]) -> bool {
    a.iter().all(|&limb| limb == E::zero())
}

pub fn fm_equal<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb], modulus: &[E::Limb]) -> bool {
    let a_biguint = limbs_to_biguint::<E>(a) % limbs_to_biguint::<E>(modulus);
    let b_biguint = limbs_to_biguint::<E>(b) % limbs_to_biguint::<E>(modulus);
    a_biguint == b_biguint
}

pub fn parse_modulus_from_hex(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)?;
    Ok(bytes)
}

pub fn bn254_modulus_limbs<E: LimbEngine>() -> Vec<E::Limb> {
    let bn254_hex = "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47";
    let modulus_biguint = BigUint::parse_bytes(bn254_hex.as_bytes(), 16).expect("valid BN254 hex");
    let num_limbs = 254_u32.div_ceil(E::bit_width()) as usize;
    biguint_to_limbs::<E>(&modulus_biguint, num_limbs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::U32Engine;

    #[test]
    fn fm_add_basic() {
        let modulus = bn254_modulus_limbs::<U32Engine>();
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(5u64), modulus.len());
        let b = biguint_to_limbs::<U32Engine>(&BigUint::from(7u64), modulus.len());
        let result = fm_add::<U32Engine>(&a, &b, &modulus);
        let result_bn = limbs_to_biguint::<U32Engine>(&result);
        assert_eq!(result_bn, BigUint::from(12u64));
    }

    #[test]
    fn fm_sub_basic() {
        let modulus = bn254_modulus_limbs::<U32Engine>();
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(10u64), modulus.len());
        let b = biguint_to_limbs::<U32Engine>(&BigUint::from(3u64), modulus.len());
        let result = fm_sub::<U32Engine>(&a, &b, &modulus);
        let result_bn = limbs_to_biguint::<U32Engine>(&result);
        assert_eq!(result_bn, BigUint::from(7u64));
    }

    #[test]
    fn fm_mul_basic() {
        let modulus = bn254_modulus_limbs::<U32Engine>();
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(6u64), modulus.len());
        let b = biguint_to_limbs::<U32Engine>(&BigUint::from(7u64), modulus.len());
        let result = fm_mul::<U32Engine>(&a, &b, &modulus);
        let result_bn = limbs_to_biguint::<U32Engine>(&result);
        assert_eq!(result_bn, BigUint::from(42u64));
    }

    #[test]
    fn fm_mul_modular_reduction() {
        let modulus = BigUint::from(13u64);
        let mod_limbs = biguint_to_limbs::<U32Engine>(&modulus, 1);
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(10u64), 1);
        let b = biguint_to_limbs::<U32Engine>(&BigUint::from(10u64), 1);
        let result = fm_mul::<U32Engine>(&a, &b, &mod_limbs);
        let result_bn = limbs_to_biguint::<U32Engine>(&result);
        assert_eq!(result_bn, BigUint::from(9u64)); // 10*10 = 100 ≡ 9 (mod 13)
    }

    #[test]
    fn fm_neg_basic() {
        let modulus = BigUint::from(13u64);
        let mod_limbs = biguint_to_limbs::<U32Engine>(&modulus, 1);
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(5u64), 1);
        let result = fm_neg::<U32Engine>(&a, &mod_limbs);
        let result_bn = limbs_to_biguint::<U32Engine>(&result);
        assert_eq!(result_bn, BigUint::from(8u64)); // 13-5 = 8
    }

    #[test]
    fn fm_equal_true() {
        let modulus = bn254_modulus_limbs::<U32Engine>();
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(42u64), modulus.len());
        let b = biguint_to_limbs::<U32Engine>(&BigUint::from(42u64), modulus.len());
        assert!(fm_equal::<U32Engine>(&a, &b, &modulus));
    }

    #[test]
    fn fm_equal_false() {
        let modulus = bn254_modulus_limbs::<U32Engine>();
        let a = biguint_to_limbs::<U32Engine>(&BigUint::from(42u64), modulus.len());
        let b = biguint_to_limbs::<U32Engine>(&BigUint::from(43u64), modulus.len());
        assert!(!fm_equal::<U32Engine>(&a, &b, &modulus));
    }
}

use crate::LimbEngine;

pub fn add_limbs<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb]) -> Vec<E::Limb> {
    let max_len = a.len().max(b.len());
    let mut result = Vec::with_capacity(max_len + 1);
    let mut carry = false;
    for i in 0..max_len {
        let ai = if i < a.len() { a[i] } else { E::zero() };
        let bi = if i < b.len() { b[i] } else { E::zero() };
        let (sum, c1) = E::add(ai, bi);
        let (sum_with_carry, c2) = if carry {
            E::add(sum, E::one())
        } else {
            (sum, false)
        };
        carry = c1 || c2;
        result.push(sum_with_carry);
    }
    if carry {
        result.push(E::one());
    }
    result
}

pub fn sub_limbs<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb]) -> Vec<E::Limb> {
    let max_len = a.len().max(b.len());
    let mut result = Vec::with_capacity(max_len);
    let mut borrow = false;
    for i in 0..max_len {
        let ai = if i < a.len() { a[i] } else { E::zero() };
        let bi = if i < b.len() { b[i] } else { E::zero() };
        let (diff, c1) = E::sub(ai, bi);
        let (diff_with_borrow, c2) = if borrow {
            E::sub(diff, E::one())
        } else {
            (diff, false)
        };
        borrow = c1 || c2;
        result.push(diff_with_borrow);
    }
    result
}

pub fn mul_limbs<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb]) -> Vec<E::Limb> {
    let mut result = vec![E::zero(); a.len() + b.len()];
    for (i, &ai) in a.iter().enumerate() {
        let mut carry = E::zero();
        for (j, &bj) in b.iter().enumerate() {
            let (lo, hi) = E::widemul(ai, bj);
            let (s1, c1) = E::add(result[i + j], lo);
            let (s2, c2) = E::add(s1, carry);
            result[i + j] = s2;
            carry = if c1 || c2 {
                E::from_u64(1 + E::to_u64(hi))
            } else {
                hi
            };
        }
        result[i + b.len()] = E::add(result[i + b.len()], carry).0;
    }
    result
}

pub fn limbs_from_u64<E: LimbEngine>(value: u64, num_limbs: usize) -> Vec<E::Limb> {
    let limb_mask = if E::bit_width() < 64 {
        (1u64 << E::bit_width()) - 1
    } else {
        u64::MAX
    };
    let mut limbs = Vec::with_capacity(num_limbs);
    let mut remaining = value;
    for _ in 0..num_limbs {
        let chunk = remaining & limb_mask;
        limbs.push(E::from_u64(chunk));
        remaining >>= E::bit_width();
    }
    limbs
}

pub fn limbs_to_u64<E: LimbEngine>(limbs: &[E::Limb]) -> u64 {
    let mut result = 0u64;
    for (i, &limb) in limbs.iter().enumerate() {
        let val: u64 = E::to_u64(limb);
        let shift = i * E::bit_width() as usize;
        if shift >= 64 {
            assert!(val == 0, "value too large for u64");
            continue;
        }
        result |= val << shift;
    }
    result
}

pub fn is_less_than<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb]) -> bool {
    let max_len = a.len().max(b.len());
    for i in (0..max_len).rev() {
        let ai = if i < a.len() { E::to_u64(a[i]) } else { 0 };
        let bi = if i < b.len() { E::to_u64(b[i]) } else { 0 };
        if ai < bi {
            return true;
        } else if ai > bi {
            return false;
        }
    }
    false
}

pub fn is_zero_limbs<E: LimbEngine>(limbs: &[E::Limb]) -> bool {
    limbs.iter().all(|&l| l == E::zero())
}

pub fn compare_limbs<E: LimbEngine>(a: &[E::Limb], b: &[E::Limb]) -> core::cmp::Ordering {
    let max_len = a.len().max(b.len());
    for i in (0..max_len).rev() {
        let ai = if i < a.len() { E::to_u64(a[i]) } else { 0 };
        let bi = if i < b.len() { E::to_u64(b[i]) } else { 0 };
        if ai < bi {
            return core::cmp::Ordering::Less;
        } else if ai > bi {
            return core::cmp::Ordering::Greater;
        }
    }
    core::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::U32Engine;

    #[test]
    fn add_limbs_works_for_u32() {
        let a = limbs_from_u64::<U32Engine>(5, 2);
        let b = limbs_from_u64::<U32Engine>(7, 2);
        let sum = add_limbs::<U32Engine>(&a, &b);
        assert_eq!(limbs_to_u64::<U32Engine>(&sum), 12);
    }

    #[test]
    fn sub_limbs_works_for_u32() {
        let a = limbs_from_u64::<U32Engine>(10, 2);
        let b = limbs_from_u64::<U32Engine>(3, 2);
        let diff = sub_limbs::<U32Engine>(&a, &b);
        assert_eq!(limbs_to_u64::<U32Engine>(&diff), 7);
    }

    #[test]
    fn mul_limbs_works_for_u32() {
        let a = limbs_from_u64::<U32Engine>(6, 2);
        let b = limbs_from_u64::<U32Engine>(7, 2);
        let prod = mul_limbs::<U32Engine>(&a, &b);
        assert_eq!(limbs_to_u64::<U32Engine>(&prod), 42);
    }

    #[test]
    fn is_less_than_true() {
        let a = limbs_from_u64::<U32Engine>(5, 2);
        let b = limbs_from_u64::<U32Engine>(10, 2);
        assert!(is_less_than::<U32Engine>(&a, &b));
        assert!(!is_less_than::<U32Engine>(&b, &a));
    }

    #[test]
    fn is_less_than_equal() {
        let a = limbs_from_u64::<U32Engine>(7, 2);
        let b = limbs_from_u64::<U32Engine>(7, 2);
        assert!(!is_less_than::<U32Engine>(&a, &b));
    }

    #[test]
    fn is_zero_limbs_works() {
        let zero = limbs_from_u64::<U32Engine>(0, 3);
        let non_zero = limbs_from_u64::<U32Engine>(1, 3);
        assert!(is_zero_limbs::<U32Engine>(&zero));
        assert!(!is_zero_limbs::<U32Engine>(&non_zero));
    }

    #[test]
    fn compare_limbs_less() {
        let a = limbs_from_u64::<U32Engine>(3, 2);
        let b = limbs_from_u64::<U32Engine>(7, 2);
        assert_eq!(
            compare_limbs::<U32Engine>(&a, &b),
            core::cmp::Ordering::Less
        );
    }
}

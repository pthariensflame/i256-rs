//! An unsigned 256-bit integer type.
//!
//! This aims to have feature parity with Rust's unsigned
//! integer types, such as [u32][core::u32]. The documentation
//! is based off of [u32][core::u32] for each method/member.
//!
//! A large portion of the implementation for helper functions
//! are based off of the Rust core implementation, such as for
//! [`checked_pow`][u128::checked_pow], [`isqrt`][u128::isqrt],
//! and more. Any non-performance critical functions, or those
//! crucial to parsing or serialization ([`add`][`u256::add`],
//! [`mul`][`u256::mul`], [`div`][`u256::div`], and
//! [`sub`][`u256::sub`]), as well as their `wrapping_*`,
//! `checked_*`, `overflowing_*` and `*_wide` variants are
//! likely based on the core implementations.

use core::ops::*;

use super::shared_macros::*;
use crate::{i256, math, ULimb};

int_define!(
    name => u256,
    bits => 256,
    kind => unsigned,
);

impl u256 {
    uint_impl_define!(
        self => u256,
        signed_t => i256,
        signed_wide_t => i128,
        unsigned_wide_t => u128,
        bits => 256,
        max_digits => 78,
        kind => unsigned,
        short_circuit => false,
    );

    /// Shifts the bits to the left by a specified amount, `n`,
    /// wrapping the truncated bits to the end of the resulting integer.
    ///
    /// Please note this isn't the same operation as the `<<` shifting operator!
    ///
    /// See [`u128::rotate_left`].
    #[inline(always)]
    pub const fn rotate_left(self, n: u32) -> Self {
        let (lo, hi) = math::rotate_left_u128(self.low(), self.high(), n);
        Self::new(lo, hi)
    }

    /// Shifts the bits to the right by a specified amount, `n`,
    /// wrapping the truncated bits to the beginning of the resulting
    /// integer.
    ///
    /// Please note this isn't the same operation as the `>>` shifting operator!
    ///
    /// See [`u128::rotate_right`].
    #[inline(always)]
    pub const fn rotate_right(self, n: u32) -> Self {
        let (lo, hi) = math::rotate_right_u128(self.low(), self.high(), n);
        Self::new(lo, hi)
    }

    /// Panic-free bitwise shift-left; yields `self << mask(rhs)`,
    /// where `mask` removes any high-order bits of `rhs` that
    /// would cause the shift to exceed the bitwidth of the type.
    ///
    /// Note that this is *not* the same as a rotate-left; the
    /// RHS of a wrapping shift-left is restricted to the range
    /// of the type, rather than the bits shifted out of the LHS
    /// being returned to the other end. The primitive integer
    /// types all implement a [`rotate_left`](Self::rotate_left) function,
    /// which may be what you want instead.
    ///
    /// See [`u128::wrapping_shl`].
    #[inline(always)]
    pub const fn wrapping_shl(self, rhs: u32) -> Self {
        let (lo, hi) = math::shl_u128(self.low(), self.high(), rhs % Self::BITS);
        Self::new(lo, hi)
    }

    /// Panic-free bitwise shift-right; yields `self >> mask(rhs)`,
    /// where `mask` removes any high-order bits of `rhs` that
    /// would cause the shift to exceed the bitwidth of the type.
    ///
    /// Note that this is *not* the same as a rotate-right; the
    /// RHS of a wrapping shift-right is restricted to the range
    /// of the type, rather than the bits shifted out of the LHS
    /// being returned to the other end. The primitive integer
    /// types all implement a [`rotate_right`](Self::rotate_right) function,
    /// which may be what you want instead.
    ///
    /// See [`u128::wrapping_shr`].
    #[inline(always)]
    pub const fn wrapping_shr(self, rhs: u32) -> Self {
        let (lo, hi) = math::shr_u128(self.low(), self.high(), rhs % Self::BITS);
        Self::new(lo, hi)
    }
}

uint_traits_define!(type => u256, signed_type => i256);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParseIntError;

    #[test]
    fn add_test() {
        // NOTE: This is mostly covered elsewhere
        assert_eq!(u256::from_u8(1).wrapping_add(u256::from_u8(1)), u256::from_u8(2));
        assert_eq!(
            u256::MAX.wrapping_add(u256::MAX),
            u256::from_le_u64([u64::MAX - 1, u64::MAX, u64::MAX, u64::MAX])
        );

        assert_eq!(
            u256::from_u8(1).overflowing_add(u256::from_u8(1)).0,
            u256::from_u8(1).wrapping_add(u256::from_u8(1))
        );
        assert_eq!(u256::MAX.overflowing_add(u256::MAX).0, u256::MAX.wrapping_add(u256::MAX));
    }

    #[test]
    fn endian_tests() {
        let data = [0x123456u64, 0x789abcu64, 0xdef012u64, 0x345678u64];
        let int = u256::from_le_u64(data);
        assert_eq!(int, u256::from_le_bytes(int.to_le_bytes()));
        assert_eq!(int, u256::from_be_bytes(int.to_be_bytes()));
        assert_eq!(int, u256::from_ne_bytes(int.to_ne_bytes()));

        assert_eq!(int, u256::from_le_limbs(int.to_le_limbs()));
        assert_eq!(int, u256::from_be_limbs(int.to_be_limbs()));
        assert_eq!(int, u256::from_ne_limbs(int.to_ne_limbs()));

        assert_eq!(int, u256::from_le_wide(int.to_le_wide()));
        assert_eq!(int, u256::from_be_wide(int.to_be_wide()));
        assert_eq!(int, u256::from_ne_wide(int.to_ne_wide()));

        assert_eq!(int, u256::from_le_u32(int.to_le_u32()));
        assert_eq!(int, u256::from_be_u32(int.to_be_u32()));
        assert_eq!(int, u256::from_ne_u32(int.to_ne_u32()));

        assert_eq!(int, u256::from_le_u64(int.to_le_u64()));
        assert_eq!(int, u256::from_be_u64(int.to_be_u64()));
        assert_eq!(int, u256::from_ne_u64(int.to_ne_u64()));
    }

    #[test]
    fn display_test() {
        let max = u256::MAX;
        let result = max.to_string();
        assert_eq!(
            "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            result
        );

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = value.to_string();
        assert_eq!("340282366920938463463374607431768211456", result);
    }

    #[test]
    fn lower_exp_test() {
        let max = u256::MAX;
        let result = format!("{:e}", max);
        assert_eq!(
            "1.15792089237316195423570985008687907853269984665640564039457584007913129639935e77",
            result
        );

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = format!("{:e}", value);
        assert_eq!("3.40282366920938463463374607431768211456e38", result);
    }

    #[test]
    fn upper_exp_test() {
        let max = u256::MAX;
        let result = format!("{:E}", max);
        assert_eq!(
            "1.15792089237316195423570985008687907853269984665640564039457584007913129639935E77",
            result
        );

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = format!("{:E}", value);
        assert_eq!("3.40282366920938463463374607431768211456E38", result);
    }

    #[test]
    fn octal_test() {
        let max = u256::MAX;
        let result = format!("{:o}", max);
        assert_eq!(
            "17777777777777777777777777777777777777777777777777777777777777777777777777777777777777",
            result
        );

        let result = format!("{:#o}", max);
        assert_eq!(
            "0o17777777777777777777777777777777777777777777777777777777777777777777777777777777777777",
            result
        );

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = format!("{:o}", value);
        assert_eq!("4000000000000000000000000000000000000000000", result);
    }

    #[test]
    fn binary_test() {
        let max = u256::MAX;
        let result = format!("{:b}", max);
        assert_eq!(
            "1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
            result
        );

        let result = format!("{:#b}", max);
        assert_eq!(
            "0b1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
            result
        );

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = format!("{:b}", value);
        assert_eq!(
            "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            result
        );
    }

    #[test]
    fn lower_hex_test() {
        let max = u256::MAX;
        let result = format!("{:x}", max);
        assert_eq!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", result);

        let result = format!("{:#x}", max);
        assert_eq!("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", result);

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = format!("{:x}", value);
        assert_eq!("100000000000000000000000000000000", result);
    }

    #[test]
    fn upper_hex_test() {
        let max = u256::MAX;
        let result = format!("{:X}", max);
        assert_eq!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", result);

        let result = format!("{:#X}", max);
        assert_eq!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", result);

        let value = u256::from_le_u64([0, 0, 1, 0]);
        let result = format!("{:X}", value);
        assert_eq!("100000000000000000000000000000000", result);
    }

    #[inline(always)]
    fn parse(expected: u256, radix: u32, s: &str) {
        // check a full roundtrip
        let res: Result<u256, ParseIntError> = u256::from_str_radix(s, radix);
        assert!(res.is_ok());
        let actual = res.unwrap();
        assert_eq!(expected, actual);

        let as_str = actual.to_string();
        let res: Result<u256, ParseIntError> = u256::from_str_radix(&as_str, 10);
        assert!(res.is_ok());
        let actual = res.unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str_radix_test() {
        let cases = [
            (
                u256::MAX,
                10,
                "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            ),
            (
                u256::MAX,
                10,
                "+115792089237316195423570985008687907853269984665640564039457584007913129639935",
            ),
            (u256::MAX, 16, "+ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            (0xffffffffffffffffu128.into(), 16, "+ffffffffffffffff"),
            (0x123456789ab123u128.into(), 10, "5124095576027427"),
            (0x123456789ab123u128.into(), 16, "+123456789ab123"),
        ];
        for case in cases {
            parse(case.0, case.1, case.2);
        }

        let failing = [
            (10, "-15"),
            (16, "-0xFF"),
            (16, "+0xFF"),
            (16, "0xFF"),
            (10, "FF"),
            (10, "a9"),
            (10, "12.34"),
            (10, "1234_67"),
            (10, "115792089237316195423570985008687907853269984665640564039457584007913129639936"),
        ];
        for case in failing {
            let res: Result<u256, ParseIntError> = u256::from_str_radix(case.1, case.0);
            assert!(res.is_err());
        }
    }

    #[test]
    #[should_panic]
    fn from_str_radix_neg_test() {
        _ = u256::from_str_radix("-123", 10).unwrap();
    }
}

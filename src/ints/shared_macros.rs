//! Macros shared between signed and unsigned types.

// FIXME: Add support for [Saturating][core::num::Saturating] and
// [Wrapping][core::num::Wrapping] when we drop support for <1.74.0.

#[rustfmt::skip]
macro_rules! int_define {
    (
        name => $name:ident,
        bits => $bits:literal,
        kind => $kind:ident $(,)?
    ) => {
        #[rustfmt::skip]
        #[doc = concat!("The ", stringify!($bits), "-bit ", stringify!($kind), " integer type.")]
        ///
        /// The high and low words depend on the target endianness.
        /// Conversion to and from big endian should be done via
        /// [`to_le_bytes`] and [`to_be_bytes`].
        ///
        /// Our formatting specifications are limited: we ignore a
        /// lot of settings, and only respect [`alternate`] among the
        /// formatter flags. So, we implement all the main formatters
        /// ([`Binary`], etc.), but ignore all flags like `width`.
        ///
        /// Note that this type is **NOT** safe to use in FFIs, since the
        /// underlying storage may use [`128-bit`] integers in the future
        /// which are not FFI-safe. If you would like to use this type
        /// within a FFI, use [`to_le_bytes`] and [`to_be_bytes`].
        ///
        #[doc = concat!("[`to_le_bytes`]: ", stringify!($name), "::to_le_bytes")]
        #[doc = concat!("[`to_be_bytes`]: ", stringify!($name), "::to_be_bytes")]
        /// [`alternate`]: core::fmt::Formatter::alternate
        /// [`Binary`]: core::fmt::Binary
        /// [`128-bit`]: https://rust-lang.github.io/unsafe-code-guidelines/layout/scalars.html#fixed-width-integer-types
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
        pub struct $name {
            // NOTE: This is currently FFI-safe (if we did repr(C)) but we
            // intentionally make  no guarantees so we're free to re-arrange
            // the layout.
            limbs: [$crate::ULimb; $bits / core::mem::size_of::<$crate::ULimb>() / 8],
        }
    };
}

#[rustfmt::skip]
macro_rules! associated_consts_define {
    (
        bits =>
        $bits:expr,max_digits =>
        $max_digits:expr,wide_type =>
        $wide_t:ty,low_type =>
        $lo_t:ty,high_type =>
        $hi_t:ty $(,)?
    ) => {
        /// The smallest value that can be represented by this integer type.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::MIN`].")]
        #[allow(deprecated)]
        pub const MIN: Self = Self::min_value();

        /// The largest value that can be represented by this integer type
        /// (2<sup>256</sup> - 1).
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::MAX`].")]
        #[allow(deprecated)]
        pub const MAX: Self = Self::max_value();

        /// The size of this integer type in bits.
        ///
        /// # Examples
        ///
        /// ```rust
        /// # use i256::u256;
        /// assert_eq!(u256::BITS, 256);
        /// ```
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::BITS`].")]
        pub const BITS: u32 = $bits;

        /// The number of decimal digits for the largest magnitude value.
        pub const MAX_DIGITS: usize = $max_digits;
    };
}

/// Define the high and low implementations for 4 limb implementations.
///
/// This is specific for **ONLY** our 256-bit integers (4x 64-bit limbs).
macro_rules! high_low_define {
    (
        self => $t:ty,
        low_type => $lo_t:ty,
        high_type => $hi_t:ty,
        kind => $kind:ident $(,)?
    ) => {
        /// Flatten two 128-bit integers as bytes to flat, 32 bytes.
        ///
        /// We keep this as a standalone function since Rust can sometimes
        /// vectorize this in a way using purely safe Rust cannot, which
        /// improves performance while ensuring we are very careful.
        /// These are guaranteed to be plain old [`data`] with a fixed size
        /// alignment, and padding.
        ///
        /// [`data`]: https://rust-lang.github.io/unsafe-code-guidelines/layout/scalars.html#fixed-width-integer-types
        #[inline(always)]
        const fn to_flat_bytes(x: [u8; 16], y: [u8; 16]) -> [u8; Self::BYTES] {
            // SAFETY: plain old data
            unsafe { core::mem::transmute::<[[u8; 16]; 2], [u8; Self::BYTES]>([x, y]) }
        }

        #[doc = concat!("Create a new `", stringify!($t), "` from the low and high bits.")]
        #[inline(always)]
        pub const fn new(lo: $lo_t, hi: $hi_t) -> Self {
            let inst = if cfg!(target_endian = "big") {
                Self::from_be_bytes(Self::to_flat_bytes(hi.to_be_bytes(), lo.to_be_bytes()))
            } else {
                Self::from_le_bytes(Self::to_flat_bytes(lo.to_le_bytes(), hi.to_le_bytes()))
            };
            assert!(inst.limbs.len() ==  4, "cannot create type with more than 4 limbs.");

            inst
        }

        #[doc = concat!("Get the high ", stringify!($crate::ULimb::BITS), " bits of the ", stringify!($kind), " integer.")]
        #[inline(always)]
        pub const fn high(self) -> $hi_t {
            assert!(self.limbs.len() ==  4, "cannot get high bits with more than 4 limbs.");
            self.get_wide(1) as $hi_t
        }

        #[doc = concat!("Get the low ", stringify!($crate::ULimb::BITS), " bits of the ", stringify!($kind), " integer.")]
        #[inline(always)]
        pub const fn low(self) -> $lo_t {
            assert!(self.limbs.len() ==  4, "cannot get low bits with more than 4 limbs.");
            self.get_wide(0) as $lo_t
        }
    };
}

macro_rules! cmp_define {
    (
        @ord
        $lhs:ident,
        $rhs:ident,
        low_type => $lo_t:ty,
        high_type => $hi_t:ty,
        op1 => $op1:tt ,
        op2 => $op2:tt $(,)?
    ) => {{
        // The implied methods that are identical between short and non-circuiting options.
        let lhs = $lhs.to_ne_wide();
        let rhs = $rhs.to_ne_wide();

        let mut i = Self::WIDE - 1;
        let lhs_0 = ne_index!(lhs[i]) as $hi_t;
        let rhs_0 = ne_index!(rhs[i]) as $hi_t;
        let mut is_ord = lhs_0 $op1 rhs_0;
        let mut is_eq = lhs_0 == rhs_0;

        while i > 0 && !is_ord && is_eq {
            i -= 1;
            let lhs_i = ne_index!(lhs[i]) as $lo_t;
            let rhs_i = ne_index!(rhs[i]) as $lo_t;
            is_ord = lhs_i $op2 rhs_i;
            is_eq = lhs_i == rhs_i;
        }
        is_ord
    }};

    (
        @cmp
        $lhs:ident,
        $rhs:ident,
        low_type => $lo_t:ty,
        high_type => $hi_t:ty,
    ) => {{
        // The implied methods that are identical between short and non-circuiting options.
        let lhs = $lhs.to_ne_wide();
        let rhs = $rhs.to_ne_wide();

        let mut i = Self::WIDE - 1;
        let lhs_0 = ne_index!(lhs[i]) as $hi_t;
        let rhs_0 = ne_index!(rhs[i]) as $hi_t;
        let mut is_eq = lhs_0 == rhs_0;
        let mut is_lt = lhs_0 < rhs_0;
        let mut is_gt = lhs_0 > rhs_0;

        while i > 0 && !is_lt && !is_gt && is_eq {
            i -= 1;
            let lhs_i = ne_index!(lhs[i]) as $lo_t;
            let rhs_i = ne_index!(rhs[i]) as $lo_t;
            is_eq = lhs_i == rhs_i;
            is_lt = lhs_i < rhs_i;
            is_gt = lhs_i > rhs_i;
        }

        if is_lt {
            core::cmp::Ordering::Less
        } else if is_gt {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }};

    (
        low_type => $lo_t:ty,
        high_type => $hi_t:ty,
        short_circuit => false $(,)?
    ) => {
        /// Non-short circuiting const implementation of `Eq`.
        #[inline(always)]
        pub const fn eq_const(self, rhs: Self) -> bool {
            let lhs = self.to_ne_wide();
            let rhs = rhs.to_ne_wide();
            let mut is_eq = true;
            let mut i = 0;
            while i < Self::WIDE {
                // NOTE: This can be in any order
                is_eq &= (lhs[i] == rhs[i]);
                i += 1;
            }
            is_eq
        }

        // NOTE: Because of two's complement, these comparisons are always normal.
        // This can always be implemented in terms of the highest wide bit, then the
        // rest as low.

        /// Non-short circuiting const implementation of `PartialOrd::lt`.
        #[inline(always)]
        pub const fn lt_const(self, rhs: Self) -> bool {
            cmp_define!(
                @ord
                self,
                rhs,
                low_type => $lo_t,
                high_type => $hi_t,
                op1 => <,
                op2 => <,
            )
        }

        /// Non-short circuiting const implementation of `PartialOrd::le`.
        #[inline(always)]
        pub const fn le_const(self, rhs: Self) -> bool {
            cmp_define!(
                @ord
                self,
                rhs,
                low_type => $lo_t,
                high_type => $hi_t,
                op1 => <,
                op2 => <=,
            )
        }

        /// Non-short circuiting const implementation of `PartialOrd::gt`.
        #[inline(always)]
        pub const fn gt_const(self, rhs: Self) -> bool {
            !self.le_const(rhs)
        }

        /// Non-short circuiting const implementation of `PartialOrd::ge`.
        #[inline(always)]
        pub const fn ge_const(self, rhs: Self) -> bool {
            !self.lt_const(rhs)
        }

        /// Non-short circuiting const implementation of `PartialOrd::cmp`.
        #[inline(always)]
        pub const fn cmp_const(self, rhs: Self) -> core::cmp::Ordering {
            cmp_define!(
                @cmp
                self,
                rhs,
                low_type => $lo_t,
                high_type => $hi_t,
            )
        }
    };

    (
        low_type => $lo_t:ty,
        high_type => $hi_t:ty,
        short_circuit => true $(,)?
    ) => {
        /// Short-circuiting const implementation of `Eq`.
        #[inline(always)]
        pub const fn eq_const(self, rhs: Self) -> bool {
            let lhs = self.to_ne_wide();
            let rhs = rhs.to_ne_wide();
            let mut is_eq = true;
            let mut i = 0;
            while i < Self::WIDE && is_eq {
                is_eq &= (lhs[i] == rhs[i]);
                i += 1;
            }
            is_eq
        }

        // NOTE: Because of two's complement, these comparisons are always normal.
        // This can always be implemented in terms of the highest wide bit, then the
        // rest as low.

        /// Short circuiting const implementation of `PartialOrd::lt`.
        #[inline(always)]
        pub const fn lt_const(self, rhs: Self) -> bool {
            cmp_define!(
                @ord
                self,
                rhs,
                low_type => $lo_t,
                high_type => $hi_t,
                op1 => <,
                op2 => <,
            )
        }

        /// Short circuiting const implementation of `PartialOrd::le`.
        #[inline(always)]
        pub const fn le_const(self, rhs: Self) -> bool {
            cmp_define!(
                @ord
                self,
                rhs,
                low_type => $lo_t,
                high_type => $hi_t,
                op1 => <,
                op2 => <=,
            )
        }

        /// Short circuiting const implementation of `PartialOrd::gt`.
        #[inline(always)]
        pub const fn gt_const(self, rhs: Self) -> bool {
            !self.le_const(rhs)
        }

        /// Short circuiting const implementation of `PartialOrd::ge`.
        #[inline(always)]
        pub const fn ge_const(self, rhs: Self) -> bool {
            !self.lt_const(rhs)
        }

        /// Short-circuiting const implementation of `PartialOrd::cmp`.
        #[inline(always)]
        pub const fn cmp_const(self, rhs: Self) -> core::cmp::Ordering {
            cmp_define!(
                @cmp
                self,
                rhs,
                low_type => $lo_t,
                high_type => $hi_t,
            )
        }
    };
}

macro_rules! extensions_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Get if the integer is even.
        #[inline(always)]
        pub const fn is_even(&self) -> bool {
            self.get_limb(0) % 2 == 0
        }

        /// Get if the integer is odd.
        #[inline(always)]
        pub const fn is_odd(&self) -> bool {
            !self.is_even()
        }

        /// Get the least significant limb in the buiffer.
        #[inline(always)]
        pub const fn least_significant_limb(&self) -> $crate::ULimb {
            self.get_limb(0)
        }
    };
}

// NOTE: Validation of the bit patterns for types can be done as:
//
// ```python
// from bitstring import BitArray
//
// def sbin(n, l, be='big'):
//     bits = BitArray(n.to_bytes(l, signed=True, byteorder=be)).bin
//     return '0b' + bits
//
// def ubin(n, l, be='big'):
//     bits = BitArray(n.to_bytes(l, signed=False, byteorder=be)).bin
//     return '0b' + bits
// ```
//
// These are output in big-endian order. Great testing includes
// unique bit patterns, like `ubin(0x123579, 4)`, which has a unique
// bit order (`0b00000000000100100011010101111001`), which we can
// check for truncation to `u16` from `u32`, etc., as well as conversions
// to signed and conversions to `i16` from `i32`. Casting to `u16` leaves
// `0x3579`, as does `i32` to `i16`. Similarly, `-0x123579i32 as i16` is
// then truncated to `-0x3579`.
//
// Meanwhile, `sbin(-0x123579`, 4) == 0b11111111111011011100101010000111`.
//
// **Big:**
// - +0x123579i32: 0b00000000 00010010 00110101 01111001
// - -0x123579i32: 0b11111111 11101101 11001010 10000111
//
// - +0x3579i16:   0b                  00110101 01111001
// - -0x3579i16:   0b                  11001010 10000111
//
// **Little:**
// - +0x123579i32: 0b01111001 00110101 00010010 00000000
// - -0x123579i32: 0b10000111 11001010 11101101 11111111
//
// - +0x3579i16:   0b01111001 00110101
// - -0x3579i16:   0b10000111 11001010
//
// Or, the `!0x123579i32 + 1`, as documented. Since we're doing
// a big-endian representation, it means truncation is just taking the high
// words and going from there.

/// And any `as_` and `from_` methods for higher-order types.
macro_rules! casts_define {
    (
        bits => $bits:expr,
        kind => $kind:ident $(,)?
    ) => {
        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a `u8`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_u8(value: u8) -> Self {
            Self::from_u32(value as u32)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a `u16`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_u16(value: u16) -> Self {
            Self::from_u32(value as u32)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a `u32`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_u32(value: u32) -> Self {
            let mut limbs = [0; Self::LIMBS];
            ne_index!(limbs[0] = value as $crate::ULimb);
            Self::from_ne_limbs(limbs)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a `u64`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_u64(value: u64) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);

            let mut limbs = [0; Self::LIMBS];
            if BITS == 32 {
                ne_index!(limbs[0] = value as $crate::ULimb);
                ne_index!(limbs[1] = (value >> 32) as $crate::ULimb);
            } else {
                ne_index!(limbs[0] = value as $crate::ULimb);
            }
            Self::from_ne_limbs(limbs)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a `u128`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_u128(value: u128) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);

            let mut limbs = [0; Self::LIMBS];
            if BITS == 32 {
                ne_index!(limbs[0] = value as $crate::ULimb);
                ne_index!(limbs[1] = (value >> 32) as $crate::ULimb);
                ne_index!(limbs[2] = (value >> 64) as $crate::ULimb);
                ne_index!(limbs[3] = (value >> 96) as $crate::ULimb);
            } else {
                ne_index!(limbs[0] = value as $crate::ULimb);
                ne_index!(limbs[1] = (value >> 64) as $crate::ULimb);
            }
            Self::from_ne_limbs(limbs)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from an unsigned limb, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn from_ulimb(value: $crate::ULimb) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);
            if BITS == 32 {
                Self::from_u32(value as u32)
            } else {
                Self::from_u64(value as u64)
            }
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from an unsigned wide type, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn from_uwide(value: $crate::UWide) -> Self {
            const BITS: u32 = $crate::UWide::BITS;
            assert!(BITS == 64 || BITS == 128);
            if BITS == 64 {
                Self::from_u64(value as u64)
            } else {
                Self::from_u128(value as u128)
            }
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from an `i8`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_i8(value: i8) -> Self {
            Self::from_i32(value as i32)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from an `i16`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_i16(value: i16) -> Self {
            Self::from_i32(value as i32)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from an `i32`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_i32(value: i32) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);
            if BITS == 32 {
                let sign_bit = $crate::ULimb::MIN.wrapping_sub(value.is_negative() as $crate::ULimb);
                let mut limbs = [sign_bit; Self::LIMBS];
                let value = value as $crate::ULimb;
                ne_index!(limbs[0] = value);
                Self::from_ne_limbs(limbs)
            } else {
                Self::from_i64(value as i64)
            }
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from an `i64`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn from_i64(value: i64) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);
            if BITS == 32 {
                Self::from_i32(value as i32)
            } else {
                let sign_bit = $crate::ULimb::MIN.wrapping_sub(value.is_negative() as $crate::ULimb);
                let mut limbs = [sign_bit; Self::LIMBS];
                let value = value as $crate::ULimb;
                ne_index!(limbs[0] = value);
                Self::from_ne_limbs(limbs)
            }
        }

        /// Create the 256-bit unsigned integer from an `i128`, as if by an `as`
        /// cast.
        #[inline(always)]
        pub const fn from_i128(value: i128) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);

            let sign_bit = $crate::ULimb::MIN.wrapping_sub(value.is_negative() as $crate::ULimb);
            let mut limbs = [sign_bit; Self::LIMBS];
            let value = value as u128;
            if BITS == 32 {
                ne_index!(limbs[0] = value as $crate::ULimb);
                ne_index!(limbs[1] = (value >> 32) as $crate::ULimb);
                ne_index!(limbs[2] = (value >> 64) as $crate::ULimb);
                ne_index!(limbs[3] = (value >> 96) as $crate::ULimb);
            } else {
                ne_index!(limbs[0] = value as $crate::ULimb);
                ne_index!(limbs[1] = (value >> 64) as $crate::ULimb);
            }
            Self::from_ne_limbs(limbs)
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a signed limb, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn from_ilimb(value: $crate::ILimb) -> Self {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);
            if BITS == 32 {
                Self::from_i32(value as i32)
            } else {
                Self::from_i64(value as i64)
            }
        }

        #[doc = concat!("Create the ", stringify!($bits), "-bit ", stringify!($kind), " integer from a wide type, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn from_iwide(value: $crate::IWide) -> Self {
            const BITS: u32 = $crate::UWide::BITS;
            assert!(BITS == 64 || BITS == 128);
            if BITS == 64 {
                Self::from_i64(value as i64)
            } else {
                Self::from_i128(value as i128)
            }
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to a `u8`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_u8(&self) -> u8 {
            self.as_u32() as u8
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to a `u16`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_u16(&self) -> u16 {
            self.as_u32() as u16
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to a `u32`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_u32(&self) -> u32 {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);
            let limbs = self.to_ne_limbs();
            ne_index!(limbs[0]) as u32
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to a `u64`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_u64(&self) -> u64 {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);

            let limbs = self.to_ne_limbs();
            if BITS == 32 {
                let x0 = ne_index!(limbs[0]) as u64;
                let x1 = ne_index!(limbs[1]) as u64;
                (x0 | (x1 << 32))
            } else {
                ne_index!(limbs[0]) as u64
            }
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " an unsigned limb, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn as_ulimb(&self) -> $crate::ULimb {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);
            if BITS == 32 {
                self.as_u32() as $crate::ULimb
            } else {
                self.as_u64() as $crate::ULimb
            }
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to a `u128`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_u128(&self) -> u128 {
            const BITS: u32 = $crate::ULimb::BITS;
            assert!(BITS == 32 || BITS == 64);

            let limbs = self.to_ne_limbs();
            if BITS == 32 {
                let x0 = ne_index!(limbs[0]) as u128;
                let x1 = ne_index!(limbs[1]) as u128;
                let x2 = ne_index!(limbs[2]) as u128;
                let x3 = ne_index!(limbs[3]) as u128;
                (x0 | (x1 << 32) | (x2 << 64)| (x3 << 96))
            } else {
                let x0 = ne_index!(limbs[0]) as u128;
                let x1 = ne_index!(limbs[1]) as u128;
                (x0 | (x1 << 64))
            }
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " an unsigned wide type, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn as_uwide(&self) -> $crate::UWide {
            const BITS: u32 = $crate::UWide::BITS;
            assert!(BITS == 64 || BITS == 128);
            if BITS == 64 {
                self.as_u64() as $crate::UWide
            } else {
                self.as_u128() as $crate::UWide
            }
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to an `i8`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_i8(&self) -> i8 {
            self.as_u8() as i8
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to an `i16`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_i16(&self) -> i16 {
            self.as_u16() as i16
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to an `i32`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_i32(&self) -> i32 {
            self.as_u32() as i32
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to an `i64`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_i64(&self) -> i64 {
            self.as_u64() as i64
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " to a `i128`, as if by an `as` cast.")]
        #[inline(always)]
        pub const fn as_i128(&self) -> i128 {
            self.as_u128() as i128
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " a signed limb, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn as_ilimb(&self) -> $crate::ILimb {
            self.as_ulimb() as $crate::ILimb
        }

        #[doc = concat!("Convert the ", stringify!($bits), "-bit ", stringify!($kind), " a signed wide type, as if by an `as` cast.")]
        #[inline(always)]
        #[allow(clippy::unnecessary_cast)]
        pub const fn as_iwide(&self) -> $crate::IWide {
            self.as_uwide() as $crate::IWide
        }
    };
}

#[rustfmt::skip]
macro_rules! byte_order_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// The number of bytes in the type.
        pub(crate) const BYTES: usize = Self::BITS as usize / 8;
        pub(crate) const U32_LEN: usize = Self::BYTES / 4;
        pub(crate) const U64_LEN: usize = Self::BYTES / 8;

        /// The number of limbs in the type.
        pub(crate) const LIMBS: usize = Self::BYTES / core::mem::size_of::<$crate::ULimb>();

        /// The number of wide values in the type.
        pub(crate) const WIDE: usize = Self::BYTES / core::mem::size_of::<$crate::UWide>();

        /// Get the limb indexing from the least-significant order.
        #[inline(always)]
        pub const fn get_limb(&self, index: usize) -> $crate::ULimb {
            let limbs = &self.limbs;
            ne_index!(limbs[index])
        }

        /// Get the wide value indexing from the least-significant order.
        ///
        /// This optimizes extremely well, if the index is known ahead of time
        /// into 2 `mov` instructions, that is, as efficient as can be.
        #[inline(always)]
        pub const fn get_wide(&self, index: usize) -> $crate::UWide {
            const LIMB_BYTES: usize = core::mem::size_of::<$crate::ULimb>();
            const WIDE_BYTES: usize = core::mem::size_of::<$crate::UWide>();
            assert!(WIDE_BYTES / LIMB_BYTES == 2);
            assert!(index < Self::WIDE, "index must be less than the total wide values");

            // NOTE: We can just grab the bytes based on the indexes,
            // and break it into 2 limbs and then build it in native
            // ending order.
            let offset = if cfg!(target_endian = "big") {
                Self::LIMBS - 2 * (index + 1)
            } else {
                2 * index
            };
            let lo = self.limbs[offset].to_ne_bytes();
            let hi = self.limbs[offset + 1].to_ne_bytes();

            // convert as via a transmute to our wide type and transmute
            // SAFETY: plain old data
            let bytes = unsafe {
                core::mem::transmute::<[[u8; LIMB_BYTES]; 2], [u8; WIDE_BYTES]>([lo, hi])
            };
            $crate::UWide::from_ne_bytes(bytes)
        }

        /// Reverses the byte order of the integer.
        ///
        /// # Assembly
        ///
        /// This optimizes very nicely, with efficient `bswap` or `rol`
        /// implementations for each.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::swap_bytes`].")]
        #[inline]
        pub const fn swap_bytes(&self) -> Self {
            let mut r = Self {
                limbs: [0; Self::LIMBS],
            };
            let mut i = 0;
            while i < Self::LIMBS {
                r.limbs[i] = self.limbs[Self::LIMBS - 1 - i].swap_bytes();
                i += 1;
            }
            r
        }

        /// Reverses the order of bits in the integer. The least significant
        /// bit becomes the most significant bit, second least-significant bit
        /// becomes second most-significant bit, etc.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::reverse_bits`].")]
        #[inline(always)]
        pub const fn reverse_bits(&self) -> Self {
            let mut r = Self {
                limbs: [0; Self::LIMBS],
            };
            let mut i = 0;
            while i < 4 {
                r.limbs[i] = self.limbs[Self::LIMBS - 1 - i].reverse_bits();
                i += 1;
            }
            r
        }

        /// Converts an integer from big endian to the target's endianness.
        ///
        /// On big endian this is a no-op. On little endian the bytes are
        /// swapped.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::from_be`].")]
        #[inline(always)]
        pub const fn from_be(x: Self) -> Self {
            if cfg!(target_endian = "big") {
                x
            } else {
                x.swap_bytes()
            }
        }

        /// Converts an integer from little endian to the target's endianness.
        ///
        /// On little endian this is a no-op. On big endian the bytes are
        /// swapped.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::from_le`].")]
        #[inline(always)]
        pub const fn from_le(x: Self) -> Self {
            if cfg!(target_endian = "little") {
                x
            } else {
                x.swap_bytes()
            }
        }

        /// Converts `self` to big endian from the target's endianness.
        ///
        /// On big endian this is a no-op. On little endian the bytes are
        /// swapped.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::to_be`].")]
        #[inline(always)]
        pub const fn to_be(self) -> Self {
            if cfg!(target_endian = "big") {
                self
            } else {
                self.swap_bytes()
            }
        }

        /// Converts `self` to little endian from the target's endianness.
        ///
        /// On little endian this is a no-op. On big endian the bytes are
        /// swapped.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::to_le`].")]
        #[inline(always)]
        pub const fn to_le(self) -> Self {
            if cfg!(target_endian = "little") {
                self
            } else {
                self.swap_bytes()
            }
        }

        /// Returns the memory representation of this integer as a byte array in
        /// big-endian (network) byte order.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::to_be_bytes`].")]
        #[inline(always)]
        pub const fn to_be_bytes(self) -> [u8; Self::BYTES] {
            self.to_be().to_ne_bytes()
        }

        /// Returns the memory representation of this integer as a byte array in
        /// little-endian byte order.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::to_le_bytes`].")]
        #[inline(always)]
        pub const fn to_le_bytes(self) -> [u8; Self::BYTES] {
            self.to_le().to_ne_bytes()
        }

        /// Returns the memory representation of this integer as a byte array in
        /// native byte order.
        ///
        /// As the target platform's native endianness is used, portable code
        /// should use [`to_be_bytes`] or [`to_le_bytes`], as appropriate,
        /// instead.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::to_ne_bytes`].")]
        ///
        /// [`to_be_bytes`]: Self::to_be_bytes
        /// [`to_le_bytes`]: Self::to_le_bytes
        #[inline(always)]
        pub const fn to_ne_bytes(self) -> [u8; Self::BYTES] {
            // SAFETY: plain old data
            unsafe {
                core::mem::transmute::<[$crate::ULimb; Self::LIMBS], [u8; Self::BYTES]>(self.limbs)
            }
        }

        /// Creates a native endian integer value from its representation
        /// as a byte array in big endian.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::from_be_bytes`].")]
        #[inline(always)]
        pub const fn from_be_bytes(bytes: [u8; Self::BYTES]) -> Self {
            Self::from_ne_bytes(bytes).to_be()
        }

        /// Creates a native endian integer value from its representation
        /// as a byte array in little endian.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::from_le_bytes`].")]
        #[inline(always)]
        pub const fn from_le_bytes(bytes: [u8; Self::BYTES]) -> Self {
            Self::from_ne_bytes(bytes).to_le()
        }

        /// Creates a native endian integer value from its memory representation
        /// as a byte array in native endianness.
        ///
        /// As the target platform's native endianness is used, portable code
        /// likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as
        /// appropriate instead.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::from_ne_bytes`].")]
        ///
        /// [`from_be_bytes`]: Self::from_be_bytes
        /// [`from_le_bytes`]: Self::from_le_bytes
        #[inline(always)]
        pub const fn from_ne_bytes(bytes: [u8; Self::BYTES]) -> Self {
            // SAFETY: plain old data
            let limbs = unsafe {
                core::mem::transmute::<[u8; Self::BYTES], [$crate::ULimb; Self::LIMBS]>(bytes)
            };
            Self::from_ne_limbs(limbs)
        }

        /// Returns the memory representation of this as a series of limbs in
        /// big-endian (network) byte order.
        ///
        /// The value of each limb stays the same, however, the order that each
        /// is stored within the buffer is in big-endian order.
        #[inline(always)]
        pub const fn to_be_limbs(self) -> [$crate::ULimb; Self::LIMBS] {
            if cfg!(target_endian = "little") {
                swap_array!(self.to_ne_limbs())
            } else {
                self.to_ne_limbs()
            }
        }

        /// Returns the memory representation of this as a series of limbs in
        /// little-endian byte order.
        ///
        /// The value of each limb stays the same, however, the order that each
        /// is stored within the buffer is in little-endian order.
        #[inline(always)]
        pub const fn to_le_limbs(self) -> [$crate::ULimb; Self::LIMBS] {
            if cfg!(target_endian = "little") {
                self.to_ne_limbs()
            } else {
                swap_array!(self.to_ne_limbs())
            }
        }

        /// Returns the memory representation of this as a series of limbs.
        ///
        /// As the target platform's native endianness is used, portable code
        /// should use [`to_be_limbs`] or [`to_le_limbs`], as appropriate,
        /// instead.
        ///
        /// [`to_be_limbs`]: Self::to_be_limbs
        /// [`to_le_limbs`]: Self::to_le_limbs
        #[inline(always)]
        pub const fn to_ne_limbs(self) -> [$crate::ULimb; Self::LIMBS] {
            self.limbs
        }

        /// Creates a native endian integer value from its representation
        /// as limbs in big endian.
        ///
        /// The value of each limb stays the same, however, the order that each
        /// is stored within the buffer as if it was from big-endian order.
        #[inline(always)]
        pub const fn from_be_limbs(limbs: [$crate::ULimb; Self::LIMBS]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_limbs(limbs)
            } else {
                Self::from_ne_limbs(swap_array!(limbs))
            }
        }

        /// Creates a native endian integer value from its representation
        /// as limbs in little endian.
        ///
        /// The value of each limb stays the same, however, the order that each
        /// is stored within the buffer as if it was from little-endian order.
        #[inline(always)]
        pub const fn from_le_limbs(limbs: [$crate::ULimb; Self::LIMBS]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_limbs(swap_array!(limbs))
            } else {
                Self::from_ne_limbs(limbs)
            }
        }

        /// Creates a native endian integer value from its memory representation
        /// as limbs in native endianness.
        ///
        /// As the target platform's native endianness is used, portable code
        /// likely wants to use [`from_be_limbs`] or [`from_le_limbs`], as
        /// appropriate instead.
        ///
        /// [`from_be_limbs`]: Self::from_be_limbs
        /// [`from_le_limbs`]: Self::from_le_limbs
        #[inline(always)]
        pub const fn from_ne_limbs(limbs: [$crate::ULimb; Self::LIMBS]) -> Self {
            Self {
                limbs,
            }
        }

        /// Returns the memory representation of this as a series of wide in
        /// big-endian (network) byte order.
        #[inline(always)]
        pub const fn to_be_wide(self) -> [$crate::UWide; Self::WIDE] {
            if cfg!(target_endian = "little") {
                swap_array!(self.to_ne_wide())
            } else {
                self.to_ne_wide()
            }
        }

        /// Returns the memory representation of this as a series of wide in
        /// little-endian byte order.
        #[inline(always)]
        pub const fn to_le_wide(self) -> [$crate::UWide; Self::WIDE] {
            if cfg!(target_endian = "little") {
                self.to_ne_wide()
            } else {
                swap_array!(self.to_ne_wide())
            }
        }

        /// Returns the memory representation of this as a series of wide types.
        ///
        /// As the target platform's native endianness is used, portable code
        /// should use [`to_be_wide`] or [`to_le_wide`], as appropriate,
        /// instead.
        ///
        /// [`to_be_wide`]: Self::to_be_wide
        /// [`to_le_wide`]: Self::to_le_wide
        #[inline(always)]
        pub const fn to_ne_wide(self) -> [$crate::UWide; Self::WIDE] {
            let bytes = self.to_ne_bytes();
            // SAFETY: plain old data
            unsafe {
                core::mem::transmute::<[u8; Self::BYTES], [$crate::UWide; Self::WIDE]>(bytes)
            }
        }

        /// Creates a native endian integer value from its representation
        /// as a wide type in big endian.
        #[inline(always)]
        pub const fn from_be_wide(wide: [$crate::UWide; Self::WIDE]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_wide(wide)
            } else {
                Self::from_ne_wide(swap_array!(wide))
            }
        }

        /// Creates a native endian integer value from its representation
        /// as a wide type in little endian.
        #[inline(always)]
        pub const fn from_le_wide(wide: [$crate::UWide; Self::WIDE]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_wide(swap_array!(wide))
            } else {
                Self::from_ne_wide(wide)
            }
        }

        /// Creates a native endian integer value from its memory representation
        /// as a wide type in native endianness.
        ///
        /// As the target platform's native endianness is used, portable code
        /// likely wants to use [`from_be_wide`] or [`from_le_wide`], as
        /// appropriate instead.
        ///
        /// [`from_be_wide`]: Self::from_be_wide
        /// [`from_le_wide`]: Self::from_le_wide
        #[inline(always)]
        pub const fn from_ne_wide(wide: [$crate::UWide; Self::WIDE]) -> Self {
            // SAFETY: plain old data
            let bytes = unsafe {
                core::mem::transmute::<[$crate::UWide; Self::WIDE], [u8; Self::BYTES]>(wide)
            };
            Self::from_ne_bytes(bytes)
        }

        /// Returns the memory representation of this as a series of `u32` digits
        /// in big-endian order.
        #[inline(always)]
        pub const fn to_be_u32(self) -> [u32; Self::U32_LEN] {
            if cfg!(target_endian = "little") {
                swap_array!(self.to_ne_u32())
            } else {
                self.to_ne_u32()
            }
        }

        /// Returns the memory representation of this as a series of `u32` digits
        /// in litte-endian order.
        #[inline(always)]
        pub const fn to_le_u32(self) -> [u32; Self::U32_LEN] {
            if cfg!(target_endian = "little") {
                self.to_ne_u32()
            } else {
                swap_array!(self.to_ne_u32())
            }
        }

        /// Returns the memory representation of this as a series of `u32`.
        ///
        /// As the target platform's native endianness is used, portable code
        /// should use [`to_be_u32`] or [`to_le_u32`], as appropriate,
        /// instead.
        ///
        /// [`to_be_u32`]: Self::to_be_u32
        /// [`to_le_u32`]: Self::to_le_u32
        #[inline(always)]
        pub const fn to_ne_u32(self) -> [u32; Self::U32_LEN] {
            let bytes = self.to_ne_bytes();
            // SAFETY: plain old data
            unsafe {
                core::mem::transmute::<[u8; Self::BYTES], [u32; Self::U32_LEN]>(bytes)
            }
        }

        /// Creates a native endian integer value from its representation
        /// as `u32` elements in big-endian.
        #[inline(always)]
        pub const fn from_be_u32(value: [u32; Self::U32_LEN]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_u32(value)
            } else {
                Self::from_ne_u32(swap_array!(value))
            }
        }

        /// Creates a native endian integer value from its representation
        /// as `u32` elements in little-endian.
        #[inline(always)]
        pub const fn from_le_u32(value: [u32; Self::U32_LEN]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_u32(swap_array!(value))
            } else {
                Self::from_ne_u32(value)
            }
        }

        /// Creates a native endian integer value from its memory representation
        /// as `u32` in native endianness.
        ///
        /// As the target platform's native endianness is used, portable code
        /// likely wants to use [`from_be_u32`] or [`from_le_u32`], as
        /// appropriate instead.
        ///
        /// [`from_be_u32`]: Self::from_be_u32
        /// [`from_le_u32`]: Self::from_le_u32
        #[inline(always)]
        pub const fn from_ne_u32(value: [u32; Self::U32_LEN]) -> Self {
            // SAFETY: plain old data
            let bytes = unsafe {
                core::mem::transmute::<[u32; Self::U32_LEN], [u8; Self::BYTES]>(value)
            };
            Self::from_ne_bytes(bytes)
        }

        /// Returns the memory representation of this as a series of `u64` digits
        /// in big-endian order.
        #[inline(always)]
        pub const fn to_be_u64(self) -> [u64; Self::U64_LEN] {
            if cfg!(target_endian = "little") {
                swap_array!(self.to_ne_u64())
            } else {
                self.to_ne_u64()
            }
        }

        /// Returns the memory representation of this as a series of `u64` digits
        /// in litte-endian order.
        #[inline(always)]
        pub const fn to_le_u64(self) -> [u64; Self::U64_LEN] {
            if cfg!(target_endian = "little") {
                self.to_ne_u64()
            } else {
                swap_array!(self.to_ne_u64())
            }
        }

        /// Returns the memory representation of this as a series of `u64`.
        ///
        /// As the target platform's native endianness is used, portable code
        /// should use [`to_be_u64`] or [`to_le_u64`], as appropriate,
        /// instead.
        ///
        /// [`to_be_u64`]: Self::to_be_u64
        /// [`to_le_u64`]: Self::to_le_u64
        #[inline(always)]
        pub const fn to_ne_u64(self) -> [u64; Self::U64_LEN] {
            let bytes = self.to_ne_bytes();
            // SAFETY: plain old data
            unsafe {
                core::mem::transmute::<[u8; Self::BYTES], [u64; Self::U64_LEN]>(bytes)
            }
        }

        /// Creates a native endian integer value from its representation
        /// as `u64` elements in big-endian.
        #[inline(always)]
        pub const fn from_be_u64(value: [u64; Self::U64_LEN]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_u64(value)
            } else {
                Self::from_ne_u64(swap_array!(value))
            }
        }

        /// Creates a native endian integer value from its representation
        /// as `u64` elements in little-endian.
        #[inline(always)]
        pub const fn from_le_u64(value: [u64; Self::U64_LEN]) -> Self {
            if cfg!(target_endian = "big") {
                Self::from_ne_u64(swap_array!(value))
            } else {
                Self::from_ne_u64(value)
            }
        }

        /// Creates a native endian integer value from its memory representation
        /// as `u64` in native endianness.
        ///
        /// As the target platform's native endianness is used, portable code
        /// likely wants to use [`from_be_u64`] or [`from_le_u64`], as
        /// appropriate instead.
        ///
        /// [`from_be_u64`]: Self::from_be_u64
        /// [`from_le_u64`]: Self::from_le_u64
        #[inline(always)]
        pub const fn from_ne_u64(value: [u64; Self::U64_LEN]) -> Self {
            // SAFETY: plain old data
            let bytes = unsafe {
                core::mem::transmute::<[u64; Self::U64_LEN], [u8; Self::BYTES]>(value)
            };
            Self::from_ne_bytes(bytes)
        }
    };
}

/// Defines some of the bitwise operator definitions.
///
/// See the bench on `bit_algos` for some of the choices.
#[rustfmt::skip]
macro_rules! bitops_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Returns the number of ones in the binary representation of `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::count_ones`].")]
        #[inline(always)]
        pub const fn count_ones(self) -> u32 {
            // NOTE: Rust vectorizes this nicely on x86_64.
            let mut count = 0;
            let mut i = 0;
            while i < Self::LIMBS {
                count += self.limbs[i].count_ones();
                i += 1;
            }
            count
        }

        /// Returns the number of zeros in the binary representation of `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::count_zeros`].")]
        #[inline(always)]
        pub const fn count_zeros(self) -> u32 {
            self.not_const().count_ones()
        }

        /// Returns the number of leading zeros in the binary representation of
        /// `self`.
        ///
        /// Depending on what you're doing with the value, you might also be
        /// interested in the `ilog2` function which returns a consistent
        /// number, even if the type widens.
        ///
        /// # Examples
        ///
        /// ```rust
        /// # use i256::i256;
        /// let n = i256::MAX >> 2i32;
        /// assert_eq!(n.leading_zeros(), 3);
        ///
        /// let min = i256::MIN;
        /// assert_eq!(min.leading_zeros(), 0);
        ///
        /// let zero = i256::from_u8(0);
        /// assert_eq!(zero.leading_zeros(), 256);
        ///
        /// let max = i256::MAX;
        /// assert_eq!(max.leading_zeros(), 1);
        /// ```
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::leading_zeros`].")]
        #[inline]
        pub const fn leading_zeros(self) -> u32 {
            let mut count = 0;
            let mut i = 0;
            while i < Self::LIMBS && count == i as u32 * $crate::ULimb::BITS {
                count += self.get_limb(Self::LIMBS - i - 1).leading_zeros();
                i += 1;
            }
            count
        }

        /// Returns the number of trailing zeros in the binary representation of
        /// `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::trailing_zeros`].")]
        #[inline]
        pub const fn trailing_zeros(self) -> u32 {
            let mut count = 0;
            let mut i = 0;
            while i < Self::LIMBS && count == i as u32 * $crate::ULimb::BITS {
                count += self.get_limb(i).trailing_zeros();
                i += 1;
            }
            count
        }

        /// Returns the number of leading ones in the binary representation of
        /// `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::leading_ones`].")]
        #[inline(always)]
        pub const fn leading_ones(self) -> u32 {
            self.not_const().leading_zeros()
        }

        /// Returns the number of trailing ones in the binary representation of
        /// `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::trailing_ones`].")]
        #[inline(always)]
        pub const fn trailing_ones(self) -> u32 {
            self.not_const().trailing_zeros()
        }

        // NOTE: These optimize super well, and flatten out entirely.

        /// Const implementation of `BitAnd`.
        #[inline(always)]
        pub const fn bitand_const(self, rhs: Self) -> Self {
            let lhs_limbs = self.to_ne_limbs();
            let rhs_limbs = rhs.to_ne_limbs();
            let mut result = [0; Self::LIMBS];
            let mut i = 0;
            while i < Self::LIMBS {
                result[i] = lhs_limbs[i] & rhs_limbs[i];
                i += 1;
            }
            Self::from_ne_limbs(result)
        }

        /// Const implementation of `BitOr`.
        #[inline(always)]
        pub const fn bitor_const(self, rhs: Self) -> Self {
            let lhs_limbs = self.to_ne_limbs();
            let rhs_limbs = rhs.to_ne_limbs();
            let mut result = [0; Self::LIMBS];
            let mut i = 0;
            while i < Self::LIMBS {
                result[i] = lhs_limbs[i] | rhs_limbs[i];
                i += 1;
            }
            Self::from_ne_limbs(result)
        }

        /// Const implementation of `BitXor`.
        #[inline(always)]
        pub const fn bitxor_const(self, rhs: Self) -> Self {
            let lhs_limbs = self.to_ne_limbs();
            let rhs_limbs = rhs.to_ne_limbs();
            let mut result = [0; Self::LIMBS];
            let mut i = 0;
            while i < Self::LIMBS {
                result[i] = lhs_limbs[i] ^ rhs_limbs[i];
                i += 1;
            }
            Self::from_ne_limbs(result)
        }

        /// Const implementation of `Not`.
        #[inline(always)]
        pub const fn not_const(self) -> Self {
            let limbs = self.to_ne_limbs();
            let mut result = [0; Self::LIMBS];
            let mut i = 0;
            while i < Self::LIMBS {
                result[i] = !limbs[i];
                i += 1;
            }
            Self::from_ne_limbs(result)
        }
    };
}

/// Define a generic op. This isn't exposed to the crate just so it's done
/// internally. This is intended to be used within the crate so the `*_signed`,
/// `*_unsigned` variants can be added.
///
/// This requires the `wrapping_*` and `overflowing_*` variants to be defined,
/// as well as `div_euclid` and `rem_euclid` to be defined.
#[rustfmt::skip]
macro_rules! ops_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Raises self to the power of `exp`, using exponentiation by squaring.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::pow`].")]
        #[inline]
        pub const fn pow(self, exp: u32) -> Self {
            if cfg!(not(have_overflow_checks)) {
                self.wrapping_pow(exp)
            } else {
                self.strict_pow(exp)
            }
        }

        /// Get the quotient and remainder of our big integer division.
        ///
        /// This allows storing of both the quotient and remainder without
        /// making repeated calls.
        ///
        /// # Panics
        ///
        /// This panics if the divisor is 0.
        #[inline(always)]
        pub fn div_rem(self, n: Self) -> (Self, Self) {
            if cfg!(not(have_overflow_checks)) {
                self.wrapping_div_rem(n)
            } else {
                match self.checked_div_rem(n) {
                    Some(v) => v,
                    _ => core::panic!("attempt to divide with overflow"),
                }
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! bigint_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Calculates `self` + `rhs` + `carry` and returns a tuple containing
        /// the sum and the output carry.
        ///
        /// Performs "ternary addition" of two integer operands and a carry-in
        /// bit, and returns an output integer and a carry-out bit. This allows
        /// chaining together multiple additions to create a wider addition, and
        /// can be useful for bignum addition.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::carrying_add`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
            let (a, b) = self.overflowing_add(rhs);
            let (c, d) = a.overflowing_add_ulimb(carry as $crate::ULimb);
            (c, b | d)
        }

        /// Calculates `self` &minus; `rhs` &minus; `borrow` and returns a tuple
        /// containing the difference and the output borrow.
        ///
        /// Performs "ternary subtraction" by subtracting both an integer
        /// operand and a borrow-in bit from `self`, and returns an output
        /// integer and a borrow-out bit. This allows chaining together multiple
        /// subtractions to create a wider subtraction, and can be useful for
        /// bignum subtraction.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::borrowing_sub`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
            let (a, b) = self.overflowing_sub(rhs);
            let (c, d) = a.overflowing_sub_ulimb(borrow as $crate::ULimb);
            (c, b | d)
        }
    };
}

#[rustfmt::skip]
macro_rules! wrapping_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Wrapping (modular) exponentiation. Computes `self.pow(exp)`,
        /// wrapping around at the boundary of the type.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::wrapping_pow`].")]
        #[inline]
        pub const fn wrapping_pow(self, mut exp: u32) -> Self {
            if exp == 0 {
                return Self::from_u8(1);
            }
            let mut base = self;
            let mut acc = Self::from_u8(1);

            // NOTE: The exponent can never go to 0.
            loop {
                if (exp & 1) == 1 {
                    acc = acc.wrapping_mul(base);
                    // since exp!=0, finally the exp must be 1.
                    if exp == 1 {
                        return acc;
                    }
                }
                exp /= 2;
                base = base.wrapping_mul(base);
                debug_assert!(exp != 0, "logic error in exponentiation, will infinitely loop");
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! overflowing_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Raises self to the power of `exp`, using exponentiation by squaring,
        /// returning the value.
        ///
        /// Returns a tuple of the exponentiation along with a bool indicating
        /// whether an overflow happened.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::overflowing_pow`].")]
        #[inline]
        pub const fn overflowing_pow(self, mut exp: u32) -> (Self, bool) {
            if exp == 0 {
                return (Self::from_u8(1), false);
            }
            let mut base = self;
            let mut acc = Self::from_u8(1);
            let mut overflowed = false;
            let mut r: (Self, bool);

            // NOTE: The exponent can never go to 0.
            loop {
                if (exp & 1) == 1 {
                    r = acc.overflowing_mul(base);
                    // since exp!=0, finally the exp must be 1.
                    if exp == 1 {
                        r.1 |= overflowed;
                        return r;
                    }
                    acc = r.0;
                    overflowed |= r.1;
                }
                exp /= 2;
                r = base.overflowing_mul(base);
                base = r.0;
                overflowed |= r.1;
                debug_assert!(exp != 0, "logic error in exponentiation, will infinitely loop");
            }
        }

        /// Get the quotient and remainder of our big integer division,
        /// returning the value and if overflow occurred.
        ///
        /// This allows storing of both the quotient and remainder without
        /// making repeated calls.
        ///
        /// # Panics
        ///
        /// This function will panic if `rhs` is zero.
        #[inline]
        pub fn overflowing_div_rem(self, n: Self) -> ((Self, Self), bool) {
            if self.is_div_overflow(n) {
                ((self, Self::from_u8(0)), true)
            } else {
                (self.wrapping_div_rem(n), false)
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! saturating_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        // Currently a no-op
    };
}

#[rustfmt::skip]
macro_rules! checked_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Checked integer addition. Computes `self + rhs`, returning `None`
        /// if overflow occurred.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_add`].")]
        #[inline(always)]
        pub const fn checked_add(self, rhs: Self) -> Option<Self> {
            let (value, overflowed) = self.overflowing_add(rhs);
            if !overflowed {
                Some(value)
            } else {
                None
            }
        }

        /// Checked integer subtraction. Computes `self - rhs`, returning `None`
        /// if overflow occurred.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_sub`].")]
        #[inline(always)]
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            let (value, overflowed) = self.overflowing_sub(rhs);
            if !overflowed {
                Some(value)
            } else {
                None
            }
        }

        /// Checked integer multiplication. Computes `self * rhs`, returning `None`
        /// if overflow occurred.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_mul`].")]
        #[inline(always)]
        pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
            let (value, overflowed) = self.overflowing_mul(rhs);
            if !overflowed {
                Some(value)
            } else {
                None
            }
        }

        /// Checked exponentiation. Computes `self.pow(exp)`, returning `None`
        /// if overflow occurred.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_pow`].")]
        #[inline(always)]
        pub const fn checked_pow(self, base: u32) -> Option<Self> {
            match self.overflowing_pow(base) {
                (value, false) => Some(value),
                _ => None,
            }
        }

        /// Checked integer division. Computes `self / rhs`, returning `None`
        /// `rhs == 0` or the division results in overflow (signed only).
        ///
        /// This allows storing of both the quotient and remainder without
        /// making repeated calls.
        #[inline]
        pub fn checked_div_rem(self, n: Self) -> Option<(Self, Self)> {
            if self.is_div_none(n) {
                None
            } else {
                Some(self.wrapping_div_rem(n))
            }
        }

        /// Checked integer division. Computes `self / rhs`, returning `None`
        /// `rhs == 0` or the division results in overflow (signed only).
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_div`].")]
        #[inline(always)]
        pub fn checked_div(self, rhs: Self) -> Option<Self> {
            if self.is_div_none(rhs) {
                None
            } else {
                Some(self.wrapping_div(rhs))
            }
        }

        /// Checked integer division. Computes `self % rhs`, returning `None`
        /// `rhs == 0` or the division results in overflow (signed only).
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_rem`].")]
        #[inline(always)]
        pub fn checked_rem(self, rhs: Self) -> Option<Self> {
            if self.is_div_none(rhs) {
                None
            } else {
                Some(self.wrapping_rem(rhs))
            }
        }

        /// Checked Euclidean division. Computes `self.div_euclid(rhs)`,
        /// returning `None` if `rhs == 0` or the division results in
        /// overflow (signed only).
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_div_euclid`].")]
        #[inline(always)]
        pub fn checked_div_euclid(self, rhs: Self) -> Option<Self> {
            if self.is_div_none(rhs) {
                None
            } else {
                Some(self.wrapping_div_euclid(rhs))
            }
        }

        /// Checked Euclidean modulo. Computes `self.rem_euclid(rhs)`,
        /// returning `None` if `rhs == 0` or the division results in
        /// overflow (signed only).
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_rem_euclid`].")]
        #[inline(always)]
        pub fn checked_rem_euclid(self, rhs: Self) -> Option<Self> {
            if self.is_div_none(rhs) {
                None
            } else {
                Some(self.wrapping_rem_euclid(rhs))
            }
        }

        /// Checked shift left. Computes `self << rhs`, returning `None` if `rhs` is
        /// larger than or equal to the number of bits in `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_shl`].")]
        #[inline(always)]
        pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
            // Not using overflowing_shl as that's a wrapping shift
            if rhs < Self::BITS {
                Some(self.wrapping_shl(rhs))
            } else {
                None
            }
        }

        /// Checked shift right. Computes `self >> rhs`, returning `None` if `rhs`
        /// is larger than or equal to the number of bits in `self`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_shr`].")]
        #[inline(always)]
        pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
            // Not using overflowing_shr as that's a wrapping shift
            if rhs < Self::BITS {
                Some(self.wrapping_shr(rhs))
            } else {
                None
            }
        }

        /// Returns the base 2 logarithm of the number, rounded down.
        ///
        /// Returns `None` if the number is negative or zero.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::checked_ilog2`].")]
        #[inline]
        pub const fn checked_ilog2(self) -> Option<u32> {
            match self.le_const(Self::from_u8(0)) {
                true => None,
                false => Some(Self::BITS - 1 - self.leading_zeros()),
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! strict_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Strict integer addition. Computes `self + rhs`, panicking
        /// if overflow occurred.
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// This function will always panic on overflow, regardless of whether
        /// overflow checks are enabled.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::strict_add`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn strict_add(self, rhs: Self) -> Self {
            match self.checked_add(rhs) {
                Some(v) => v,
                None => core::panic!("attempt to add with overflow"),
            }
        }

        /// Strict integer subtraction. Computes `self - rhs`, panicking if
        /// overflow occurred.
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// This function will always panic on overflow, regardless of whether
        /// overflow checks are enabled.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::strict_sub`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn strict_sub(self, rhs: Self) -> Self {
            match self.checked_sub(rhs) {
                Some(v) => v,
                None => core::panic!("attempt to subtract with overflow"),
            }
        }

        /// Strict integer multiplication. Computes `self * rhs`, panicking if
        /// overflow occurred.
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// This function will always panic on overflow, regardless of whether
        /// overflow checks are enabled.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::strict_mul`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn strict_mul(self, rhs: Self) -> Self {
            match self.checked_mul(rhs) {
                Some(v) => v,
                None => core::panic!("attempt to subtract with overflow"),
            }
        }

        /// Strict exponentiation. Computes `self.pow(exp)`, panicking if
        /// overflow occurred.
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// This function will always panic on overflow, regardless of whether
        /// overflow checks are enabled.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::strict_pow`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn strict_pow(self, rhs: u32) -> Self {
            match self.checked_pow(rhs) {
                Some(v) => v,
                None => core::panic!("attempt to multiply with overflow"),
            }
        }

        /// Strict shift left. Computes `self << rhs`, panicking if `rhs` is larger
        /// than or equal to the number of bits in `self`.
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// This function will always panic on overflow, regardless of whether
        /// overflow checks are enabled.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::strict_shl`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn strict_shl(self, rhs: u32) -> Self {
            match self.checked_shl(rhs) {
                Some(v) => v,
                None => core::panic!("attempt to shift left with overflow"),
            }
        }

        /// Strict shift right. Computes `self >> rhs`, panicking `rhs` is
        /// larger than or equal to the number of bits in `self`.
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// This function will always panic on overflow, regardless of whether
        /// overflow checks are enabled.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::strict_shr`].")]
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn strict_shr(self, rhs: u32) -> Self {
            match self.checked_shr(rhs) {
                Some(v) => v,
                None => core::panic!("attempt to shift right with overflow"),
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! unchecked_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Unchecked integer addition. Computes `self + rhs`, assuming overflow
        /// cannot occur.
        ///
        /// Calling `x.unchecked_add(y)` is semantically equivalent to calling
        /// `x.`[`checked_add`]`(y).`[`unwrap_unchecked`]`()`.
        ///
        /// If you're just trying to avoid the panic in debug mode, then **do not**
        /// use this.  Instead, you're looking for [`wrapping_add`].
        ///
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when the value overflows.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::unchecked_add`].")]
        ///
        /// [`checked_add`]: Self::checked_add
        /// [`wrapping_add`]: Self::wrapping_add
        /// [`unwrap_unchecked`]: Option::unwrap_unchecked
        #[must_use]
        #[inline(always)]
        pub unsafe fn unchecked_add(self, rhs: Self) -> Self {
            match self.checked_add(rhs) {
                Some(value) => value,
                // SAFETY: this is guaranteed to be safe by the caller.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }

        /// Unchecked integer subtraction. Computes `self - rhs`, assuming overflow
        /// cannot occur.
        ///
        /// Calling `x.unchecked_sub(y)` is semantically equivalent to calling
        /// `x.`[`checked_sub`]`(y).`[`unwrap_unchecked`]`()`.
        ///
        /// If you're just trying to avoid the panic in debug mode, then **do not**
        /// use this.  Instead, you're looking for [`wrapping_sub`].
        ///
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when the value overflows.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::unchecked_sub`].")]
        ///
        /// [`checked_sub`]: Self::checked_sub
        /// [`wrapping_sub`]: Self::wrapping_sub
        /// [`unwrap_unchecked`]: Option::unwrap_unchecked
        #[must_use]
        #[inline(always)]
        pub unsafe fn unchecked_sub(self, rhs: Self) -> Self {
            match self.checked_sub(rhs) {
                Some(value) => value,
                // SAFETY: this is guaranteed to be safe by the caller.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }

        /// Unchecked integer multiplication. Computes `self * rhs`, assuming
        /// overflow cannot occur.
        ///
        /// Calling `x.unchecked_mul(y)` is semantically equivalent to calling
        /// `x.`[`checked_mul`]`(y).`[`unwrap_unchecked`]`()`.
        ///
        /// If you're just trying to avoid the panic in debug mode, then **do not**
        /// use this.  Instead, you're looking for [`wrapping_mul`].
        ///
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior when the value overflows.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::unchecked_mul`].")]
        ///
        /// [`wrapping_mul`]: Self::wrapping_mul
        /// [`checked_mul`]: Self::checked_mul
        /// [`unwrap_unchecked`]: Option::unwrap_unchecked
        #[must_use]
        #[inline(always)]
        pub const unsafe fn unchecked_mul(self, rhs: Self) -> Self {
            match self.checked_mul(rhs) {
                Some(value) => value,
                // SAFETY: this is guaranteed to be safe by the caller.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }

        /// Unchecked shift left. Computes `self << rhs`, assuming that
        /// `rhs` is less than the number of bits in `self`.
        ///
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior if `rhs` is larger than
        /// or equal to the number of bits in `self`,
        /// i.e. when [`checked_shl`] would return `None`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::unchecked_shl`].")]
        ///
        /// [`checked_shl`]: Self::checked_shl
        #[must_use]
        #[inline(always)]
        pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
            match self.checked_shl(rhs) {
                Some(value) => value,
                // SAFETY: this is guaranteed to be safe by the caller.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }

        /// Unchecked shift right. Computes `self >> rhs`, assuming that
        /// `rhs` is less than the number of bits in `self`.
        ///
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        ///
        /// # Safety
        ///
        /// This results in undefined behavior if `rhs` is larger than
        /// or equal to the number of bits in `self`,
        /// i.e. when [`checked_shr`] would return `None`.
        ///
        #[doc = concat!("See [`", stringify!($wide_t), "::unchecked_shr`].")]
        ///
        /// [`checked_shr`]: Self::checked_shr
        #[must_use]
        #[inline(always)]
        pub const unsafe fn unchecked_shr(self, rhs: u32) -> Self {
            match self.checked_shr(rhs) {
                Some(value) => value,
                // SAFETY: this is guaranteed to be safe by the caller.
                None => unsafe { core::hint::unreachable_unchecked() },
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! unbounded_define {
    (type => $t:ty,wide_type => $wide_t:ty) => {
        /// Unbounded shift left. Computes `self << rhs`, without bounding the value
        /// of `rhs`.
        ///
        /// If `rhs` is larger or equal to the number of bits in `self`,
        /// the entire value is shifted out, and `0` is returned.
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn unbounded_shl(self, rhs: u32) -> Self {
            if rhs < Self::BITS {
                self.wrapping_shl(rhs)
            } else {
                Self::from_u8(0)
            }
        }

        /// Unbounded shift right. Computes `self >> rhs`, without bounding the
        /// value of `rhs`.
        ///
        /// If `rhs` is larger or equal to the number of bits in `self`,
        /// the entire value is shifted out, and `0` is returned.
        ///
        /// <div class="warning">
        /// This is a nightly-only experimental API in the Rust core implementation,
        /// and therefore is subject to change at any time.
        /// </div>
        #[inline]
        #[must_use]
        pub const fn unbounded_shr(self, rhs: u32) -> Self {
            if rhs < Self::BITS {
                self.wrapping_shr(rhs)
            } else {
                Self::from_u8(0)
            }
        }
    };
}

macro_rules! limb_ops_define {
    () => {
        /// Add an unsigned limb to the big integer.
        ///
        /// This allows optimizations a full addition cannot do.
        #[inline(always)]
        pub const fn add_ulimb(self, n: $crate::ULimb) -> Self {
            if cfg!(not(have_overflow_checks)) {
                self.wrapping_add_ulimb(n)
            } else {
                match self.checked_add_ulimb(n) {
                    Some(v) => v,
                    None => core::panic!("attempt to add with overflow"),
                }
            }
        }

        /// Subtract an unsigned limb from the big integer.
        ///
        /// This allows optimizations a full subtraction cannot do.
        #[inline(always)]
        pub const fn sub_ulimb(self, n: $crate::ULimb) -> Self {
            if cfg!(not(have_overflow_checks)) {
                self.wrapping_sub_ulimb(n)
            } else {
                match self.checked_sub_ulimb(n) {
                    Some(v) => v,
                    _ => core::panic!("attempt to subtract with overflow"),
                }
            }
        }

        /// Multiply our big integer by an unsigned limb.
        ///
        /// This allows optimizations a full multiplication cannot do.
        #[inline(always)]
        pub const fn mul_ulimb(self, n: $crate::ULimb) -> Self {
            if cfg!(not(have_overflow_checks)) {
                self.wrapping_mul_ulimb(n)
            } else {
                match self.checked_mul_ulimb(n) {
                    Some(v) => v,
                    None => core::panic!("attempt to multiply with overflow"),
                }
            }
        }

        /// Get the quotient and remainder of our big integer divided
        /// by an unsigned limb.
        ///
        /// This allows optimizations a full division cannot do.
        ///
        /// # Panics
        ///
        /// This panics if the divisor is 0.
        #[inline(always)]
        pub fn div_rem_ulimb(self, n: $crate::ULimb) -> (Self, $crate::ULimb) {
            if cfg!(not(have_overflow_checks)) {
                self.wrapping_div_rem_ulimb(n)
            } else {
                match self.checked_div_rem_ulimb(n) {
                    Some(v) => v,
                    _ => core::panic!("attempt to divide with overflow"),
                }
            }
        }

        /// Get the quotient of our big integer divided by an unsigned limb.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn div_ulimb(self, n: $crate::ULimb) -> Self {
            self.div_rem_ulimb(n).0
        }

        /// Get the remainder of our big integer divided by an unsigned limb.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn rem_ulimb(self, n: $crate::ULimb) -> $crate::ULimb {
            self.div_rem_ulimb(n).1
        }
    };

    (@wrapping) => {
        /// Get the quotient of our big integer divided by an unsigned limb,
        /// wrapping on overflow.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn wrapping_div_ulimb(self, n: $crate::ULimb) -> Self {
            self.wrapping_div_rem_ulimb(n).0
        }

        /// Get the remainder of our big integer divided by an unsigned limb,
        /// wrapping on overflow.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn wrapping_rem_ulimb(self, n: $crate::ULimb) -> $crate::ULimb {
            self.wrapping_div_rem_ulimb(n).1
        }
    };

    (@overflowing) => {
        /// Get the quotient and remainder of our big integer divided
        /// by an unsigned limb, returning the value and if overflow
        /// occurred.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline]
        pub fn overflowing_div_rem_ulimb(self, n: $crate::ULimb) -> ((Self, $crate::ULimb), bool) {
            (self.wrapping_div_rem_ulimb(n), false)
        }

        /// Get the quotient of our big integer divided
        /// by an unsigned limb, returning the value and if overflow
        /// occurred.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn overflowing_div_ulimb(self, n: $crate::ULimb) -> (Self, bool) {
            let (value, overflowed) = self.overflowing_div_rem_ulimb(n);
            (value.0, overflowed)
        }

        /// Get the remainder of our big integer divided
        /// by an unsigned limb, returning the value and if overflow
        /// occurred.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn overflowing_rem_ulimb(self, n: $crate::ULimb) -> ($crate::ULimb, bool) {
            let (value, overflowed) = self.overflowing_div_rem_ulimb(n);
            (value.1, overflowed)
        }
    };

    (@checked) => {
        /// Add an unsigned limb to the big integer, returning None on overflow.
        ///
        /// This allows optimizations a full addition cannot do.
        #[inline(always)]
        pub const fn checked_add_ulimb(self, n: $crate::ULimb) -> Option<Self> {
            let (value, overflowed) = self.overflowing_add_ulimb(n);
            if overflowed {
                None
            } else {
                Some(value)
            }
        }

        /// Subtract an unsigned limb from the big integer, returning None on overflow.
        ///
        /// This allows optimizations a full addition cannot do.
        #[inline(always)]
        pub const fn checked_sub_ulimb(self, n: $crate::ULimb) -> Option<Self> {
            let (value, overflowed) = self.overflowing_sub_ulimb(n);
            if overflowed {
                None
            } else {
                Some(value)
            }
        }

        /// Multiply our big integer by an unsigned limb, returning None on overflow.
        ///
        /// This allows optimizations a full multiplication cannot do.
        #[inline(always)]
        pub const fn checked_mul_ulimb(self, n: $crate::ULimb) -> Option<Self> {
            let (value, overflowed) = self.overflowing_mul_ulimb(n);
            if overflowed {
                None
            } else {
                Some(value)
            }
        }

        /// Get the quotient of our big integer divided by an unsigned
        /// limb, returning None on overflow or division by 0.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline]
        pub fn checked_div_rem_ulimb(self, n: $crate::ULimb) -> Option<(Self, $crate::ULimb)> {
            if n == 0 {
                None
            } else {
                Some(self.wrapping_div_rem_ulimb(n))
            }
        }

        /// Get the quotient of our big integer divided by an unsigned
        /// limb, returning None on overflow or division by 0.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn checked_div_ulimb(self, n: $crate::ULimb) -> Option<Self> {
            Some(self.checked_div_rem_ulimb(n)?.0)
        }

        /// Get the remainder of our big integer divided by a signed
        /// limb, returning None on overflow or division by 0.
        ///
        /// This allows optimizations a full division cannot do.
        #[inline(always)]
        pub fn checked_rem_ulimb(self, n: $crate::ULimb) -> Option<$crate::ULimb> {
            Some(self.checked_div_rem_ulimb(n)?.1)
        }
    };

    (@all) => {
        limb_define!();
        limb_define!(@wrapping);
        limb_define!(@overflowing);
        limb_define!(@checked);
    };
}

macro_rules! binop_trait_define {
    ($t:ty, $trait:ident, $assign:ident, $op:ident, $op_assign:ident) => {
        impl $trait<&$t> for $t {
            type Output = <Self as $trait>::Output;

            #[inline(always)]
            fn $op(self, rhs: &Self) -> Self::Output {
                self.$op(*rhs)
            }
        }

        impl $assign for $t {
            #[inline(always)]
            fn $op_assign(&mut self, other: Self) {
                *self = self.$op(other);
            }
        }

        impl $assign<&$t> for $t {
            #[inline(always)]
            fn $op_assign(&mut self, other: &Self) {
                *self = self.$op(other);
            }
        }
    };
}

macro_rules! ref_trait_define {
    ($t:ty, $trait:ident, $op:ident $(, $args:tt:$type:ty)*) => {
        impl $trait for &$t {
            type Output = <$t as $trait>::Output;

            #[inline(always)]
            fn $op(self $(, $args: $type)*) -> Self::Output {
                $trait::$op(*self $(, $args)*)
            }
        }
    };
}

macro_rules! from_trait_define {
    ($to:ty, $from:ty, $op:ident) => {
        impl From<$from> for $to {
            #[inline(always)]
            fn from(value: $from) -> Self {
                Self::$op(value)
            }
        }
    };
}

macro_rules! binop_ref_trait_define {
    ($t:ty, $trait:ident, $op:ident) => {
        impl $trait<&$t> for $t {
            type Output = <$t as $trait>::Output;

            #[inline(always)]
            fn $op(self, other: &$t) -> Self::Output {
                $trait::$op(self, *other)
            }
        }
    };
}

macro_rules! shift_define {
    (@mod base => $base:ty, impl => $($t:ty)*) => ($(
        impl Shl<$t> for $base {
            type Output = Self;

            #[inline(always)]
            #[allow(unused_comparisons)]
            #[allow(clippy::suspicious_arithmetic_impl)]
            fn shl(self, other: $t) -> Self::Output {
                if cfg!(have_overflow_checks) {
                    assert!(other < Self::BITS as $t && other >= 0, "attempt to shift left with overflow");
                }
                self.wrapping_shl(other as u32)
            }
        }

        impl Shr<$t> for $base {
            type Output = Self;

            #[inline(always)]
            #[allow(unused_comparisons)]
            #[allow(clippy::suspicious_arithmetic_impl)]
            fn shr(self, other: $t) -> Self::Output {
                if cfg!(have_overflow_checks) {
                    assert!(other < Self::BITS as $t && other >= 0, "attempt to shift right with overflow");
                }
                self.wrapping_shr(other as u32)
            }
        }
    )*);

    (@256 base => $base:ty, impl => $($t:ty)*) => ($(
        impl Shl<$t> for $base {
            type Output = Self;

            #[inline(always)]
            #[allow(clippy::suspicious_arithmetic_impl)]
            fn shl(self, other: $t) -> Self::Output {
                if cfg!(have_overflow_checks) {
                    let is_above = other.ge_const(<$t>::from_u32(Self::BITS));
                    let is_below = other.lt_const(<$t>::from_u32(0));
                    let is_overflow = is_above || is_below;
                    assert!(!is_overflow, "attempt to shift right with overflow");
                }
                self.wrapping_shl(other.as_u32())
            }
        }

        impl Shr<$t> for $base {
            type Output = Self;

            #[inline(always)]
            #[allow(clippy::suspicious_arithmetic_impl)]
            fn shr(self, other: $t) -> Self::Output {
                if cfg!(have_overflow_checks) {
                    let is_above = other.ge_const(<$t>::from_u32(Self::BITS));
                    let is_below = other.lt_const(<$t>::from_u32(0));
                    let is_overflow = is_above || is_below;
                    assert!(!is_overflow, "attempt to shift right with overflow");
                }
                self.wrapping_shr(other.as_u32())
            }
        }
    )*);

    (base => $base:ty, impl => $($t:ty)*) => ($(
        impl Shl<&$t> for $base {
            type Output = <Self as Shl>::Output;

            #[inline(always)]
            fn shl(self, other: &$t) -> Self::Output {
                self.shl(*other)
            }
        }

        impl ShlAssign<$t> for $base {
            #[inline(always)]
            fn shl_assign(&mut self, other: $t) {
                *self = self.shl(other);
            }
        }

        impl ShlAssign<&$t> for $base {
            #[inline(always)]
            fn shl_assign(&mut self, other: &$t) {
                *self = self.shl(other);
            }
        }

        impl Shr<&$t> for $base {
            type Output = <Self as Shr>::Output;

            #[inline(always)]
            fn shr(self, other: &$t) -> Self::Output {
                self.shr(*other)
            }
        }

        impl ShrAssign<$t> for $base {
            #[inline(always)]
            fn shr_assign(&mut self, other: $t) {
                *self = self.shr(other);
            }
        }

        impl ShrAssign<&$t> for $base {
            #[inline(always)]
            fn shr_assign(&mut self, other: &$t) {
                *self = self.shr(other);
            }
        }
    )*);
}

macro_rules! traits_define {
    ($t:ty) => {
        impl Add for $t {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                if cfg!(not(have_overflow_checks)) {
                    self.wrapping_add(rhs)
                } else {
                    match self.checked_add(rhs) {
                        Some(v) => v,
                        _ => core::panic!("attempt to add with overflow"),
                    }
                }
            }
        }

        binop_trait_define!($t, Add, AddAssign, add, add_assign);

        impl BitAnd for $t {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                self.bitand_const(rhs)
            }
        }

        binop_trait_define!($t, BitAnd, BitAndAssign, bitand, bitand_assign);

        impl BitOr for $t {
            type Output = $t;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                self.bitor_const(rhs)
            }
        }

        binop_trait_define!($t, BitOr, BitOrAssign, bitor, bitor_assign);

        impl BitXor for $t {
            type Output = Self;

            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self::Output {
                self.bitxor_const(rhs)
            }
        }

        binop_trait_define!($t, BitXor, BitXorAssign, bitxor, bitxor_assign);

        impl Div for $t {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                if cfg!(not(have_overflow_checks)) {
                    self.wrapping_div(rhs)
                } else {
                    match self.checked_div(rhs) {
                        Some(v) => v,
                        _ => core::panic!("attempt to divide with overflow"),
                    }
                }
            }
        }

        binop_trait_define!($t, Div, DivAssign, div, div_assign);

        impl Mul for $t {
            type Output = $t;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                if cfg!(not(have_overflow_checks)) {
                    self.wrapping_mul(rhs)
                } else {
                    match self.checked_mul(rhs) {
                        Some(v) => v,
                        _ => core::panic!("attempt to multiply with overflow"),
                    }
                }
            }
        }

        binop_trait_define!($t, Mul, MulAssign, mul, mul_assign);

        impl Not for $t {
            type Output = $t;

            #[inline(always)]
            fn not(self) -> Self::Output {
                self.not_const()
            }
        }

        ref_trait_define!($t, Not, not);

        impl core::cmp::Ord for $t {
            #[inline(always)]
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.cmp_const(*other)
            }
        }

        impl core::cmp::PartialOrd for $t {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }

            #[inline(always)]
            fn lt(&self, other: &Self) -> bool {
                self.lt_const(*other)
            }

            #[inline(always)]
            fn le(&self, other: &Self) -> bool {
                self.le_const(*other)
            }

            #[inline(always)]
            fn gt(&self, other: &Self) -> bool {
                self.gt_const(*other)
            }

            #[inline(always)]
            fn ge(&self, other: &Self) -> bool {
                self.ge_const(*other)
            }
        }

        impl Rem for $t {
            type Output = $t;

            #[inline(always)]
            fn rem(self, rhs: Self) -> Self::Output {
                if cfg!(not(have_overflow_checks)) {
                    self.wrapping_rem(rhs)
                } else {
                    match self.checked_rem(rhs) {
                        Some(v) => v,
                        _ => core::panic!("attempt to divide with overflow"),
                    }
                }
            }
        }

        binop_trait_define!($t, Rem, RemAssign, rem, rem_assign);

        impl Sub for $t {
            type Output = $t;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                if cfg!(not(have_overflow_checks)) {
                    self.wrapping_sub(rhs)
                } else {
                    match self.checked_sub(rhs) {
                        Some(v) => v,
                        _ => core::panic!("attempt to subtract with overflow"),
                    }
                }
            }
        }

        binop_trait_define!($t, Sub, SubAssign, sub, sub_assign);

        impl Shl for $t {
            type Output = Self;

            #[inline(always)]
            #[allow(clippy::suspicious_arithmetic_impl)]
            fn shl(self, other: Self) -> Self::Output {
                let shift = other.low() as u32 & u32::MAX;
                self.wrapping_shl(shift)
            }
        }

        ref_trait_define!($t, Shl, shl, other: &$t);
        binop_ref_trait_define!($t, Shl, shl);

        impl Shr for $t {
            type Output = Self;

            #[inline(always)]
            #[allow(clippy::suspicious_arithmetic_impl)]
            fn shr(self, other: Self) -> Self::Output {
                let shift = other.low() as u32 & u32::MAX;
                self.wrapping_shr(shift)
            }
        }

        ref_trait_define!($t, Shr, shr, other: &$t);
        binop_ref_trait_define!($t, Shr, shr);
        shift_define! { @mod base => $t, impl => i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }
        shift_define! { base => $t, impl => i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }

        impl core::fmt::Debug for $t {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                core::fmt::Display::fmt(self, f)
            }
        }

        impl From<bool> for $t {
            #[inline(always)]
            fn from(small: bool) -> Self {
                Self::from_u8(small as u8)
            }
        }

        impl From<char> for $t {
            #[inline(always)]
            fn from(c: char) -> Self {
                Self::from_u32(c as u32)
            }
        }

        from_trait_define!($t, u8, from_u8);
        from_trait_define!($t, u16, from_u16);
        from_trait_define!($t, u32, from_u32);
        from_trait_define!($t, u64, from_u64);
        from_trait_define!($t, u128, from_u128);
    };
}

macro_rules! try_from_define {
    (base => $base:ty, from => $($t:ty)*) => ($(
        impl TryFrom<$t> for $base {
            type Error = $crate::TryFromIntError;

            #[inline(always)]
            fn try_from(u: $t) -> Result<Self, $crate::TryFromIntError> {
                if u >= 0 {
                    Ok(Self::from_u128(u as u128))
                } else {
                    Err($crate::TryFromIntError {})
                }
            }
        }
    )*);
}

// Internal implementation helpers.
pub(crate) use associated_consts_define;
pub(crate) use bigint_define;
pub(crate) use binop_ref_trait_define;
pub(crate) use binop_trait_define;
pub(crate) use bitops_define;
pub(crate) use byte_order_define;
pub(crate) use casts_define;
pub(crate) use checked_define;
pub(crate) use cmp_define;
pub(crate) use extensions_define;
pub(crate) use from_trait_define;
pub(crate) use high_low_define;
pub(crate) use int_define;
pub(crate) use limb_ops_define;
pub(crate) use ops_define;
pub(crate) use overflowing_define;
pub(crate) use ref_trait_define;
pub(crate) use saturating_define;
pub(crate) use shift_define;
pub(crate) use strict_define;
pub(crate) use traits_define;
pub(crate) use try_from_define;
pub(crate) use unbounded_define;
pub(crate) use unchecked_define;
pub(crate) use wrapping_define;

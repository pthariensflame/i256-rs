//! An unsigned big integer type.
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

pub(crate) mod bitops;
pub(crate) mod casts;
pub(crate) mod checked;
pub(crate) mod constants;
pub(crate) mod limb;
pub(crate) mod ops;
pub(crate) mod overflowing;
pub(crate) mod saturating;
pub(crate) mod strict;
pub(crate) mod traits;
pub(crate) mod wrapping;

/// Define an unsigned integer.
///
/// Sample use is:
///
/// ```rust,ignore
/// crate::uint::define!(
///     name => u256,
///     signed_t => crate::i256,
///     signed_wide_t => i128,
///     unsigned_wide_t => u128,
///     bits => 256,
/// );
/// ```
macro_rules! define {
    (
        name => $name:ident,
        signed_t => $s_t:ty,
        signed_wide_t => $wide_s_t:ty,
        unsigned_wide_t => $wide_u_t:ty,
        bits => $bits:expr $(,)?
    ) => {
        $crate::shared::int_struct_define!(
            name => $name,
            bits => $bits,
            kind => unsigned,
        );

        impl $name {
            $crate::uint::constants::define!(
                bits => $bits,
                wide_type => $wide_u_t,
            );
            $crate::uint::bitops::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::shared::endian::define!(type => $s_t, wide_type => $wide_s_t);
            $crate::shared::ord::define!(
                low_type => $wide_u_t,
                high_type => $wide_u_t,
            );
            $crate::uint::casts::define!(
                signed_type => $s_t,
                bits => $bits,
                wide_type => $wide_u_t,
                kind => unsigned,
            );
            $crate::shared::extensions::define!(high_type => $crate::ULimb);
            $crate::uint::ops::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::shared::bigint::define!(wide_type => $wide_u_t);
            $crate::uint::wrapping::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::uint::overflowing::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::uint::saturating::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::uint::checked::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::uint::strict::define!(signed_type => $s_t, wide_type => $wide_u_t);
            $crate::shared::unchecked::define!(type => $s_t, wide_type => $wide_u_t);
            $crate::shared::unbounded::define!(type => $s_t, wide_type => $wide_u_t);
            $crate::uint::limb::define!(@all);

            $crate::parse::define!(false);
            $crate::write::define!(false);
        }

        $crate::uint::traits::define!(type => $name, signed_type => $s_t);
    };
}

pub(crate) use define;

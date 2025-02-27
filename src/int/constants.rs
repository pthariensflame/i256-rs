//! Associated constants for our types.

#[rustfmt::skip]
macro_rules! define {
    (
        bits => $bits:expr,
        wide_type => $wide_t:ty,
        see_type => $see_t:ty $(,)?
    ) => {
        $crate::shared::constants::define!(
            bits => $bits,
            wide_type => $wide_t,
            low_type => $crate::ULimb,
            high_type => $crate::ILimb,
            see_type => $see_t,
        );

        #[deprecated]
        #[inline(always)]
        #[doc = $crate::shared::constants::min_value_doc!($see_t)]
        pub const fn min_value() -> Self {
            let mut limbs = [0; Self::LIMBS];
            ne_index!(limbs[Self::LIMBS - 1] = $crate::ILimb::MIN as $crate::ULimb);
            Self::from_ne_limbs(limbs)
        }

        #[deprecated]
        #[inline(always)]
        #[doc = $crate::shared::constants::max_value_doc!($see_t)]
        pub const fn max_value() -> Self {
            let mut limbs = [$crate::ULimb::MAX; Self::LIMBS];
            ne_index!(limbs[Self::LIMBS - 1] = $crate::ILimb::MAX as $crate::ULimb);
            Self::from_ne_limbs(limbs)
        }

        #[doc = $crate::shared::constants::is_signed_doc!()]
        pub const IS_SIGNED: bool = true;
    };
}

pub(crate) use define;

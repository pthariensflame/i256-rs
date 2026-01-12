macro_rules! define {
    (
        type => $t:ty,
    ) => {
        impl core::iter::Sum for $t {
            # [inline(always)]
            fn sum<I> (iter: I) -> Self
            where
                I: core::iter::Iterator<Item = Self>,
            {
                iter.fold(Self::from_u8(0), |acc, element| acc + element)
            }
        }

        impl core::iter::Product for $t {
            # [inline(always)]
            fn product<I> (iter: I) -> Self
            where
                I: core::iter::Iterator<Item = Self>,
            {
                iter.fold(Self::from_u8(1), |acc, element| acc * element)
            }
        }
    }
}

pub(crate) use define;

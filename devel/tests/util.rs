#![allow(dead_code, unused_macros, unused_macro_rules)]

use std::mem;

pub use bnum::types::{I256 as Bi256, U256 as Bu256};
pub use i256::ULimb;

pub fn to_ubnum(x: u128, y: u128) -> Bu256 {
    let buf = [x.to_le_bytes(), y.to_le_bytes()];
    // SAFETY: plain old data
    let slc = unsafe { mem::transmute::<[[u8; 16]; 2], [u8; 32]>(buf) };
    Bu256::from_le_slice(&slc).unwrap()
}

pub fn to_ibnum(x: u128, y: i128) -> Bi256 {
    let buf = [x.to_le_bytes(), y.to_le_bytes()];
    // SAFETY: plain old data
    let slc = unsafe { mem::transmute::<[[u8; 16]; 2], [u8; 32]>(buf) };
    Bi256::from_le_slice(&slc).unwrap()
}

pub fn to_u256(x: u128, y: u128) -> i256::u256 {
    if ULimb::BITS == 64 {
        i256::u256::from_le_u64([x as u64, (x >> 64) as u64, y as u64, (y >> 64) as u64])
    } else {
        i256::u256::from_le_u32([
            x as u32,
            (x >> 32) as u32,
            (x >> 64) as u32,
            (x >> 96) as u32,
            y as u32,
            (y >> 32) as u32,
            (y >> 64) as u32,
            (y >> 96) as u32,
        ])
    }
}

pub fn to_i256(x: u128, y: i128) -> i256::i256 {
    to_u256(x, y as u128).as_signed()
}

macro_rules! unsigned_op_equal {
    ($x0:ident, $x1:ident, $op:ident, $cmp:expr) => {{
        let bx = util::to_ubnum($x0, $x1);
        let bres = bx.$op();

        let ux = util::to_u256($x0, $x1);
        let ures = ux.$op();

        $cmp(bres, ures)
    }};

    ($x0:ident, $x1:ident, $y:ident, $op:ident, $cmp:expr) => {{
        let bx = util::to_ubnum($x0, $x1);
        let bres = bx.$op($y);

        let ux = util::to_u256($x0, $x1);
        let ures = ux.$op($y);

        $cmp(bres, ures)
    }};

    ($x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident, $cmp:expr) => {{
        let bx = util::to_ubnum($x0, $x1);
        let by = util::to_ubnum($y0, $y1);
        let bres = bx.$op(by);

        let ux = util::to_u256($x0, $x1);
        let uy = util::to_u256($y0, $y1);
        let ures = ux.$op(uy);

        $cmp(bres, ures)
    }};

    (wrap $x0:ident, $x1:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $op, |x: util::Bu256, y: i256::u256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (wrap $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $y, $op, |x: util::Bu256, y: i256::u256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (wrap $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $y0, $y1, $op, |x: util::Bu256, y: i256::u256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (over $x0:ident, $x1:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $op, |x: (util::Bu256, bool), y: (i256::u256, bool)| {
            x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
        })
    }};

    (over $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $y, $op, |x: (util::Bu256, bool), y: (i256::u256, bool)| {
            x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
        })
    }};

    (over $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        unsigned_op_equal!(
            $x0,
            $x1,
            $y0,
            $y1,
            $op,
            |x: (util::Bu256, bool), y: (i256::u256, bool)| {
                x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
            }
        )
    }};

    (check $x0:ident, $x1:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $op, |x: Option<util::Bu256>, y: Option<i256::u256>| {
            x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
        })
    }};

    (check $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $y, $op, |x: Option<util::Bu256>, y: Option<i256::u256>| {
            x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
        })
    }};

    (check $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        unsigned_op_equal!(
            $x0,
            $x1,
            $y0,
            $y1,
            $op,
            |x: Option<util::Bu256>, y: Option<i256::u256>| {
                x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
            }
        )
    }};

    (scalar $x0:ident, $x1:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $op, |x, y| x == y)
    }};

    (scalar $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $y, $o, |x, y| x == y)
    }};

    (scalar $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        unsigned_op_equal!($x0, $x1, $y0, $y1, $op, |x, y| x == y)
    }};
}

macro_rules! signed_op_equal {
    ($x0:ident, $x1:ident, $op:ident, $cmp:expr) => {{
        let bx = util::to_ibnum($x0, $x1);
        let bres = bx.$op();

        let ux = util::to_i256($x0, $x1);
        let ures = ux.$op();

        $cmp(bres, ures)
    }};

    ($x0:ident, $x1:ident, $y:ident, $op:ident, $cmp:expr) => {{
        let bx = util::to_ibnum($x0, $x1);
        let bres = bx.$op($y);

        let ux = util::to_i256($x0, $x1);
        let ures = ux.$op($y);

        $cmp(bres, ures)
    }};

    ($x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident, $cmp:expr) => {{
        let bx = util::to_ibnum($x0, $x1);
        let by = util::to_ibnum($y0, $y1);
        let bres = bx.$op(by);

        let ux = util::to_i256($x0, $x1);
        let uy = util::to_i256($y0, $y1);
        let ures = ux.$op(uy);

        $cmp(bres, ures)
    }};

    (wrap $x0:ident, $x1:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $op, |x: util::Bi256, y: i256::i256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (wrap $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $y, $op, |x: util::Bi256, y: i256::i256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (wrap $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $y0, $y1, $op, |x: util::Bi256, y: i256::i256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (over $x0:ident, $x1:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $op, |x: (util::Bi256, bool), y: (i256::i256, bool)| {
            x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
        })
    }};

    (over $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $y, $op, |x: (util::Bi256, bool), y: (i256::i256, bool)| {
            x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
        })
    }};

    (over $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        signed_op_equal!(
            $x0,
            $x1,
            $y0,
            $y1,
            $op,
            |x: (util::Bi256, bool), y: (i256::i256, bool)| {
                x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
            }
        )
    }};

    (check $x0:ident, $x1:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $op, |x: Option<util::Bi256>, y: Option<i256::i256>| {
            x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
        })
    }};

    (check $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $y, $op, |x: Option<util::Bi256>, y: Option<i256::i256>| {
            x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
        })
    }};

    (check $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        signed_op_equal!(
            $x0,
            $x1,
            $y0,
            $y1,
            $op,
            |x: Option<util::Bi256>, y: Option<i256::i256>| {
                x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
            }
        )
    }};

    (scalar $x0:ident, $x1:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $op, |x, y| x == y)
    }};

    (scalar $x0:ident, $x1:ident, $y:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $y, $o, |x, y| x == y)
    }};

    (scalar $x0:ident, $x1:ident, $y0:ident, $y1:ident, $op:ident) => {{
        signed_op_equal!($x0, $x1, $y0, $y1, $op, |x, y| x == y)
    }};
}

macro_rules! unsigned_limb_op_equal {
    ($x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident, $cmp:expr) => {{
        let x = util::to_u256($x0, $x1);
        let fres = x.$full(i256::u256::from($y));
        let lres = x.$limb($y);

        $cmp(fres, lres)
    }};

    (wrap $x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident) => {{
        unsigned_limb_op_equal!($x0, $x1, $y, $full, $limb, |x: i256::u256, y: i256::u256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (over $x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident) => {{
        unsigned_limb_op_equal!(
            $x0,
            $x1,
            $y,
            $full,
            $limb,
            |x: (i256::u256, bool), y: (i256::u256, bool)| {
                x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
            }
        )
    }};

    (check $x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident) => {{
        unsigned_limb_op_equal!(
            $x0,
            $x1,
            $y,
            $full,
            $limb,
            |x: Option<i256::u256>, y: Option<i256::u256>| {
                x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
            }
        )
    }};
}

macro_rules! signed_limb_op_equal {
    ($x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident, $cmp:expr) => {{
        let x = util::to_i256($x0, $x1);
        let fres = x.$full($y.into());
        let lres = x.$limb($y);

        $cmp(fres, lres)
    }};

    (wrap $x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident) => {{
        signed_limb_op_equal!($x0, $x1, $y, $full, $limb, |x: i256::i256, y: i256::i256| {
            x.to_le_bytes() == y.to_le_bytes()
        })
    }};

    (over $x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident) => {{
        signed_limb_op_equal!(
            $x0,
            $x1,
            $y,
            $full,
            $limb,
            |x: (i256::i256, bool), y: (i256::i256, bool)| {
                x.0.to_le_bytes() == y.0.to_le_bytes() && x.1 == y.1
            }
        )
    }};

    (check $x0:ident, $x1:ident, $y:ident, $full:ident, $limb:ident) => {{
        signed_limb_op_equal!(
            $x0,
            $x1,
            $y,
            $full,
            $limb,
            |x: Option<i256::i256>, y: Option<i256::i256>| {
                x.map(|v| v.to_le_bytes()) == y.map(|v| v.to_le_bytes())
            }
        )
    }};
}

use super::permutation::Permutation;
use paste::paste;

const FACTORIALS_U8: [u8; 6] = {
    let mut f = [1u8; 6];
    let mut i = 1;
    // compute f[1] … f[n] inclusive
    while i <= 5 {
        f[i] = f[i - 1] * (i as u8);
        i += 1;
    }
    f
};

const FACTORIALS_U16: [u16; 9] = {
    let mut f = [1u16; 9];
    let mut i = 1;
    // compute f[1] … f[n] inclusive
    while i <= 8 {
        f[i] = f[i - 1] * (i as u16);
        i += 1;
    }
    f
};

const FACTORIALS_U32: [u32; 13] = {
    let mut f = [1u32; 13];
    let mut i = 1;
    // compute f[1] … f[n] inclusive
    while i <= 12 {
        f[i] = f[i - 1] * (i as u32);
        i += 1;
    }
    f
};

const FACTORIALS_U64: [u64; 21] = {
    let mut f = [1u64; 21];
    let mut i = 1;
    // compute f[1] … f[n] inclusive
    while i <= 20 {
        f[i] = f[i - 1] * (i as u64);
        i += 1;
    }
    f
};

macro_rules! permutation_coord {
    ($n:expr, $t:ty, $dense:expr) => {
        paste! {
            const [<FACTORIALS_ $n>]: [$t; $n + 1] = {
                let mut f: [$t; $n + 1] = [1; $n + 1];
                let mut i = 1;
                // compute f[1] … f[n] inclusive
                while i <= $n {
                    f[i] = f[i - 1] * (i as $t);
                    i += 1;
                }
                f
            };

            const [<FIRST_PERM_ $n>]: [u8; $n] = {
                let mut f: [u8; $n] = [0; $n];
                let mut i = 0;
                while i < $n {
                    f[i] = i as u8;
                    i += 1;
                }
                f
            };

            const fn [<permutation_coord_ $n>](perm: &Permutation<$n>) -> $t {
                let mut sum = 0;
                let mut i = 1;
                while i < $n {
                    let mut j = 0;
                    while j < i {
                        if perm.0[j] > perm.0[i] {
                            sum += [<FACTORIALS_ $n>][i]
                        }

                        j += 1;
                    }

                    i += 1;
                }
                sum
            }

            const fn [<permutation_coord_ $n _inverse>](mut coord: $t) -> Permutation<$n> {
                let mut f = $n - 1;
                let mut result = [<FIRST_PERM_ $n>];
                let mut c = 0;
                loop {
                    if [<FACTORIALS_ $n>][f] <= coord {
                        coord -= [<FACTORIALS_ $n>][f];
                        c += 1;
                    } else {
                        if c != 0 {
                            let swap = result[f - c];
                            let mut i = f - c;
                            while i < f {
                                result[i] = result[i + 1];
                                i += 1;
                            }
                            // todo!();
                            result[f] = swap;
                            c = 0;
                        }
                        if f == 0 {
                            break;
                        }
                        f -= 1;
                    }
                }

                Permutation(result)
            }

            const fn [<permutation_coord_ $n _parity>](perm: &Permutation<$n>) -> $t {
                let mut r = [<permutation_coord_ $n>](perm);
                // is_odd returns bool, cast into integer 0/1
                if (r & 1) != (perm.is_odd() as $t) {
                    r ^= 1;
                }
                r
            }

            const fn [<permutation_coord_ $n _parity_inverse>](code: $t) -> Permutation<$n> {
                let p = [<permutation_coord_ $n _inverse>](code);
                if (p.is_odd() as $t) == (code & 1) {
                    p
                } else {
                    [<permutation_coord_ $n _inverse>](code ^ 1)
                }
            }

            #[test]
            fn [<permutation_coord_ $n _test>]() {
                if $dense {
                    // the domain is small enough so we just check the whole thing.
                    for i in 0..[<FACTORIALS_ $n>][$n] {
                        let s = [<permutation_coord_ $n _inverse>](i);
                        assert_eq!([<permutation_coord_ $n>](&s), i);
                    }
                } else {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

                    // the domain is too big, sample randomly
                    for _ in 0..1000 {
                        let i = rng.random_range(0..[<FACTORIALS_ $n>][$n]);
                        let s = [<permutation_coord_ $n _inverse>](i);
                        assert_eq!([<permutation_coord_ $n>](&s), i);
                    }
                }
            }

            #[test]
            fn [<permutation_coord_ $n _parity_test>]() {
                if $dense {
                    // the domain is small enough so we just check the whole thing.
                    for i in 0..[<FACTORIALS_ $n>][$n] {
                        let s = [<permutation_coord_ $n _parity_inverse>](i);
                        assert_eq!(s.is_odd() as $t, i % 2);
                        assert_eq!([<permutation_coord_ $n _parity>](&s), i);
                    }
                } else {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

                    // the domain is too big, sample randomly
                    for _ in 0..1000 {
                        let i = rng.random_range(0..[<FACTORIALS_ $n>][$n]);
                        let s = [<permutation_coord_ $n _parity_inverse>](i);
                        assert_eq!(s.is_odd() as $t, i % 2);
                        assert_eq!([<permutation_coord_ $n _parity>](&s), i);
                    }
                }
            }
        }
    };
}

pub trait LehmerRank {
    type Code;
    fn into_coord(self) -> Self::Code;
    fn from_coord(code: Self::Code) -> Self;
}

pub trait ParityPreservingRank {
    type Code;
    fn into_coord(self) -> Self::Code;
    fn from_coord(code: Self::Code) -> Self;
}

permutation_coord!(1, u8, true);
permutation_coord!(2, u8, true);
permutation_coord!(3, u8, true);
permutation_coord!(4, u8, true);
permutation_coord!(5, u8, true);
permutation_coord!(6, u16, true);
permutation_coord!(7, u16, true);
permutation_coord!(8, u16, true);
permutation_coord!(9, u32, false);
permutation_coord!(10, u32, false);
permutation_coord!(11, u32, false);
permutation_coord!(12, u32, false);
permutation_coord!(13, u64, false);
permutation_coord!(14, u64, false);
permutation_coord!(15, u64, false);
permutation_coord!(16, u64, false);
permutation_coord!(17, u64, false);
permutation_coord!(18, u64, false);
permutation_coord!(19, u64, false);
permutation_coord!(20, u64, false);

/// Macro to implement LehmerRank for a single (N,Code,rank_fn,unrank_fn).
macro_rules! impl_lehmer_rank {
    (
        $N:expr,                // the const‐generic N
        $Code:ty,               // the integer type (u8,u16,u32,u64,…)
        $rank_fn:path,          // e.g. permutation_coord_5
        $unrank_fn:path,          // e.g. permutation_coord_5_inverse
        $FACTORIALS:expr
    ) => {
        impl Permutation<$N> {
            #[inline(always)]
            pub const fn const_into_coord(self) -> $Code {
                $rank_fn(&self)
            }

            #[inline(always)]
            pub const fn const_from_coord(coord: $Code) -> Self {
                debug_assert!(coord < $FACTORIALS[$N]);
                $unrank_fn(coord)
            }
        }

        impl LehmerRank for Permutation<$N> {
            type Code = $Code;

            #[inline(always)]
            fn into_coord(self) -> $Code {
                self.const_into_coord()
            }

            #[inline(always)]
            fn from_coord(coord: $Code) -> Self {
                Self::const_from_coord(coord)
            }
        }
    };
}

impl_lehmer_rank!(
    1,
    u8,
    permutation_coord_1_parity,
    permutation_coord_1_parity_inverse,
    FACTORIALS_U8
);
impl_lehmer_rank!(
    2,
    u8,
    permutation_coord_2_parity,
    permutation_coord_2_parity_inverse,
    FACTORIALS_U8
);
impl_lehmer_rank!(
    3,
    u8,
    permutation_coord_3_parity,
    permutation_coord_3_parity_inverse,
    FACTORIALS_U8
);
impl_lehmer_rank!(
    4,
    u8,
    permutation_coord_4_parity,
    permutation_coord_4_parity_inverse,
    FACTORIALS_U8
);
impl_lehmer_rank!(
    5,
    u8,
    permutation_coord_5_parity,
    permutation_coord_5_parity_inverse,
    FACTORIALS_U8
);
impl_lehmer_rank!(
    6,
    u16,
    permutation_coord_6_parity,
    permutation_coord_6_parity_inverse,
    FACTORIALS_U16
);
impl_lehmer_rank!(
    7,
    u16,
    permutation_coord_7_parity,
    permutation_coord_7_parity_inverse,
    FACTORIALS_U16
);
impl_lehmer_rank!(
    8,
    u16,
    permutation_coord_8_parity,
    permutation_coord_8_parity_inverse,
    FACTORIALS_U16
);
impl_lehmer_rank!(
    9,
    u32,
    permutation_coord_9_parity,
    permutation_coord_9_parity_inverse,
    FACTORIALS_U32
);
impl_lehmer_rank!(
    10,
    u32,
    permutation_coord_10_parity,
    permutation_coord_10_parity_inverse,
    FACTORIALS_U32
);
impl_lehmer_rank!(
    11,
    u32,
    permutation_coord_11_parity,
    permutation_coord_11_parity_inverse,
    FACTORIALS_U32
);
impl_lehmer_rank!(
    12,
    u32,
    permutation_coord_12_parity,
    permutation_coord_12_parity_inverse,
    FACTORIALS_U32
);
impl_lehmer_rank!(
    13,
    u64,
    permutation_coord_13_parity,
    permutation_coord_13_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    14,
    u64,
    permutation_coord_14_parity,
    permutation_coord_14_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    15,
    u64,
    permutation_coord_15_parity,
    permutation_coord_15_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    16,
    u64,
    permutation_coord_16_parity,
    permutation_coord_16_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    17,
    u64,
    permutation_coord_17_parity,
    permutation_coord_17_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    18,
    u64,
    permutation_coord_18_parity,
    permutation_coord_18_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    19,
    u64,
    permutation_coord_19_parity,
    permutation_coord_19_parity_inverse,
    FACTORIALS_U64
);
impl_lehmer_rank!(
    20,
    u64,
    permutation_coord_20_parity,
    permutation_coord_20_parity_inverse,
    FACTORIALS_U64
);

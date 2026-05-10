use super::permutation::Permutation;
use paste::paste;

const FACTORIALS_U8: [u8; 6] = {
    let mut f = [1u8; 6];
    let mut i = 1;
    while i <= 5 {
        f[i] = f[i - 1] * (i as u8);
        i += 1;
    }
    f
};

const FACTORIALS_U16: [u16; 9] = {
    let mut f = [1u16; 9];
    let mut i = 1;
    while i <= 8 {
        f[i] = f[i - 1] * (i as u16);
        i += 1;
    }
    f
};

const FACTORIALS_U32: [u32; 13] = {
    let mut f = [1u32; 13];
    let mut i = 1;
    while i <= 12 {
        f[i] = f[i - 1] * (i as u32);
        i += 1;
    }
    f
};

const FACTORIALS_U64: [u64; 21] = {
    let mut f = [1u64; 21];
    let mut i = 1;
    while i <= 20 {
        f[i] = f[i - 1] * (i as u64);
        i += 1;
    }
    f
};

/// implement the encode/decode scheme for Permutation<N>
macro_rules! lehmer_code {
    ($n:expr, $t:ty, $factorials:expr, $dense:expr) => {
        paste! {
            const fn [<lehmer_encode_ $n>](perm: &Permutation<$n>) -> $t {
                let mut sum = 0;
                let mut i = 1;
                while i < $n {
                    let mut j = 0;
                    while j < i {
                        if perm.0[j] > perm.0[i] {
                            sum += $factorials[i]
                        }

                        j += 1;
                    }

                    i += 1;
                }
                sum
            }

            const fn [<lehmer_decode_ $n>](mut coord: $t) -> Permutation<$n> {
                let mut f = $n - 1;
                let mut result = Permutation::IDENTITY.0;
                let mut c = 0;
                loop {
                    if $factorials[f] <= coord {
                        coord -= $factorials[f];
                        c += 1;
                    } else {
                        if c != 0 {
                            let swap = result[f - c];
                            let mut i = f - c;
                            while i < f {
                                result[i] = result[i + 1];
                                i += 1;
                            }
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

            const fn [<lehmer_encode_preserve_parity_ $n>](perm: &Permutation<$n>) -> $t {
                let mut r = [<lehmer_encode_ $n>](perm);
                if (r & 1) != (perm.is_odd() as $t) {
                    r ^= 1;
                }
                r
            }

            const fn [<lehmer_decode_preserve_parity_ $n>](code: $t) -> Permutation<$n> {
                let p = [<lehmer_decode_ $n>](code);
                if (p.is_odd() as $t) == (code & 1) {
                    p
                } else {
                    [<lehmer_decode_ $n>](code ^ 1)
                }
            }

            impl Permutation<$n> {
                /// Encode this permutation in a single number in the range 0..N!,
                /// with the additional property that the parity of the permutation
                /// is equal to the parity of the number it is encoded in.
                pub const fn const_lehmer_encode(self) -> $t {
                    [<lehmer_encode_preserve_parity_ $n>](&self)
                }

                /// Decode this number from the range 0..N! into a permutation,
                /// with the additional property that the parity of the permutation
                /// is equal to the parity of the number it was encoded in.
                pub const fn const_lehmer_decode(coord: $t) -> Self {
                    debug_assert!(coord < $factorials[$n]);
                    [<lehmer_decode_preserve_parity_ $n>](coord)
                }
            }

            #[test]
            fn [<lehmer_encode_decode_ $n _test>]() {
                if $n < 9 {
                    // the domain is small enough so we just check the whole thing.
                    for i in 0..$factorials[$n] {
                        let s = [<lehmer_decode_ $n>](i);
                        assert_eq!([<lehmer_encode_ $n>](&s), i);
                    }
                } else {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

                    // the domain is too big, sample randomly
                    for _ in 0..1000 {
                        let i = rng.random_range(0..$factorials[$n]);
                        let s = [<lehmer_decode_ $n>](i);
                        assert_eq!([<lehmer_encode_ $n>](&s), i);
                    }
                }
            }

            #[test]
            fn [<lehmer_encode_decode_parity_ $n>]() {
                if $n < 9 {
                    // the domain is small enough so we just check the whole thing.
                    for i in 0..$factorials[$n] {
                        let s = [<lehmer_decode_preserve_parity_ $n>](i);
                        assert_eq!(s.is_odd() as $t, i % 2);
                        assert_eq!([<lehmer_encode_preserve_parity_ $n>](&s), i);
                    }
                } else {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

                    // the domain is too big, sample randomly
                    for _ in 0..1000 {
                        let i = rng.random_range(0..$factorials[$n]);
                        let s = [<lehmer_decode_preserve_parity_ $n>](i);
                        assert_eq!(s.is_odd() as $t, i % 2);
                        assert_eq!([<lehmer_encode_preserve_parity_ $n>](&s), i);
                    }
                }
            }

            #[test]
            fn [<permutation_encode_decode_ $n>]() {
                if $n < 9 {
                    // the domain is small enough so we just check the whole thing.
                    for i in 0..$factorials[$n] {
                        let p = Permutation::<$n>::const_lehmer_decode(i);
                        assert_eq!(p.is_odd() as $t, i % 2);
                        assert_eq!(p.const_lehmer_encode(), i);
                    }
                } else {
                    use rand::{Rng, SeedableRng};
                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

                    // the domain is too big, sample randomly
                    for _ in 0..1000 {
                        let i = rng.random_range(0..$factorials[$n]);
                        let p = Permutation::<$n>::const_lehmer_decode(i);
                        assert_eq!(p.is_odd() as $t, i % 2);
                        assert_eq!(p.const_lehmer_encode(), i);
                    }
                }
            }
        }
    };
}

lehmer_code!(1, u8, FACTORIALS_U8, true);
lehmer_code!(2, u8, FACTORIALS_U8, true);
lehmer_code!(3, u8, FACTORIALS_U8, true);
lehmer_code!(4, u8, FACTORIALS_U8, true);
lehmer_code!(5, u8, FACTORIALS_U8, true);
lehmer_code!(6, u16, FACTORIALS_U16, true);
lehmer_code!(7, u16, FACTORIALS_U16, true);
lehmer_code!(8, u16, FACTORIALS_U16, true);
lehmer_code!(9, u32, FACTORIALS_U32, false);
lehmer_code!(10, u32, FACTORIALS_U32, false);
lehmer_code!(11, u32, FACTORIALS_U32, false);
lehmer_code!(12, u32, FACTORIALS_U32, false);
lehmer_code!(13, u64, FACTORIALS_U64, false);
lehmer_code!(14, u64, FACTORIALS_U64, false);
lehmer_code!(15, u64, FACTORIALS_U64, false);
lehmer_code!(16, u64, FACTORIALS_U64, false);
lehmer_code!(17, u64, FACTORIALS_U64, false);
lehmer_code!(18, u64, FACTORIALS_U64, false);
lehmer_code!(19, u64, FACTORIALS_U64, false);
lehmer_code!(20, u64, FACTORIALS_U64, false);

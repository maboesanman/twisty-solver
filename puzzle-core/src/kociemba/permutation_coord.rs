macro_rules! permutation_coord {
    ($n:expr, $t:ty,
        $id: ident,
        $id_inv: ident,
        $id_test: ident,
        $id_factorials: ident,
        $id_first_perm: ident,
    ) => {
        const $id_factorials: [$t; $n + 1] = {
            let mut f: [$t; $n + 1] = [1; $n + 1];
            let mut i = 1;
            loop {
                f[i] = f[i - 1] * (i as $t);
                i += 1;
                if i == $n {
                    break;
                }
            }
            f
        };

        const $id_first_perm: [u8; $n] = {
            let mut f: [u8; $n] = [0; $n];
            let mut i = 0;
            loop {
                f[i] = i as u8;
                i += 1;
                if i == $n {
                    break;
                }
            }
            f
        };

        pub const fn $id(perm: &[u8; $n]) -> $t {
            let mut sum = 0;
            let mut i = 1;
            while i < $n {
                let mut j = 0;
                while j < i {
                    if perm[j] > perm[i] {
                        sum += $id_factorials[i]
                    }

                    j += 1;
                }

                i += 1;
            }
            sum
        }

        pub fn $id_inv(mut coord: $t) -> [u8; $n] {
            let mut f = $n - 1;
            let mut result = $id_first_perm;
            let mut c = 0;
            loop {
                if $id_factorials[f] <= coord {
                    coord -= $id_factorials[f];
                    c += 1;
                } else {
                    if c != 0 {
                        let swap = result[f - c];
                        result.copy_within((f - c + 1)..=f, f - c);
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

            result
        }

        #[test]
        fn $id_test() {
            // the domain is small enough so we just check the whole thing.
            for i in 0..$id_factorials[$n] {
                let s = $id_inv(i);
                assert_eq!($id(&s), i);
            }
        }
    };
}

permutation_coord!(
    8,
    u16,
    permutation_coord_8,
    permutation_coord_8_inverse,
    permutation_coord_8_test,
    FACTORIALS_8,
    FIRST_PERM_8,
);

permutation_coord!(
    4,
    u8,
    permutation_coord_4,
    permutation_coord_4_inverse,
    permutation_coord_4_test,
    FACTORIALS_4,
    FIRST_PERM_4,
);

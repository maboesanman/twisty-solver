use paste::paste;

macro_rules! permutation_coord {
    ($n:expr, $t:ty) => {
        paste! {
            const [<FACTORIALS_ $n>]: [$t; $n + 1] = {
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

            const [<FIRST_PERM_ $n>]: [u8; $n] = {
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

            pub const fn [<permutation_coord_ $n>](perm: &[u8; $n]) -> $t {
                let mut sum = 0;
                let mut i = 1;
                while i < $n {
                    let mut j = 0;
                    while j < i {
                        if perm[j] > perm[i] {
                            sum += [<FACTORIALS_ $n>][i]
                        }

                        j += 1;
                    }

                    i += 1;
                }
                sum
            }

            pub const fn [<permutation_coord_ $n _inverse>](mut coord: $t) -> [u8; $n] {
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

                result
            }

            #[test]
            fn [<permutation_coord_ $n _test>]() {
                // the domain is small enough so we just check the whole thing.
                for i in 0..[<FACTORIALS_ $n>][$n] {
                    let s = [<permutation_coord_ $n _inverse>](i);
                    assert_eq!([<permutation_coord_ $n>](&s), i);
                }
            }
        }
    };
}

permutation_coord!(8, u16);

permutation_coord!(4, u8);

const COMBINATIONS: [[u16; 4]; 12] = {
    const FACTORIALS: [u32; 12] = [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
    ];

    let mut buf = [[0u16; 4]; 12];

    let mut i = 0;
    while i < 12 {
        let mut j = 0;
        while j < 4 && j <= i {
            buf[i][j] = (FACTORIALS[i] / FACTORIALS[j] / FACTORIALS[i - j]) as u16;
            j += 1;
        }
        i += 1;
    }

    buf
};

pub const fn edge_grouping(items: &[bool; 12]) -> u16 {
    let mut sum = 0;
    let mut k = 3;
    let mut n = 11;

    loop {
        if !items[n] {
            sum += COMBINATIONS[n][k]
        } else if k == 0 {
            break;
        } else {
            k -= 1;
        }

        if n == 0 {
            break;
        }
        n -= 1;
    }

    sum
}

pub const fn edge_grouping_inverse(mut coord: u16) -> [bool; 12] {
    let mut buf = [false; 12];
    let mut k = 11;
    let mut i = 3;
    loop {
        let c = COMBINATIONS[k][i];
        if coord >= c {
            coord -= c;
            k -= 1;
        } else {
            buf[k] = true;
            if i == 0 {
                break;
            }
            i -= 1;
            k -= 1;
        }
    }

    buf
}

#[test]
fn edge_grouping_test() {
    // the domain is small enough so we just check the whole thing.
    for i in 0..495 {
        let s = edge_grouping_inverse(i);
        assert_eq!(edge_grouping(&s), i);
    }
}

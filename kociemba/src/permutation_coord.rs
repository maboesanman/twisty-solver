use paste::paste;

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

            // ——— parity-interleaved rank: even perms→even codes, odd→odd
            pub const fn [<permutation_coord_ $n _parity>](perm: &[u8; $n]) -> $t {
                let mut r = [<permutation_coord_ $n>](perm);
                // is_odd returns bool, cast into integer 0/1
                if (r & 1) != (is_odd(perm) as $t) {
                    r ^= 1;
                }
                r
            }

            // ——— parity-aware unrank: pick the branch whose parity matches code&1
            pub const fn [<permutation_coord_ $n _parity_inverse>](code: $t) -> [u8; $n] {
                let p = [<permutation_coord_ $n _inverse>](code);
                if (is_odd(&p) as $t) == (code & 1) {
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
                    // the domain is too big, sample randomly
                    for i in 0..[<FACTORIALS_ $n>][$n] {
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
                        assert_eq!(is_odd(&s) as $t, i % 2);
                        assert_eq!([<permutation_coord_ $n _parity>](&s), i);
                    }
                } else {

                }
            }
        }
    };
}

permutation_coord!(12, u32, false);

permutation_coord!(8, u16, true);

permutation_coord!(4, u8, true);

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

pub const fn is_odd<const N: usize>(perm: &[u8; N]) -> bool {
    let mut seen = [false; N];
    let mut parity = 0;

    let mut i = 0;
    while i < N {
        if !seen[i] {
            let mut len = 0;
            let mut j = i;
            while !seen[j] {
                seen[j] = true;
                j = perm[j] as usize;
                len += 1;
            }
            if len > 0 {
                parity ^= (len - 1) & 1;
            }
        }
        i += 1;
    }

    parity == 1
}

/// Returns `true` if `arr` is a permutation of `1..=N`
/// (no duplicates, no out‐of‐range), else `false`.
pub const fn is_perm<const N: usize>(arr: &[u8; N]) -> bool {
    // a little boolean table indexed 1..=N
    let mut seen = [false; N];
    let mut i = 0;
    while i < N {
        let v = arr[i] as usize;
        // out of 1..=N ?
        if v >= N {
            return false;
        }
        // duplicate?
        if seen[v] {
            return false;
        }
        seen[v] = true;
        i += 1;
    }
    // if we saw exactly N distinct values in 1..=N, it's guaranteed to be complete
    true
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Perm12Slots([u8; 6]);

pub const SORTED_PERM_12: Perm12Slots = Perm12Slots([
    0b0000_0001,
    0b0010_0011,
    0b0100_0101,
    0b0110_0111,
    0b1000_1001,
    0b1010_1011,
]);

impl Perm12Slots {
    /// Unpack 6 bytes → 12 nibbles (0..11)
    pub const fn unpack12(self) -> [u8; 12] {
        let mut out = [0u8; 12];
        let mut i = 0;
        while i < 6 {
            let byte = self.0[i];
            out[2 * i] = byte & 0x0F; // low nibble
            out[2 * i + 1] = (byte >> 4) & 0x0F; // high nibble
            i += 1;
        }
        out
    }

    /// Pack 12 nibbles (each <16) → 6 bytes
    pub const fn pack12(nibs: [u8; 12]) -> Self {
        let mut out = [0u8; 6];
        let mut i = 0;
        while i < 6 {
            let lo = nibs[2 * i] & 0x0F;
            let hi = nibs[2 * i + 1] & 0x0F;
            out[i] = lo | (hi << 4);
            i += 1;
        }
        Self(out)
    }

    pub const fn then(self, other: Self) -> Self {
        let a = self.unpack12();
        let b = other.unpack12();
        let mut c = [0u8; 12];
        let mut i = 0;
        while i < 12 {
            c[i] = a[b[i] as usize];
            i += 1;
        }
        Self::pack12(c)
    }

    pub const fn inverse(self) -> Self {
        let p = self.unpack12();
        let mut inv = [0u8; 12];
        let mut i = 0;
        while i < 12 {
            let v = p[i] as usize;
            inv[v] = i as u8;
            i += 1;
        }
        Self::pack12(inv)
    }
}

pub struct Perm8Slots([u8; 3]);

const SORTED_PERM_8: Perm8Slots = Perm8Slots([0b000_001_01, 0b0_011_100_1, 0b01_110_111]);

impl Perm8Slots {
    pub fn then(self, other: Self) -> Self {
        todo!()
    }

    pub fn inverse(self) -> Self {
        todo!()
    }
}

pub struct Perm4Slots(u8);

const SORTED_PERM_4: Perm4Slots = Perm4Slots(0b_00_01_10_11);

impl Perm4Slots {
    pub fn then(self, other: Self) -> Self {
        todo!()
    }

    pub fn inverse(self) -> Self {
        todo!()
    }
}

#[test]
fn edge_grouping_test() {
    // the domain is small enough so we just check the whole thing.
    for i in 0..495 {
        let s = edge_grouping_inverse(i);
        assert_eq!(edge_grouping(&s), i);
    }
}

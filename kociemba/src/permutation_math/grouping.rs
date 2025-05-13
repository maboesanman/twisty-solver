use super::permutation::Permutation;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EdgeCombination(pub [bool; 12]);

impl EdgeCombination {
    pub const SOLVED: Self = Self([
        false, false, false, false, false, false, false, false, true, true, true, true,
    ]);

    const fn is_valid(array: [bool; 12]) -> bool {
        let mut seen = 0;
        let mut i = 0;
        while i < 12 {
            if array[i] {
                seen += 1;
            }
            i += 1;
        }

        seen == 4
    }

    pub const fn const_into_array(self) -> [bool; 12] {
        self.0
    }

    pub const fn try_const_from_array(array: [bool; 12]) -> Result<Self, [bool; 12]> {
        if Self::is_valid(array) {
            Ok(Self(array))
        } else {
            Err(array)
        }
    }

    pub const fn const_from_array(array: [bool; 12]) -> Self {
        match Self::try_const_from_array(array) {
            Ok(val) => val,
            Err(_) => panic!("invalid permutation array"),
        }
    }

    pub const unsafe fn const_from_array_unchecked(array: [bool; 12]) -> Self {
        debug_assert!(Self::is_valid(array));
        Self(array)
    }

    pub const fn permute(mut self, perm: Permutation<12>) -> Self {
        perm.apply_to(&mut self.0);
        self
    }

    pub const fn into_coord(self) -> u16 {
        let mut sum = 0;
        let mut k = 3;
        let mut n = 11;

        loop {
            if !self.0[n] {
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

    pub const fn from_coord(mut coord: u16) -> Self {
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

        Self(buf)
    }
}

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

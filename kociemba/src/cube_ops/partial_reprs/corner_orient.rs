use crate::cube_ops::coords::CornerOrientRawCoord;

use super::corner_perm::CornerPerm;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CornerOrient(pub [u8; 8]);

impl CornerOrient {
    pub const SOLVED: Self = Self([0; 8]);

    const fn is_valid(array: [u8; 8]) -> bool {
        let mut s = 0;
        let mut i = 0;
        while i < 8 {
            s += array[i];
            i += 1;
        }
        s % 3 == 0
    }

    pub const fn const_from_array(array: [u8; 8]) -> Self {
        match Self::try_from_array(array) {
            Ok(x) => x,
            Err(_) => panic!(),
        }
    }

    pub const fn try_from_array(array: [u8; 8]) -> Result<Self, [u8; 8]> {
        if Self::is_valid(array) {
            Ok(Self(array))
        } else {
            Err(array)
        }
    }

    pub const fn permute(mut self, perm: CornerPerm) -> Self {
        perm.0.apply_to(&mut self.0);
        self
    }

    pub const fn correct(mut self, correct: Self) -> Self {
        let mut i = 0;
        while i < 8 {
            self.0[i] = (self.0[i] + correct.0[i]) % 3;
            i += 1;
        }

        self
    }

    pub const fn uncorrect(mut self, correct: Self) -> Self {
        let mut i = 0;
        while i < 8 {
            self.0[i] = (3 + self.0[i] - correct.0[i]) % 3;
            i += 1;
        }

        self
    }

    pub const fn mirror(mut self) -> Self {
        let mut i = 0;
        while i < 8 {
            self.0[i] = (3 - self.0[i]) % 3;
            i += 1;
        }

        self
    }

    pub const fn from_coord(mut coord: CornerOrientRawCoord) -> Self {
        let mut corner_orient = [0; 8];

        let mut sum = 0;
        let mut i = 7;
        while i > 0 {
            i -= 1;
            let value = (coord.0 % 3) as u8;
            corner_orient[i] = value;
            sum += value as u16;
            coord.0 /= 3;
        }

        corner_orient[7] = ((3 - (sum % 3)) % 3) as u8;

        Self(corner_orient)
    }

    pub const fn into_coord(self) -> CornerOrientRawCoord {
        let mut total = 0u16;
        let mut i = 0;
        while i < 7 {
            total = total * 3 + self.0[i] as u16;
            i += 1;
        }

        CornerOrientRawCoord(total)
    }

    pub const fn const_eq(self, other: Self) -> bool {
        let mut i = 1;
        while i < 8 {
            if self.0[i] != other.0[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

use crate::kociemba::coords::CornerOrientRawCoord;

use super::corner_perm::CornerPerm;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
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
        let mut i = 0;
        while i < 7 {
            let value = (coord.0 % 3) as u8;
            corner_orient[i] = value;
            sum += value as u16;
            coord.0 /= 3;
            i += 1;
        }
        corner_orient[7] = ((3 - (sum % 3)) % 3) as u8;
        Self(corner_orient)
    }

    pub const fn into_coord(self) -> CornerOrientRawCoord {
        let mut total = 0u16;
        let mut i = 7;
        while i > 0 {
            i -= 1;
            total = total * 3 + self.0[i] as u16;
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

    pub const fn into_dense(self) -> u16 {
        let mut dense = 0u16;
        let mut i = 0;
        while i < 8 {
            dense |= (self.0[i] as u16) << (i * 2);
            i += 1;
        }
        dense
    }

    pub const fn from_dense(dense: u16) -> Self {
        let mut array = [0u8; 8];
        let mut i = 0;
        while i < 8 {
            array[i] = ((dense >> (i * 2)) & 0b11) as u8;
            i += 1;
        }
        Self(array)
    }
}

#[test]
fn from_into_coord_inverses() {
    for i in 0..2187 {
        let c = CornerOrient::from_coord(CornerOrientRawCoord(i));
        assert_eq!(i, c.into_coord().0);
    }
}

#[test]
fn from_dense_inverses() {
    for i in 0..2187 {
        let c = CornerOrient::from_coord(CornerOrientRawCoord(i));

        let d = c.into_dense();
        let c2 = CornerOrient::from_dense(d);
        assert_eq!(c, c2);
    }
}
use crate::cube_ops::{coords::EdgeOrientRawCoord, cube_move::CubeMove};

use super::edge_perm::EdgePerm;



#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EdgeOrient(pub [u8; 12]);

impl EdgeOrient {
    pub const SOLVED: Self = Self([0; 12]);

    const fn is_valid(array: [u8; 12]) -> bool {
        let mut s = 0;
        let mut i = 0;
        while i < 12 {
            s += array[i];
            i += 1;
        }
        s % 2 == 0
    }

    pub const fn const_from_array(array: [u8; 12]) -> Self {
        match Self::try_from_array(array) {
            Ok(x) => x,
            Err(_) => panic!(),
        }
    }

    pub const fn try_from_array(array: [u8; 12]) -> Result<Self, [u8; 12]> {
        if Self::is_valid(array) {
            Ok(Self(array))
        } else {
            Err(array)
        }
    }

    pub const fn permute(mut self, perm: EdgePerm) -> Self {
        perm.0.apply_to(&mut self.0);
        self
    }

    pub const fn correct(mut self, correct: Self) -> Self {
        let mut i = 0;
        while i < 12 {
            self.0[i] = (self.0[i] + correct.0[i]) % 2;
            i += 1;
        }

        self
    }

    pub const fn from_coord(mut coord: EdgeOrientRawCoord) -> Self {
        let mut edge_orient = [0; 12];

        let mut sum = 0;
        let mut i = 7;
        while i > 0 {
            i -= 1;
            let value = (coord.0 % 2) as u8;
            edge_orient[i] = value;
            sum += value as u16;
            coord.0 /= 2;
        }

        edge_orient[7] = ((2 - (sum % 2)) % 2) as u8;

        Self(edge_orient)
    }

    pub const fn into_coord(self) -> EdgeOrientRawCoord {
        let mut total = 0u16;
        let mut i = 0;
        while i < 7 {
            total = total * 3 + self.0[i] as u16;
            i += 1;
        }

        EdgeOrientRawCoord(total)
    }

    pub const fn const_eq(self, other: Self) -> bool {
        let mut i = 1;
        while i < 12 {
            if self.0[i] != other.0[i] {
                return false
            }
            i += 1;
        }
        true
    }
}
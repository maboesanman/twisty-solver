use crate::cube_ops::{coords::{EdgeGroupRawCoord, EdgeOrientRawCoord}, cube_move::{CubeMove, DominoMove}};

use super::{edge_group::EdgeGroup, edge_orient::EdgeOrient, edge_perm::EdgePerm};



// fits in 20 bits
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EdgeGroupOrientRawCoord(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EdgeGroupOrient(pub EdgeGroup, pub EdgeOrient);

impl EdgeGroupOrient {
    pub const SOLVED: Self = Self(EdgeGroup::SOLVED, EdgeOrient::SOLVED);

    pub const fn from_coord(coord: EdgeGroupOrientRawCoord) -> Self {
        let group_coord = (coord.0 >> 11) as u16;
        let orient_coord = (coord.0 & 0b11111111111) as u16;
        Self(EdgeGroup::from_coord(EdgeGroupRawCoord(group_coord)), EdgeOrient::from_coord(EdgeOrientRawCoord(orient_coord)))
    }

    pub const fn into_coord(self) -> EdgeGroupOrientRawCoord {
        let group_coord = self.0.into_coord().0 as u32;
        let orient_coord = self.1.into_coord().0 as u32;
        EdgeGroupOrientRawCoord((group_coord << 11) + orient_coord)
    }

    pub const fn then(self, perm: EdgePerm) -> Self {
        Self(self.0.permute(perm), self.1.permute(perm))
    }
}
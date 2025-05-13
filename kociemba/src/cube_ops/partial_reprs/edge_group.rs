use crate::{cube_ops::coords::EdgeGroupRawCoord, permutation_math::grouping::EdgeCombination};

use super::edge_perm::EdgePerm;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgeGroup(pub EdgeCombination);

impl EdgeGroup {
    pub const SOLVED: Self = Self(EdgeCombination::SOLVED);

    pub const fn from_coord(coord: EdgeGroupRawCoord) -> Self {
        Self(EdgeCombination::from_coord(coord.0))
    }

    pub const fn into_coord(self) -> EdgeGroupRawCoord {
        EdgeGroupRawCoord(self.0.into_coord())
    }

    pub const fn permute(self, perm: EdgePerm) -> Self {
        Self(self.0.permute(perm.0))
    }
}

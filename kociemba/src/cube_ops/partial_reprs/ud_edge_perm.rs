use crate::{cube_ops::coords::UDEdgePermRawCoord, permutation_math::permutation::Permutation};

/// The slot representation for corner permutation.
/// While `Permutation<N>` represents an element of the permutation group, this represents
/// a permutation when specifically applied to the cube's UD edges, while the group coord is 0.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct UDEdgePerm(pub Permutation<8>);

impl UDEdgePerm {
    pub const SOLVED: Self = Self(Permutation::IDENTITY);

    pub const fn from_coord(coord: UDEdgePermRawCoord) -> Self {
        Self(Permutation::<8>::const_from_coord(coord.0))
    }

    pub const fn into_coord(self) -> UDEdgePermRawCoord {
        UDEdgePermRawCoord(self.0.const_into_coord())
    }

    pub const fn then(self, other: Self) -> Self {
        Self(self.0.then(other.0))
    }

    pub const fn inverse(self) -> Self {
        Self(self.0.invert())
    }

    pub const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }
}

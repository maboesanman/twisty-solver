use crate::permutation_math::{grouping::EdgeCombination, permutation::Permutation};

use super::{e_edge_perm::EEdgePerm, edge_group::EdgeGroup, ud_edge_perm::UDEdgePerm};

/// The slot representation for edge permutation.
/// While `Permutation<N>` represents an element of the permutation group, this represents
/// a permutation when specifically applied to the cube's edges.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgePerm(pub Permutation<12>);

impl EdgePerm {
    pub const SOLVED: Self = Self(Permutation::IDENTITY);

    pub const fn into_grouping(self) -> EdgeGroup {
        let mut items = [false; 12];
        let mut i = 0;
        while i < 12 {
            items[i] = self.0.0[i] > 7;
            i += 1;
        }
        EdgeGroup(unsafe { EdgeCombination::const_from_array_unchecked(items) })
    }

    pub const fn into_grouped(self) -> Result<(UDEdgePerm, EEdgePerm), Self> {
        let mut ud = [0; 8];
        let mut e = [0; 4];

        let mut i = 0;
        while i < 8 {
            ud[i] = self.0.0[i];
            i += 1;
        }
        while i < 12 {
            if self.0.0[i] < 8 {
                return Err(self);
            }
            e[i - 8] = self.0.0[i] - 8;
            i += 1;
        }

        Ok((
            UDEdgePerm(unsafe { Permutation::const_from_array_unchecked(ud) }),
            EEdgePerm(unsafe { Permutation::const_from_array_unchecked(e) }),
        ))
    }

    pub const unsafe fn into_grouped_unchecked(self) -> (UDEdgePerm, EEdgePerm) {
        let mut ud = [0; 8];
        let mut e = [0; 4];

        let mut i = 0;
        while i < 8 {
            ud[i] = self.0.0[i];
            i += 1;
        }
        while i < 12 {
            e[i - 8] = self.0.0[i] - 8;
            i += 1;
        }

        (
            UDEdgePerm(unsafe { Permutation::const_from_array_unchecked(ud) }),
            EEdgePerm(unsafe { Permutation::const_from_array_unchecked(e) }),
        )
    }

    pub const unsafe fn into_ud_perm_unchecked(self) -> UDEdgePerm {
        let mut ud = [0; 8];

        let mut i = 0;
        while i < 8 {
            ud[i] = self.0.0[i];
            i += 1;
        }

        UDEdgePerm(unsafe { Permutation::const_from_array_unchecked(ud) })
    }

    pub const unsafe fn into_e_perm_unchecked(self) -> EEdgePerm {
        let mut e = [0; 4];

        let mut i = 0;
        while i < 4 {
            e[i] = self.0.0[i + 8] - 8;
            i += 1;
        }

        EEdgePerm(unsafe { Permutation::const_from_array_unchecked(e) })
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

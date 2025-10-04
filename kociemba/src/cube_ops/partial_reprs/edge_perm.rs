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

    pub const fn split(self) -> (EdgeGroup, UDEdgePerm, EEdgePerm) {
        // example of how to decompose:
        // 1, 8, 9, 3, 5, 4, 2, 11, 0, 10, 6, 7
        // f, t, t, f, f, f, f, t, f, t, f, f
        // 1, 3, 5, 4, 2, 0, 6, 7,
        // 0, 1, 3, 2

        let mut ud = [0u8; 8];
        let mut e = [0u8; 4];
        let mut g = [false; 12]; // false = UD slot, true = E slot

        let mut i = 0; // E index
        let mut j = 0; // UD index
        while i + j < 12 {
            let k = i + j;
            let val = self.0.0[k];
            let is_e = val > 7;
            g[k] = is_e;

            if is_e {
                // E-edge values are stored as 8..11; normalize to 0..3
                e[i] = val - 8;
                i += 1;
            } else {
                // UD-edge values are 0..7 already
                ud[j] = val;
                j += 1;
            }
        }

        (
            EdgeGroup(unsafe { EdgeCombination::const_from_array_unchecked(g) }),
            UDEdgePerm(unsafe { Permutation::const_from_array_unchecked(ud) }),
            EEdgePerm(unsafe { Permutation::const_from_array_unchecked(e) }),
        )
    }

    pub const fn join(group: EdgeGroup, ud: UDEdgePerm, e: EEdgePerm) -> Self {
        let mut perm = [0u8; 12];

        let mut i = 0; // E index
        let mut j = 0; // UD index
        while i + j < 12 {
            let k = i + j;
            if group.0.0[k] {
                // true => E-edge slot: re-add the +8 offset
                perm[k] = e.0.0[i] + 8;
                i += 1;
            } else {
                // false => UD-edge slot
                perm[k] = ud.0.0[j];
                j += 1;
            }
        }

        Self(Permutation(perm))
    }

    // pub const fn into_grouped(self) -> (UDEdgePerm, EEdgePerm) {
    //     let (_g, ud, e) = self.split();
    //     (ud, e)
    // }

    // pub const fn into_grouping(self) -> EdgeGroup {
    //     let (g, _ud, _e) = self.split();
    //     g
    // }

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

#[cfg(test)]
mod test {
    use super::*;
    use rand::distr::StandardUniform;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn basic_join_split() {
        // Seeded RNG for reproducibility
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..500 {
            let p: Permutation<12> = rng.sample(StandardUniform);

            let edge_perm = EdgePerm(p);

            let (g, ud, e) = edge_perm.split();

            let join = EdgePerm::join(g, ud, e);

            assert_eq!(edge_perm, join);
        }
    }
}

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
        let g = self.get_grouping();

        let divided = self.0.then(g.0.into_representative_even_perm().invert()).0;

        let mut ud = [0; 8];
        let mut e = [0; 4];

        let mut i = 0;
        while i < 8 {
            ud[i] = divided[i];
            i += 1;
        }
        while i < 12 {
            e[i - 8] = divided[i] - 8;
            i += 1;
        }

        (
            g,
            UDEdgePerm(unsafe { Permutation::const_from_array_unchecked(ud) }),
            EEdgePerm(unsafe { Permutation::const_from_array_unchecked(e) }),
        )
    }

    pub const fn join(group: EdgeGroup, ud: UDEdgePerm, e: EEdgePerm) -> Self {
        let mut perm = [0u8; 12];

        let mut i = 0;
        while i < 8 {
            perm[i] = ud.0.0[i];
            i += 1;
        }
        while i < 12 {
            perm[i] = e.0.0[i - 8] + 8;
            i += 1;
        }

        Self(Permutation(perm).then(group.0.into_representative_even_perm()))
    }

    pub const fn get_grouping(self) -> EdgeGroup {
        let mut g = [false; 12]; // false = UD slot, true = E slot

        let mut i = 0;
        while i < 12 {
            let val = self.0.0[i];
            let is_e = val > 7;
            g[i] = is_e;
            i += 1;
        }

        EdgeGroup(unsafe { EdgeCombination::const_from_array_unchecked(g) })
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

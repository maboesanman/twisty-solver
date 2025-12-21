use crate::permutation_math::permutation::Permutation;

/// The slot representation for edge permutation.
/// While `Permutation<N>` represents an element of the permutation group, this represents
/// a permutation when specifically applied to the cube's edges.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgePerm(pub Permutation<12>);

impl EdgePerm {
    pub const SOLVED: Self = Self(Permutation::IDENTITY);

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

// #[cfg(test)]
// mod test {
//     use super::*;
//     use rand::distr::StandardUniform;
//     use rand::rngs::StdRng;
//     use rand::{Rng, SeedableRng};

//     #[test]
//     fn basic_join_split() {
//         // Seeded RNG for reproducibility
//         let mut rng = StdRng::seed_from_u64(42);

//         for _ in 0..500 {
//             let p: Permutation<12> = rng.sample(StandardUniform);

//             let edge_perm = EdgePerm(p);

//             let (g, ud, e) = edge_perm.split();

//             let join = EdgePerm::join(g, ud, e);

//             assert_eq!(edge_perm, join);
//         }
//     }
// }

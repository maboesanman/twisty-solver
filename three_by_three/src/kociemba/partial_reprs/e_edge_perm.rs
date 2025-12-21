use crate::{
    cube_ops::{cube_move::DominoMove, cube_sym::DominoSymmetry},
    kociemba::{
        coords::coords::EEdgePermRawCoord, partial_reprs::edge_positions::split_edge_positions,
    },
    permutation_math::permutation::Permutation,
};

/// The slot representation for corner permutation.
/// While `Permutation<N>` represents an element of the permutation group, this represents
/// a permutation when specifically applied to the cube's E edges,
/// wherever they might be after the grouping has been applied.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EEdgePerm(pub Permutation<4>);

impl EEdgePerm {
    pub const SOLVED: Self = Self(Permutation::IDENTITY);

    pub const fn from_coord(coord: EEdgePermRawCoord) -> Self {
        Self(Permutation::<4>::const_from_coord(coord.0))
    }

    pub const fn into_coord(self) -> EEdgePermRawCoord {
        EEdgePermRawCoord(self.0.const_into_coord())
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

    pub const fn from_domino_move(mv: DominoMove) -> EEdgePerm {
        use crate::cube_ops::cube_move::{
            B_EDGE_PERM, D_EDGE_PERM, F_EDGE_PERM, L_EDGE_PERM, R_EDGE_PERM, U_EDGE_PERM,
        };

        const TABLE: [EEdgePerm; 10] = const {
            let mut val = [EEdgePerm::SOLVED; 10];
            let mut i = 0;
            while i < 10 {
                let mv: DominoMove = unsafe { core::mem::transmute(i as u8) };
                val[i] = EEdgePerm(Permutation::<4>::const_from_coord(
                    (split_edge_positions(match mv {
                        DominoMove::U1 => U_EDGE_PERM,
                        DominoMove::U2 => U_EDGE_PERM.then(U_EDGE_PERM),
                        DominoMove::U3 => U_EDGE_PERM.then(U_EDGE_PERM).then(U_EDGE_PERM),
                        DominoMove::D1 => D_EDGE_PERM,
                        DominoMove::D2 => D_EDGE_PERM.then(D_EDGE_PERM),
                        DominoMove::D3 => D_EDGE_PERM.then(D_EDGE_PERM).then(D_EDGE_PERM),
                        DominoMove::F2 => F_EDGE_PERM.then(F_EDGE_PERM),
                        DominoMove::B2 => B_EDGE_PERM.then(B_EDGE_PERM),
                        DominoMove::R2 => R_EDGE_PERM.then(R_EDGE_PERM),
                        DominoMove::L2 => L_EDGE_PERM.then(L_EDGE_PERM),
                    })
                    .2
                    .into_inner()
                        % 24) as u8,
                ));
                i += 1;
            }

            val
        };
        TABLE[mv.into_index()]
    }

    pub const fn apply_domino_move(self, mv: DominoMove) -> Self {
        self.then(Self::from_domino_move(mv))
    }

    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = Self(split_edge_positions(crate::cube_ops::cube_sym::EDGE_PERM_LOOKUP[sym.0 as usize]).1.0.split().1);
        let inv_perm = perm.inverse();
        inv_perm.then(self).then(perm)
    }
}

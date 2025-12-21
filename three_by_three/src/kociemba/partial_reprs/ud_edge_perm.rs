use crate::{
    EdgePerm, cube_ops::{cube_move::DominoMove, cube_sym::DominoSymmetry}, kociemba::{coords::coords::UDEdgePermRawCoord, partial_reprs::edge_positions::{DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, combine_edge_positions, split_edge_positions}}, permutation_math::permutation::Permutation
};

/// The slot representation for corner permutation.
/// While `Permutation<N>` represents an element of the permutation group, this represents
/// a permutation when specifically applied to the cube's UD edges,
/// wherever they might be after the grouping has been applied.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct UDEdgePerm(pub UEdgePositions, pub DEdgePositions);

impl UDEdgePerm {
    pub const SOLVED: Self = const { Self::from_coord(UDEdgePermRawCoord(0)) };

    pub const fn from_coord(coord: UDEdgePermRawCoord) -> Self {
        let raw = coord.0;

        let d_perm = raw % 24;
        let tmp = raw / 24;

        let u_perm = tmp % 24;

        let d_group_residue = tmp / 24;

        let d_group = d_group_residue + 425;

        let d = DEdgePositions::from_inner(d_group * 24 + d_perm);
        let u = UEdgePositions::get_phase_2_u(d_group_residue, u_perm);

        Self(u, d)
    }

    pub const fn into_coord(self) -> UDEdgePermRawCoord {
        // solved groupings: u: 494, d: 425, e: 0
        // this means we want (d grouping - 425, u perm, d perm), which are all 0 when solved

        let u_perm = self.0.into_inner() % 24;
        let d_group = self.1.into_inner() / 24;
        let d_perm = self.1.into_inner() % 24;
        let d_group_residue = d_group - 425; // 0..70
        UDEdgePermRawCoord((d_group_residue * 24 + u_perm ) * 24 + d_perm)
    }

    const fn into_full_perm(self) -> EdgePerm {
        combine_edge_positions(self.0, self.1, EEdgePositions::SOLVED)
    }

    const fn from_full_perm(perm: EdgePerm) -> Self {
        let (u, d, _) = split_edge_positions(perm);

        Self(u, d)
    }

    pub const fn then(self, other: Self) -> Self {
        Self::from_full_perm(self.into_full_perm().then(other.into_full_perm()))
    }

    pub const fn inverse(self) -> Self {
        Self::from_full_perm(self.into_full_perm().inverse())
    }

    pub const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }

    pub const fn from_domino_move(mv: DominoMove) -> Self {
        use crate::cube_ops::cube_move::{
            B_EDGE_PERM, D_EDGE_PERM, F_EDGE_PERM, L_EDGE_PERM, R_EDGE_PERM, U_EDGE_PERM,
        };

        const TABLE: [UDEdgePerm; 10] = const {
            let mut val = [UDEdgePerm::SOLVED; 10];
            let mut i = 0;
            while i < 10 {
                let mv: DominoMove = unsafe { core::mem::transmute(i as u8) };
                let perm = split_edge_positions(match mv {
                    DominoMove::U1 => crate::cube_ops::cube_move::U_EDGE_PERM,
                    DominoMove::U2 => U_EDGE_PERM.then(U_EDGE_PERM),
                    DominoMove::U3 => U_EDGE_PERM.then(U_EDGE_PERM).then(U_EDGE_PERM),
                    DominoMove::D1 => D_EDGE_PERM,
                    DominoMove::D2 => D_EDGE_PERM.then(D_EDGE_PERM),
                    DominoMove::D3 => D_EDGE_PERM.then(D_EDGE_PERM).then(D_EDGE_PERM),
                    DominoMove::F2 => F_EDGE_PERM.then(F_EDGE_PERM),
                    DominoMove::B2 => B_EDGE_PERM.then(B_EDGE_PERM),
                    DominoMove::R2 => R_EDGE_PERM.then(R_EDGE_PERM),
                    DominoMove::L2 => L_EDGE_PERM.then(L_EDGE_PERM),
                });
                val[i] = UDEdgePerm(perm.0, perm.1);
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
        let perm = split_edge_positions(crate::cube_ops::cube_sym::EDGE_PERM_LOOKUP[sym.0 as usize]);
        let perm = Self(perm.0, perm.1);
        let inv_perm = perm.inverse();
        inv_perm.then(self).then(perm)
    }
}

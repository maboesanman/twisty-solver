use std::marker::PhantomData;

use super::{
    coords::{
        CornerOrientRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientSymCoord,
        UDEdgePermRawCoord,
    },
    cube_sym::DominoSymmetry,
    partial_reprs::edge_perm::EdgePerm,
    repr_cube::ReprCube,
};

struct TwoPhaseTables<'a> {
    phantom: PhantomData<&'a [u8]>,
}

/// used to determine the initial distance in htm from domino reduction
/// which seeds Phase1Repr::htm_dist_to_domino
pub struct Phase1InitRepr {
    edge_group_orient: EdgeGroupOrientSymCoord,
    corner_orient: CornerOrientRawCoord,
}

impl Phase1InitRepr {
    pub fn from_cube(cube: ReprCube, tables: TwoPhaseTables<'_>) -> Self {
        todo!()
    }

    pub fn adjacent(self) -> impl IntoIterator<Item = Self> {
        todo!();
        None
    }
}

pub struct Phase1Repr {
    edge_group_orient: EdgeGroupOrientSymCoord,
    corner_orient: CornerOrientRawCoord,
    edge_perm: EdgePerm,
    corner_perm: CornerPermSymCoord,
    corner_perm_correct: DominoSymmetry,

    // used as distance heuristic
    htm_dist_to_domino: u8,
}

impl Phase1Repr {
    pub fn from_cube(cube: ReprCube, tables: TwoPhaseTables<'_>, htm_dist_to_domino: u8) -> Self {
        todo!()
    }
}

/// used to determine the initial distance in htm from "solved ud oriented e"
/// which seeds Phase2Repr::htm_dist_to_solved_ud_oriented_e
/// should only be done if the last move was a non-domino move (F, F', B, B', R, R', L, L')
pub struct Phase2InitRepr {
    corner_perm: CornerPermSymCoord,
    ud_edge_perm: UDEdgePermRawCoord,
}

impl Phase2InitRepr {
    pub fn from_phase1_repr(cube: Phase1Repr, tables: TwoPhaseTables<'_>) -> Self {
        todo!()
    }
}

pub struct Phase2Repr {
    corner_perm: CornerPermSymCoord,
    ud_edge_perm: UDEdgePermRawCoord,
    e_edge_perm: EEdgePermRawCoord,

    // used as distance heuristic
    htm_dist_to_solved_ud_oriented_e: u8,
}

impl Phase2Repr {
    pub fn from_phase1_repr(cube: ReprCube, tables: TwoPhaseTables<'_>) -> Self {
        todo!()
    }
}

use pathfinding::num_traits::Euclid;

use crate::tables::{lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, move_raw_corner_orient::MoveRawCornerOrientTable, move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable, Tables};

use super::{
    coords::{
        CornerOrientRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientSymCoord,
        UDEdgePermRawCoord,
    }, cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::{edge_group_orient::{self, EdgeGroupOrient}, edge_perm::EdgePerm}, repr_cube::ReprCube
};

/// used to determine the initial distance in htm from domino reduction
/// which seeds Phase1Repr::htm_dist_to_domino
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Phase1InitRepr {
    pub edge_group_orient: EdgeGroupOrientSymCoord,
    pub corner_orient: CornerOrientRawCoord,
}

impl Phase1InitRepr {
    pub const SOLVED: Self = Self {
        edge_group_orient: EdgeGroupOrientSymCoord(0),
        corner_orient: CornerOrientRawCoord(0),
    };

    pub fn from_cube(cube: ReprCube, 
        move_raw_corner_orient: &MoveRawCornerOrientTable,
        lookup_sym_edge_group_orient: &LookupSymEdgeGroupOrientTable,
    ) -> Self {
        let corner_orient = cube.corner_orient.into_coord();
        let edge_group_orient = EdgeGroupOrient(cube.edge_perm.into_grouping(), cube.edge_orient).into_coord();

        let (edge_group_orient, sym) = lookup_sym_edge_group_orient.get_sym_from_raw(edge_group_orient);
        let corner_orient = move_raw_corner_orient.domino_conjugate(corner_orient, sym);
        Self {
            edge_group_orient,
            corner_orient,
        }
    }

    pub fn apply_cube_move(
        self,
        mv: CubeMove, 
        move_sym_edge_group_orient: &MoveSymEdgeGroupOrientTable,
        move_raw_corner_orient: &MoveRawCornerOrientTable,
    ) -> Self {
        let (edge_group_orient, sym) = move_sym_edge_group_orient.apply_cube_move(self.edge_group_orient, mv);
        let corner_orient_pre_conj = move_raw_corner_orient.apply_cube_move(self.corner_orient, mv);
        let corner_orient = move_raw_corner_orient.domino_conjugate(corner_orient_pre_conj, sym);

        Self {
            edge_group_orient,
            corner_orient,
        }
    }

    pub fn adjacent<'t>(self,
        move_sym_edge_group_orient: &'t MoveSymEdgeGroupOrientTable,
        move_raw_corner_orient: &'t MoveRawCornerOrientTable,) -> impl Send + Sync + 't + Iterator<Item = Self> {
        CubeMove::all_iter().map(move |mv| {
            self.apply_cube_move(mv, move_sym_edge_group_orient,move_raw_corner_orient)
        })
    }

    pub fn is_solved(self) -> bool {
        self.corner_orient.0 == 0 && self.edge_group_orient.0 == 0
    }

    pub fn into_index(self) -> usize {
        self.edge_group_orient.0 as usize * 2187 + self.corner_orient.0 as usize
    }

    pub fn from_index(index: usize) -> Self {
        let (edges, corners) = index.div_rem_euclid(&2187);
        Self {
            edge_group_orient: EdgeGroupOrientSymCoord(edges as u16),
            corner_orient: CornerOrientRawCoord(corners as u16)
        }
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

// impl Phase1Repr {
//     pub fn from_cube(cube: ReprCube, tables: TwoPhaseTables<'_>, htm_dist_to_domino: u8) -> Self {
//         todo!()
//     }
// }

/// used to determine the initial distance in htm from "solved ud oriented e"
/// which seeds Phase2Repr::htm_dist_to_solved_ud_oriented_e
/// should only be done if the last move was a non-domino move (F, F', B, B', R, R', L, L')
pub struct Phase2InitRepr {
    corner_perm: CornerPermSymCoord,
    ud_edge_perm: UDEdgePermRawCoord,
}

// impl Phase2InitRepr {
//     pub fn from_phase1_repr(cube: Phase1Repr, tables: TwoPhaseTables<'_>) -> Self {
//         todo!()
//     }
// }

pub struct Phase2Repr {
    corner_perm: CornerPermSymCoord,
    ud_edge_perm: UDEdgePermRawCoord,
    e_edge_perm: EEdgePermRawCoord,

    // used as distance heuristic
    htm_dist_to_solved_ud_oriented_e: u8,
}

// impl Phase2Repr {
//     pub fn from_phase1_repr(cube: ReprCube, tables: TwoPhaseTables<'_>) -> Self {
//         todo!()
//     }
// }

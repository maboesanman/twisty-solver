use pathfinding::num_traits::Euclid;

use crate::tables::{
    Tables, lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable,
    move_raw_corner_orient::MoveRawCornerOrientTable,
    move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
};

use super::{
    coords::{
        CornerOrientRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientSymCoord,
        UDEdgePermRawCoord,
    },
    cube_move::CubeMove,
    cube_sym::DominoSymmetry,
    partial_reprs::{
        edge_group_orient::{self, EdgeGroupOrient},
        edge_perm::EdgePerm,
    },
    repr_cube::ReprCube,
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

    pub fn from_cube(
        cube: ReprCube,
        move_raw_corner_orient: &MoveRawCornerOrientTable,
        lookup_sym_edge_group_orient: &LookupSymEdgeGroupOrientTable,
    ) -> Self {
        let corner_orient = cube.corner_orient.into_coord();
        let edge_group_orient =
            EdgeGroupOrient(cube.edge_perm.split().0, cube.edge_orient).into_coord();

        let (edge_group_orient, sym) =
            lookup_sym_edge_group_orient.get_sym_from_raw(edge_group_orient);
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
    ) -> (Self, DominoSymmetry) {
        let (edge_group_orient, sym) =
            move_sym_edge_group_orient.apply_cube_move(self.edge_group_orient, mv);
        let corner_orient_pre_conj = move_raw_corner_orient.apply_cube_move(self.corner_orient, mv);
        let corner_orient = move_raw_corner_orient.domino_conjugate(corner_orient_pre_conj, sym);

        (
            Self {
                edge_group_orient,
                corner_orient,
            },
            sym,
        )
    }

    pub fn adjacent<'t>(
        self,
        move_sym_edge_group_orient: &'t MoveSymEdgeGroupOrientTable,
        move_raw_corner_orient: &'t MoveRawCornerOrientTable,
    ) -> impl Send + Sync + 't + Iterator<Item = (Self, DominoSymmetry)> {
        CubeMove::all_iter().map(move |mv| {
            self.apply_cube_move(mv, move_sym_edge_group_orient, move_raw_corner_orient)
        })
    }

    pub fn is_domino_reduced(self) -> bool {
        self.corner_orient.0 == 0 && self.edge_group_orient.0 == 0
    }

    pub fn into_index(self) -> usize {
        self.edge_group_orient.0 as usize * 2187 + self.corner_orient.0 as usize
    }

    pub fn from_index(index: usize) -> Self {
        let (edges, corners) = index.div_rem_euclid(&2187);
        Self {
            edge_group_orient: EdgeGroupOrientSymCoord(edges as u16),
            corner_orient: CornerOrientRawCoord(corners as u16),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rayon::iter::IntoParallelIterator;
    use rayon::iter::ParallelIterator;

    #[test]
    fn adjacency_reversible() {
        let lookup_sym_edge_group_orient = LookupSymEdgeGroupOrientTable::load(
            "edge_group_orient_sym_lookup_table.dat",
        ).unwrap();

        let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
            "edge_group_orient_sym_move_table.dat",
            &lookup_sym_edge_group_orient,
        ).unwrap();

        let move_raw_corner_orient = MoveRawCornerOrientTable::load("corner_orient_move_table.dat").unwrap();

        let move_sym_edge_group_orient_ref = &move_sym_edge_group_orient;
        let move_raw_corner_orient_ref = &move_raw_corner_orient;

        (0..2187 * 64430)
            .into_par_iter()
            .map(Phase1InitRepr::from_index)
            .for_each(|cube| {
                assert!(CubeMove::all_iter().flat_map(|mv| {
                    let cube_2 = cube.apply_cube_move(mv, move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).0;

                    CubeMove::all_iter().map(move |mv_2| {
                        cube_2.apply_cube_move(mv_2, move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).0
                    })
                }).any(|c| c == cube))
            })
    }
}

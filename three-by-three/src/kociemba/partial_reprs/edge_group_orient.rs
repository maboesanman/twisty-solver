use crate::{cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::{edge_orient::EdgeOrient, edge_perm::EdgePerm}}, kociemba::coords::coords::{EdgeGroupOrientRawCoord, EdgeGroupRawCoord, EdgeOrientRawCoord}};

use super::{edge_group::EdgeGroup};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EdgeGroupOrient(pub EdgeGroup, pub EdgeOrient);

impl EdgeGroupOrient {
    pub const SOLVED: Self = Self(EdgeGroup::SOLVED, EdgeOrient::SOLVED);

    pub const fn from_coord(coord: EdgeGroupOrientRawCoord) -> Self {
        let group_coord = (coord.0 >> 11) as u16;
        let orient_coord = (coord.0 & 0b11111111111) as u16;
        Self(
            EdgeGroup::from_coord(EdgeGroupRawCoord(group_coord)),
            EdgeOrient::from_coord(EdgeOrientRawCoord(orient_coord)),
        )
    }

    pub const fn into_coord(self) -> EdgeGroupOrientRawCoord {
        let group_coord = self.0.into_coord().0 as u32;
        let orient_coord = self.1.into_coord().0 as u32;
        EdgeGroupOrientRawCoord((group_coord << 11) | orient_coord)
    }

    pub const fn then(self, perm: EdgePerm) -> Self {
        Self(self.0.permute(perm), self.1.permute(perm))
    }
}

impl EdgeGroupOrient {
    pub const fn apply_cube_move(self, mv: CubeMove) -> Self {
        Self(self.0.apply_cube_move(mv), self.1.apply_cube_move(mv))
    }

    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = crate::cube_ops::cube_sym::EDGE_PERM_LOOKUP[sym.0 as usize];
        let new_group = self.0.permute(perm);
        let mut orient = self.1.permute(perm);
    
        if ((sym.0 >> 1) & 1) == 1 {
            let mut i = 0;
            while i < 12 {
                if new_group.0.0[i] {
                    orient.0[i] ^= 1;
                }
                i += 1;
            }
            orient = orient.correct(crate::cube_ops::cube_sym::S_U4_1_EDGE_ORIENT_CORRECT);
        }
    
        EdgeGroupOrient(new_group, orient)
    }
}

#[cfg(test)]
mod test {

    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use crate::cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry};

    use super::*;

    #[test]
    fn test() {
        (0..(2048 * 495)).into_par_iter().for_each(|i| {
            let coord = EdgeGroupOrientRawCoord(i as u32);
            let group_orient = EdgeGroupOrient::from_coord(coord);
            assert_eq!(coord, group_orient.into_coord())
        })
    }

    #[test]
    fn move_adjacency() {
        (0..(2048 * 495)).into_par_iter().for_each(|i| {
            let coord = EdgeGroupOrientRawCoord(i as u32);
            let group_orient = EdgeGroupOrient::from_coord(coord);

            for adj in CubeMove::all_iter().map(|mv| group_orient.apply_cube_move(mv)) {
                assert!(
                    CubeMove::all_iter()
                        .map(|mv| adj.apply_cube_move(mv))
                        .any(|adj_adj| adj_adj == group_orient)
                )
            }
        })
    }

    #[test]
    fn conjugation_adjacency() {
        (0..(2048 * 495)).into_par_iter().for_each(|i| {
            let coord = EdgeGroupOrientRawCoord(i as u32);
            let group_orient = EdgeGroupOrient::from_coord(coord);

            let one_step: Vec<_> = DominoSymmetry::all_iter()
                .map(|sym| group_orient.domino_conjugate(sym))
                .collect();

            for &elem in &one_step {
                for sym in DominoSymmetry::nontrivial_iter() {
                    let two_step = elem.domino_conjugate(sym);
                    assert!(one_step.contains(&two_step))
                }
            }
        })
    }
}

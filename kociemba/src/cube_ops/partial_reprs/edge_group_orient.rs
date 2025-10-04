use crate::cube_ops::coords::{EdgeGroupRawCoord, EdgeOrientRawCoord};

use super::{edge_group::EdgeGroup, edge_orient::EdgeOrient, edge_perm::EdgePerm};

// fits in 20 bits
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EdgeGroupOrientRawCoord(pub u32);

impl EdgeGroupOrientRawCoord {
    pub fn split(self) -> (EdgeGroupRawCoord, EdgeOrientRawCoord) {
        (
            EdgeGroupRawCoord((self.0 >> 11) as u16),
            EdgeOrientRawCoord((self.0 & 0b11111111111) as u16),
        )
    }

    pub fn join(group: EdgeGroupRawCoord, orient: EdgeOrientRawCoord) -> Self {
        Self(((group.0 as u32) << 11) & (orient.0 as u32))
    }
}

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

use crate::cube_ops::{
    coords::{EdgeGroupOrientRawCoord, EdgeGroupOrientSymCoord},
    cube_move::CubeMove,
    cube_sym::DominoSymmetry,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EdgeGroupOrientComboCoord {
    pub sym_coord: EdgeGroupOrientSymCoord,
    pub domino_conjugation: DominoSymmetry,
}

impl EdgeGroupOrientComboCoord {
    pub fn from_raw(tables: &crate::tables::Tables, raw_coord: EdgeGroupOrientRawCoord) -> Self {
        tables
            .lookup_sym_edge_group_orient
            .get_combo_from_raw(raw_coord)
    }

    pub fn into_raw(self, tables: &crate::tables::Tables) -> EdgeGroupOrientRawCoord {
        tables.lookup_sym_edge_group_orient.get_raw_from_combo(self)
    }

    pub fn apply_cube_move(self, tables: &crate::tables::Tables, cube_move: CubeMove) -> Self {
        let preimage_move = cube_move.domino_conjugate(self.domino_conjugation);
        let mut result = tables
            .move_sym_edge_group_orient
            .apply_cube_move(self.sym_coord, preimage_move);

        result.domino_conjugation = self.domino_conjugation.then(result.domino_conjugation);

        result
    }

    pub fn domino_conjugate(self, domino_symmetry: DominoSymmetry) -> Self {
        Self {
            sym_coord: self.sym_coord,
            domino_conjugation: domino_symmetry.inverse().then(self.domino_conjugation),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, sync::atomic::AtomicU32};

    use itertools::Itertools;
    use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

    use crate::{cube_ops::partial_reprs::edge_group_orient::EdgeGroupOrient, tables::Tables};

    use super::*;

    #[test]
    fn round_trip() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        for i in 0..(495 * 2048) {
            let raw = EdgeGroupOrientRawCoord(i);
            let combo = EdgeGroupOrientComboCoord::from_raw(&tables, raw);
            let raw_again = combo.into_raw(&tables);

            assert_eq!(raw, raw_again);
        }

        Ok(())
    }

    #[test]
    fn moves_match_raw() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        (0..495 * 2048).into_par_iter().for_each(|i| {
            let raw = EdgeGroupOrientRawCoord(i);
            let combo = EdgeGroupOrientComboCoord::from_raw(&tables, raw);
            let group_orient = EdgeGroupOrient::from_coord(raw);

            for cube_move in CubeMove::all_iter() {
                let new_group_orient = group_orient.apply_cube_move(cube_move);
                let new_raw = new_group_orient.into_coord();
                let new_combo = combo.apply_cube_move(&tables, cube_move);

                assert_eq!(new_raw, new_combo.into_raw(&tables));
            }
        });

        Ok(())
    }

    #[test]
    fn conjugations_match_raw() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        (0..495 * 2048).into_par_iter().for_each(|i| {
            let raw = EdgeGroupOrientRawCoord(i);
            let combo = EdgeGroupOrientComboCoord::from_raw(&tables, raw);
            let group_orient = EdgeGroupOrient::from_coord(raw);

            for sym in DominoSymmetry::all_iter() {
                let new_group_orient = group_orient.domino_conjugate(sym);
                let new_raw = new_group_orient.into_coord();
                let new_combo = combo.domino_conjugate(sym);

                assert_eq!(new_raw, new_combo.into_raw(&tables));
            }
        });

        Ok(())
    }
}

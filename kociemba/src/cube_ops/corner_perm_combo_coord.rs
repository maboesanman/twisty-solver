use crate::cube_ops::{
    coords::{CornerPermRawCoord, CornerPermSymCoord},
    cube_move::CubeMove,
    cube_sym::DominoSymmetry,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CornerPermComboCoord {
    pub sym_coord: CornerPermSymCoord,
    pub domino_conjugation: DominoSymmetry,
}

impl CornerPermComboCoord {
    pub fn from_raw(tables: &crate::tables::Tables, raw_coord: CornerPermRawCoord) -> Self {
        tables.lookup_sym_corner_perm.get_combo_from_raw(raw_coord)
    }

    pub fn into_raw(self, tables: &crate::tables::Tables) -> CornerPermRawCoord {
        tables.lookup_sym_corner_perm.get_raw_from_combo(self)
    }

    pub fn apply_cube_move(self, tables: &crate::tables::Tables, cube_move: CubeMove) -> Self {
        let preimage_move = cube_move.domino_conjugate(self.domino_conjugation);
        let mut result = tables
            .move_sym_corner_perm
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
    use super::*;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use crate::{cube_ops::partial_reprs::corner_perm::CornerPerm, tables::Tables};

    #[test]
    fn round_trip() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        for i in 0..40320 {
            let raw = CornerPermRawCoord(i);
            let combo = CornerPermComboCoord::from_raw(&tables, raw);
            let raw_again = combo.into_raw(&tables);

            assert_eq!(raw, raw_again);
        }

        Ok(())
    }

    #[test]
    fn moves_match_raw() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        (0..40320).into_iter().for_each(|i| {
            let raw = CornerPermRawCoord(i);
            let combo = CornerPermComboCoord::from_raw(&tables, raw);
            let corner_perm = CornerPerm::from_coord(raw);

            for cube_move in CubeMove::all_iter() {
                let new_corner_perm = corner_perm.apply_cube_move(cube_move);
                let new_raw = new_corner_perm.into_coord();
                let new_combo = combo.apply_cube_move(&tables, cube_move);

                assert_eq!(new_raw, new_combo.into_raw(&tables));
            }
        });

        Ok(())
    }

    #[test]
    fn conjugations_match_raw() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        (0..40320).into_par_iter().for_each(|i| {
            let raw = CornerPermRawCoord(i);
            let combo = CornerPermComboCoord::from_raw(&tables, raw);
            let corner_perm = CornerPerm::from_coord(raw);

            for sym in DominoSymmetry::all_iter() {
                let new_corner_perm = corner_perm.domino_conjugate(sym);
                let new_raw = new_corner_perm.into_coord();
                let new_combo = combo.domino_conjugate(sym);

                assert_eq!(new_raw, new_combo.into_raw(&tables));
            }
        });

        Ok(())
    }
}

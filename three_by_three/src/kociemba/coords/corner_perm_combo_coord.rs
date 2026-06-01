use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{CornerPermRawCoord, CornerPermSymCoord},
        tables::{
            lookup_sym_corner_perm::LookupSymCornerPermTable,
            move_sym_corner_perm::MoveSymCornerPermTable,
        },
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CornerPermComboCoord {
    pub sym_coord: CornerPermSymCoord,
    pub domino_conjugation: DominoSymmetry,
}

impl CornerPermComboCoord {
    pub fn from_raw(
        tables: impl AsRef<LookupSymCornerPermTable>,
        raw_coord: CornerPermRawCoord,
    ) -> Self {
        tables.as_ref().get_combo_from_raw(raw_coord)
    }

    pub fn into_raw(self, tables: impl AsRef<LookupSymCornerPermTable>) -> CornerPermRawCoord {
        tables.as_ref().get_raw_from_combo(self)
    }

    pub fn into_dense(self) -> u16 {
        (self.sym_coord.0 & 0x0FFF) | ((self.domino_conjugation.0 as u16) << 12)
    }

    pub fn from_dense(dense: u16) -> Self {
        Self {
            sym_coord: CornerPermSymCoord(dense & 0x0FFF),
            domino_conjugation: DominoSymmetry((dense >> 12) as u8),
        }
    }

    pub fn apply_cube_move(
        self,
        tables: impl AsRef<MoveSymCornerPermTable>,
        cube_move: CubeMove,
    ) -> Self {
        let preimage_move = cube_move.domino_conjugate(self.domino_conjugation);
        let mut result = tables
            .as_ref()
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

    use crate::{cube_ops::partial_reprs::corner_perm::CornerPerm, kociemba::tables::Tables};

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

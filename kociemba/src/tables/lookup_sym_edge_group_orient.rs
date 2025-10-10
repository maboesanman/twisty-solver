use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        combo_coords::EdgeGroupOrientComboCoord, coords::{EdgeGroupOrientRawCoord, EdgeGroupOrientSymCoord}, cube_sym::DominoSymmetry, partial_reprs::edge_group_orient::EdgeGroupOrient
    },
    tables::table_loader::{as_u32_slice, collect_unique_sorted_parallel},
};

use super::table_loader::{as_u32_slice_mut, load_table};

const TABLE_SIZE_BYTES: usize = 64430 * 4;
const FILE_CHECKSUM: u32 = 4005177882;

pub struct LookupSymEdgeGroupOrientTable(Mmap);

impl LookupSymEdgeGroupOrientTable {
    pub fn get_rep_from_sym(&self, sym_coord: EdgeGroupOrientSymCoord) -> EdgeGroupOrientRawCoord {
        let buffer = as_u32_slice(&self.0);
        EdgeGroupOrientRawCoord(buffer[sym_coord.0 as usize])
    }

    pub fn get_raw_from_combo(&self, combo_coord: EdgeGroupOrientComboCoord) -> EdgeGroupOrientRawCoord {
        EdgeGroupOrient::from_coord(self.get_rep_from_sym(combo_coord.sym_coord)).domino_conjugate(combo_coord.domino_conjugation).into_coord()
    }

    pub fn get_combo_from_raw(
        &self,
        raw_coord: EdgeGroupOrientRawCoord,
    ) -> EdgeGroupOrientComboCoord {
        let buffer = as_u32_slice(&self.0);
        let edge_group_orient = EdgeGroupOrient::from_coord(raw_coord);
        let (rep_coord, domino_conjugation) = DominoSymmetry::all_iter()
            .map(|sym| (edge_group_orient.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        EdgeGroupOrientComboCoord {
            sym_coord: EdgeGroupOrientSymCoord(buffer.binary_search(&rep_coord.0).unwrap() as u16),
            domino_conjugation,
        }
    }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u32_slice_mut(buffer);
        let reps = (0u32..(2048 * 495)).into_par_iter().map(|i| {
            let raw_coord = EdgeGroupOrientRawCoord(i);
            let edge_group_orient = EdgeGroupOrient::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| edge_group_orient.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        for (i, rep) in collect_unique_sorted_parallel(reps).enumerate() {
            buffer[i] = rep.0
        }

        debug_assert!(buffer.is_sorted())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf)
        })
        .map(Self)
    }
}

#[cfg(test)]
mod test {
    use crate::tables::Tables;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_edge_group_orient;

        (0u32..(2048 * 495)).into_par_iter().for_each(|i| {
            let a = EdgeGroupOrientRawCoord(i);
            let combo = table.get_combo_from_raw(a);
            let b = table.get_raw_from_combo(combo);

            assert_eq!(a, b)
        });

        Ok(())
    }
}

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        coords::EdgeGroupOrientSymCoord,
        cube_sym::DominoSymmetry,
        partial_reprs::edge_group_orient::{EdgeGroupOrient, EdgeGroupOrientRawCoord},
    },
    tables::table_loader::{as_u32_slice, collect_unique_sorted_parallel},
};

use super::table_loader::{as_u32_slice_mut, load_table};

const TABLE_SIZE_BYTES: usize = 64430 * 4;
const FILE_CHECKSUM: u32 = 4005177882;

pub struct LookupSymEdgeGroupOrientTable(Mmap);

impl LookupSymEdgeGroupOrientTable {
    pub fn get_raw_from_sym(&self, sym_coord: EdgeGroupOrientSymCoord) -> EdgeGroupOrientRawCoord {
        let buffer = as_u32_slice(&self.0);
        EdgeGroupOrientRawCoord(buffer[sym_coord.0 as usize])
    }

    pub fn get_sym_from_raw(
        &self,
        raw_coord: EdgeGroupOrientRawCoord,
    ) -> (EdgeGroupOrientSymCoord, DominoSymmetry) {
        let buffer = as_u32_slice(&self.0);
        let edge_group_orient = EdgeGroupOrient::from_coord(raw_coord);
        let (rep_coord, sym) = DominoSymmetry::all_iter()
            .map(|sym| (edge_group_orient.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        (
            EdgeGroupOrientSymCoord(buffer.binary_search(&rep_coord.0).unwrap() as u16),
            sym,
        )
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
    use num_integer::Integer;

    use crate::{cube_ops::coords::{EdgeGroupRawCoord, EdgeOrientRawCoord}, tables::Tables};

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_edge_group_orient;

        (0u32..(2048 * 495)).into_par_iter().for_each(|i| {
            let raw_coord = EdgeGroupOrientRawCoord(i);
            let edge_group_orient = EdgeGroupOrient::from_coord(raw_coord);
            
            let (sym_coord, sym) = table.get_sym_from_raw(raw_coord);
            let updated_raw = edge_group_orient.domino_conjugate(sym).into_coord();
            let rep_coord = table.get_raw_from_sym(sym_coord);

            assert_eq!(rep_coord, updated_raw)
        });
    
        Ok(())
    }
}

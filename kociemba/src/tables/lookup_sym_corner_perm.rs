use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::cube_ops::{
    coords::{CornerPermRawCoord, CornerPermSymCoord},
    cube_sym::DominoSymmetry,
    partial_reprs::corner_perm::CornerPerm,
};

use super::table_loader::{
    as_u16_slice, as_u16_slice_mut, collect_unique_sorted_parallel, load_table,
};

const TABLE_SIZE_BYTES: usize = 2768 * 2;
const FILE_CHECKSUM: u32 = 2748406986;

pub struct LookupSymCornerPermTable(Mmap);

impl LookupSymCornerPermTable {
    pub fn get_raw_from_sym(&self, sym_coord: CornerPermSymCoord) -> CornerPermRawCoord {
        let buffer = as_u16_slice(&self.0);
        CornerPermRawCoord(buffer[sym_coord.0 as usize])
    }

    pub fn get_sym_from_raw(
        &self,
        raw_coord: CornerPermRawCoord,
    ) -> (CornerPermSymCoord, DominoSymmetry) {
        let buffer = as_u16_slice(&self.0);
        let corner_perm = CornerPerm::from_coord(raw_coord);
        let (rep_coord, sym) = DominoSymmetry::all_iter()
            .map(|sym| (corner_perm.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        (
            CornerPermSymCoord(buffer.binary_search(&rep_coord.0).unwrap() as u16),
            sym,
        )
    }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u16_slice_mut(buffer);
        let reps = (0..40320).into_par_iter().map(|i| {
            let raw_coord = CornerPermRawCoord(i);
            let corner_perm = CornerPerm::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| corner_perm.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        for (i, rep) in collect_unique_sorted_parallel(reps).enumerate() {
            buffer[i] = rep.0
        }
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

        let table = &tables.lookup_sym_corner_perm;

        (0..40320).into_par_iter().for_each(|i| {
            let raw_coord = CornerPermRawCoord(i);
            let corner_perm = CornerPerm::from_coord(raw_coord);
            
            let (sym_coord, sym) = table.get_sym_from_raw(raw_coord);
            let updated_raw = corner_perm.domino_conjugate(sym).into_coord();
            let rep_coord = table.get_raw_from_sym(sym_coord);

            assert_eq!(rep_coord, updated_raw)
        });
    
        Ok(())
    }
}



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
const FILE_CHECKSUM: u32 = 188933558;

pub struct LookupSymCornerPermTable(Mmap);

impl LookupSymCornerPermTable {
    pub fn get_raw_from_sym(&self, sym_coord: CornerPermSymCoord) -> CornerPermRawCoord {
        let buffer = as_u16_slice(&self.0);
        let (even, odd) = buffer.split_at(2768 / 2);

        let buffer = if sym_coord.0 % 2 == 0 {
            even
        } else {
            odd
        };

        CornerPermRawCoord(buffer[sym_coord.0 as usize / 2])
    }

    pub fn get_sym_from_raw(
        &self,
        raw_coord: CornerPermRawCoord,
    ) -> (CornerPermSymCoord, DominoSymmetry) {
        let buffer = as_u16_slice(&self.0);
        let (even, odd) = buffer.split_at(2768 >> 1);
        let corner_perm = CornerPerm::from_coord(raw_coord);
        let (rep_coord, sym) = DominoSymmetry::all_iter()
            .map(|sym| (corner_perm.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        // index within its parity half
        let pos_in_half = if raw_coord.0 % 2 == 0 {
            even.binary_search(&rep_coord.0).unwrap()
        } else {
            odd.binary_search(&rep_coord.0).unwrap()
        };

        // pack: (pos << 1) | parity
        let packed = ((pos_in_half as u16) << 1) | ((raw_coord.0 & 1) as u16);

        (CornerPermSymCoord(packed), sym)
    }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u16_slice_mut(buffer);
        let (even, odd) = buffer.split_at_mut(2768 >> 1);

        let even_reps = (0..(40320 >> 1)).into_par_iter().map(|i| {
            let raw_coord = CornerPermRawCoord(i << 1);
            let corner_perm = CornerPerm::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| corner_perm.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        let odd_reps = (0..(40320 >> 1)).into_par_iter().map(|i| {
            let raw_coord = CornerPermRawCoord((i << 1) + 1);
            let corner_perm = CornerPerm::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| corner_perm.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        for (i, rep) in collect_unique_sorted_parallel(even_reps).enumerate() {
            even[i] = rep.0
        }

        for (i, rep) in collect_unique_sorted_parallel(odd_reps).enumerate() {
            odd[i] = rep.0
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

        (0..40320).into_iter().for_each(|i| {
            let raw_coord = CornerPermRawCoord(i);
            let corner_perm = CornerPerm::from_coord(raw_coord);

            let (sym_coord, sym) = table.get_sym_from_raw(raw_coord);
            let updated_raw = corner_perm.domino_conjugate(sym).into_coord();
            let rep_coord = table.get_raw_from_sym(sym_coord);

            assert_eq!(rep_coord, updated_raw)
        });

        Ok(())
    }

    #[test]
    fn test_parity_preserved() -> Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_corner_perm;

        (0..2768).into_iter().for_each(|i| {
            let sym_coord = CornerPermSymCoord(i);
            let raw_coord = table.get_raw_from_sym(sym_coord);
            let corner_perm = CornerPerm::from_coord(raw_coord);

            assert_eq!(corner_perm.0.is_odd(), i & 0b1 == 1);
        });

        Ok(())
    }
}

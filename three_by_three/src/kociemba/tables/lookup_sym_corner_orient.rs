use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    CornerOrient, cube_ops::cube_sym::DominoSymmetry, kociemba::{
        coords::CornerOrientRawCoord, tables::table_loader::{as_u16_slice, collect_unique_sorted_parallel}
    }
};

use super::table_loader::{as_u16_slice_mut, load_table};

const TABLE_SIZE_BYTES: usize = 168 * 2;
const FILE_CHECKSUM: u32 = 638929069;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CornerOrientSymCoord(pub u8);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CornerOrientComboCoord {
    pub sym_coord: CornerOrientSymCoord,
    pub domino_conjugation: DominoSymmetry
}

pub struct LookupSymCornerOrientTable([u8]);

impl LookupSymCornerOrientTable {
    pub fn get_rep_from_sym(&self, sym_coord: CornerOrientSymCoord) -> CornerOrientRawCoord {
        let buffer = as_u16_slice(&self.0);
        CornerOrientRawCoord(buffer[sym_coord.0 as usize])
    }

    pub fn get_raw_from_combo(
        &self,
        combo_coord: CornerOrientComboCoord,
    ) -> CornerOrientRawCoord {
        CornerOrient::from_coord(self.get_rep_from_sym(combo_coord.sym_coord))
            .domino_conjugate(combo_coord.domino_conjugation.inverse())
            .into_coord()
    }

    pub fn get_combo_from_raw(
        &self,
        raw_coord: CornerOrientRawCoord,
    ) -> CornerOrientComboCoord {
        let buffer = as_u16_slice(&self.0);
        let corner_orient = CornerOrient::from_coord(raw_coord);
        let (rep_coord, domino_conjugation) = DominoSymmetry::all_iter()
            .map(|sym| (corner_orient.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        CornerOrientComboCoord {
            sym_coord: CornerOrientSymCoord(buffer.binary_search(&rep_coord.0).unwrap() as u8),
            domino_conjugation,
        }
    }

    // /// includes the identity
    // pub fn get_all_stabilizing_conjugations(
    //     &self,
    //     sym_coord: CornerOrientSymCoord,
    // ) -> impl IntoIterator<Item = DominoSymmetry> {
    //     let rep = self.get_rep_from_sym(sym_coord);
    //     let group_orient = CornerOrient::from_coord(rep);
    //     DominoSymmetry::all_iter()
    //         .filter(move |sym| group_orient.domino_conjugate(*sym) == group_orient)
    // }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u16_slice_mut(buffer);
        let reps = (0u16..(2187)).into_par_iter().map(|i| {
            let raw_coord = CornerOrientRawCoord(i);
            let corner_orient = CornerOrient::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| corner_orient.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        let mut n = 0;
        for (i, rep) in collect_unique_sorted_parallel(reps).enumerate() {
            buffer[i] = rep.0;
            n += 1;
        }

        println!("NUM SYM COORDS: {n}");

        debug_assert!(buffer.is_sorted())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, Self::generate)
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use itertools::Itertools;

    use crate::kociemba::tables::Tables;

    use super::*;

    #[test]
    fn round_trip() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table: &LookupSymCornerOrientTable = tables.as_ref();

        for i in 0..(2187) {
            let raw = CornerOrientRawCoord(i);
            let combo = table.get_combo_from_raw(raw);
            let raw_again = table.get_raw_from_combo(combo);

            assert_eq!(raw, raw_again);
        }

        Ok(())
    }
}

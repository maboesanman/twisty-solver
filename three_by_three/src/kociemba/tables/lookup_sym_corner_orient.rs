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

const TABLE_SIZE_BYTES: usize = 168 * 4;
const FILE_CHECKSUM: u32 = 4005177882;

pub struct CornerOrientSymCoord(u16);

pub struct LookupSymCornerOrientTable([u8]);

impl LookupSymCornerOrientTable {
    pub fn get_rep_from_sym(&self, sym_coord: CornerOrientSymCoord) -> CornerOrientRawCoord {
        let buffer = as_u16_slice(&self.0);
        CornerOrientRawCoord(buffer[sym_coord.0 as usize])
    }

    // pub fn get_raw_from_combo(
    //     &self,
    //     combo_coord: CornerOrientComboCoord,
    // ) -> CornerOrientRawCoord {
    //     CornerOrient::from_coord(self.get_rep_from_sym(combo_coord.sym_coord))
    //         .domino_conjugate(combo_coord.domino_conjugation.inverse())
    //         .into_coord()
    // }

    // pub fn get_combo_from_raw(
    //     &self,
    //     raw_coord: CornerOrientRawCoord,
    // ) -> CornerOrientComboCoord {
    //     let buffer = as_u16_slice(&self.0);
    //     let edge_group_orient = CornerOrient::from_coord(raw_coord);
    //     let (rep_coord, domino_conjugation) = DominoSymmetry::all_iter()
    //         .map(|sym| (edge_group_orient.domino_conjugate(sym).into_coord(), sym))
    //         .min_by_key(|x| x.0)
    //         .unwrap();

    //     CornerOrientComboCoord {
    //         sym_coord: CornerOrientSymCoord(buffer.binary_search(&rep_coord.0).unwrap() as u16),
    //         domino_conjugation,
    //     }
    // }

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
            let edge_group_orient = CornerOrient::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| edge_group_orient.domino_conjugate(sym).into_coord())
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
    fn calc_cardinality() -> anyhow::Result<()> {
        let mut buf = [0; 2187 * 2];
        LookupSymCornerOrientTable::generate(&mut buf);

        Ok(())
    }

    // #[test]
    // fn round_trip() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let table: &LookupSymCornerOrientTable = tables.as_ref();

    //     for i in 0..(495 * 2048) {
    //         let raw = CornerOrientRawCoord(i);
    //         let combo = table.get_combo_from_raw(raw);
    //         let raw_again = table.get_raw_from_combo(combo);

    //         assert_eq!(raw, raw_again);
    //     }

    //     Ok(())
    // }

    // #[test]
    // fn test_stabilizing_conjugations() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let table: &LookupSymCornerOrientTable = tables.as_ref();

    //     (0..64430).into_par_iter().for_each(|i| {
    //         let sym = CornerOrientSymCoord(i);
    //         let rep = table.get_rep_from_sym(sym);
    //         let group_orient = CornerOrient::from_coord(rep);
    //         let stabilizing_conjugations = table
    //             .get_all_stabilizing_conjugations(sym)
    //             .into_iter()
    //             .collect_vec();
    //         for s in DominoSymmetry::all_iter() {
    //             let stabilized = group_orient == group_orient.domino_conjugate(s);
    //             assert_eq!(stabilized, stabilizing_conjugations.contains(&s))
    //         }
    //     });

    //     Ok(())
    // }

    // #[test]
    // fn check_for_stabilizing_conj() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let table: &LookupSymCornerOrientTable = tables.as_ref();

    //     // 2033 of the 64430 sym coords have nontrivial stabilizing symmetries
    //     // there are 26 possible cardinalities

    //     let nonzero_count: HashMap<_, _> = (0..64430)
    //         .into_par_iter()
    //         .map(|i| {
    //             let sym = CornerOrientSymCoord(i);
    //             let rep = table.get_rep_from_sym(sym);
    //             let group_orient = CornerOrient::from_coord(rep);

    //             (
    //                 sym.0,
    //                 DominoSymmetry::all_iter().fold(0u16, |acc, sym| {
    //                     acc | ((group_orient == group_orient.domino_conjugate(sym)) as u16) << sym.0
    //                 }),
    //             )
    //         })
    //         .filter(|x| x.1 != 1)
    //         .collect();

    //     let mut reversed: HashMap<u16, Vec<u16>> = HashMap::new();

    //     for (k, v) in nonzero_count {
    //         reversed.entry(v).or_default().push(k);
    //     }

    //     let mut out_string =
    //         "static STABILIZING_CONJUGATIONS: phf::Map<u16, u16> = phf::phf_map! {\n".to_string();
    //     for (k, v) in reversed {
    //         out_string.push_str(&format!(
    //             "    {} => {},\n",
    //             v.into_iter().map(|x| format!("{x}")).join(" | "),
    //             k
    //         ));
    //     }
    //     out_string.push_str("};");

    //     println!("{out_string}");

    //     Ok(())
    // }
}

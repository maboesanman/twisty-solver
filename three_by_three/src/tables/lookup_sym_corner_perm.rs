use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{cube_sym::DominoSymmetry, partial_reprs::corner_perm::CornerPerm},
    kociemba::coords::{
        coords::{CornerPermRawCoord, CornerPermSymCoord},
        corner_perm_combo_coord::CornerPermComboCoord,
    },
};

use super::table_loader::{
    as_u16_slice, as_u16_slice_mut, collect_unique_sorted_parallel, load_table,
};

const TABLE_SIZE_BYTES: usize = 2768 * 2;
const FILE_CHECKSUM: u32 = 188933558;

pub struct LookupSymCornerPermTable(Mmap);

impl LookupSymCornerPermTable {
    pub fn get_rep_from_sym(&self, sym_coord: CornerPermSymCoord) -> CornerPermRawCoord {
        let buffer = as_u16_slice(&self.0);
        let (even, odd) = buffer.split_at(2768 / 2);

        let buffer = if sym_coord.0.is_multiple_of(2) {
            even
        } else {
            odd
        };

        CornerPermRawCoord(buffer[sym_coord.0 as usize / 2])
    }

    pub fn get_raw_from_combo(&self, combo_coord: CornerPermComboCoord) -> CornerPermRawCoord {
        CornerPerm::from_coord(self.get_rep_from_sym(combo_coord.sym_coord))
            .domino_conjugate(combo_coord.domino_conjugation.inverse())
            .into_coord()
    }

    pub fn get_combo_from_raw(&self, raw_coord: CornerPermRawCoord) -> CornerPermComboCoord {
        let buffer = as_u16_slice(&self.0);
        let (even, odd) = buffer.split_at(2768 >> 1);
        let corner_perm = CornerPerm::from_coord(raw_coord);
        let (rep_coord, sym) = DominoSymmetry::all_iter()
            .map(|sym| (corner_perm.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        // index within its parity half
        let pos_in_half = if raw_coord.0.is_multiple_of(2) {
            even.binary_search(&rep_coord.0).unwrap()
        } else {
            odd.binary_search(&rep_coord.0).unwrap()
        };

        // pack: (pos << 1) | parity
        let packed = ((pos_in_half as u16) << 1) | (raw_coord.0 & 1);

        CornerPermComboCoord {
            sym_coord: CornerPermSymCoord(packed),
            domino_conjugation: sym,
        }
    }

    /// includes the identity
    pub fn get_all_stabilizing_conjugations(
        &self,
        sym_coord: CornerPermSymCoord,
    ) -> impl IntoIterator<Item = DominoSymmetry> {
        let rep = self.get_rep_from_sym(sym_coord);
        let corner_perm = CornerPerm::from_coord(rep);
        DominoSymmetry::all_iter()
            .filter(move |sym| corner_perm.domino_conjugate(*sym) == corner_perm)
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
    use std::collections::HashMap;

    use itertools::Itertools;

    use crate::tables::Tables;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_corner_perm;

        (0..40320).into_iter().for_each(|i| {
            let raw_coord = CornerPermRawCoord(i);
            let corner_perm = CornerPerm::from_coord(raw_coord);

            let CornerPermComboCoord {
                sym_coord,
                domino_conjugation: sym,
            } = table.get_combo_from_raw(raw_coord);
            let updated_raw = corner_perm.domino_conjugate(sym).into_coord();
            let rep_coord = table.get_rep_from_sym(sym_coord);

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
            let raw_coord = table.get_rep_from_sym(sym_coord);
            let corner_perm = CornerPerm::from_coord(raw_coord);

            assert_eq!(corner_perm.0.is_odd(), i & 0b1 == 1);
        });

        Ok(())
    }

    #[test]
    fn test_stabilizing_conjugations() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_corner_perm;

        (0..2768).into_par_iter().for_each(|i| {
            let sym = CornerPermSymCoord(i);
            let rep = tables.lookup_sym_corner_perm.get_rep_from_sym(sym);
            let corner_perm = CornerPerm::from_coord(rep);
            let stabilizing_conjugations = table
                .get_all_stabilizing_conjugations(sym)
                .into_iter()
                .collect_vec();
            for s in DominoSymmetry::all_iter() {
                let stabilized = corner_perm == corner_perm.domino_conjugate(s);
                assert_eq!(stabilized, stabilizing_conjugations.contains(&s))
            }
        });

        Ok(())
    }

    #[test]
    fn check_for_stabilizing_conj() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        // 444 of the 2768 sym coords have nontrivial stabilizing symmetries
        // there are 34 possible cardinalities

        let nonzero_count: HashMap<_, _> = (0..2768)
            .into_par_iter()
            .map(|i| {
                let sym = CornerPermSymCoord(i);
                let rep = tables.lookup_sym_corner_perm.get_rep_from_sym(sym);
                let perm = CornerPerm::from_coord(rep);

                (
                    sym.0,
                    DominoSymmetry::all_iter().fold(0u16, |acc, sym| {
                        acc | ((perm == perm.domino_conjugate(sym)) as u16) << sym.0
                    }),
                )
            })
            .filter(|x| x.1 != 1)
            .collect();

        let mut reversed: HashMap<u16, Vec<u16>> = HashMap::new();

        for (k, v) in nonzero_count {
            reversed.entry(v).or_default().push(k);
        }

        let mut out_string =
            "static STABILIZING_CONJUGATIONS: phf::Map<u16, u16> = phf::phf_map! {\n".to_string();
        for (k, v) in reversed {
            out_string.push_str(&format!(
                "    {} => {},\n",
                v.into_iter().map(|x| format!("{x}")).join(" | "),
                k
            ));
        }
        out_string.push_str("};");

        println!("{out_string}");

        Ok(())
    }
}

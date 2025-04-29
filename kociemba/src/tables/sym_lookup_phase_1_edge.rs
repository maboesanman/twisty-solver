use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    coords::{EdgeGroupCoord, EdgeOrientCoord, Phase1EdgeSymCoord},
    symmetries::SubGroupTransform,
};

use super::{
    move_table_raw_edge_grouping::EdgeGroupingMoveTable,
    move_table_raw_edge_orient::EdgeOrientMoveTable,
    table_loader::{as_u16_2_slice, as_u16_slice, as_u16_slice_mut, load_table},
};

const PHASE_1_EDGE_SYM_LOOKUP_TABLE_SIZE_BYTES: usize = 64430 * 2 * 2;
const PHASE_1_EDGE_SYM_LOOKUP_TABLE_CHECKSUM: u32 = 416901822;

fn generate_phase_1_edge_sym_lookup_table(
    buffer: &mut [u8],
    edge_orient_move_table: &EdgeOrientMoveTable,
    edge_group_move_table: &EdgeGroupingMoveTable,
) {
    let mut reps: Vec<[u16; 2]> = (0..2048u16)
        .into_par_iter()
        .flat_map(|i| {
            (0..495u16).into_par_iter().map(move |j| {
                let edge_orient_syms = edge_orient_move_table.get_sym_array_ref(i.into());
                let edge_group_syms = edge_group_move_table.get_sym_array_ref(j.into());

                edge_orient_syms
                    .iter()
                    .zip(edge_group_syms)
                    .map(|(a, b)| [*a, *b])
                    .min()
                    .unwrap()
            })
        })
        .collect();

    // 2) sort + dedup to get the same ordering as BTreeSet
    reps.sort();
    println!("reps len {}", reps.len());

    reps.dedup();

    println!("reps len {}", reps.len());
    assert!(reps.len() == 64430);

    // 3) write into the u16‐view of the buffer
    let buf16 = as_u16_slice_mut(buffer);
    for (i, [orient, group]) in reps.into_iter().enumerate() {
        buf16[2 * i] = orient;
        buf16[2 * i + 1] = group;
    }

    let buf = as_u16_2_slice(buffer);
    assert!(buf.is_sorted())
}

pub fn load_phase_1_edge_sym_lookup_table<P: AsRef<Path>>(
    path: P,

    edge_orient_move_table: &EdgeOrientMoveTable,
    edge_group_move_table: &EdgeGroupingMoveTable,
) -> Result<Phase1EdgeSymLookupTable> {
    load_table(
        path,
        PHASE_1_EDGE_SYM_LOOKUP_TABLE_SIZE_BYTES,
        PHASE_1_EDGE_SYM_LOOKUP_TABLE_CHECKSUM,
        |buf| {
            generate_phase_1_edge_sym_lookup_table(
                buf,
                edge_orient_move_table,
                edge_group_move_table,
            )
        },
    )
    .map(Phase1EdgeSymLookupTable)
}

pub struct Phase1EdgeSymLookupTable(Mmap);

impl Phase1EdgeSymLookupTable {
    pub fn get_raw_from_sym(
        &self,
        sym_coord: Phase1EdgeSymCoord,
    ) -> (EdgeOrientCoord, EdgeGroupCoord) {
        let slice = as_u16_slice(&self.0);
        let i = sym_coord.inner() as usize * 2;
        println!("{:?}", &slice[i..i + 2]);
        (slice[i].into(), slice[i + 1].into())
    }

    pub fn get_sym_from_raw(
        &self,
        edge_orient: EdgeOrientCoord,
        edge_group: EdgeGroupCoord,
        edge_orient_move_table: &EdgeOrientMoveTable,
        edge_group_move_table: &EdgeGroupingMoveTable,
    ) -> (Phase1EdgeSymCoord, SubGroupTransform) {
        let edge_orient_syms = edge_orient_move_table.get_sym_array_ref(edge_orient);
        let edge_group_syms = edge_group_move_table.get_sym_array_ref(edge_group);

        let (transform, val) = edge_orient_syms
            .iter()
            .zip(edge_group_syms)
            .map(|(a, b)| [*a, *b])
            .enumerate()
            .min_by_key(|(_, x)| *x)
            .unwrap();
        let transform = SubGroupTransform(transform as u8);

        let slice = as_u16_2_slice(&self.0);
        (
            (slice.binary_search(&val).unwrap() as u16).into(),
            transform,
        )
    }
}

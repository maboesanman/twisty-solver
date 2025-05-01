use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    coords::{EdgeGroupCoord, EdgeOrientCoord, Phase1EdgeSymCoord},
    symmetries::SubGroupTransform,
};

use super::{
    move_table_raw_edge_group_and_orient::EdgeGroupAndOrientMoveTable,
    table_loader::{as_u16_2_slice, as_u16_slice_mut, load_table},
};

const PHASE_1_EDGE_SYM_LOOKUP_TABLE_SIZE_BYTES: usize = 64430 * 2 * 2;
const PHASE_1_EDGE_SYM_LOOKUP_TABLE_CHECKSUM: u32 = 1283221251;

fn generate_phase_1_edge_sym_lookup_table(
    buffer: &mut [u8],
    move_table: &EdgeGroupAndOrientMoveTable,
) {
    let mut reps: Vec<[u16; 2]> = (0..2048u16)
        .into_par_iter()
        .flat_map(|i| {
            (0..495u16).into_par_iter().map(move |j| {
                let (group, orient, _) = move_table.get_sym_representative(j.into(), i.into());

                [group.into(), orient.into()]
            })
        })
        .collect();

    // 2) sort + dedup to get the same ordering as BTreeSet
    reps.sort();
    reps.dedup();

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
    move_table: &EdgeGroupAndOrientMoveTable,
) -> Result<Phase1EdgeSymLookupTable> {
    load_table(
        path,
        PHASE_1_EDGE_SYM_LOOKUP_TABLE_SIZE_BYTES,
        PHASE_1_EDGE_SYM_LOOKUP_TABLE_CHECKSUM,
        |buf| generate_phase_1_edge_sym_lookup_table(buf, move_table),
    )
    .map(Phase1EdgeSymLookupTable)
}

pub struct Phase1EdgeSymLookupTable(Mmap);

impl Phase1EdgeSymLookupTable {
    pub fn get_raw_from_sym(
        &self,
        sym_coord: Phase1EdgeSymCoord,
    ) -> (EdgeGroupCoord, EdgeOrientCoord) {
        let slice = as_u16_2_slice(&self.0);
        let i = sym_coord.inner() as usize;
        let [group, orient] = slice[i];
        (group.into(), orient.into())
    }

    pub fn get_sym_from_raw(
        &self,
        move_table: &EdgeGroupAndOrientMoveTable,
        raw_group: EdgeGroupCoord,
        raw_orient: EdgeOrientCoord,
    ) -> (Phase1EdgeSymCoord, SubGroupTransform) {
        let (rep_group, rep_orient, transform) =
            move_table.get_sym_representative(raw_group, raw_orient);
        let slice = as_u16_2_slice(&self.0);
        let sym_coord = (slice
            .binary_search(&[rep_group.inner(), rep_orient.inner()])
            .unwrap() as u16)
            .into();

        (sym_coord, transform)
    }
}

#[test]
fn test() -> Result<()> {
    let move_table =
        super::move_table_raw_edge_group_and_orient::load_edge_group_and_orient_move_table(
            "edge_group_and_orient_move_table.dat",
        )?;
    let table =
        load_phase_1_edge_sym_lookup_table("phase_1_edge_sym_lookup_table.dat", &move_table)?;

    itertools::iproduct!(0..2048, 0..495)
        .par_bridge()
        .for_each(|(i, j)| {
            let orient = EdgeOrientCoord::from(i);
            let group = EdgeGroupCoord::from(j);
            let (sym_coord, transform) = table.get_sym_from_raw(&move_table, group, orient);
            let rep_coord = table.get_raw_from_sym(sym_coord);
            let recovered_raw_coord = move_table.conjugate_by_transform(group, orient, transform);
            assert_eq!(rep_coord, recovered_raw_coord);
        });

    Ok(())
}

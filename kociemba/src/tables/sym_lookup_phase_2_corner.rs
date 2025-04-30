use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    coords::{CornerPermCoord, Phase2CornerSymCoord},
    symmetries::SubGroupTransform,
};

use super::{
    move_table_raw_corner_perm::CornerPermMoveTable,
    table_loader::{as_u16_slice, as_u16_slice_mut, load_table},
};

const PHASE_2_CORNER_SYM_LOOKUP_TABLE_SIZE_BYTES: usize = 2768 * 2;
const PHASE_2_CORNER_SYM_LOOKUP_TABLE_CHECKSUM: u32 = 2748406986;

fn generate_phase_2_corner_sym_lookup_table(
    buffer: &mut [u8],
    corner_perm_move_table: &CornerPermMoveTable,
) {
    let mut reps: Vec<u16> = (0..40320u16)
        .into_par_iter()
        .map(move |i| {
            corner_perm_move_table
                .get_sym_representative(i.into())
                .0
                .inner()
        })
        .collect();

    // 2) sort + dedup to get the same ordering as BTreeSet
    reps.par_sort_unstable();
    reps.dedup();

    // 3) write into the u16‐view of the buffer
    let buf16 = as_u16_slice_mut(buffer);
    for (i, corners) in reps.into_iter().enumerate() {
        buf16[i] = corners;
    }
}

pub fn load_phase_2_corner_sym_lookup_table<P: AsRef<Path>>(
    path: P,
    corner_perm_move_table: &CornerPermMoveTable,
) -> Result<Phase2CornerSymLookupTable> {
    load_table(
        path,
        PHASE_2_CORNER_SYM_LOOKUP_TABLE_SIZE_BYTES,
        PHASE_2_CORNER_SYM_LOOKUP_TABLE_CHECKSUM,
        |buf| generate_phase_2_corner_sym_lookup_table(buf, corner_perm_move_table),
    )
    .map(Phase2CornerSymLookupTable)
}

pub struct Phase2CornerSymLookupTable(Mmap);

impl Phase2CornerSymLookupTable {
    pub fn get_raw_from_sym(&self, sym_coord: Phase2CornerSymCoord) -> CornerPermCoord {
        let slice = as_u16_slice(&self.0);
        let i = sym_coord.inner() as usize;
        slice[i].into()
    }

    pub fn get_sym_from_raw(
        &self,
        corner_perm_move_table: &CornerPermMoveTable,
        raw_coord: CornerPermCoord,
    ) -> (Phase2CornerSymCoord, SubGroupTransform) {
        let (c, transform) = corner_perm_move_table.get_sym_representative(raw_coord);
        let slice = as_u16_slice(&self.0);
        let sym_coord = (slice.binary_search(&c.inner()).unwrap() as u16).into();

        (sym_coord, transform)
    }
}

#[test]
fn test() -> Result<()> {
    let corner_table = super::move_table_raw_corner_perm::load_corner_perm_move_table(
        "corner_perm_move_table.dat",
    )?;
    let table =
        load_phase_2_corner_sym_lookup_table("phase_2_corner_sym_lookup_table.dat", &corner_table)?;

    (0..40320).par_bridge().for_each(|i| {
        let raw_coord = CornerPermCoord::from(i);
        let (sym_coord, transform) = table.get_sym_from_raw(&corner_table, raw_coord);
        let rep_coord = table.get_raw_from_sym(sym_coord);
        let recovered_raw_coord = corner_table.conjugate_by_transform(raw_coord, transform);
        assert_eq!(rep_coord, recovered_raw_coord);
    });

    Ok(())
}

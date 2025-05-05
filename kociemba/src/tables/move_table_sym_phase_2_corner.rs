use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::Phase1EdgeSymCoord, moves::Phase2Move, symmetries::SubGroupTransform, tables::table_loader::as_u16_slice_mut
};

use super::{
    move_table_raw_corner_perm::CornerPermMoveTable, sym_lookup_phase_2_corner::Phase2CornerSymLookupTable, table_loader::{as_u16_slice, load_table}
};

const PHASE_2_CORNER_MOVE_TABLE_SIZE_BYTES: usize = (2768 * 10) * 2 * 2;
const PHASE_2_CORNER_MOVE_TABLE_CHECKSUM: u32 = 4143049956;

fn generate_phase_2_corner_sym_move_table(
    buffer: &mut [u8],
    sym_lookup_table: &Phase2CornerSymLookupTable,
    move_table: &CornerPermMoveTable,
) {
    assert_eq!(buffer.len(), PHASE_2_CORNER_MOVE_TABLE_SIZE_BYTES);
    let buffer = as_u16_slice_mut(buffer);

    buffer
        .chunks_mut(10 * 2)
        .enumerate()
        .for_each(|(sym_coord, row)| {
            let raw_coord = sym_lookup_table.get_raw_from_sym((sym_coord as u16).into());
            for (j, mv) in Phase2Move::all_iter().enumerate() {
                let new_raw_coord = move_table.apply_move(raw_coord, mv);
                let (sym_coord, transform) =
                    sym_lookup_table.get_sym_from_raw(move_table, new_raw_coord);

                row[2 * j] = sym_coord.into();
                row[2 * j + 1] = transform.0 as u16;
            }
        });
}

pub fn load_phase_2_corner_sym_move_table<P: AsRef<Path>>(
    path: P,
    sym_lookup_table: &Phase2CornerSymLookupTable,
    move_table: &CornerPermMoveTable,
) -> Result<Phase2CornerSymMoveTable> {
    load_table(
        path,
        PHASE_2_CORNER_MOVE_TABLE_SIZE_BYTES,
        PHASE_2_CORNER_MOVE_TABLE_CHECKSUM,
        |buf| generate_phase_2_corner_sym_move_table(buf, sym_lookup_table, move_table),
    )
    .map(Phase2CornerSymMoveTable)
}

pub struct Phase2CornerSymMoveTable(Mmap);

impl Phase2CornerSymMoveTable {
    pub fn apply_move(
        &self,
        coord: Phase1EdgeSymCoord,
        mv: Phase2Move,
    ) -> (Phase1EdgeSymCoord, SubGroupTransform) {
        let slice = as_u16_slice(&self.0);
        let i = coord.inner() as usize * 10 * 2 + mv as u8 as usize * 2;
        (slice[i].into(), SubGroupTransform(slice[i + 1] as u8))
    }
}

#[test]
fn test_inversion() -> anyhow::Result<()> {
    use rayon::prelude::*;

    let phase_2_move_corner_raw_table = crate::tables::move_table_raw_corner_perm::load_corner_perm_move_table("corner_perm_move_table.dat")?;
    let phase_2_lookup_corner_sym_table = crate::tables::sym_lookup_phase_2_corner::load_phase_2_corner_sym_lookup_table(
        "phase_2_corner_sym_lookup_table.dat",
        &phase_2_move_corner_raw_table,
    )?;
    let phase_2_move_corner_sym_table = load_phase_2_corner_sym_move_table(
        "phase_2_corner_sym_move_table.dat",
        &phase_2_lookup_corner_sym_table,
        &phase_2_move_corner_raw_table,
    )?;
    (0..2768u16).into_par_iter().for_each(|i| {
        let coord = Phase1EdgeSymCoord::from(i);

        for mv in Phase2Move::all_iter() {
            let move_cube = crate::repr_cubie::ReprCube::from(mv);
            let (next, transform1) = phase_2_move_corner_sym_table.apply_move(coord, mv);
            let inv_move_cube = Phase2Move::try_from(move_cube.conjugate_by_subgroup_transform(transform1).inverse()).unwrap();
            let (recovered,transform2) = phase_2_move_corner_sym_table.apply_move(next, inv_move_cube);

            assert_eq!(coord, recovered);
            assert_eq!(crate::repr_cubie::SOLVED_CUBE, crate::repr_cubie::SOLVED_CUBE.conjugate_by_subgroup_transform(transform1).conjugate_by_subgroup_transform(transform2));
        }
    });

    Ok(())
}

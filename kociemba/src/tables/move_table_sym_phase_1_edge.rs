use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    coords::Phase1EdgeSymCoord, moves::Move, symmetries::SubGroupTransform, tables::table_loader::as_u16_slice_mut
};

use super::{
    move_table_raw_edge_group_and_orient::EdgeGroupAndOrientMoveTable, sym_lookup_phase_1_edge::Phase1EdgeSymLookupTable, table_loader::{as_u16_slice, load_table}
};

const PHASE_1_EDGE_MOVE_TABLE_SIZE_BYTES: usize = (64430 * 18) * 2 * 2;
const PHASE_1_EDGE_MOVE_TABLE_CHECKSUM: u32 = 3661454509;

fn generate_phase_1_edge_sym_move_table(
    buffer: &mut [u8],
    sym_lookup_table: &Phase1EdgeSymLookupTable,
    move_table: &EdgeGroupAndOrientMoveTable,
) {
    assert_eq!(buffer.len(), PHASE_1_EDGE_MOVE_TABLE_SIZE_BYTES);
    let buffer = as_u16_slice_mut(buffer);

    buffer
        .par_chunks_mut(18 * 2)
        .enumerate()
        .for_each(|(sym_coord, row)| {
            let (edge_group_coord, edge_orient_coord) =
                sym_lookup_table.get_raw_from_sym((sym_coord as u16).into());
            for mv in Move::all_iter() {
                let (new_edge_group_coord, new_edge_orient_coord) =
                    move_table.apply_move(edge_group_coord, edge_orient_coord, mv);
                let (sym_coord, transform) = sym_lookup_table.get_sym_from_raw(
                    move_table,
                    new_edge_group_coord,
                    new_edge_orient_coord,
                );

                row[2 * (mv as u8 as usize)] = sym_coord.into();
                row[2 * (mv as u8 as usize) + 1] = transform.0 as u16;
            }
        });
}

pub fn load_phase_1_edge_sym_move_table<P: AsRef<Path>>(
    path: P,
    sym_lookup_table: &Phase1EdgeSymLookupTable,
    move_table: &EdgeGroupAndOrientMoveTable,
) -> Result<Phase1EdgeSymMoveTable> {
    load_table(
        path,
        PHASE_1_EDGE_MOVE_TABLE_SIZE_BYTES,
        PHASE_1_EDGE_MOVE_TABLE_CHECKSUM,
        |buf| generate_phase_1_edge_sym_move_table(buf, sym_lookup_table, move_table),
    )
    .map(Phase1EdgeSymMoveTable)
}

pub struct Phase1EdgeSymMoveTable(Mmap);

impl Phase1EdgeSymMoveTable {
    pub fn apply_move(
        &self,
        coord: Phase1EdgeSymCoord,
        mv: Move,
    ) -> (Phase1EdgeSymCoord, SubGroupTransform) {
        let slice = as_u16_slice(&self.0);
        let i = coord.inner() as usize * 18 * 2 + mv as u8 as usize * 2;
        (slice[i].into(), SubGroupTransform(slice[i + 1] as u8))
    }
}

#[test]
fn test_inversion() -> anyhow::Result<()> {
    use rayon::prelude::*;
    let phase_1_move_edge_raw_table =
        crate::tables::move_table_raw_edge_group_and_orient::load_edge_group_and_orient_move_table("edge_group_and_orient_move_table.dat")?;
    let phase_1_lookup_edge_sym_table = crate::tables::sym_lookup_phase_1_edge::load_phase_1_edge_sym_lookup_table(
        "phase_1_edge_sym_lookup_table.dat",
        &phase_1_move_edge_raw_table,
    )?;
    let phase_1_move_edge_sym_table = load_phase_1_edge_sym_move_table(
        "phase_1_edge_sym_move_table.dat",
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
    )?;
    (0..64430u16).into_par_iter().for_each(|i| {
        let coord = Phase1EdgeSymCoord::from(i);

        for mv in Move::all_iter() {
            let move_cube = crate::repr_cubie::ReprCube::from(mv);
            let (next, transform1) = phase_1_move_edge_sym_table.apply_move(coord, mv);
            let inv_move_cube = Move::try_from(move_cube.conjugate_by_subgroup_transform(transform1).inverse()).unwrap();
            let (recovered,transform2) = phase_1_move_edge_sym_table.apply_move(next, inv_move_cube);

            assert_eq!(coord, recovered);
            assert_eq!(crate::repr_cubie::SOLVED_CUBE, crate::repr_cubie::SOLVED_CUBE.conjugate_by_subgroup_transform(transform1).conjugate_by_subgroup_transform(transform2));
        }
    });

    Ok(())
}
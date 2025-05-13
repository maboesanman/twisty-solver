use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::SymCornerPermCoord, moves::{Move}, symmetries::SubGroupTransform,
    tables::table_loader::as_u16_slice_mut,
};

use super::{
    lookup_sym_corner_perm::LookupSymCornerPermTable,
    move_raw_corner_perm::MoveRawCornerPermTable,
    table_loader::{as_u16_slice, load_table},
};

const TABLE_SIZE_BYTES: usize = (2768 * 18) * 2 * 2;
const FILE_CHECKSUM: u32 = 110890093;

pub struct MoveSymCornerPermTable(Mmap);

impl MoveSymCornerPermTable {
    pub fn apply_move(
        &self,
        coord: SymCornerPermCoord,
        mv: Move,
    ) -> (SymCornerPermCoord, SubGroupTransform) {
        let slice = as_u16_slice(&self.0);
        let i = coord.inner() as usize * 18 * 2 + mv as u8 as usize * 2;
        (slice[i].into(), SubGroupTransform(slice[i + 1] as u8))
    }

    fn generate(
        buffer: &mut [u8],
        sym_lookup_table: &LookupSymCornerPermTable,
        move_table: &MoveRawCornerPermTable,
    ) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u16_slice_mut(buffer);

        buffer
            .chunks_mut(18 * 2)
            .enumerate()
            .for_each(|(sym_coord, row)| {
                let raw_coord = sym_lookup_table.get_raw_from_sym((sym_coord as u16).into());
                for (j, mv) in Move::all_iter().enumerate() {
                    let new_raw_coord = move_table.apply_move(raw_coord, mv);
                    let (sym_coord, transform) =
                        sym_lookup_table.get_sym_from_raw(move_table, new_raw_coord);

                    row[2 * j] = sym_coord.into();
                    row[2 * j + 1] = transform.0 as u16;
                }
            });
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        sym_lookup_table: &LookupSymCornerPermTable,
        move_table: &MoveRawCornerPermTable,
    ) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table, move_table)
        })
        .map(Self)
    }
}

// #[test]
// fn test_inversion() -> anyhow::Result<()> {
//     use rayon::prelude::*;

//     let phase_2_move_corner_raw_table = crate::tables::move_raw_corner_perm::load("corner_perm_move_table.dat")?;
//     let phase_2_lookup_corner_sym_table = crate::tables::lookup_sym_corner_perm::load(
//         "phase_2_corner_sym_lookup_table.dat",
//         &phase_2_move_corner_raw_table,
//     )?;
//     let phase_2_move_corner_sym_table = load(
//         "phase_2_corner_sym_move_table.dat",
//         &phase_2_lookup_corner_sym_table,
//         &phase_2_move_corner_raw_table,
//     )?;
//     (0..2768u16).into_par_iter().for_each(|i| {
//         let coord = SymEdgeGroupFlipCoord::from(i);

//         for mv in Phase2Move::all_iter() {
//             let move_cube = crate::repr_cubie::ReprCube::from(mv);
//             let (next, transform1) = phase_2_move_corner_sym_table.apply_move(coord, mv);
//             let inv_move_cube = Phase2Move::try_from(move_cube.conjugate_by_subgroup_transform(transform1).inverse()).unwrap();
//             let (recovered,transform2) = phase_2_move_corner_sym_table.apply_move(next, inv_move_cube);

//             assert_eq!(coord, recovered);
//             assert_eq!(crate::repr_cubie::SOLVED_CUBE, crate::repr_cubie::SOLVED_CUBE.conjugate_by_subgroup_transform(transform1).conjugate_by_subgroup_transform(transform2));
//         }
//     });

//     Ok(())
// }

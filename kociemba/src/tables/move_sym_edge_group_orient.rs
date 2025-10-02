use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        coords::EdgeGroupOrientSymCoord, cube_move::CubeMove, cube_sym::DominoSymmetry,
        partial_reprs::edge_group_orient::EdgeGroupOrient,
    },
    tables::table_loader::{as_u16_slice, as_u16_slice_mut},
};

use super::{
    lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, table_loader::load_table,
};

const TABLE_SIZE_BYTES: usize = (64430 * 18) * 2 * 2;
const FILE_CHECKSUM: u32 = 3661454509;

pub struct MoveSymEdgeGroupOrientTable(Mmap);

impl MoveSymEdgeGroupOrientTable {
    fn chunks(&self) -> &[[u16; 36]] {
        let buffer = as_u16_slice(&self.0);
        unsafe { buffer.as_chunks_unchecked() }
    }

    fn chunk(&self, coord: EdgeGroupOrientSymCoord) -> &[u16; 36] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: EdgeGroupOrientSymCoord,
        mv: CubeMove,
    ) -> (EdgeGroupOrientSymCoord, DominoSymmetry) {
        (
            EdgeGroupOrientSymCoord(self.chunk(coord)[mv.into_index() * 2]),
            DominoSymmetry(self.chunk(coord)[mv.into_index() * 2 + 1] as u8),
        )
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymEdgeGroupOrientTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u16_slice_mut(buffer);

        buffer
            .par_chunks_mut(36)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = EdgeGroupOrientSymCoord(i as u16);
                let rep = sym_lookup_table.get_raw_from_sym(sym_coord);
                let group_orient = EdgeGroupOrient::from_coord(rep);

                CubeMove::all_iter()
                    .zip(store.as_chunks_mut::<2>().0)
                    .for_each(|(mv, slot)| {
                        let new_rep = group_orient.apply_cube_move(mv).into_coord();
                        let (sym_coord, sym_correct) = sym_lookup_table.get_sym_from_raw(new_rep);
                        *slot = [sym_coord.0, sym_correct.0 as u16];
                    });
            })
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        sym_lookup_table: &LookupSymEdgeGroupOrientTable,
    ) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table)
        })
        .map(Self)
    }
}

#[test]
fn test() -> anyhow::Result<()> {
    let phase_1_lookup_edge_sym_table =
        LookupSymEdgeGroupOrientTable::load("edge_group_orient_sym_lookup_table.dat")?;
    let phase_1_move_edge_sym_table = MoveSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_move_table.dat",
        &phase_1_lookup_edge_sym_table,
    )?;

    Ok(())
}

// #[test]
// fn test_inversion() -> anyhow::Result<()> {
//     use rayon::prelude::*;
//     let phase_1_move_edge_raw_table =
//         crate::tables::move_raw_edge_group_flip::load("edge_group_and_orient_move_table.dat")?;
//     let phase_1_lookup_edge_sym_table = crate::tables::lookup_sym_edge_group_flip::load(
//         "phase_1_edge_sym_lookup_table.dat",
//         &phase_1_move_edge_raw_table,
//     )?;
//     let phase_1_move_edge_sym_table = load(
//         "phase_1_edge_sym_move_table.dat",
//         &phase_1_lookup_edge_sym_table,
//         &phase_1_move_edge_raw_table,
//     )?;
//     (0..64430u16).into_par_iter().for_each(|i| {
//         let coord = SymEdgeGroupFlipCoord::from(i);

//         for mv in Move::all_iter() {
//             let move_cube = crate::repr_cubie::ReprCube::from(mv);
//             let (next, transform1) = phase_1_move_edge_sym_table.apply_move(coord, mv);
//             let inv_move_cube = Move::try_from(move_cube.conjugate_by_subgroup_transform(transform1).inverse()).unwrap();
//             let (recovered,transform2) = phase_1_move_edge_sym_table.apply_move(next, inv_move_cube);

//             assert_eq!(coord, recovered);
//             assert_eq!(crate::repr_cubie::SOLVED_CUBE, crate::repr_cubie::SOLVED_CUBE.conjugate_by_subgroup_transform(transform1).conjugate_by_subgroup_transform(transform2));
//         }
//     });

//     Ok(())
// }

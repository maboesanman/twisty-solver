use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        coords::EdgeGroupOrientSymCoord, cube_move::CubeMove, cube_sym::DominoSymmetry,
        partial_reprs::edge_group_orient::EdgeGroupOrient,
    },
    tables::table_loader::as_u32_slice_mut,
};

use super::{
    lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable,
    table_loader::{as_u32_slice, load_table},
};

const TABLE_SIZE_BYTES: usize = (64430 * 18) * 2 * 2;
const FILE_CHECKSUM: u32 = 3661454509;

pub struct MoveSymEdgeGroupOrientTable(Mmap);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Entry {
    pub sym_coord: EdgeGroupOrientSymCoord,
    pub sym_correct: DominoSymmetry,
}

impl MoveSymEdgeGroupOrientTable {
    fn chunks(&self) -> &[[Entry; 18]] {
        let buffer = as_u32_slice(&self.0);
        unsafe {
            let slice: &[[u32; 18]] = buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const [Entry; 18], slice.len())
        }
    }

    fn chunk(&self, coord: EdgeGroupOrientSymCoord) -> &[Entry; 18] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: EdgeGroupOrientSymCoord,
        mv: CubeMove,
    ) -> (EdgeGroupOrientSymCoord, DominoSymmetry) {
        let entry = &self.chunk(coord)[mv.into_index()];
        (entry.sym_coord, entry.sym_correct)
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymEdgeGroupOrientTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u32_slice_mut(buffer);

        buffer
            .par_chunks_mut(18)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = EdgeGroupOrientSymCoord(i as u16);
                let rep = sym_lookup_table.get_raw_from_sym(sym_coord);
                let group_orient = EdgeGroupOrient::from_coord(rep);

                CubeMove::all_iter()
                    .zip(store.iter_mut())
                    .for_each(|(mv, slot)| {
                        let new_rep = group_orient.apply_cube_move(mv).into_coord();
                        let (sym_coord, sym_correct) = sym_lookup_table.get_sym_from_raw(new_rep);
                        *slot = unsafe {
                            core::mem::transmute(Entry {
                                sym_coord,
                                sym_correct,
                            })
                        };
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
    let phase_1_lookup_edge_sym_table = LookupSymEdgeGroupOrientTable::load(
            "edge_group_orient_sym_lookup_table.dat",
        )?;
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

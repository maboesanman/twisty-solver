use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        coords::CornerPermSymCoord, cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_perm::CornerPerm,
    },
    tables::{lookup_sym_corner_perm::LookupSymCornerPermTable, table_loader::as_u32_slice_mut},
};

use super::{
    table_loader::{as_u32_slice, load_table},
};

const TABLE_SIZE_BYTES: usize = (2768 * 18) * 2 * 2;
const FILE_CHECKSUM: u32 = 3661454509;

pub struct MoveSymCornerPermTable(Mmap);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Entry {
    pub sym_coord: CornerPermSymCoord,
    pub sym_correct: DominoSymmetry,
}

impl MoveSymCornerPermTable {
    fn chunks(&self) -> &[[Entry; 18]] {
        let buffer = as_u32_slice(&self.0);
        unsafe {
            let slice: &[[u32; 18]] = buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const [Entry; 18], slice.len())
        }
    }

    fn chunk(&self, coord: CornerPermSymCoord) -> &[Entry; 18] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: CornerPermSymCoord,
        mv: CubeMove,
    ) -> (CornerPermSymCoord, DominoSymmetry) {
        let entry = &self.chunk(coord)[mv.into_index()];
        (entry.sym_coord, entry.sym_correct)
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymCornerPermTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u32_slice_mut(buffer);

        buffer
            .par_chunks_mut(18)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = CornerPermSymCoord(i as u16);
                let rep = sym_lookup_table.get_raw_from_sym(sym_coord);
                let group_orient = CornerPerm::from_coord(rep);

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
        sym_lookup_table: &LookupSymCornerPermTable,
    ) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table)
        })
        .map(Self)
    }
}
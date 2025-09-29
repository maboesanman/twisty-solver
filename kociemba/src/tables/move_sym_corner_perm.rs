use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        coords::CornerPermSymCoord, cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_perm::CornerPerm,
    },
    tables::{lookup_sym_corner_perm::LookupSymCornerPermTable, table_loader::{as_u16_slice, as_u16_slice_mut, as_u32_slice_mut}},
};

use super::{
    table_loader::{as_u32_slice, load_table},
};

const TABLE_SIZE_BYTES: usize = (2768 * 18) * 2 * 2;
const FILE_CHECKSUM: u32 = 110890093;

pub struct MoveSymCornerPermTable(Mmap);

impl MoveSymCornerPermTable {
    fn chunks(&self) -> &[[u16; 36]] {
        let buffer = as_u16_slice(&self.0);
        unsafe {
            buffer.as_chunks_unchecked()
        }
    }

    fn chunk(&self, coord: CornerPermSymCoord) -> &[u16; 36] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: CornerPermSymCoord,
        mv: CubeMove,
    ) -> (CornerPermSymCoord, DominoSymmetry) {
        (
            CornerPermSymCoord(self.chunk(coord)[mv.into_index() * 2]),
            DominoSymmetry(self.chunk(coord)[mv.into_index() * 2 + 1] as u8),
        )
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymCornerPermTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u16_slice_mut(buffer);

        buffer
            .par_chunks_mut(36)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = CornerPermSymCoord(i as u16);
                let rep = sym_lookup_table.get_raw_from_sym(sym_coord);
                let group_orient = CornerPerm::from_coord(rep);

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
        sym_lookup_table: &LookupSymCornerPermTable,
    ) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table)
        })
        .map(Self)
    }
}
use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_perm::CornerPerm,
    },
    kociemba::coords::{CornerPermSymCoord, corner_perm_combo_coord::CornerPermComboCoord},
    kociemba::tables::{
        lookup_sym_corner_perm::LookupSymCornerPermTable, table_loader::as_u16_slice_mut,
    },
};

use super::table_loader::load_table;

const TABLE_SIZE_BYTES: usize = 2768 * core::mem::size_of::<Row>();
const FILE_CHECKSUM: u32 = 1209655720;

#[repr(C)]
#[repr(align(64))]
struct Row([u16; 18]);

pub struct MoveSymCornerPermTable([u8]);

impl MoveSymCornerPermTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr() as *const u16
    }

    fn chunks(&self) -> &[Row] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<Row>()]] = self.0.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const Row, slice.len())
        }
    }

    fn chunk(&self, coord: CornerPermSymCoord) -> &[u16; 18] {
        &self.chunks()[coord.0 as usize].0
    }

    pub fn apply_cube_move(&self, coord: CornerPermSymCoord, mv: CubeMove) -> CornerPermComboCoord {
        CornerPermComboCoord::from_dense(self.chunk(coord)[mv.into_index()])
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymCornerPermTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u16_slice_mut(buffer);

        buffer
            .par_chunks_mut(32)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = CornerPermSymCoord(i as u16);
                let combo = CornerPermComboCoord {
                    sym_coord,
                    domino_conjugation: DominoSymmetry::IDENTITY,
                };
                let raw = sym_lookup_table.get_raw_from_combo(combo);
                let corner_perm = CornerPerm::from_coord(raw);

                CubeMove::all_iter().zip(store).for_each(|(mv, slot)| {
                    let new_raw = corner_perm.apply_cube_move(mv).into_coord();
                    let new_combo = sym_lookup_table.get_combo_from_raw(new_raw);
                    *slot = new_combo.sym_coord.0 | ((new_combo.domino_conjugation.0 as u16) << 12);
                });
            })
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        sym_lookup_table: &LookupSymCornerPermTable,
    ) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table)
        })
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    CornerOrient, cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry}, kociemba::tables::lookup_sym_corner_orient::{CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable}
};

use super::{
    lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, table_loader::load_table,
};

pub(crate) const TABLE_SIZE_BYTES: usize =
    168 * core::mem::size_of::<MoveSymCornerOrientRow>();
const FILE_CHECKSUM: u32 = 1176224547;

#[repr(C)]
#[repr(align(64))]
pub struct MoveSymCornerOrientRow {
    pub coords: [u8; 18],
    pub padding: [u8; 18],
    pub conjugations: [u8; 18],
}

impl MoveSymCornerOrientRow {
    pub fn as_u16_array(&self) -> &[u16; 32] {
        unsafe { core::mem::transmute(self) }
    }

    pub fn as_u8_arrays(&self) -> &[[u8; 32]; 2] {
        unsafe { core::mem::transmute::<_, &[[u8; 32]; 2]>(self) }
    }
}

pub struct MoveSymCornerOrientTable {
    buffer: [u8],
}

impl MoveSymCornerOrientTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.buffer.as_ptr() as *const u16
    }

    fn rows(&self) -> &[MoveSymCornerOrientRow] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<MoveSymCornerOrientRow>()]] =
                self.buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const MoveSymCornerOrientRow,
                slice.len(),
            )
        }
    }

    pub fn row(&self, coord: CornerOrientSymCoord) -> &MoveSymCornerOrientRow {
        unsafe { self.rows().get_unchecked(coord.0 as usize) }
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymCornerOrientTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = unsafe {
            let slice: &mut [[u8; core::mem::size_of::<MoveSymCornerOrientRow>()]] =
                buffer.as_chunks_unchecked_mut();
            core::slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut MoveSymCornerOrientRow,
                slice.len(),
            )
        };

        buffer.par_iter_mut().enumerate().for_each(|(i, row)| {
            let sym_coord = CornerOrientSymCoord(i as u8);
            let combo = CornerOrientComboCoord {
                sym_coord,
                domino_conjugation: DominoSymmetry::IDENTITY,
            };
            let raw = sym_lookup_table.get_raw_from_combo(combo);
            let group_orient = CornerOrient::from_coord(raw);

            for (i, mv) in CubeMove::all_iter().enumerate() {
                let new_raw = group_orient.apply_cube_move(mv).into_coord();
                let new_combo = sym_lookup_table.get_combo_from_raw(new_raw);
                row.coords[i] = new_combo.sym_coord.0;
                row.conjugations[i] = new_combo.domino_conjugation.0;
            }
        })
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        sym_lookup_table: &LookupSymCornerOrientTable,
    ) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table)
        })
    }

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

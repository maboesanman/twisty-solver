use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_perm::CornerPerm,
    },
    kociemba::coords::{CornerPermSymCoord, corner_perm_combo_coord::CornerPermComboCoord},
    kociemba::tables::lookup_sym_corner_perm::LookupSymCornerPermTable,
};

use super::table_loader::load_table;

pub(crate) const TABLE_SIZE_BYTES: usize = 2768 * core::mem::size_of::<MoveSymCornerPermRow>();
const FILE_CHECKSUM: u32 = 3645794727;

#[repr(C)]
#[repr(align(64))]
pub struct MoveSymCornerPermRow {
    pub coords: [u16; 18],
    pub conjugations: [u8; 18],
}

pub struct MoveSymCornerPermTable([u8]);

impl MoveSymCornerPermTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr() as *const u16
    }

    pub(crate) fn rows(&self) -> &[MoveSymCornerPermRow] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<MoveSymCornerPermRow>()]] =
                self.0.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const MoveSymCornerPermRow, slice.len())
        }
    }

    pub fn row(&self, coord: CornerPermSymCoord) -> &MoveSymCornerPermRow {
        &self.rows()[coord.0 as usize]
    }

    pub fn apply_cube_move(&self, coord: CornerPermSymCoord, mv: CubeMove) -> CornerPermComboCoord {
        let row = self.row(coord);
        CornerPermComboCoord {
            sym_coord: CornerPermSymCoord(row.coords[mv.into_index()]),
            domino_conjugation: DominoSymmetry(row.conjugations[mv.into_index()]),
        }
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymCornerPermTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = unsafe {
            let slice: &mut [[u8; core::mem::size_of::<MoveSymCornerPermRow>()]] =
                buffer.as_chunks_unchecked_mut();
            core::slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut MoveSymCornerPermRow,
                slice.len(),
            )
        };

        buffer.par_iter_mut().enumerate().for_each(|(i, row)| {
            let sym_coord = CornerPermSymCoord(i as u16);
            let combo = CornerPermComboCoord {
                sym_coord,
                domino_conjugation: DominoSymmetry::IDENTITY,
            };
            let raw = sym_lookup_table.get_raw_from_combo(combo);
            let corner_perm = CornerPerm::from_coord(raw);

            for (i, mv) in CubeMove::all_iter().enumerate() {
                let new_raw = corner_perm.apply_cube_move(mv).into_coord();
                let new_combo = sym_lookup_table.get_combo_from_raw(new_raw);
                row.coords[i] = new_combo.sym_coord.0;
                row.conjugations[i] = new_combo.domino_conjugation.0;
            }
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

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

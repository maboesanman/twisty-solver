use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            EdgeGroupOrientSymCoord, edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::edge_group_orient::EdgeGroupOrient,
    },
};

use super::{
    lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, table_loader::load_table,
};

pub(crate) const TABLE_SIZE_BYTES: usize =
    64430 * core::mem::size_of::<MoveSymEdgeGroupOrientRow>();
const FILE_CHECKSUM: u32 = 3694283469;

#[repr(C)]
#[repr(align(64))]
pub struct MoveSymEdgeGroupOrientRow {
    pub coords: [u16; 18],
    pub conjugations: [u8; 18],
}

pub struct MoveSymEdgeGroupOrientTable {
    buffer: [u8],
}

impl MoveSymEdgeGroupOrientTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.buffer.as_ptr() as *const u16
    }

    fn rows(&self) -> &[MoveSymEdgeGroupOrientRow] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<MoveSymEdgeGroupOrientRow>()]] =
                self.buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const MoveSymEdgeGroupOrientRow,
                slice.len(),
            )
        }
    }

    pub fn row(&self, coord: EdgeGroupOrientSymCoord) -> &MoveSymEdgeGroupOrientRow {
        &self.rows()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: EdgeGroupOrientSymCoord,
        mv: CubeMove,
    ) -> EdgeGroupOrientComboCoord {
        let row = self.row(coord);
        EdgeGroupOrientComboCoord {
            sym_coord: EdgeGroupOrientSymCoord(row.coords[mv.into_index()]),
            domino_conjugation: DominoSymmetry(row.conjugations[mv.into_index()]),
        }
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymEdgeGroupOrientTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = unsafe {
            let slice: &mut [[u8; core::mem::size_of::<MoveSymEdgeGroupOrientRow>()]] =
                buffer.as_chunks_unchecked_mut();
            core::slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut MoveSymEdgeGroupOrientRow,
                slice.len(),
            )
        };

        buffer.par_iter_mut().enumerate().for_each(|(i, row)| {
            let sym_coord = EdgeGroupOrientSymCoord(i as u16);
            let combo = EdgeGroupOrientComboCoord {
                sym_coord,
                domino_conjugation: DominoSymmetry::IDENTITY,
            };
            let raw = sym_lookup_table.get_raw_from_combo(combo);
            let group_orient = EdgeGroupOrient::from_coord(raw);

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
        sym_lookup_table: &LookupSymEdgeGroupOrientTable,
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

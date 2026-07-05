use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            CornerPermSymCoord, UDEdgePermRawCoord, corner_perm_combo_coord::CornerPermComboCoord,
        },
        tables::{move_sym_corner_perm::MoveSymCornerPermTable, prune_phase_2::PrunePhase2Table},
    },
};

use super::table_loader::load_table;

pub(crate) const TABLE_SIZE_BYTES: usize =
    2768 * core::mem::size_of::<MoveSymCornerPermAugmentedRow>();
const FILE_CHECKSUM: u32 = 3522146883;

#[repr(C)]
#[repr(align(64))]
pub struct MoveSymCornerPermAugmentedRow {
    pub coords: [u16; 18],
    pub conjugations: [u8; 18],
}

pub struct MoveSymCornerPermAugmentedTable([u8]);

impl MoveSymCornerPermAugmentedTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr() as *const u16
    }

    fn rows(&self) -> &[MoveSymCornerPermAugmentedRow] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<MoveSymCornerPermAugmentedRow>()]] =
                self.0.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const MoveSymCornerPermAugmentedRow,
                slice.len(),
            )
        }
    }

    pub fn row(&self, coord: CornerPermSymCoord) -> &MoveSymCornerPermAugmentedRow {
        &self.rows()[(coord.0 & 0x0FFF) as usize]
    }

    pub fn apply_cube_move(&self, coord: CornerPermSymCoord, mv: CubeMove) -> CornerPermComboCoord {
        let row = self.row(coord);
        CornerPermComboCoord {
            sym_coord: CornerPermSymCoord(row.coords[mv.into_index()]),
            domino_conjugation: DominoSymmetry(row.conjugations[mv.into_index()]),
        }
    }

    fn generate(
        buffer: &mut [u8],
        move_sym_table: &MoveSymCornerPermTable,
        prune_phase_2: &PrunePhase2Table,
    ) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = unsafe {
            let slice: &mut [[u8; core::mem::size_of::<MoveSymCornerPermAugmentedRow>()]] =
                buffer.as_chunks_unchecked_mut();
            core::slice::from_raw_parts_mut(
                slice.as_mut_ptr() as *mut MoveSymCornerPermAugmentedRow,
                slice.len(),
            )
        };

        let coords: Vec<_> = (0..2768)
            .into_iter()
            .map(|i| {
                let x = (0u16..40320)
                    .map(|j| prune_phase_2.get_value(CornerPermSymCoord(i), UDEdgePermRawCoord(j)))
                    .min()
                    .unwrap();

                x as u16
            })
            .collect();

        buffer
            .iter_mut()
            .zip(move_sym_table.rows())
            .for_each(|(aug, row)| {
                *aug = MoveSymCornerPermAugmentedRow {
                    coords: row.coords,
                    conjugations: row.conjugations,
                };

                for coord in aug.coords.iter_mut() {
                    *coord = (coords[*coord as usize] << 12) | *coord;
                }
            })
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        move_sym_table: &MoveSymCornerPermTable,
        prune_phase_2: &PrunePhase2Table,
    ) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, move_sym_table, prune_phase_2)
        })
    }

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

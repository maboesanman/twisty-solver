use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            coords::EdgeGroupOrientSymCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::edge_group_orient::EdgeGroupOrient,
        tables::table_loader::{as_u16_slice, as_u16_slice_mut, as_u32_slice_mut},
    },
};

use super::{
    lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, table_loader::load_table,
};

const TABLE_SIZE_BYTES: usize = (64430 * 16 * 18) * 4;
const FILE_CHECKSUM: u32 = 1523055528;

pub struct MoveComboEdgeGroupOrientTable(Mmap);

impl MoveComboEdgeGroupOrientTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr() as *const u16
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymEdgeGroupOrientTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u32_slice_mut(buffer);

        buffer
            .par_chunks_mut(18)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = EdgeGroupOrientSymCoord((i >> 4) as u16);
                let domino_conjugation = DominoSymmetry((i as u8) & 0b1111);
                let combo = EdgeGroupOrientComboCoord {
                    sym_coord,
                    domino_conjugation,
                };
                let raw = sym_lookup_table.get_raw_from_combo(combo);
                let group_orient = EdgeGroupOrient::from_coord(raw);

                CubeMove::all_iter()
                    .zip(store)
                    .for_each(|(mv, slot)| {
                        let new_raw = group_orient.apply_cube_move(mv).into_coord();
                        let new_combo = sym_lookup_table.get_combo_from_raw(new_raw);
                        *slot = ((new_combo.sym_coord.0 as u32) << 4) | new_combo.domino_conjugation.0 as u32;
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

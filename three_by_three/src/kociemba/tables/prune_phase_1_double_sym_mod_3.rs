use bitvec::field::BitField;
use bitvec::view::BitView;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{Ordering, fence};

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::Tables;
use crate::cube_ops::cube_sym::DominoSymmetry;
use crate::kociemba::coords::{CornerOrientRawCoord, EdgeGroupOrientSymCoord};
use crate::kociemba::tables::lookup_sym_corner_orient::{CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable};
use crate::kociemba::tables::lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use crate::kociemba::tables::move_raw_corner_orient::MoveRawCornerOrientTable;
use crate::kociemba::tables::move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;
use crate::kociemba::tables::prune_phase_1::PrunePhase1Table;
use crate::kociemba::tables::table_loader::as_u32_slice;

use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_ENTRY_COUNT: usize = 65536 * 168 * 16;
pub(crate) const TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 4;
const FILE_CHECKSUM: u32 = 270889050;

pub struct PrunePhase1DoubleSymMod3Table {
    buffer: [u8],
}

impl PrunePhase1DoubleSymMod3Table {
    #[inline(always)]
    pub fn get_value(
        &self,
        edge_group_orient_sym_coord: EdgeGroupOrientSymCoord,
        corner_orient_combo_coord: CornerOrientComboCoord,
    ) -> u8 {
        let ego_coord = edge_group_orient_sym_coord.0;
        let co_coord = corner_orient_combo_coord.sym_coord.0;
        let co_sym = corner_orient_combo_coord.domino_conjugation.0;
        let co_sym_hi = co_sym >> 2;
        let co_sym_lo = co_sym & 0b11;

        let i = ((ego_coord as usize) << 2) | (co_sym_lo as usize) | ((co_coord as usize) << 18);
        let byte = self.buffer[i];

        (byte >> (co_sym_hi << 1)) & 0b11
    }
}

impl PrunePhase1DoubleSymMod3Table {
    fn generate(
        buffer: &mut [u8],
        tables: &(
             impl Send
             + Sync
             + AsRef<LookupSymCornerOrientTable>
             + AsRef<PrunePhase1Table>
         ),
    ) {
        let co_lookup: &LookupSymCornerOrientTable = tables.as_ref();
        let prune: &PrunePhase1Table = tables.as_ref();

        buffer.par_iter_mut().enumerate().for_each(|(i, out)| {
            let ego_coord = ((i >> 2) & 0xFFFF) as u16;
            if ego_coord >= 64430 {
                return;
            }
            let co_coord = i >> 18;
            let co_sym_lo = i & 0b11;

            let mut byte = 0;
            for co_sym_hi in 0..4 {
                let co_sym = (co_sym_hi << 2) | co_sym_lo;
                let co_combo = CornerOrientComboCoord {
                    sym_coord: CornerOrientSymCoord(co_coord as u8),
                    domino_conjugation: DominoSymmetry(co_sym as u8),
                };
                let co_raw = co_lookup.get_raw_from_combo(co_combo);
                let dist = prune.get_value(EdgeGroupOrientSymCoord(ego_coord), co_raw);
                let contrib = (dist % 3) << (co_sym_hi << 1);
                byte |= contrib;
            }

            *out = byte;
        });
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        tables: &(
             impl Send
             + Sync
             + AsRef<LookupSymCornerOrientTable>
             + AsRef<PrunePhase1Table>
         ),
    ) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, tables)
        })
    }

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) fn as_u32_buffer(&self) -> &[u32] {
        as_u32_slice(self.as_buffer())
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

#[test]
fn coherence() -> anyhow::Result<()> {
    let tables = Tables::new("tables")?;
    let original: &PrunePhase1Table = tables.as_ref();
    let transformed: &PrunePhase1DoubleSymMod3Table = tables.as_ref();
    let lookup: &LookupSymCornerOrientTable = tables.as_ref();

    (0..64430u16).into_par_iter().for_each(|ego| {
        let ego = EdgeGroupOrientSymCoord(ego);

        for i in 0..2187 {
            let co_raw = CornerOrientRawCoord(i);
            let co_combo = lookup.get_combo_from_raw(co_raw);
            let original_dist = original.get_value(ego, co_raw);
            let transformed_dist = transformed.get_value(ego, co_combo);
            assert_eq!(original_dist % 3, transformed_dist)
        }

        for i in 0..168 {
            let i = CornerOrientSymCoord(i);
            for j in DominoSymmetry::all_iter() {
                let co_combo = CornerOrientComboCoord { sym_coord: i, domino_conjugation: j };
                let co_raw = lookup.get_raw_from_combo(co_combo);
                let original_dist = original.get_value(ego, co_raw);
                let transformed_dist = transformed.get_value(ego, co_combo);
                assert_eq!(original_dist % 3, transformed_dist)
            }
        }
    });

    Ok(())
}
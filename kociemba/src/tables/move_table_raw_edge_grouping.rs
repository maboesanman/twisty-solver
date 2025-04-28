use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_1_cubies, CornerOrientCoord, EdgeGroupCoord, EdgeOrientCoord},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_full_move_table, load_table};

const EDGE_GROUPING_MOVE_TABLE_SIZE_BYTES: usize = (495 * (18 + 16)) * 2;
const EDGE_GROUPING_MOVE_TABLE_CHECKSUM: u32 = 253579695;

fn generate_edge_grouping_move_table(buffer: &mut [u8]) {
    generate_full_move_table::<EDGE_GROUPING_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| {
            phase_1_cubies(
                CornerOrientCoord(0),
                EdgeOrientCoord(0),
                EdgeGroupCoord(i as u16),
            )
        },
        |c| CornerOrientCoord::from_cubie(c).into(),
    );
}

pub fn load_edge_grouping_move_table<P: AsRef<Path>>(path: P) -> Result<EdgeGroupingMoveTable> {
    load_table(
        path,
        EDGE_GROUPING_MOVE_TABLE_SIZE_BYTES,
        EDGE_GROUPING_MOVE_TABLE_CHECKSUM,
        generate_edge_grouping_move_table,
    )
    .map(EdgeGroupingMoveTable)
}

pub struct EdgeGroupingMoveTable(Mmap);

impl EdgeGroupingMoveTable {
    fn get_slice_for_coord(&self, coord: EdgeGroupCoord) -> &[u16; 34] {
        let i = (coord.0 as usize) * 34;
        as_u16_slice(&self.0)[i..i + 34].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: EdgeGroupCoord, mv: Move) -> EdgeGroupCoord {
        self.get_slice_for_coord(coord)[mv as usize].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: EdgeGroupCoord,
        transform: SubGroupTransform,
    ) -> EdgeGroupCoord {
        self.get_slice_for_coord(coord)[transform.0 as usize + 18].into()
    }

    pub fn get_sym_array_ref(&self, coord: EdgeGroupCoord) -> &[u16; 16] {
        self.get_slice_for_coord(coord)[18..].as_array().unwrap()
    }
}

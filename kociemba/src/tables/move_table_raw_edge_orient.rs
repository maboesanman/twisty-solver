use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_1_cubies, CornerOrientCoord, EdgeGroupCoord, EdgeOrientCoord},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_full_move_table, load_table};

const EDGE_ORIENT_MOVE_TABLE_SIZE_BYTES: usize = (2048 * (18 + 16)) * 2;
const EDGE_ORIENT_MOVE_TABLE_CHECKSUM: u32 = 3006511453;

fn generate_edge_orient_move_table(buffer: &mut [u8]) {
    generate_full_move_table::<EDGE_ORIENT_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| {
            phase_1_cubies(
                CornerOrientCoord(0),
                EdgeOrientCoord(i as u16),
                EdgeGroupCoord(0),
            )
        },
        |c| EdgeOrientCoord::from_cubie(c).into(),
    );
}

pub fn load_edge_orient_move_table<P: AsRef<Path>>(path: P) -> Result<EdgeOrientMoveTable> {
    load_table(
        path,
        EDGE_ORIENT_MOVE_TABLE_SIZE_BYTES,
        EDGE_ORIENT_MOVE_TABLE_CHECKSUM,
        generate_edge_orient_move_table,
    )
    .map(EdgeOrientMoveTable)
}

pub struct EdgeOrientMoveTable(Mmap);

impl EdgeOrientMoveTable {
    fn get_slice_for_coord(&self, coord: EdgeOrientCoord) -> &[u16; 34] {
        let i = (coord.0 as usize) * 34;
        as_u16_slice(&self.0)[i..i + 34].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: EdgeOrientCoord, mv: Move) -> EdgeOrientCoord {
        self.get_slice_for_coord(coord)[mv as usize].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: EdgeOrientCoord,
        transform: SubGroupTransform,
    ) -> EdgeOrientCoord {
        self.get_slice_for_coord(coord)[transform.0 as usize + 18].into()
    }

    pub fn get_sym_array_ref(&self, coord: EdgeOrientCoord) -> &[u16; 16] {
        self.get_slice_for_coord(coord)[18..].as_array().unwrap()
    }
}

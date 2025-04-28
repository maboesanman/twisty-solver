use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_1_cubies, CornerOrientCoord, EdgeGroupCoord, EdgeOrientCoord},
    moves::Move,
    symmetries::SubGroupTransform,
    tables::table_loader::generate_full_move_table,
};

use super::table_loader::{as_u16_slice, load_table};

const CORNER_ORIENT_MOVE_TABLE_SIZE_BYTES: usize = (2187 * (18 + 16)) * 2;
const CORNER_ORIENT_MOVE_TABLE_CHECKSUM: u32 = 402471466;

fn generate_corner_orient_move_table(buffer: &mut [u8]) {
    generate_full_move_table::<CORNER_ORIENT_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| {
            phase_1_cubies(
                CornerOrientCoord(i as u16),
                EdgeOrientCoord(0),
                EdgeGroupCoord(0),
            )
        },
        |c| CornerOrientCoord::from_cubie(c).into(),
    );
}

pub fn load_corner_orient_move_table<P: AsRef<Path>>(path: P) -> Result<CornerOrientMoveTable> {
    load_table(
        path,
        CORNER_ORIENT_MOVE_TABLE_SIZE_BYTES,
        CORNER_ORIENT_MOVE_TABLE_CHECKSUM,
        generate_corner_orient_move_table,
    )
    .map(CornerOrientMoveTable)
}

pub struct CornerOrientMoveTable(Mmap);

impl CornerOrientMoveTable {
    pub fn apply_move(&self, coord: CornerOrientCoord, mv: Move) -> CornerOrientCoord {
        let i = (coord.0 as usize) * 34 + (mv as u8 as usize);
        as_u16_slice(&self.0)[i].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: CornerOrientCoord,
        transform: SubGroupTransform,
    ) -> CornerOrientCoord {
        let i = (coord.0 as usize) * 34 + (transform.0 as usize + 18);
        as_u16_slice(&self.0)[i].into()
    }
}

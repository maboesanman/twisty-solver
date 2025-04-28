use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_2_cubies, CornerPermCoord, EEdgePermCoord, UDEdgePermCoord},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_full_move_table, load_table};

const UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES: usize = (40320 * (18 + 16)) * 2;
const UD_EDGE_PERM_MOVE_TABLE_CHECKSUM: u32 = 37629438;

fn generate_ud_edge_perm_move_table(buffer: &mut [u8]) {
    generate_full_move_table::<UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| {
            phase_2_cubies(
                CornerPermCoord(0),
                UDEdgePermCoord(i as u16),
                EEdgePermCoord(0),
            )
        },
        |c| UDEdgePermCoord::from_cubie(c).into(),
    );
}

pub fn load_ud_edge_perm_move_table<P: AsRef<Path>>(path: P) -> Result<UDEdgePermMoveTable> {
    load_table(
        path,
        UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES,
        UD_EDGE_PERM_MOVE_TABLE_CHECKSUM,
        generate_ud_edge_perm_move_table,
    )
    .map(UDEdgePermMoveTable)
}

pub struct UDEdgePermMoveTable(Mmap);

impl UDEdgePermMoveTable {
    pub fn apply_move(&self, coord: UDEdgePermCoord, mv: Move) -> UDEdgePermCoord {
        let i = (coord.0 as usize) * 34 + (mv as u8 as usize);
        as_u16_slice(&self.0)[i].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: UDEdgePermCoord,
        transform: SubGroupTransform,
    ) -> UDEdgePermCoord {
        let i = (coord.0 as usize) * 34 + (transform.0 as usize + 18);
        as_u16_slice(&self.0)[i].into()
    }
}

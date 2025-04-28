use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_2_cubies, CornerPermCoord, EEdgePermCoord, UDEdgePermCoord},
    moves::Move,
    symmetries::{SubGroupTransform, Transform},
};

use super::table_loader::load_table;

const E_EDGE_PERM_MOVE_TABLE_SIZE_BYTES: usize = 24 * (18 + 16);
const E_EDGE_PERM_MOVE_TABLE_CHECKSUM: u32 = 1568500842;

fn generate_e_edge_perm_move_table(buffer: &mut [u8]) {
    assert_eq!(buffer.len(), E_EDGE_PERM_MOVE_TABLE_SIZE_BYTES);

    for i in 0..24 {
        let cube = phase_2_cubies(
            CornerPermCoord(0),
            UDEdgePermCoord(0),
            EEdgePermCoord(i as u8),
        );
        let mut j = 0usize;
        while j < 18 {
            let m = unsafe { core::mem::transmute(j as u8) };
            buffer[i * 34 + j] = EEdgePermCoord::from_cubie(cube.const_move(m)).into();
            j += 1;
        }
        while j < 34 {
            let t = Transform((j - 18) as u8);
            buffer[i * 34 + j] = EEdgePermCoord::from_cubie(cube.conjugate_by_transform(t)).into();
            j += 1;
        }
    }
}

pub fn load_e_edge_perm_move_table<P: AsRef<Path>>(path: P) -> Result<EEdgePermMoveTable> {
    load_table(
        path,
        E_EDGE_PERM_MOVE_TABLE_SIZE_BYTES,
        E_EDGE_PERM_MOVE_TABLE_CHECKSUM,
        generate_e_edge_perm_move_table,
    )
    .map(EEdgePermMoveTable)
}

pub struct EEdgePermMoveTable(Mmap);

impl EEdgePermMoveTable {
    pub fn apply_move(&self, coord: EEdgePermCoord, mv: Move) -> EEdgePermCoord {
        let i = (coord.0 as usize) * 34 + (mv as u8 as usize);
        self.0[i].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: EEdgePermCoord,
        transform: SubGroupTransform,
    ) -> EEdgePermCoord {
        let i = (coord.0 as usize) * 34 + (transform.0 as usize + 18);
        self.0[i].into()
    }
}

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_2_cubies, CornerPermCoord, EEdgePermCoord, UDEdgePermCoord},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_full_move_table, load_table};

const CORNER_PERM_MOVE_TABLE_SIZE_BYTES: usize = (40320 * (18 + 16)) * 2;
const CORNER_PERM_MOVE_TABLE_CHECKSUM: u32 = 683523999;

fn generate_corner_perm_move_table(buffer: &mut [u8]) {
    generate_full_move_table::<CORNER_PERM_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| {
            phase_2_cubies(
                CornerPermCoord(i as u16),
                UDEdgePermCoord(0),
                EEdgePermCoord(0),
            )
        },
        |c| CornerPermCoord::from_cubie(c).into(),
    );
}

pub fn load_corner_perm_move_table<P: AsRef<Path>>(path: P) -> Result<CornerPermMoveTable> {
    load_table(
        path,
        CORNER_PERM_MOVE_TABLE_SIZE_BYTES,
        CORNER_PERM_MOVE_TABLE_CHECKSUM,
        generate_corner_perm_move_table,
    )
    .map(CornerPermMoveTable)
}

pub struct CornerPermMoveTable(Mmap);

impl CornerPermMoveTable {
    fn get_slice_for_coord(&self, coord: CornerPermCoord) -> &[u16; 34] {
        let i = (coord.0 as usize) * 34;
        as_u16_slice(&self.0)[i..i + 34].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: CornerPermCoord, mv: Move) -> CornerPermCoord {
        let entry = self.get_slice_for_coord(coord);
        entry[mv as u8 as usize].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: CornerPermCoord,
        transform: SubGroupTransform,
    ) -> CornerPermCoord {
        let entry = self.get_slice_for_coord(coord);
        entry[transform.0 as usize + 18].into()
    }

    pub fn get_sym_representative(
        &self,
        coord: CornerPermCoord,
    ) -> (CornerPermCoord, SubGroupTransform) {
        let entry = &self.get_slice_for_coord(coord)[18..];
        let (i, representative) = entry.iter().enumerate().min_by_key(|(_, x)| *x).unwrap();
        ((*representative).into(), SubGroupTransform(i as u8))
    }
}

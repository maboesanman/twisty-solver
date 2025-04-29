use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_2_cubies, UDEdgePermCoord},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_full_move_table, load_table};

const UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES: usize = (40320 * (18 + 16)) * 2;
const UD_EDGE_PERM_MOVE_TABLE_CHECKSUM: u32 = 37629438;

fn generate_ud_edge_perm_move_table(buffer: &mut [u8]) {
    generate_full_move_table::<UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| phase_2_cubies(0.into(), (i as u16).into(), 0.into()),
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
        let i = (coord.inner() as usize) * 34 + (mv as u8 as usize);
        as_u16_slice(&self.0)[i].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: UDEdgePermCoord,
        transform: SubGroupTransform,
    ) -> UDEdgePermCoord {
        let i = (coord.inner() as usize) * 34 + (transform.0 as usize + 18);
        as_u16_slice(&self.0)[i].into()
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_ud_edge_perm_move_table("ud_edge_perm_move_table.dat")?;
    for i in 0..40320 {
        let coord = UDEdgePermCoord::from(i);
        let cube = phase_2_cubies(0.into(), coord, 0.into());

        for i in 0..18 {
            let mv: Move = unsafe { core::mem::transmute(i as u8) };
            let cubie_moved = UDEdgePermCoord::from_cubie(cube.const_move(mv));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                UDEdgePermCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
            let table_conjugated = table.conjugate_by_transform(coord, transform);
            assert_eq!(cubie_conjugated, table_conjugated);
        }
    }

    Ok(())
}

#[test]
fn test_random() -> Result<()> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    let table = load_ud_edge_perm_move_table("ud_edge_perm_move_table.dat")?;
    for i in 0..40320 {
        let coord = UDEdgePermCoord::from(i);
        let cube = phase_2_cubies(
            rng.random_range(0..40320u16).into(),
            coord,
            rng.random_range(0..24u8).into(),
        );

        for i in 0..18 {
            let mv: Move = unsafe { core::mem::transmute(i as u8) };
            let cubie_moved = UDEdgePermCoord::from_cubie(cube.const_move(mv));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                UDEdgePermCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
            let table_conjugated = table.conjugate_by_transform(coord, transform);
            assert_eq!(cubie_conjugated, table_conjugated);
        }
    }

    Ok(())
}

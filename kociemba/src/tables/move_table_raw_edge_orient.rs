use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_1_cubies, EdgeOrientCoord},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_phase_1_move_table, load_table};

const EDGE_ORIENT_MOVE_TABLE_SIZE_BYTES: usize = 2048 * 18 * 2;
const EDGE_ORIENT_MOVE_TABLE_CHECKSUM: u32 = 1158719283;

fn generate_edge_orient_move_table(buffer: &mut [u8]) {
    generate_phase_1_move_table::<EDGE_ORIENT_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| phase_1_cubies(0.into(), (i as u16).into(), 0.into()),
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
    fn get_slice_for_coord(&self, coord: EdgeOrientCoord) -> &[u16; 18] {
        let i = (coord.inner() as usize) * 18;
        as_u16_slice(&self.0)[i..i + 18].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: EdgeOrientCoord, mv: Move) -> EdgeOrientCoord {
        self.get_slice_for_coord(coord)[mv as usize].into()
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_edge_orient_move_table("edge_orient_move_table.dat")?;
    for i in 0..2048 {
        let coord = EdgeOrientCoord::from(i);
        let cube = phase_1_cubies(0.into(), coord, 0.into());

        for i in 0..18 {
            let mv: Move = unsafe { core::mem::transmute(i as u8) };
            let cubie_moved = EdgeOrientCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }
    }

    Ok(())
}

#[test]
fn test_random() -> Result<()> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    let table = load_edge_orient_move_table("edge_orient_move_table.dat")?;
    for i in 0..2048 {
        let coord = EdgeOrientCoord::from(i);
        let cube = phase_1_cubies(
            rng.random_range(0..2187u16).into(),
            coord,
            rng.random_range(0..495u16).into(),
        );

        for j in 0..18 {
            let mv: Move = unsafe { core::mem::transmute(j as u8) };
            let cubie_moved = EdgeOrientCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }
    }

    Ok(())
}

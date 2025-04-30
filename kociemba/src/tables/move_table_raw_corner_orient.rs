use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{CornerOrientCoord, phase_1_cubies},
    moves::Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_phase_1_move_and_sym_table, load_table};

const CORNER_ORIENT_MOVE_TABLE_SIZE_BYTES: usize = (2187 * 33) * 2;
const CORNER_ORIENT_MOVE_TABLE_CHECKSUM: u32 = 1089186443;

fn generate_corner_orient_move_table(buffer: &mut [u8]) {
    generate_phase_1_move_and_sym_table::<CORNER_ORIENT_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| phase_1_cubies((i as u16).into(), 0.into(), 0.into()),
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
        let i = (coord.inner() as usize) * 33 + (mv as u8 as usize);
        as_u16_slice(&self.0)[i].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: CornerOrientCoord,
        transform: SubGroupTransform,
    ) -> CornerOrientCoord {
        if transform.0 == 0 {
            return coord;
        }
        let i = (coord.inner() as usize) * 33 + (transform.0 as usize + 17);
        as_u16_slice(&self.0)[i].into()
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_corner_orient_move_table("corner_orient_move_table.dat")?;
    for i in 0..2187 {
        let coord = i.into();
        let cube = phase_1_cubies(coord, 0.into(), 0.into());

        for i in 0..18 {
            let mv: Move = unsafe { core::mem::transmute(i as u8) };
            let cubie_moved = CornerOrientCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                CornerOrientCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
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
    let table = load_corner_orient_move_table("corner_orient_move_table.dat")?;
    for i in 0..2187 {
        let coord = CornerOrientCoord::from(i);
        let cube = phase_1_cubies(
            coord,
            rng.random_range(0..2048u16).into(),
            rng.random_range(0..495u16).into(),
        );

        for i in 0..18 {
            let mv: Move = unsafe { core::mem::transmute(i as u8) };
            let cubie_moved = CornerOrientCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                CornerOrientCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
            let table_conjugated = table.conjugate_by_transform(coord, transform);
            assert_eq!(cubie_conjugated, table_conjugated);
        }
    }

    Ok(())
}

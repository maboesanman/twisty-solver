use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{CornerPermCoord, phase_2_cubies},
    moves::Phase2Move,
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_phase_2_move_table, load_table};

const CORNER_PERM_MOVE_TABLE_SIZE_BYTES: usize = (40320 * (10 + 15)) * 2;
const CORNER_PERM_MOVE_TABLE_CHECKSUM: u32 = 1827037757;

fn generate_corner_perm_move_table(buffer: &mut [u8]) {
    generate_phase_2_move_table::<CORNER_PERM_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| phase_2_cubies((i as u16).into(), 0.into(), 0.into()),
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
    fn get_slice_for_coord(&self, coord: CornerPermCoord) -> &[u16; 25] {
        let i = (coord.inner() as usize) * 25;
        as_u16_slice(&self.0)[i..i + 25].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: CornerPermCoord, mv: Phase2Move) -> CornerPermCoord {
        let entry = self.get_slice_for_coord(coord);
        entry[mv.into_index()].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: CornerPermCoord,
        transform: SubGroupTransform,
    ) -> CornerPermCoord {
        if transform.0 == 0 {
            return coord;
        }
        let entry = self.get_slice_for_coord(coord);
        entry[transform.0 as usize + 9].into()
    }

    pub fn get_sym_representative(
        &self,
        coord: CornerPermCoord,
    ) -> (CornerPermCoord, SubGroupTransform) {
        let entry = &self.get_slice_for_coord(coord)[10..];
        let coord_val: u16 = coord.into();
        let (i, representative) = Some(&coord_val)
            .into_iter()
            .chain(entry.iter())
            .enumerate()
            .min_by_key(|(_, x)| *x)
            .unwrap();
        ((*representative).into(), SubGroupTransform(i as u8))
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_corner_perm_move_table("corner_perm_move_table.dat")?;
    for i in 0..40320 {
        let coord = CornerPermCoord::from(i);
        let cube = phase_2_cubies(coord, 0.into(), 0.into());

        for mv in Phase2Move::all_iter() {
            let cubie_moved = CornerPermCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                CornerPermCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
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
    let table = load_corner_perm_move_table("corner_perm_move_table.dat")?;
    for i in 0..40320 {
        let coord = CornerPermCoord::from(i);
        let cube = phase_2_cubies(
            coord,
            rng.random_range(0..40320u16).into(),
            rng.random_range(0..24u8).into(),
        );

        for mv in Phase2Move::all_iter() {
            let cubie_moved = CornerPermCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv);
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                CornerPermCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
            let table_conjugated = table.conjugate_by_transform(coord, transform);
            assert_eq!(cubie_conjugated, table_conjugated);
        }
    }

    Ok(())
}

#[test]
fn test_sym() -> Result<()> {
    let table = load_corner_perm_move_table("corner_perm_move_table.dat")?;
    let mut reps = std::collections::HashSet::new();
    for i in 0..40320 {
        let coord = CornerPermCoord::from(i);

        reps.insert(table.get_sym_representative(coord).0);
    }
    assert_eq!(reps.len(), 2768);

    Ok(())
}

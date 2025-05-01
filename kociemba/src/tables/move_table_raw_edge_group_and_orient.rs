use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    coords::{EdgeGroupCoord, EdgeOrientCoord, phase_1_cubies},
    moves::Move,
    symmetries::SubGroupTransform,
    tables::table_loader::as_u16_slice_mut,
};

use super::table_loader::{as_u16_slice, load_table};

// 2048 edge orient values, 495 edge group values, (18 cube moves + 15 non-identity transforms), 2 coordinates out, 2 bytes each
// this move table should be (group, orient, move/transform) -> (group, orient)
const EDGE_GROUP_AND_ORIENT_MOVE_TABLE_SIZE_BYTES: usize = (2048 * 495 * 33) * 2 * 2;
const EDGE_GROUP_AND_ORIENT_MOVE_TABLE_CHECKSUM: u32 = 3948581847;

fn generate_edge_group_and_orient_move_table(buffer: &mut [u8]) {
    assert_eq!(buffer.len(), EDGE_GROUP_AND_ORIENT_MOVE_TABLE_SIZE_BYTES);
    let buffer = as_u16_slice_mut(buffer);

    buffer.par_chunks_mut(66).enumerate().for_each(|(i, row)| {
        let edges = (i & 0b11111111111) as u16;
        let edge_group = (i >> 11) as u16;
        for (j, coord) in phase_1_cubies(0.into(), edges.into(), edge_group.into())
            .phase_1_move_table_entry_cubes()
            .flat_map(|c| {
                [
                    EdgeGroupCoord::from_cubie(c).into(),
                    EdgeOrientCoord::from_cubie(c).into(),
                ]
            })
            .enumerate()
        {
            row[j] = coord
        }
    });
}

pub fn load_edge_group_and_orient_move_table<P: AsRef<Path>>(
    path: P,
) -> Result<EdgeGroupAndOrientMoveTable> {
    load_table(
        path,
        EDGE_GROUP_AND_ORIENT_MOVE_TABLE_SIZE_BYTES,
        EDGE_GROUP_AND_ORIENT_MOVE_TABLE_CHECKSUM,
        generate_edge_group_and_orient_move_table,
    )
    .map(EdgeGroupAndOrientMoveTable)
}

pub struct EdgeGroupAndOrientMoveTable(Mmap);

impl EdgeGroupAndOrientMoveTable {
    fn get_slice_for_coords(&self, group: EdgeGroupCoord, orient: EdgeOrientCoord) -> &[u16; 66] {
        let group: u16 = group.into();
        let orient: u16 = orient.into();
        let i = ((group as usize) << 11) + orient as usize;
        as_u16_slice(&self.0)[66 * i..66 * (i + 1)]
            .as_array()
            .unwrap()
    }

    pub fn apply_move(
        &self,
        edge_group_coord: EdgeGroupCoord,
        edge_orient_coord: EdgeOrientCoord,
        mv: Move,
    ) -> (EdgeGroupCoord, EdgeOrientCoord) {
        let slice = self.get_slice_for_coords(edge_group_coord, edge_orient_coord);
        let mv_index = mv.into_index() * 2;
        (slice[mv_index].into(), slice[mv_index + 1].into())
    }

    pub fn conjugate_by_transform(
        &self,
        edge_group_coord: EdgeGroupCoord,
        edge_orient_coord: EdgeOrientCoord,
        transform: SubGroupTransform,
    ) -> (EdgeGroupCoord, EdgeOrientCoord) {
        if transform.0 == 0 {
            return (edge_group_coord, edge_orient_coord);
        }
        let slice = self.get_slice_for_coords(edge_group_coord, edge_orient_coord);
        let transform_index = 34 + transform.0 as usize * 2;
        (
            slice[transform_index].into(),
            slice[transform_index + 1].into(),
        )
    }

    pub fn get_sym_representative(
        &self,
        group: EdgeGroupCoord,
        orient: EdgeOrientCoord,
    ) -> (EdgeGroupCoord, EdgeOrientCoord, SubGroupTransform) {
        let entry =
            unsafe { self.get_slice_for_coords(group, orient)[36..].as_chunks_unchecked::<2>() };
        let first = [group.into(), orient.into()];
        let (i, representative) = Some(&first)
            .into_iter()
            .chain(entry.iter())
            .enumerate()
            .min_by_key(|(i, x)| (*x, *i))
            .unwrap();
        (
            representative[0].into(),
            representative[1].into(),
            SubGroupTransform(i as u8),
        )
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_edge_group_and_orient_move_table("edge_group_and_orient_move_table.dat")?;
    itertools::iproduct!(0..2048, 0..495)
        .par_bridge()
        .for_each(|(i, j)| {
            let orient_coord = i.into();
            let group_coord = j.into();
            let cube = phase_1_cubies(0.into(), orient_coord, group_coord);

            for mv in Move::all_iter() {
                let moved = cube.then(mv.into());
                let cubie_moved = (
                    EdgeGroupCoord::from_cubie(moved),
                    EdgeOrientCoord::from_cubie(moved),
                );
                let table_moved = table.apply_move(group_coord, orient_coord, mv);
                assert_eq!(cubie_moved, table_moved);
            }

            for transform in SubGroupTransform::all_iter() {
                let moved = cube.conjugate_by_subgroup_transform(transform);
                let cubie_conjugated = (
                    EdgeGroupCoord::from_cubie(moved),
                    EdgeOrientCoord::from_cubie(moved),
                );
                let table_conjugated =
                    table.conjugate_by_transform(group_coord, orient_coord, transform);
                assert_eq!(cubie_conjugated, table_conjugated);
            }
        });

    Ok(())
}

#[test]
fn test_random() -> Result<()> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    let table = load_edge_group_and_orient_move_table("edge_group_and_orient_move_table.dat")?;
    itertools::iproduct!(0..2048, 0..495)
        .map(|(i, j)| (i, j, rng.random_range(0..2187)))
        .par_bridge()
        .for_each(|(i, j, k)| {
            let orient_coord = i.into();
            let group_coord = j.into();
            let cube = phase_1_cubies(k.into(), orient_coord, group_coord);

            for mv in Move::all_iter() {
                let moved = cube.then(mv.into());
                let cubie_moved = (
                    EdgeGroupCoord::from_cubie(moved),
                    EdgeOrientCoord::from_cubie(moved),
                );
                let table_moved = table.apply_move(group_coord, orient_coord, mv);
                assert_eq!(cubie_moved, table_moved);
            }

            for transform in SubGroupTransform::all_iter() {
                let moved = cube.conjugate_by_subgroup_transform(transform);
                let cubie_conjugated = (
                    EdgeGroupCoord::from_cubie(moved),
                    EdgeOrientCoord::from_cubie(moved),
                );
                let table_conjugated =
                    table.conjugate_by_transform(group_coord, orient_coord, transform);
                assert_eq!(cubie_conjugated, table_conjugated);
            }
        });

    Ok(())
}

#[test]
fn test_sym() -> Result<()> {
    let table = load_edge_group_and_orient_move_table("edge_group_and_orient_move_table.dat")?;
    let mut reps = std::collections::HashSet::new();
    reps.reserve(80000);
    itertools::iproduct!(0..2048, 0..495).for_each(|(i, j)| {
        let orient_coord = i.into();
        let group_coord = j.into();

        let rep = table.get_sym_representative(group_coord, orient_coord);
        reps.insert((rep.0, rep.1));
    });
    assert_eq!(reps.len(), 64430);

    Ok(())
}

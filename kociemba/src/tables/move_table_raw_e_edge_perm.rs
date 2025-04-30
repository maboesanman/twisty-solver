use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    coords::{phase_2_cubies, EEdgePermCoord},
    moves::{Move, Phase2Move},
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_phase_2_move_table, load_table};

const E_EDGE_PERM_MOVE_TABLE_SIZE_BYTES: usize = 24 * (10 + 15);
const E_EDGE_PERM_MOVE_TABLE_CHECKSUM: u32 = 665180893;

fn generate_e_edge_perm_move_table(buffer: &mut [u8]) {
    assert_eq!(buffer.len(), E_EDGE_PERM_MOVE_TABLE_SIZE_BYTES);
    buffer.par_chunks_mut(25).enumerate().for_each(|(i, row)| {
        for (j, coord) in phase_2_cubies(0.into(),0.into() ,(i as u8).into())
        .phase_2_move_table_entry_cubes().map(|c| EEdgePermCoord::from_cubie(c).into()).enumerate() {
            row[j] = coord
        }
    });
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
    fn get_slice_for_coord(&self, coord: EEdgePermCoord) -> &[u8; 25] {
        let i = (coord.inner() as usize) * 25;
        &self.0[i..i + 25].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: EEdgePermCoord, mv: Phase2Move) -> EEdgePermCoord {
        let entry = self.get_slice_for_coord(coord);
        entry[mv.into_index()].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: EEdgePermCoord,
        transform: SubGroupTransform,
    ) -> EEdgePermCoord {
        if transform.0 == 0 {
            return coord;
        }
        let entry = self.get_slice_for_coord(coord);
        entry[transform.0 as usize + 9].into()
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_e_edge_perm_move_table("e_edge_perm_move_table.dat")?;
    for i in 0..24 {
        let coord = EEdgePermCoord::from(i);
        let cube = phase_2_cubies(0.into(), 0.into(), coord);

        for mv in Phase2Move::all_iter() {
            let cubie_moved = EEdgePermCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv.into());
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                EEdgePermCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
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
    let table = load_e_edge_perm_move_table("e_edge_perm_move_table.dat")?;
    for i in 0..24 {
        let coord = EEdgePermCoord::from(i);
        let cube = phase_2_cubies(
            rng.random_range(0..40320u16).into(),
            rng.random_range(0..40320u16).into(),
            coord,
        );

        for mv in Phase2Move::all_iter() {
            let cubie_moved = EEdgePermCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv.into());
            assert_eq!(cubie_moved, table_moved);
        }

        for i in 0..16 {
            let transform = SubGroupTransform(i as u8);
            let cubie_conjugated =
                EEdgePermCoord::from_cubie(cube.conjugate_by_subgroup_transform(transform));
            let table_conjugated = table.conjugate_by_transform(coord, transform);
            assert_eq!(cubie_conjugated, table_conjugated);
        }
    }

    Ok(())
}

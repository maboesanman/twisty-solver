use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::{
    coords::{phase_2_cubies, UDEdgePermCoord},
    moves::{Move, Phase2Move},
    symmetries::SubGroupTransform,
};

use super::table_loader::{as_u16_slice, generate_phase_2_move_table, load_table};

const UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES: usize = (40320 * (10 + 15)) * 2;
const UD_EDGE_PERM_MOVE_TABLE_CHECKSUM: u32 = 3029666453;

fn generate_ud_edge_perm_move_table(buffer: &mut [u8]) {
    generate_phase_2_move_table::<UD_EDGE_PERM_MOVE_TABLE_SIZE_BYTES, _, _>(
        buffer,
        |i| phase_2_cubies(0.into(),(i as u16).into() ,0.into()),
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
    fn get_slice_for_coord(&self, coord: UDEdgePermCoord) -> &[u16; 25] {
        let i = (coord.inner() as usize) * 25;
        as_u16_slice(&self.0)[i..i + 25].as_array().unwrap()
    }

    pub fn apply_move(&self, coord: UDEdgePermCoord, mv: Phase2Move) -> UDEdgePermCoord {
        let entry = self.get_slice_for_coord(coord);
        entry[mv.into_index()].into()
    }

    pub fn conjugate_by_transform(
        &self,
        coord: UDEdgePermCoord,
        transform: SubGroupTransform,
    ) -> UDEdgePermCoord {
        if transform.0 == 0 {
            return coord;
        }
        let entry = self.get_slice_for_coord(coord);
        entry[transform.0 as usize + 9].into()
    }
}

#[test]
fn test() -> Result<()> {
    let table = load_ud_edge_perm_move_table("ud_edge_perm_move_table.dat")?;
    for i in 0..40320 {
        let coord = UDEdgePermCoord::from(i);
        let cube = phase_2_cubies(0.into(), coord, 0.into());

        for mv in Phase2Move::all_iter() {
            let cubie_moved = UDEdgePermCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv.into());
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

        for mv in Phase2Move::all_iter() {
            let cubie_moved = UDEdgePermCoord::from_cubie(cube.then(mv.into()));
            let table_moved = table.apply_move(coord, mv.into());
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

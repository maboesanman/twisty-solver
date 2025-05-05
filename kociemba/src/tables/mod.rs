use move_table_raw_corner_orient::CornerOrientMoveTable;
use move_table_raw_edge_group_and_orient::EdgeGroupAndOrientMoveTable;
use move_table_sym_phase_1_edge::Phase1EdgeSymMoveTable;
use pruning_table_phase_1_working::Phase1PruningTable;
use sym_lookup_phase_1_edge::Phase1EdgeSymLookupTable;

use crate::{coords::{CornerOrientCoord, EdgeGroupCoord, EdgeOrientCoord, Phase1EdgeSymCoord}, moves::Move, repr_cubie::ReprCube, symmetries::SubGroupTransform};

pub mod move_table_raw_corner_orient;
pub mod move_table_raw_corner_perm;
pub mod move_table_raw_e_edge_perm;
pub mod move_table_raw_ud_edge_perm;
pub mod sym_lookup_phase_1_edge;
pub mod sym_lookup_phase_2_corner;

pub mod move_table_raw_edge_group_and_orient;

pub mod move_table_sym_phase_1_edge;

pub mod move_table_sym_phase_2_corner;

pub mod pruning_table_phase_1_working;

mod table_loader;

pub struct Tables {
    pub phase_1_move_edge_raw_table: EdgeGroupAndOrientMoveTable,
    pub phase_1_move_corner_raw_table: CornerOrientMoveTable,
    pub phase_1_lookup_edge_sym_table: Phase1EdgeSymLookupTable,
    pub phase_1_move_edge_sym_table: Phase1EdgeSymMoveTable,
    // pub phase_1_pruning_table: Phase1PruningTable,

    // pub phase_2_move_corner_raw_table
    // pub phase_2_lookup_corner_sym_table
    // pub phase_2_move_corner_sym_table
}

impl Tables {
    pub fn new() -> anyhow::Result<Self> {
        let phase_1_move_edge_raw_table =
            move_table_raw_edge_group_and_orient::load_edge_group_and_orient_move_table("edge_group_and_orient_move_table.dat")?;
        let phase_1_move_corner_raw_table =
            move_table_raw_corner_orient::load_corner_orient_move_table("corner_orient_move_table.dat")?;
        let phase_1_lookup_edge_sym_table = sym_lookup_phase_1_edge::load_phase_1_edge_sym_lookup_table(
            "phase_1_edge_sym_lookup_table.dat",
            &phase_1_move_edge_raw_table,
        )?;
        let phase_1_move_edge_sym_table = move_table_sym_phase_1_edge::load_phase_1_edge_sym_move_table(
            "phase_1_edge_sym_move_table.dat",
            &phase_1_lookup_edge_sym_table,
            &phase_1_move_edge_raw_table,
        )?;

        // let phase_1_pruning_table = pruning_table_phase_1_working::load_phase_1_pruning_table("phase_1_pruning_table.dat", &phase_1_move_edge_sym_table, &phase_1_move_corner_raw_table)?;

        Ok(Self {
            phase_1_move_edge_raw_table,
            phase_1_move_corner_raw_table,
            phase_1_lookup_edge_sym_table,
            phase_1_move_edge_sym_table,
            // phase_1_pruning_table,
        })
    }

    pub fn phase_1_sym_coords_from_cube(&self, cube: ReprCube) -> ((Phase1EdgeSymCoord, CornerOrientCoord), SubGroupTransform) {
        let (corner_orient, edge_orient, edge_group) = cube.into_phase_1_raw_coords();
        self.phase_1_sym_coords_from_raw(corner_orient, edge_orient, edge_group)
    }

    pub fn phase_1_sym_coords_from_raw(&self, corner_orient: CornerOrientCoord, edge_orient: EdgeOrientCoord, edge_group: EdgeGroupCoord) -> ((Phase1EdgeSymCoord, CornerOrientCoord), SubGroupTransform) {
        let (sym, transform) = self.phase_1_lookup_edge_sym_table.get_sym_from_raw(&self.phase_1_move_edge_raw_table, edge_group, edge_orient);
        let raw = self.phase_1_move_corner_raw_table.conjugate_by_transform(corner_orient, transform);
        ((sym, raw), transform)
    }

    pub fn phase_1_neighbors(&self, coords: (Phase1EdgeSymCoord, CornerOrientCoord)) -> impl Iterator<Item = ((Phase1EdgeSymCoord, CornerOrientCoord), SubGroupTransform)> {
        let (edge_group_coord, edge_orient_coord) = self.phase_1_lookup_edge_sym_table.get_raw_from_sym(coords.0);
        Move::all_iter().map(move |mv| {
            let edges = self.phase_1_move_edge_raw_table.apply_move(edge_group_coord, edge_orient_coord, mv);
            let corners = self.phase_1_move_corner_raw_table.apply_move(coords.1, mv);

            self.phase_1_sym_coords_from_raw(corners, edges.1, edges.0)
        })
    }
}


#[test]
fn phase_1_sym_coords_from_cube_test() -> anyhow::Result<()> {
    let tables = Tables::new()?;

    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    use rand::distr::Distribution;
    
    for _ in 0..1000 {
        let cube: ReprCube = rand::distr::StandardUniform.sample(&mut rng);

        let (p1_coords, _transform) = tables.phase_1_sym_coords_from_cube(cube);
        println!("{:?}", (p1_coords.0, p1_coords.1));
        let neighbors_of_sym_coords: std::collections::BTreeSet<_> = tables.phase_1_neighbors(p1_coords).map(|((a, b), _)| (a.inner(), b.inner())).collect();
        let sym_coords_of_neighbors: std::collections::BTreeSet<_> = cube.phase_1_move_table_entry_cubes().take(18).map(|c| tables.phase_1_sym_coords_from_cube(c).0).map(|(a, b)| (a.inner(), b.inner())).collect();

        assert_eq!(neighbors_of_sym_coords, sym_coords_of_neighbors);
    }

    Ok(())
}
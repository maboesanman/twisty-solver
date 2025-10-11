use std::{fs::create_dir_all, mem::MaybeUninit, path::Path};

use lookup_sym_corner_perm::LookupSymCornerPermTable;
use lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use move_raw_corner_orient::MoveRawCornerOrientTable;
use move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;

use crate::{
    cube_ops::{
        coords::EdgeGroupOrientRawCoord,
        cube_move::{CubeMove, DominoMove},
        partial_reprs::{
            corner_orient::CornerOrient, corner_perm::CornerPerm, e_edge_perm::EEdgePerm,
            edge_group::EdgeGroup, edge_orient::EdgeOrient, edge_perm::EdgePerm,
            ud_edge_perm::UDEdgePerm,
        },
        repr_cube::ReprCube,
    },
    tables::{
        self, grouped_edge_moves::GroupedEdgeMovesTable,
        move_sym_corner_perm::MoveSymCornerPermTable,
    },
};

pub mod lookup_sym_corner_perm;
pub mod lookup_sym_edge_group_orient;

pub mod move_raw_corner_orient;
pub mod move_sym_corner_perm;
pub mod move_sym_edge_group_orient;

pub mod prune_phase_1;
// pub mod prune_phase_2;
// pub mod prune_phase_2_corner_perm;

pub mod grouped_edge_moves;

mod table_loader;

const MOVE_RAW_CORNER_ORIENT_TABLE_NAME: &str = "move_raw_corner_orient_table.dat";
const MOVE_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "move_sym_edge_group_orient_table.dat";
const LOOKUP_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "lookup_sym_edge_group_orient_table.dat";
const LOOKUP_SYM_CORNER_PERM_TABLE_NAME: &str = "lookup_sym_corner_perm_table.dat";
const PRUNE_PHASE_1_TABLE_NAME: &str = "prune_phase_1_table.dat";
const GROUPED_EDGE_MOVES_RESTRICT_TABLE_NAME: &str = "grouped_edge_moves_restrict_table.dat";
const GROUPED_EDGE_MOVES_UD_TABLE_NAME: &str = "grouped_edge_moves_ud_table.dat";
const GROUPED_EDGE_MOVES_E_TABLE_NAME: &str = "grouped_edge_moves_e_table.dat";
const MOVE_SYM_CORNER_PERM_TABLE_NAME: &str = "move_sym_corner_perm_table.dat";
const PRUNE_PHASE_2_TABLE_NAME: &str = "prune_phase_2_table.dat";
const PRUNE_PHASE_2_CORNER_PERM_TABLE_NAME: &str = "prune_phase_2_corner_perm_table.dat";

pub struct Tables {
    pub(crate) lookup_sym_edge_group_orient: LookupSymEdgeGroupOrientTable,
    pub(crate) lookup_sym_corner_perm: LookupSymCornerPermTable,

    pub(crate) move_raw_corner_orient: MoveRawCornerOrientTable,
    pub(crate) move_sym_edge_group_orient: MoveSymEdgeGroupOrientTable,
    pub(crate) move_sym_corner_perm: MoveSymCornerPermTable,
    pub(crate) grouped_edge_moves: GroupedEdgeMovesTable,
    // pub(crate) prune_phase_1: MaybeUninit<PrunePhase1Table>,
    // pub(crate) prune_phase_2: MaybeUninit<PrunePhase2Table>,
    // pub(crate) prune_phase_2_corner_perm: MaybeUninit<PrunePhaseCornerTable>,
}

impl Tables {
    pub fn new<P>(folder: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let folder = folder.as_ref();

        create_dir_all(folder)?;

        let move_raw_corner_orient =
            MoveRawCornerOrientTable::load(folder.join(MOVE_RAW_CORNER_ORIENT_TABLE_NAME))?;
        let lookup_sym_edge_group_orient = LookupSymEdgeGroupOrientTable::load(
            folder.join(LOOKUP_SYM_EDGE_GROUP_ORIENT_TABLE_NAME),
        )?;

        let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
            folder.join(MOVE_SYM_EDGE_GROUP_ORIENT_TABLE_NAME),
            &lookup_sym_edge_group_orient,
        )?;
        let grouped_edge_moves = GroupedEdgeMovesTable::load(
            folder.join(GROUPED_EDGE_MOVES_RESTRICT_TABLE_NAME),
            folder.join(GROUPED_EDGE_MOVES_UD_TABLE_NAME),
            folder.join(GROUPED_EDGE_MOVES_E_TABLE_NAME),
        )?;
        let lookup_sym_corner_perm =
            LookupSymCornerPermTable::load(folder.join(LOOKUP_SYM_CORNER_PERM_TABLE_NAME))?;
        let move_sym_corner_perm = MoveSymCornerPermTable::load(
            folder.join(MOVE_SYM_CORNER_PERM_TABLE_NAME),
            &lookup_sym_corner_perm,
        )?;

        let mut working = Tables {
            move_raw_corner_orient,
            move_sym_edge_group_orient,
            lookup_sym_edge_group_orient,
            grouped_edge_moves,
            lookup_sym_corner_perm,
            move_sym_corner_perm,
            // prune_phase_1: MaybeUninit::uninit(),
            // prune_phase_2: MaybeUninit::uninit(),
            // prune_phase_2_corner_perm: MaybeUninit::uninit(),
        };

        // working.prune_phase_1.write(PrunePhase1Table::load(
        //     folder.join(PRUNE_PHASE_1_TABLE_NAME),
        //     &working,
        // )?);
        // working.prune_phase_2.write(PrunePhase2Table::load(
        //     folder.join(PRUNE_PHASE_2_TABLE_NAME),
        //     &working,
        // )?);
        // working.prune_phase_2_corner_perm.write(PrunePhaseCornerTable::load(folder.join(PRUNE_PHASE_2_CORNER_PERM_TABLE_NAME), &working)?);

        Ok(working)
    }

    // pub fn sym_reduce_cube(&self, cube: ReprCube) -> SymReducedPhase1Repr {
    //     let (edge_group, _, _) = cube.edge_perm.split();
    //     let edge_group_raw = edge_group.into_coord();

    //     let edge_orient_raw = cube.edge_orient.into_coord();
    //     let (edge_group_orient, sym) =
    //         self.lookup_sym_edge_group_orient
    //             .get_combo_from_raw(EdgeGroupOrientRawCoord::join(
    //                 edge_group_raw,
    //                 edge_orient_raw,
    //             ));

    //     let cube = cube.domino_conjugate(sym);

    //     let (_, ud_edge_perm, e_edge_perm) = cube.edge_perm.split();
    //     let corner_orient = cube.corner_orient.into_coord();
    //     let ud_edge_perm = ud_edge_perm.into_coord();
    //     let e_edge_perm = e_edge_perm.into_coord();

    //     let corner_perm_raw = cube.corner_perm.into_coord();
    //     let (corner_perm, corner_perm_inv) = self
    //         .lookup_sym_corner_perm
    //         .get_sym_from_raw(corner_perm_raw);

    //     let corner_perm_correction = corner_perm_inv.inverse();

    //     SymReducedPhase1Repr::from_coords(
    //         corner_orient,
    //         edge_group_orient,
    //         e_edge_perm,
    //         ud_edge_perm,
    //         corner_perm,
    //         corner_perm_correction,
    //     )
    // }

    // pub fn repr_cube_from_phase_1(&self, cube: SymReducedPhase1Repr) -> ReprCube {
    //     // extract coords from sym reduced
    //     let corner_orient_raw_coord = cube.get_corner_orient_coord();
    //     let edge_group_orient_sym_coord = cube.get_edge_group_orient_sym_coord();
    //     let (ud_edge_perm_raw_coord, corner_perm_sym_coord) = cube.ud_edge_and_corner_perm_coords();
    //     let e_edge_perm_raw_coord = cube.e_edge_perm_coord();
    //     let corner_perm_sym_correct = cube.corner_perm_sym_correction();

    //     // corner perm
    //     let corner_perm_rep_coord = self
    //         .lookup_sym_corner_perm
    //         .get_raw_from_sym(corner_perm_sym_coord);
    //     let corner_perm =
    //         CornerPerm::from_coord(corner_perm_rep_coord).domino_conjugate(corner_perm_sym_correct);

    //     // corner orient
    //     let corner_orient = CornerOrient::from_coord(corner_orient_raw_coord);

    //     // edge perm
    //     let edge_group_orient_raw_coord = self
    //         .lookup_sym_edge_group_orient
    //         .get_raw_from_sym(edge_group_orient_sym_coord);
    //     let (edge_group_raw_coord, edge_orient_raw_coord) = edge_group_orient_raw_coord.split();
    //     let edge_perm = EdgePerm::join(
    //         EdgeGroup::from_coord(edge_group_raw_coord),
    //         UDEdgePerm::from_coord(ud_edge_perm_raw_coord),
    //         EEdgePerm::from_coord(e_edge_perm_raw_coord),
    //     );

    //     // edge orient
    //     let edge_orient = EdgeOrient::from_coord(edge_orient_raw_coord);

    //     ReprCube {
    //         corner_perm,
    //         corner_orient,
    //         edge_perm,
    //         edge_orient,
    //     }
    // }

    // pub fn phase_1_adjacent(
    //     &self,
    //     cube: SymReducedPhase1Repr,
    // ) -> impl IntoIterator<Item = SymReducedPhase1Repr> {
    //     let edge_group_orient_sym = cube.get_edge_group_orient_sym_coord();
    //     let edge_group_orient_raw = self
    //         .lookup_sym_edge_group_orient
    //         .get_raw_from_sym(edge_group_orient_sym);
    //     let edge_group = edge_group_orient_raw.split().0;
    //     let e_edge_perm = cube.e_edge_perm_coord();
    //     let (ud_edge_perm, corner_perm) = cube.ud_edge_and_corner_perm_coords();

    //     let corner_orient = cube.get_corner_orient_coord();
    //     let corner_perm_sym_correction = cube.corner_perm_sym_correction();

    //     CubeMove::all_iter().map(move |mv| {
    //         let (new_edge_group_orient, conj) = self
    //             .move_sym_edge_group_orient
    //             .apply_cube_move(edge_group_orient_sym, mv);

    //         let (moved_edge_group, moved_ud_edge_perm, moved_e_edge_perm) = self
    //             .grouped_edge_moves
    //             .update_edge_perms_cube_move(edge_group, mv, ud_edge_perm, e_edge_perm);

    //         // let moved_edge_group_orient_raw = self
    //         //     .lookup_sym_edge_group_orient
    //         //     .get_raw_from_sym(new_edge_group_orient);

    //         // let moved_edge_group = moved_edge_group_orient_raw.split().0;

    //         let (new_edge_group, new_ud_edge_perm, new_e_edge_perm) =
    //             self.grouped_edge_moves.update_edge_perms_domino_conjugate(
    //                 moved_edge_group,
    //                 conj,
    //                 moved_ud_edge_perm,
    //                 moved_e_edge_perm,
    //             );

    //         debug_assert_eq!(self.lookup_sym_edge_group_orient.get_raw_from_sym(new_edge_group_orient).split().0, new_edge_group);

    //         let moved_corner_orient = self
    //             .move_raw_corner_orient
    //             .apply_cube_move(corner_orient, mv);

    //         let (new_corner_perm, correction_adjust) =
    //             self.move_sym_corner_perm.apply_cube_move(corner_perm, mv);

    //         let new_corner_perm_correction = correction_adjust
    //             .then(corner_perm_sym_correction)
    //             .then(conj);

    //         let new_corner_orient = self
    //             .move_raw_corner_orient
    //             .domino_conjugate(moved_corner_orient, conj);

    //         SymReducedPhase1Repr::from_coords(
    //             new_corner_orient,
    //             new_edge_group_orient,
    //             new_e_edge_perm,
    //             new_ud_edge_perm,
    //             new_corner_perm,
    //             new_corner_perm_correction,
    //         )
    //     })
    // }

    // pub fn phase_2_adjacent(
    //     &self,
    //     cube: SymReducedPhase2Repr,
    // ) -> impl IntoIterator<Item = SymReducedPhase2Repr> {
    //     let (ud_edge_perm, corner_perm) = cube.ud_edge_and_corner_perm_coords();
    //     let e_edge_perm = cube.e_edge_perm_coord();

    //     DominoMove::all_iter().map(move |mv| {
    //         let (new_corner_perm, conj) = self
    //             .move_sym_corner_perm
    //             .apply_cube_move(corner_perm, mv.into());

    //         let (moved_ud_edge_perm, moved_e_edge_perm) = self
    //             .grouped_edge_moves
    //             .update_edge_perm_phase_2_domino_move(mv, ud_edge_perm, e_edge_perm);

    //         let (new_ud_edge_perm, new_e_edge_perm) = self
    //             .grouped_edge_moves
    //             .update_edge_perm_phase_2_domino_symmetry(
    //                 conj,
    //                 moved_ud_edge_perm,
    //                 moved_e_edge_perm,
    //             );

    //         SymReducedPhase2Repr::from_coords(new_e_edge_perm, new_ud_edge_perm, new_corner_perm)
    //     })
    // }

    // pub fn phase_1_partial_adjacent(
    //     &self,
    //     cube: SymReducedPhase1PartialRepr,
    // ) -> impl IntoIterator<Item = SymReducedPhase1PartialRepr> {
    //     let edge_group_orient_sym = cube.get_edge_group_orient_sym_coord();
    //     let corner_orient = cube.get_corner_orient_coord();

    //     CubeMove::all_iter().map(move |mv| {
    //         let (new_edge_group_orient, conj) = self
    //             .move_sym_edge_group_orient
    //             .apply_cube_move(edge_group_orient_sym, mv);

    //         let moved_corner_orient = self
    //             .move_raw_corner_orient
    //             .apply_cube_move(corner_orient, mv);

    //         let new_corner_orient = self
    //             .move_raw_corner_orient
    //             .domino_conjugate(moved_corner_orient, conj);

    //         SymReducedPhase1PartialRepr::from_coords(new_edge_group_orient, new_corner_orient)
    //     })
    // }

    // pub fn phase_2_partial_adjacent(
    //     &self,
    //     cube: SymReducedPhase2PartialRepr,
    // ) -> impl IntoIterator<Item = SymReducedPhase2PartialRepr> {
    //     let ud_edge_perm = cube.get_ud_edge_perm_coord();
    //     let corner_perm = cube.get_corner_perm_sym_coord();

    //     DominoMove::all_iter().map(move |mv| {
    //         let (new_corner_perm, conj) = self
    //             .move_sym_corner_perm
    //             .apply_cube_move(corner_perm, mv.into());

    //         let moved_ud_edge_perm = self
    //             .grouped_edge_moves
    //             .update_edge_perm_phase_2_partial_domino_move(mv, ud_edge_perm);

    //         let new_ud_edge_perm = self
    //             .grouped_edge_moves
    //             .update_edge_perm_phase_2_partial_domino_symmetry(conj, moved_ud_edge_perm);

    //         SymReducedPhase2PartialRepr::from_coords(new_corner_perm, new_ud_edge_perm)
    //     })
    // }

    // pub fn phase_change(
    //     &self,
    //     cube: SymReducedPhase1Repr,
    // ) -> Result<SymReducedPhase2Repr, SymReducedPhase1Repr> {
    //     if 0x0FFFFFFF00000000 & cube.0 == 0 {
    //         let conj = cube.corner_perm_sym_correction().inverse();

    //         let (ud_edge_perm, corner_perm) = cube.ud_edge_and_corner_perm_coords();
    //         let e_edge_perm = cube.e_edge_perm_coord();

    //         let (new_ud_edge_perm, new_e_edge_perm) = self
    //             .grouped_edge_moves
    //             .update_edge_perm_phase_2_domino_symmetry(conj, ud_edge_perm, e_edge_perm);

    //         Ok(SymReducedPhase2Repr::from_coords(
    //             new_e_edge_perm,
    //             new_ud_edge_perm,
    //             corner_perm,
    //         ))
    //     } else {
    //         Err(cube)
    //     }
    // }

    // pub fn phase_1_prune_dist_mod_3(&self, cube: SymReducedPhase1Repr) -> u8 {
    //     unsafe { self.prune_phase_1.assume_init_ref() }.get_value(cube.into_pruning_index())
    // }

    // pub fn phase_2_prune_dist_mod_3(&self, cube: SymReducedPhase2Repr) -> u8 {
    //     unsafe { self.prune_phase_2.assume_init_ref() }.get_value(cube.into_pruning_index())
    // }
}

// #[cfg(test)]
// mod test {
//     use std::collections::{BTreeSet, HashSet};

//     use itertools::Itertools;
//     use rand::distr::{Distribution, StandardUniform};

//     use super::*;
//     #[test]
//     fn gen_tables() -> anyhow::Result<()> {
//         let _ = Tables::new("tables")?;

//         Ok(())
//     }

//     #[test]
//     fn move_adjacency() -> anyhow::Result<()> {
//         let tables = Tables::new("tables")?;
//         use rand::SeedableRng;
//         let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(18);

//         for _ in 0..10 {
//             let cube: ReprCube = StandardUniform.sample(&mut rng);

//             let move_then_sym: HashSet<_> = CubeMove::all_iter()
//                 .map(|mv| cube.apply_cube_move(mv))
//                 .map(|c| tables.sym_reduce_cube(c))
//                 .collect();

//             let sym_then_move: HashSet<_> = tables
//                 .phase_1_adjacent(tables.sym_reduce_cube(cube))
//                 .into_iter()
//                 .collect();

//             assert_eq!(move_then_sym, sym_then_move)
//         }

//         Ok(())
//     }

//     #[test]
//     fn move_adjacency_diagnostic() -> anyhow::Result<()> {
//         let tables = Tables::new("tables")?;
//         // use rand::SeedableRng;
//         // let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(18);

//         for mv in CubeMove::all_iter() {
//             let cube = ReprCube::SOLVED.apply_cube_move(mv);

//             let mut move_then_sym = CubeMove::all_iter()
//                 .map(|mv| cube.apply_cube_move(mv))
//                 .map(|c| tables.sym_reduce_cube(c))
//                 .map(|c| {
//                     let c = tables.repr_cube_from_phase_1(c);
//                     (c.edge_perm.0.0, c.edge_orient.0)
//                 })
//                 .collect_vec();

//             move_then_sym.sort();

//             let mut sym_then_move = tables
//                 .phase_1_adjacent(tables.sym_reduce_cube(cube))
//                 .into_iter()
//                 .map(|c| {
//                     let c = tables.repr_cube_from_phase_1(c);
//                     (c.edge_perm.0.0, c.edge_orient.0)
//                 })
//                 .collect_vec();

//             sym_then_move.sort();

//             assert_eq!(move_then_sym, sym_then_move)
//         }

//         Ok(())
//     }

//     #[test]
//     fn repr_phase_1_roundtrip() -> anyhow::Result<()> {
//         let tables = Tables::new("tables")?;

//         use rand::SeedableRng;
//         use rand::prelude::Distribution;

//         let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(18);

//         for _ in 0..10_000 {
//             let cube: ReprCube = StandardUniform.sample(&mut rng);

//             let reduced_a = tables.sym_reduce_cube(cube);
//             let a = tables.repr_cube_from_phase_1(reduced_a);

//             let reduced_b = tables.sym_reduce_cube(a);
//             let b = tables.repr_cube_from_phase_1(reduced_b);

//             assert_eq!(a, b);
//             assert_eq!(reduced_a, reduced_b);
//         }

//         Ok(())
//     }
// }

use lookup_sym_corner_perm::LookupSymCornerPermTable;
use lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use move_raw_corner_orient::MoveRawCornerOrientTable;
use move_raw_corner_perm::MoveRawCornerPermTable;
use move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;
use prune_phase_1::PrunePhase1Table;

use crate::{
    cube_ops::{
        cube_move::{CubeMove, DominoMove}, partial_reprs::e_edge_perm, repr_coord::{SymReducedPhase1Repr, SymReducedPhase2Repr}
    },
    tables::{
        grouped_edge_moves::GroupedEdgeMovesTable, move_sym_corner_perm::MoveSymCornerPermTable,
    },
};

pub mod lookup_sym_corner_perm;
pub mod lookup_sym_edge_group_orient;

pub mod move_raw_corner_orient;
pub mod move_raw_corner_perm;
pub mod move_sym_corner_perm;
pub mod move_sym_edge_group_orient;

pub mod prune_phase_1;

pub mod grouped_edge_moves;

mod table_loader;

pub struct Tables {
    // phase 1 tables
    pub move_raw_corner_orient: MoveRawCornerOrientTable,
    pub move_sym_edge_group_orient: MoveSymEdgeGroupOrientTable,
    pub lookup_sym_edge_group_orient: LookupSymEdgeGroupOrientTable,
    pub prune_phase_1: PrunePhase1Table,

    // multi phase tables
    pub grouped_edge_moves: GroupedEdgeMovesTable,
    pub move_sym_corner_perm: MoveSymCornerPermTable,
    // phase 2 tables
    // prune_phase_2
}

impl Tables {
    pub fn new() -> anyhow::Result<Self> {
        // let move_raw_corner_orient =
        //     MoveRawCornerOrientTable::load("move_raw_corner_orient_table.dat")?;
        // let lookup_sym_edge_group_orient =
        //     LookupSymEdgeGroupOrientTable::load("lookup_sym_edge_group_orient_table.dat")?;
        // let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
        //     "move_sym_edge_group_orient_table.dat",
        //     &lookup_sym_edge_group_orient,
        // )?;
        // let prune_phase_1 = PrunePhase1Table::load(
        //     "prune_phase_1_table.dat",
        //     &move_sym_edge_group_orient,
        //     &move_raw_corner_orient,
        // )?;

        // Ok(Self {
        //     move_raw_corner_orient,
        //     move_sym_edge_group_orient,
        //     lookup_sym_edge_group_orient,
        //     prune_phase_1,
        // })

        todo!()
    }

    pub fn phase_1_adjacent(
        &self,
        cube: SymReducedPhase1Repr,
    ) -> impl IntoIterator<Item = SymReducedPhase1Repr> {
        let edge_group_orient_sym = cube.get_edge_group_orient_sym_coord();
        let edge_group_orient_raw = self
            .lookup_sym_edge_group_orient
            .get_raw_from_sym(edge_group_orient_sym);
        let edge_group = edge_group_orient_raw.split().0;
        let corner_orient = cube.get_corner_orient_coord();
        let (ud_edge_perm, corner_perm) = cube.ud_edge_and_corner_perm_coords();
        let e_edge_perm = cube.e_edge_perm_coord();
        let corner_perm_sym_correction = cube.corner_perm_sym_correction();

        CubeMove::all_iter().map(move |mv| {
            let (new_edge_group_orient, conj) = self
                .move_sym_edge_group_orient
                .apply_cube_move(edge_group_orient_sym, mv);

            let (moved_ud_edge_perm, moved_e_edge_perm) = self
                .grouped_edge_moves
                .update_edge_perms_cube_move(edge_group, mv, ud_edge_perm, e_edge_perm);

            let moved_corner_orient = self
                .move_raw_corner_orient
                .apply_cube_move(corner_orient, mv);

            let (new_corner_perm, correction_adjust) =
                self.move_sym_corner_perm.apply_cube_move(corner_perm, mv);

            let new_corner_perm_correction = correction_adjust
                .then(corner_perm_sym_correction)
                .then(conj);

            let moved_edge_group_orient_raw = self
                .lookup_sym_edge_group_orient
                .get_raw_from_sym(edge_group_orient_sym);

            let moved_edge_group = moved_edge_group_orient_raw.split().0;

            let (new_ud_edge_perm, new_e_edge_perm) =
                self.grouped_edge_moves.update_edge_perms_domino_conjugate(
                    moved_edge_group,
                    conj,
                    moved_ud_edge_perm,
                    moved_e_edge_perm,
                );

            let new_corner_orient = self
                .move_raw_corner_orient
                .domino_conjugate(moved_corner_orient, conj);

            SymReducedPhase1Repr::from_coords(
                new_corner_orient,
                new_edge_group_orient,
                new_e_edge_perm,
                new_ud_edge_perm,
                new_corner_perm,
                new_corner_perm_correction,
            )
        })
    }

    pub fn phase_change(&self, cube: SymReducedPhase1Repr) -> Result<SymReducedPhase2Repr, SymReducedPhase1Repr> {
        if 0x0FFFFFFF00000000 & cube.0 == 0 {
            let conj = cube.corner_perm_sym_correction().inverse();

            let (ud_edge_perm, corner_perm) = cube.ud_edge_and_corner_perm_coords();
            let e_edge_perm = cube.e_edge_perm_coord();

            let (new_ud_edge_perm, new_e_edge_perm) = self.grouped_edge_moves.update_edge_perm_phase_2_domino_symmetry(conj, ud_edge_perm, e_edge_perm);

            Ok(SymReducedPhase2Repr::from_coords(new_e_edge_perm, new_ud_edge_perm, corner_perm))
        } else {
            Err(cube)
        }
    }

    pub fn phase_2_adjacent(
        &self,
        cube: SymReducedPhase2Repr,
    ) -> impl IntoIterator<Item = SymReducedPhase2Repr> {
        let (ud_edge_perm, corner_perm) = cube.ud_edge_and_corner_perm_coords();
        let e_edge_perm = cube.e_edge_perm_coord();

        DominoMove::all_iter().map(move |mv| {
            let (new_corner_perm, conj) = self
                .move_sym_corner_perm
                .apply_cube_move(corner_perm, mv.into());

            let (moved_ud_edge_perm, moved_e_edge_perm) = self
                .grouped_edge_moves
                .update_edge_perm_phase_2_domino_move(mv, ud_edge_perm, e_edge_perm);

            let (new_ud_edge_perm, new_e_edge_perm) = self
                .grouped_edge_moves
                .update_edge_perm_phase_2_domino_symmetry(
                    conj,
                    moved_ud_edge_perm,
                    moved_e_edge_perm,
                );

            SymReducedPhase2Repr::from_coords(new_e_edge_perm, new_ud_edge_perm, new_corner_perm)
        })
    }
}

#[test]
fn gen_tables() -> anyhow::Result<()> {
    let _ = Tables::new()?;

    Ok(())
}

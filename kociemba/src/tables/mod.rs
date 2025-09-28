use lookup_sym_corner_perm::LookupSymCornerPermTable;
use lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use move_raw_corner_orient::MoveRawCornerOrientTable;
use move_raw_corner_perm::MoveRawCornerPermTable;
use move_raw_e_edge_perm::MoveRawEEdgePermTable;
use move_raw_ud_edge_perm::MoveRawUDEdgePermTable;
use move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;
use prune_phase_1::PrunePhase1Table;

use crate::tables::grouped_edge_moves::GroupedEdgeMovesTable;

pub mod lookup_sym_corner_perm;
pub mod lookup_sym_edge_group_orient;
pub mod move_raw_corner_orient;
pub mod move_raw_corner_perm;
pub mod move_raw_e_edge_perm;
pub mod move_raw_ud_edge_perm;

pub mod move_sym_edge_group_orient;
// pub mod move_raw_edge_group_flip;

// pub mod move_sym_edge_group_flip;

// pub mod move_sym_corner_perm;

pub mod prune_phase_1;

pub mod grouped_edge_moves;

mod table_loader;

pub struct Tables {
    // phase 1 tables
    pub move_raw_corner_orient: MoveRawCornerOrientTable,
    pub move_raw_corner_perm: MoveRawCornerPermTable,
    pub move_sym_edge_group_orient: MoveSymEdgeGroupOrientTable,
    pub lookup_sym_edge_group_orient: LookupSymEdgeGroupOrientTable,
    pub prune_phase_1: PrunePhase1Table,
    
    // multi phase tables
    pub grouped_edge_moves: GroupedEdgeMovesTable,
    
    // phase 2 tables
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
}

#[test]
fn gen_tables() -> anyhow::Result<()> {
    let _ = Tables::new()?;

    Ok(())
}

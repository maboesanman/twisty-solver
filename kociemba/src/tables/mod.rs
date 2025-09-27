use lookup_sym_corner_perm::LookupSymCornerPermTable;
use lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use move_raw_corner_orient::MoveRawCornerOrientTable;
use move_raw_corner_perm::MoveRawCornerPermTable;
use move_raw_e_edge_perm::MoveRawEEdgePermTable;
use move_raw_ud_edge_perm::MoveRawUDEdgePermTable;
use move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;
use prune_phase_1::PrunePhase1Table;

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
    // raw coord complete move tables
    pub move_raw_corner_orient: MoveRawCornerOrientTable,
    // pub move_raw_corner_perm: MoveRawCornerPermTable,

    // raw coord domino move tables
    // pub move_raw_ud_edge_perm: MoveRawUDEdgePermTable,
    // pub move_raw_e_edge_perm: MoveRawEEdgePermTable,

    // // sym coord move tables
    pub move_sym_edge_group_orient: MoveSymEdgeGroupOrientTable,

    // sym coord lookup tables
    pub lookup_sym_edge_group_orient: LookupSymEdgeGroupOrientTable,
    // pub lookup_sym_corner_perm: LookupSymCornerPermTable,
    // prune tables
    pub prune_phase_1: PrunePhase1Table,
}

impl Tables {
    pub fn new() -> anyhow::Result<Self> {
        let move_raw_corner_orient =
            MoveRawCornerOrientTable::load("move_raw_corner_orient_table.dat")?;
        let lookup_sym_edge_group_orient =
            LookupSymEdgeGroupOrientTable::load("lookup_sym_edge_group_orient_table.dat")?;
        let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
            "move_sym_edge_group_orient_table.dat",
            &lookup_sym_edge_group_orient,
        )?;
        let prune_phase_1 = PrunePhase1Table::load(
            "prune_phase_1_table.dat",
            &move_sym_edge_group_orient,
            &move_raw_corner_orient,
        )?;

        Ok(Self {
            move_raw_corner_orient,
            move_sym_edge_group_orient,
            lookup_sym_edge_group_orient,
            prune_phase_1,
        })
    }
}

#[test]
fn gen_tables() -> anyhow::Result<()> {
    let _ = Tables::new()?;

    Ok(())
}

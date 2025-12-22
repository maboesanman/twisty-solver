use std::{fs::create_dir_all, mem::MaybeUninit, path::Path};

use lookup_sym_corner_perm::LookupSymCornerPermTable;
use lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use move_raw_corner_orient::MoveRawCornerOrientTable;
use move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;

use crate::kociemba::tables::{
    move_edge_positions::MoveEdgePositions, move_raw_e_edge_perm::MoveRawEEdgePermTable, move_raw_ud_edge_perm::MoveRawUDEdgePermTable, move_sym_corner_perm::MoveSymCornerPermTable, prune_phase_1::PrunePhase1Table, prune_phase_2::PrunePhase2Table
};

pub mod lookup_sym_corner_perm;
pub mod lookup_sym_edge_group_orient;

pub mod move_raw_corner_orient;
pub mod move_sym_corner_perm;
pub mod move_sym_edge_group_orient;
pub mod move_edge_positions;
pub mod move_raw_e_edge_perm;
pub mod move_raw_ud_edge_perm;

pub mod prune_phase_1;
pub mod prune_phase_2;

mod table_loader;

const MOVE_RAW_CORNER_ORIENT_TABLE_NAME: &str = "move_raw_corner_orient_table.dat";
const MOVE_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "move_sym_edge_group_orient_table.dat";
const LOOKUP_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "lookup_sym_edge_group_orient_table.dat";
const LOOKUP_SYM_CORNER_PERM_TABLE_NAME: &str = "lookup_sym_corner_perm_table.dat";
const PRUNE_PHASE_1_TABLE_NAME: &str = "prune_phase_1_table.dat";
const MOVE_E_EDGE_PERM_TABLE_NAME: &str = "move_raw_e_edge_perm.dat";
const MOVE_UD_EDGE_PERM_TABLE_NAME: &str = "move_raw_ud_edge_perm.dat";
const MOVE_SYM_CORNER_PERM_TABLE_NAME: &str = "move_sym_corner_perm_table.dat";
const PRUNE_PHASE_2_TABLE_NAME: &str = "prune_phase_2_table.dat";
const MOVE_EDGE_POSITION_TABLE_NAME: &str = "move_raw_edge_position_table.dat";

pub struct Tables {
    pub(crate) lookup_sym_edge_group_orient: LookupSymEdgeGroupOrientTable,
    pub(crate) lookup_sym_corner_perm: LookupSymCornerPermTable,

    pub(crate) move_raw_corner_orient: MoveRawCornerOrientTable,
    pub(crate) move_sym_edge_group_orient: MoveSymEdgeGroupOrientTable,
    pub(crate) move_sym_corner_perm: MoveSymCornerPermTable,
    pub(crate) move_edge_position: MoveEdgePositions,
    pub(crate) move_raw_e_edge_perm: MoveRawEEdgePermTable,
    pub(crate) move_raw_ud_edge_perm: MoveRawUDEdgePermTable,

    pub(crate) prune_phase_1: MaybeUninit<PrunePhase1Table>,
    pub(crate) prune_phase_2: MaybeUninit<PrunePhase2Table>,
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
        let lookup_sym_corner_perm =
            LookupSymCornerPermTable::load(folder.join(LOOKUP_SYM_CORNER_PERM_TABLE_NAME))?;
        let move_sym_corner_perm = MoveSymCornerPermTable::load(
            folder.join(MOVE_SYM_CORNER_PERM_TABLE_NAME),
            &lookup_sym_corner_perm,
        )?;

        let move_edge_position = MoveEdgePositions::load(folder.join(MOVE_EDGE_POSITION_TABLE_NAME))?;

        let move_raw_e_edge_perm = MoveRawEEdgePermTable::load(folder.join(MOVE_E_EDGE_PERM_TABLE_NAME))?;
        let move_raw_ud_edge_perm = MoveRawUDEdgePermTable::load(folder.join(MOVE_UD_EDGE_PERM_TABLE_NAME))?;

        let mut working = Tables {
            move_raw_corner_orient,
            move_sym_edge_group_orient,
            lookup_sym_edge_group_orient,
            lookup_sym_corner_perm,
            move_sym_corner_perm,
            move_edge_position,
            move_raw_e_edge_perm,
            move_raw_ud_edge_perm,
            prune_phase_1: MaybeUninit::uninit(),
            prune_phase_2: MaybeUninit::uninit(),
            // prune_phase_2_corner_perm: MaybeUninit::uninit(),
        };

        working.prune_phase_1.write(PrunePhase1Table::load(
            folder.join(PRUNE_PHASE_1_TABLE_NAME),
            &working,
        )?);

        working.prune_phase_2.write(PrunePhase2Table::load(
            folder.join(PRUNE_PHASE_2_TABLE_NAME),
            &working,
        )?);

        Ok(working)
    }

    pub(crate) fn get_prune_phase_1(&self) -> &PrunePhase1Table {
        unsafe { self.prune_phase_1.assume_init_ref() }
    }

    pub(crate) fn get_prune_phase_2(&self) -> &PrunePhase2Table {
        unsafe { self.prune_phase_2.assume_init_ref() }
    }
}

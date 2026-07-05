use std::{fs::create_dir_all, path::Path};

use lookup_sym_corner_perm::LookupSymCornerPermTable;
use lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use memmap2::Mmap;
use move_raw_corner_orient::MoveRawCornerOrientTable;
use move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;

use crate::kociemba::tables::{
    move_edge_positions::MoveEdgePositionsTable, move_raw_e_edge_perm::MoveRawEEdgePermTable,
    move_raw_ud_edge_perm::MoveRawUDEdgePermTable, move_sym_corner_perm::MoveSymCornerPermTable,
    move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable,
    prune_phase_1::PrunePhase1Table, prune_phase_2::PrunePhase2Table,
};

pub mod lookup_sym_corner_perm;
pub mod lookup_sym_edge_group_orient;

pub mod move_edge_positions;
pub mod move_raw_corner_orient;
pub mod move_raw_e_edge_perm;
pub mod move_raw_ud_edge_perm;
pub mod move_sym_corner_perm;
pub mod move_sym_corner_perm_augmented;
pub mod move_sym_edge_group_orient;

pub mod prune_phase_1;
pub mod prune_phase_2;

// pub mod permuted_coordinates;

// pub mod permute_sym_edge_group_orient;

mod table_loader;

const MOVE_RAW_CORNER_ORIENT_TABLE_NAME: &str = "move_raw_corner_orient_table.dat";
const MOVE_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "move_sym_edge_group_orient_table.dat";
const LOOKUP_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "lookup_sym_edge_group_orient_table.dat";
const LOOKUP_SYM_CORNER_PERM_TABLE_NAME: &str = "lookup_sym_corner_perm_table.dat";
const PRUNE_PHASE_1_TABLE_NAME: &str = "prune_phase_1_table.dat";
const MOVE_E_EDGE_PERM_TABLE_NAME: &str = "move_raw_e_edge_perm.dat";
const MOVE_UD_EDGE_PERM_TABLE_NAME: &str = "move_raw_ud_edge_perm.dat";
const MOVE_SYM_CORNER_PERM_TABLE_NAME: &str = "move_sym_corner_perm_table.dat";
const MOVE_SYM_CORNER_PERM_TABLE_AUGMENTED_NAME: &str = "move_sym_corner_perm_table_augmented.dat";
const PRUNE_PHASE_2_TABLE_NAME: &str = "prune_phase_2_table.dat";
const PRUNE_PHASE_2_CORNER_SYM_TABLE_NAME: &str = "prune_phase_2_table_corner_sym.dat";
const MOVE_EDGE_POSITION_TABLE_NAME: &str = "move_raw_edge_position_table.dat";
const PERMUTE_SYM_EDGE_GROUP_ORIENT_TABLE_NAME: &str = "permute_sym_edge_group_orient_table.dat";
const PERMUTE_RAW_CORNER_ORIENT_TABLE_NAME: &str = "permute_raw_corner_orient_table.dat";

struct MovesPreTables {
    lookup_sym_edge_group_orient: Mmap,
    lookup_sym_corner_perm: Mmap,

    move_raw_corner_orient: Mmap,
    move_sym_edge_group_orient: Mmap,
    move_sym_corner_perm: Mmap,
    move_edge_position: Mmap,
    move_raw_e_edge_perm: Mmap,
    move_raw_ud_edge_perm: Mmap,
}

struct PrunePreTables {
    moves_pre_table: MovesPreTables,

    prune_phase_1: Mmap,
    prune_phase_2: Mmap,
}

pub struct Tables {
    prune_pre_tables: PrunePreTables,
    corner_sym_augmented: Mmap,
}

#[rustfmt::skip]
mod unformatted {
    use crate::kociemba::tables::move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable;

use super::*;

    impl AsRef<LookupSymCornerPermTable> for MovesPreTables { fn as_ref(&self) -> &LookupSymCornerPermTable { unsafe { LookupSymCornerPermTable::from_buffer(&self.lookup_sym_corner_perm) } } }
    impl AsRef<LookupSymEdgeGroupOrientTable> for MovesPreTables { fn as_ref(&self) -> &LookupSymEdgeGroupOrientTable { unsafe { LookupSymEdgeGroupOrientTable::from_buffer( &self.lookup_sym_edge_group_orient) } } }
    impl AsRef<MoveRawCornerOrientTable> for MovesPreTables { fn as_ref(&self) -> &MoveRawCornerOrientTable { unsafe { MoveRawCornerOrientTable::from_buffer( &self.move_raw_corner_orient) } } }
    impl AsRef<MoveSymEdgeGroupOrientTable> for MovesPreTables { fn as_ref(&self) -> &MoveSymEdgeGroupOrientTable { unsafe { MoveSymEdgeGroupOrientTable::from_buffer( &self.move_sym_edge_group_orient) } } }
    impl AsRef<MoveSymCornerPermTable> for MovesPreTables { fn as_ref(&self) -> &MoveSymCornerPermTable { unsafe { MoveSymCornerPermTable::from_buffer( &self.move_sym_corner_perm) } } }
    impl AsRef<MoveEdgePositionsTable> for MovesPreTables { fn as_ref(&self) -> &MoveEdgePositionsTable { unsafe { MoveEdgePositionsTable::from_buffer( &self.move_edge_position) } } }
    impl AsRef<MoveRawEEdgePermTable> for MovesPreTables { fn as_ref(&self) -> &MoveRawEEdgePermTable { unsafe { MoveRawEEdgePermTable::from_buffer( &self.move_raw_e_edge_perm) } } }
    impl AsRef<MoveRawUDEdgePermTable> for MovesPreTables { fn as_ref(&self) -> &MoveRawUDEdgePermTable { unsafe { MoveRawUDEdgePermTable::from_buffer( &self.move_raw_ud_edge_perm) } } }

    impl AsRef<LookupSymEdgeGroupOrientTable> for PrunePreTables { fn as_ref(&self) -> &LookupSymEdgeGroupOrientTable { self.moves_pre_table.as_ref() } }
    impl AsRef<LookupSymCornerPermTable> for PrunePreTables { fn as_ref(&self) -> &LookupSymCornerPermTable { self.moves_pre_table.as_ref() } }
    impl AsRef<MoveRawCornerOrientTable> for PrunePreTables { fn as_ref(&self) -> &MoveRawCornerOrientTable { self.moves_pre_table.as_ref() } }
    impl AsRef<MoveSymEdgeGroupOrientTable> for PrunePreTables { fn as_ref(&self) -> &MoveSymEdgeGroupOrientTable { self.moves_pre_table.as_ref() } }
    impl AsRef<MoveSymCornerPermTable> for PrunePreTables { fn as_ref(&self) -> &MoveSymCornerPermTable { self.moves_pre_table.as_ref() } }
    impl AsRef<MoveEdgePositionsTable> for PrunePreTables { fn as_ref(&self) -> &MoveEdgePositionsTable { self.moves_pre_table.as_ref() } }
    impl AsRef<MoveRawEEdgePermTable> for PrunePreTables { fn as_ref(&self) -> &MoveRawEEdgePermTable { self.moves_pre_table.as_ref() } }
    impl AsRef<MoveRawUDEdgePermTable> for PrunePreTables { fn as_ref(&self) -> &MoveRawUDEdgePermTable { self.moves_pre_table.as_ref() } }
    impl AsRef<PrunePhase1Table> for PrunePreTables { fn as_ref(&self) -> &PrunePhase1Table { unsafe { PrunePhase1Table::from_buffer( &self.prune_phase_1) } } }
    impl AsRef<PrunePhase2Table> for PrunePreTables { fn as_ref(&self) -> &PrunePhase2Table { unsafe { PrunePhase2Table::from_buffer( &self.prune_phase_2) } } }

    impl AsRef<LookupSymEdgeGroupOrientTable> for Tables { fn as_ref(&self) -> &LookupSymEdgeGroupOrientTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<LookupSymCornerPermTable> for Tables { fn as_ref(&self) -> &LookupSymCornerPermTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveRawCornerOrientTable> for Tables { fn as_ref(&self) -> &MoveRawCornerOrientTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveSymEdgeGroupOrientTable> for Tables { fn as_ref(&self) -> &MoveSymEdgeGroupOrientTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveSymCornerPermTable> for Tables { fn as_ref(&self) -> &MoveSymCornerPermTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveEdgePositionsTable> for Tables { fn as_ref(&self) -> &MoveEdgePositionsTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveRawEEdgePermTable> for Tables { fn as_ref(&self) -> &MoveRawEEdgePermTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveRawUDEdgePermTable> for Tables { fn as_ref(&self) -> &MoveRawUDEdgePermTable { self.prune_pre_tables.as_ref() } }
    impl AsRef<PrunePhase1Table> for Tables { fn as_ref(&self) -> &PrunePhase1Table { self.prune_pre_tables.as_ref() } }
    impl AsRef<PrunePhase2Table> for Tables { fn as_ref(&self) -> &PrunePhase2Table { self.prune_pre_tables.as_ref() } }
    impl AsRef<MoveSymCornerPermAugmentedTable> for Tables { fn as_ref(&self) -> &MoveSymCornerPermAugmentedTable { unsafe { MoveSymCornerPermAugmentedTable::from_buffer(&self.corner_sym_augmented) } } }
}

impl MovesPreTables {
    fn new<P>(folder: P) -> anyhow::Result<Self>
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
            unsafe { LookupSymEdgeGroupOrientTable::from_buffer(&lookup_sym_edge_group_orient) },
        )?;
        let lookup_sym_corner_perm =
            LookupSymCornerPermTable::load(folder.join(LOOKUP_SYM_CORNER_PERM_TABLE_NAME))?;
        let move_sym_corner_perm =
            MoveSymCornerPermTable::load(folder.join(MOVE_SYM_CORNER_PERM_TABLE_NAME), unsafe {
                LookupSymCornerPermTable::from_buffer(&lookup_sym_corner_perm)
            })?;

        let move_edge_position =
            MoveEdgePositionsTable::load(folder.join(MOVE_EDGE_POSITION_TABLE_NAME))?;

        let move_raw_e_edge_perm =
            MoveRawEEdgePermTable::load(folder.join(MOVE_E_EDGE_PERM_TABLE_NAME))?;
        let move_raw_ud_edge_perm =
            MoveRawUDEdgePermTable::load(folder.join(MOVE_UD_EDGE_PERM_TABLE_NAME))?;

        Ok(Self {
            lookup_sym_edge_group_orient,
            lookup_sym_corner_perm,
            move_raw_corner_orient,
            move_sym_edge_group_orient,
            move_sym_corner_perm,
            move_edge_position,
            move_raw_e_edge_perm,
            move_raw_ud_edge_perm,
        })
    }
}

impl PrunePreTables {
    fn new<P>(folder: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let folder = folder.as_ref();
        let moves_pre_table = MovesPreTables::new(folder)?;

        let prune_phase_1 =
            PrunePhase1Table::load(folder.join(PRUNE_PHASE_1_TABLE_NAME), &moves_pre_table)?;

        let prune_phase_2 =
            PrunePhase2Table::load(folder.join(PRUNE_PHASE_2_TABLE_NAME), &moves_pre_table)?;

        Ok(Self {
            moves_pre_table,
            prune_phase_1,
            prune_phase_2,
        })
    }
}

impl std::fmt::Debug for Tables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tables").finish()
    }
}

impl Tables {
    pub fn new<P>(folder: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let folder = folder.as_ref();
        let prune_pre_tables = PrunePreTables::new(folder)?;
        let corner_sym_augmented = MoveSymCornerPermAugmentedTable::load(
            folder.join(MOVE_SYM_CORNER_PERM_TABLE_AUGMENTED_NAME),
            prune_pre_tables.as_ref(),
            prune_pre_tables.as_ref(),
        )?;

        Ok(Self {
            prune_pre_tables,
            corner_sym_augmented,
        })
    }
}

#[test]
fn generate() -> anyhow::Result<()> {
    let _tables = Tables::new("tables")?;

    Ok(())
}

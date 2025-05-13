// use lookup_sym_corner_perm::LookupSymCornerPermTable;
// use lookup_sym_edge_group_flip::LookupSymEdgeGroupFlipTable;
// use move_raw_corner_orient::MoveRawCornerOrientTable;
// use move_raw_corner_perm::MoveRawCornerPermTable;
// use move_raw_edge_group_flip::MoveRawEdgeGroupFlipTable;
// use move_sym_corner_perm::MoveSymCornerPermTable;
// use move_sym_edge_group_flip::MoveSymEdgeGroupFlipTable;
// use part_move_raw_e_edge_perm::PartMoveRawEEdgePermTable;
// use part_move_raw_ud_edge_perm::PartMoveRawUDEdgePermTable;
// use prune_phase_1::PrunePhase1Table;

pub mod lookup_sym_corner_perm;
pub mod lookup_sym_edge_group_orient;
pub mod move_raw_corner_orient;
pub mod move_raw_corner_perm;
pub mod move_raw_e_edge_perm;
pub mod move_raw_ud_edge_perm;

// pub mod move_raw_edge_group_flip;

// pub mod move_sym_edge_group_flip;

// pub mod move_sym_corner_perm;

// pub mod prune_phase_1;
pub mod kociemba_tables;

mod table_loader;

pub struct Tables {
    // // raw coord complete move tables
    // pub move_raw_edge_group_flip: Option<MoveRawEdgeGroupFlipTable>,
    // pub move_raw_corner_orient: Option<MoveRawCornerOrientTable>,
    // pub move_raw_corner_perm: Option<MoveRawCornerPermTable>,

    // // raw coord partial move tables (only phase 2 moves)
    // pub part_move_raw_ud_edge_perm: Option<PartMoveRawUDEdgePermTable>,
    // pub part_move_raw_e_edge_perm: Option<PartMoveRawEEdgePermTable>,

    // // sym coord move tables
    // pub move_sym_edge_group_flip: Option<MoveSymEdgeGroupFlipTable>,
    // pub move_sym_corner_perm: Option<MoveSymCornerPermTable>,

    // // sym coord lookup tables
    // pub lookup_sym_edge_group_flip: Option<LookupSymEdgeGroupFlipTable>,
    // pub lookup_sym_corner_perm: Option<LookupSymCornerPermTable>,

    // prune tables
    // pub prune_phase_1: PrunePhase1Table,
}

impl Tables {
    pub fn new() -> anyhow::Result<Self> {
        // let dir = Path::new("./tables");
        // create_dir_all(dir)?;

        // // edge group flip coords
        // let move_raw_edge_group_flip =
        //     MoveRawEdgeGroupFlipTable::load(dir.join("move_raw_edge_group_flip_table.dat"))?;
        // let lookup_sym_edge_group_flip =
        //     LookupSymEdgeGroupFlipTable::load(dir.join("lookup_sym_edge_group_flip_table.dat"), &move_raw_edge_group_flip)?;
        // let move_sym_edge_group_flip = MoveSymEdgeGroupFlipTable::load(
        //     dir.join("move_sym_edge_group_flip_table.dat"),
        //     &lookup_sym_edge_group_flip,
        //     &move_raw_edge_group_flip,
        // )?;

        // // corner perm coords
        // let move_raw_corner_perm =
        //     MoveRawCornerPermTable::load(dir.join("move_raw_corner_perm_table.dat"))?;
        // let lookup_sym_corner_perm =
        //     LookupSymCornerPermTable::load(dir.join("lookup_sym_corner_perm_table.dat"), &move_raw_corner_perm)?;
        // let move_sym_corner_perm = MoveSymCornerPermTable::load(
        //     dir.join("move_sym_corner_perm_table.dat"),
        //     &lookup_sym_corner_perm,
        //     &move_raw_corner_perm,
        // )?;

        // // other coords
        // let move_raw_corner_orient =
        //     MoveRawCornerOrientTable::load(dir.join("move_raw_corner_orient_table.dat"))?;
        // let part_move_raw_ud_edge_perm =
        //     PartMoveRawUDEdgePermTable::load(dir.join("part_move_raw_ud_edge_perm_table.dat"))?;
        // let part_move_raw_e_edge_perm =
        //     PartMoveRawEEdgePermTable::load(dir.join("part_move_raw_e_edge_perm_table.dat"))?;

        Ok(Self {
            // move_raw_edge_group_flip: None,
            // move_raw_corner_orient: None,
            // move_raw_corner_perm: None,
            // part_move_raw_ud_edge_perm: None,
            // part_move_raw_e_edge_perm: None,
            // move_sym_edge_group_flip: None,
            // move_sym_corner_perm: None,
            // lookup_sym_edge_group_flip: None,
            // lookup_sym_corner_perm: None,
        })
    }
}

#[test]
fn gen_tables() -> anyhow::Result<()> {
    let _ = Tables::new()?;

    Ok(())
}

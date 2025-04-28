use anyhow::Result;
use kociemba::tables::{
    move_table_raw_corner_orient::load_corner_orient_move_table,
    move_table_raw_corner_perm::load_corner_perm_move_table,
    move_table_raw_e_edge_perm::load_e_edge_perm_move_table,
    move_table_raw_edge_grouping::load_edge_grouping_move_table,
    move_table_raw_edge_orient::load_edge_orient_move_table,
    move_table_raw_ud_edge_perm::load_ud_edge_perm_move_table,
    sym_lookup_phase_1_edge::load_phase_1_edge_sym_lookup_table,
    sym_lookup_phase_2_corner::load_phase_2_corner_sym_lookup_table,
};

pub fn main() -> Result<()> {
    let _t1 = load_corner_orient_move_table("corner_orient_move_table.dat")?;
    let _t2 = load_edge_orient_move_table("edge_orient_move_table.dat")?;
    let _t3 = load_edge_grouping_move_table("edge_grouping_move_table.dat")?;
    let t4 = load_corner_perm_move_table("corner_perm_move_table.dat")?;
    let _t5 = load_ud_edge_perm_move_table("ud_edge_perm_move_table.dat")?;
    let _t6 = load_e_edge_perm_move_table("e_edge_perm_move_table.dat")?;

    let _t7 = load_phase_1_edge_sym_lookup_table("phase_1_edge_sym_lookup_table.dat")?;
    let _t8 = load_phase_2_corner_sym_lookup_table("phase_2_corner_sym_lookup_table.dat", &t4)?;

    Ok(())
}

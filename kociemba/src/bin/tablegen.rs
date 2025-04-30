use anyhow::Result;
use kociemba::tables::{
    move_table_edge_group_and_orient::load_edge_group_and_orient_move_table, move_table_raw_corner_orient::load_corner_orient_move_table, move_table_raw_corner_perm::load_corner_perm_move_table, move_table_raw_e_edge_perm::load_e_edge_perm_move_table, move_table_raw_ud_edge_perm::load_ud_edge_perm_move_table, move_table_sym_phase_1_edge::load_phase_1_edge_sym_move_table, move_table_sym_phase_2_corner::load_phase_2_corner_sym_move_table, sym_lookup_phase_1_edge::load_phase_1_edge_sym_lookup_table, sym_lookup_phase_2_corner::load_phase_2_corner_sym_lookup_table
};

pub fn main() -> Result<()> {
    let phase_1_move_edge_raw_table =
        load_edge_group_and_orient_move_table("edge_group_and_orient_move_table.dat")?;
    let phase_1_move_corner_raw_table =
        load_corner_orient_move_table("corner_orient_move_table.dat")?;
    let phase_1_lookup_edge_sym_table = load_phase_1_edge_sym_lookup_table(
        "phase_1_edge_sym_lookup_table.dat",
        &phase_1_move_edge_raw_table,
    )?;
    let phase_1_move_edge_sym_table = load_phase_1_edge_sym_move_table(
        "phase_1_edge_sym_move_table.dat",
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
    );

    let phase_2_move_corner_raw_table = load_corner_perm_move_table("corner_perm_move_table.dat")?;
    let phase_2_move_e_edge_raw_table = load_e_edge_perm_move_table("e_edge_perm_move_table.dat")?;
    let phase_2_move_ud_edge_raw_table =
        load_ud_edge_perm_move_table("ud_edge_perm_move_table.dat")?;
    let phase_2_lookup_corner_sym_table = load_phase_2_corner_sym_lookup_table(
        "phase_2_corner_sym_lookup_table.dat",
        &phase_2_move_corner_raw_table,
    )?;
    let phase_2_move_corner_sym_table = load_phase_2_corner_sym_move_table("phase_2_corner_sym_move_table.dat", &phase_2_lookup_corner_sym_table, &phase_2_move_corner_raw_table);

    // calc coord for phase 1
    let _ = phase_1_lookup_edge_sym_table;

    // coord in phase 1
    let _ = (phase_1_move_corner_raw_table, phase_1_move_edge_sym_table);

    // calc coord for phase 2
    let _ = phase_2_lookup_corner_sym_table;

    // coord in phase 2
    let _ = (
        phase_2_move_ud_edge_raw_table,
        phase_2_move_e_edge_raw_table,
        phase_2_move_corner_sym_table,
    );

    // let t7 = load_phase_1_edge_sym_lookup_table("phase_1_edge_sym_lookup_table.dat", &t2, &t3)?;
    // let _t8 = load_phase_2_corner_sym_lookup_table("phase_2_corner_sym_lookup_table.dat", &t4)?;

    // let _t9 = load_phase_1_edge_move_table("phase_1_edge_move_table.dat", &t7, &t2, &t3);

    Ok(())
}

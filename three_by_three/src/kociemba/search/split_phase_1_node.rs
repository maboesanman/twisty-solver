use std::{num::NonZeroU8, simd::{Simd, num::SimdUint}};

use crate::{
    CornerOrient, CubeMove, Tables, cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry}, kociemba::{
        coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord}, partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions,
        }, search::phase_1_node::Phase1Node, tables::{
            lookup_sym_corner_orient::{CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable}, move_edge_positions::MoveEdgePositionsTable, move_raw_corner_orient::MoveRawCornerOrientTable, move_raw_corner_orient_trits::MoveRawCornerOrientTritsTable, move_sym_corner_orient::MoveSymCornerOrientTable, move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable, move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable, prune_phase_1::PrunePhase1Table, prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table
        },
    }, simd_utils::{combine, gather16, lut_u8x16x2, lut_u16x16x3}
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeA {
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,
    pub corner_perm_sym: CornerPermSymCoord,
    pub corner_orient_sym: CornerOrientSymCoord,
    pub corner_orient_correct: DominoSymmetry,
    pub combined_phase_1_dist_previous_axis: u8,

    /// ego_hi, co_lo
    pub combined_sym_correct: u8,
}

const SPLIT_A_ZERO: SplitPhase1NodeA = SplitPhase1NodeA {
    edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
    corner_perm_sym: CornerPermSymCoord(0),
    corner_orient_sym: CornerOrientSymCoord(0),
    corner_orient_correct: DominoSymmetry::IDENTITY,
    combined_phase_1_dist_previous_axis: 0,
    combined_sym_correct: 0,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeB {
    pub u_edge_positions: UEdgePositions,
    pub d_edge_positions: DEdgePositions,
    pub e_edge_positions: EEdgePositions,
    pub previous_axis: CubePreviousAxis,
}

// a set of nodes which came from the same call to `produce_next_nodes`
pub struct Phase1FrameMetadata<I> {
    pub children: I,
}

#[inline(always)]
pub fn split(node: Phase1Node ,tables: &Tables) -> (SplitPhase1NodeA, SplitPhase1NodeB) {
    let dist = node.distance_heuristic(tables);
    let Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct,
        corner_perm_correct,
        corner_perm_sym,
        corner_orient_combo,
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = node;
    (
        SplitPhase1NodeA {
            edge_group_orient_sym,
            corner_perm_sym,
            combined_phase_1_dist_previous_axis: (previous_axis as u8 | (dist << 4)),
            combined_sym_correct: (edge_group_orient_correct.0 << 4) | corner_perm_correct.0,
            corner_orient_sym: corner_orient_combo.sym_coord,
            corner_orient_correct: corner_orient_combo.domino_conjugation,
        },
        SplitPhase1NodeB {
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis,
        },
    )
}

#[inline(always)]
pub fn merge(a: SplitPhase1NodeA, b: SplitPhase1NodeB ,tables: &Tables) -> Phase1Node {
    let SplitPhase1NodeA {
        edge_group_orient_sym,
        corner_perm_sym,
        corner_orient_sym,
        corner_orient_correct,
        combined_phase_1_dist_previous_axis,
        combined_sym_correct,
    } = a;
    let SplitPhase1NodeB {
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = b;
    let lookup_table: &LookupSymCornerOrientTable = tables.as_ref();
    Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct: DominoSymmetry(combined_sym_correct >> 4),
        corner_perm_correct: DominoSymmetry(combined_sym_correct & 0xF),
        corner_perm_sym,
        corner_orient_combo: CornerOrientComboCoord {
            sym_coord: corner_orient_sym,
            domino_conjugation: corner_orient_correct,
        },
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    }
}

const SYM_CONJ: [[u8; 32]; 16] = {
    let mut t = [[0u8; 32]; 16];
    let mut s = 0;
    while s < 16 {
        let mut m = 0;
        while m < 18 {
            let mv = unsafe { core::mem::transmute::<u8, CubeMove>(m as u8) };
            t[s][m as usize] = mv.domino_conjugate(DominoSymmetry(s as u8)) as u8;
            m += 1;
        }
        s += 1;
    }
    t
};

const N_AXIS: usize = 11; // U..=NoneAlt

const AXIS_AFTER: [[[u8; 32]; N_AXIS]; 2] = {
    let mut t = [[[0u8; 32]; N_AXIS]; 2];
    let mut r = 0;
    while r < 2 {
        // r == 1 => the child computation sees remaining_moves == 1
        // r == 0 => any non-1 value behaves identically; 2 is representative
        let rem_arg = if r == 1 { 1u8 } else { 2u8 };
        let mut p = 0;
        while p < N_AXIS {
            let prev = unsafe { core::mem::transmute::<u8, CubePreviousAxis>(p as u8) };
            let mut m = 0;
            while m < 18 {
                let mv = unsafe { core::mem::transmute::<u8, CubeMove>(m as u8) };
                t[r][p][m] = prev.update_with_new_move(mv, rem_arg) as u8;
                m += 1;
            }
            p += 1;
        }
        r += 1;
    }
    t
};

const DISTANCE_OPTIONS_LOOKUP: [[u8; 32]; 13] = const {
    let mut out = [[0; 32]; 13];

    let mut d = 0usize;
    while d < 13 {
        let mut i = -1i8;
        while i < 2 {
            let mut new_d = (d as i8) + i;
            if new_d < 0 {
                new_d = 0;
            }
            if new_d > 12 {
                new_d = 12;
            }

            let new_d = new_d as usize;

            out[d][new_d % 3] = new_d as u8;

            i += 1;
        }

        d += 1;
    }

    out
};

#[inline(always)]
fn axis_after_table(prev: CubePreviousAxis, moves_remaining: NonZeroU8) -> &'static [u8; 32] {
    let penultimate = (moves_remaining.get() == 2) as usize;
    &AXIS_AFTER[penultimate][prev as usize]
}

impl SplitPhase1NodeA {
    #[inline(always)]
    pub fn is_domino_reduced(self) -> bool {
        self.corner_orient_sym.0 == 0 && self.edge_group_orient_sym.0 == 0
    }

    #[inline(always)]
    pub fn distance_heuristic(
        self,
        // tables: impl AsRef<MoveRawCornerOrientTable> + AsRef<PrunePhase1Table>,
    ) -> u8 {
        self.combined_phase_1_dist_previous_axis >> 4
        // let move_table: &MoveRawCornerOrientTable = tables.as_ref();
        // let prune_table: &PrunePhase1Table = tables.as_ref();

        // let corner_orient_adjusted = move_table.domino_conjugate(
        //     self.corner_orient_raw,
        //     DominoSymmetry(self.combined_sym_correct >> 4),
        // );

        // prune_table.get_value(self.edge_group_orient_sym, corner_orient_adjusted)
    }

    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline(never)]
    pub fn produce_next_nodes_a(
        slice: &mut [Self; 16],
        moves_remaining: NonZeroU8,
        tables: &Tables,
    ) -> usize {
        let start_node = slice[0];

        let ego_mv_tbl: &MoveSymEdgeGroupOrientTable = tables.as_ref();
        let cp_mv_tbl: &MoveSymCornerPermAugmentedTable = tables.as_ref();
        let co_mv_tbl: &MoveSymCornerOrientTable = tables.as_ref();
        let prune_tbl: &PrunePhase1DoubleSymMod3Table = tables.as_ref();

        let ego_row = ego_mv_tbl.row(start_node.edge_group_orient_sym);
        let cp_row = cp_mv_tbl.row(start_node.corner_perm_sym);
        let co_row = co_mv_tbl.row(start_node.corner_orient_sym);

        let previous_axis = unsafe { core::mem::transmute(start_node.combined_phase_1_dist_previous_axis & 0xF) };
        let phase_1_distance = start_node.combined_phase_1_dist_previous_axis >> 4;

        let (mv, num_moves) = CubeMove::new_axis_move_array_raw(previous_axis);

        let ego_i = lut_u8x16x2(
            unsafe { SYM_CONJ.get_unchecked((start_node.combined_sym_correct >> 4) as usize) },
            mv,
        );
        let cp_i = lut_u8x16x2(
            unsafe { SYM_CONJ.get_unchecked((start_node.combined_sym_correct & 0xF) as usize) },
            mv,
        );
        let co_i = lut_u8x16x2(
            unsafe { SYM_CONJ.get_unchecked(start_node.corner_orient_correct.0 as usize) },
            mv,
        );
        let ego_c = lut_u16x16x3(ego_row.as_u16_array(), &ego_i);
        let cp_c = lut_u16x16x3(cp_row.as_u16_array(), &cp_i);
        let co_c = lut_u8x16x2(&co_row.as_u8_arrays()[0], &co_i);

        let offset = u128::from_ne_bytes([4u8; 16]);
        let ego_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(ego_i));
        let cp_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(cp_i));
        let co_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(co_i));

        let ego_j = lut_u8x16x2(&ego_row.as_u8_arrays()[1], &ego_i);
        let cp_j = lut_u8x16x2(&cp_row.as_u8_arrays()[1], &cp_i);
        let co_j = lut_u8x16x2(&co_row.as_u8_arrays()[1], &co_i);
        let axis_n = lut_u8x16x2(
            axis_after_table(previous_axis, moves_remaining),
            mv,
        );

        // --- portable, autovec-reliable from here ---
        let a = u128::from_ne_bytes([start_node.combined_sym_correct; 16]);
        let next_cmb_syms = u128::from_ne_bytes(ego_j) << 4 | u128::from_ne_bytes(cp_j);
        let cmb_syms = DominoSymmetry::bitwise_then_32(a, next_cmb_syms);
        
        let a = u128::from_ne_bytes([start_node.corner_orient_correct.0; 16]);
        let next_co_sym = u128::from_ne_bytes(co_j);
        let co_syms = DominoSymmetry::bitwise_then_32(a, next_co_sym);

        let ego_syms = cmb_syms >> 4;
        let ego_syms_inv = DominoSymmetry::bitwise_invert_32(ego_syms);
        let co_lookup = DominoSymmetry::bitwise_then_32(ego_syms_inv, co_syms)
            & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
        let co_lookup_lo = ego_syms ^ co_syms;

        let ego_c_simd = Simd::from_array(ego_c);
        let co_c_simd = Simd::from_array(co_c);
        let co_s_simd = Simd::from_array(u128::to_ne_bytes(co_lookup));
        let co_s_simd_lo = Simd::from_array(u128::to_ne_bytes(co_lookup_lo));

        let new_co_s_lo_arr = (co_s_simd_lo) & Simd::splat(0x03);
        let dist_byte_addr_arr = (ego_c_simd.cast::<u32>() << 2)
            | (new_co_s_lo_arr.cast::<u32>())
            | (co_c_simd.cast::<u32>() << 18);

        let dist_bytes = unsafe { gather16(prune_tbl.as_buffer(), dist_byte_addr_arr) };
        let new_co_s_hi_arr = (co_s_simd >> 2) & Simd::splat(0x03);

        let dist_i = ((dist_bytes >> (new_co_s_hi_arr << 1)) & Simd::splat(0x03)).to_array();
        let distance_options =
            unsafe { DISTANCE_OPTIONS_LOOKUP.get_unchecked(phase_1_distance as usize) };

        // final dists
        let new_dists = Simd::from_array(lut_u8x16x2(&distance_options, &dist_i));
        let new_prev_axis = Simd::from_array(axis_n);
        let combined_dist_axis = (new_dists << 4) | (new_prev_axis);

        let cmb_syms = u128::to_ne_bytes(cmb_syms);
        let co_syms = u128::to_ne_bytes(co_syms);

        let out = combine(
            ego_c, cp_c, co_c,
            co_syms,
            Simd::to_array(combined_dist_axis), 
            cmb_syms,
        );

        for i in 1..16 {
            unsafe {
                slice[i] = core::mem::transmute::<[u8; 8], SplitPhase1NodeA>(*out.get_unchecked(i));
            }
        }
        num_moves
    }
}


impl SplitPhase1NodeB {
    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline(always)]
    pub fn produce_next_node_b(
        self,
        moves_remaining: NonZeroU8,
        // tables: &(
        //      impl AsRef<MoveEdgePositionsTable>
        //      + AsRef<MoveRawCornerOrientTable>
        //      + AsRef<MoveSymCornerPermTable>
        //      + AsRef<MoveSymEdgeGroupOrientTable>
        //      + AsRef<PrunePhase1Table>
        //  ),
        tables: &Tables,
        i: usize,
    ) -> SplitPhase1NodeB {
        let ep_mv_tbl: &MoveEdgePositionsTable = tables.as_ref();

        let u_row = ep_mv_tbl.row(self.u_edge_positions.0);
        let d_row = ep_mv_tbl.row(self.d_edge_positions.0);
        let e_row = ep_mv_tbl.row(self.e_edge_positions.0);

        let (unaltered_move_offsets, _) = CubeMove::new_axis_move_array(self.previous_axis);

        let un_i = unaltered_move_offsets[i] as u8 as usize;
        let new_u_coord = UEdgePositions(EdgePositions(u_row.0[un_i]));
        let new_d_coord = DEdgePositions(EdgePositions(d_row.0[un_i]));
        let new_e_coord = EEdgePositions(EdgePositions(e_row.0[un_i]));

        let new_previous_axis = self
            .previous_axis
            .update_with_new_move(unaltered_move_offsets[i], moves_remaining.get() - 1);

        SplitPhase1NodeB {
            u_edge_positions: new_u_coord,
            d_edge_positions: new_d_coord,
            e_edge_positions: new_e_coord,
            previous_axis: new_previous_axis,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ReprCube;

    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use std::num::NonZeroU8;
    extern crate test;

    // #[test]
    // fn split_produce_next_nodes_matches_unsplit() {
    //     let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
    //     let mut rng = ChaCha8Rng::seed_from_u64(42);

    //     for _ in 0..100 {
    //         let cube: ReprCube =
    //             rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
    //         let moves_remaining = unsafe { NonZeroU8::new_unchecked(30) };

    //         let phase_1 = Phase1Node::from_cube(cube, &tables);
    //         let (split_a, split_b) = split(phase_1, tables);

    //         // Run the unsplit version
    //         let mut unsplit_buf = [phase_1; 16];
    //         let unsplit_count = Phase1Node::produce_next_nodes(&mut unsplit_buf, moves_remaining, tables);

    //         // Run the split A version
    //         let mut split_a_buf = [split_a; 16];
    //         let split_count = SplitPhase1NodeA::produce_next_nodes_a(&mut split_a_buf, moves_remaining, tables);

    //         assert_eq!(unsplit_count, split_count, "produce_next_nodes count mismatch");

    //         // For each child, compute the split B counterpart and merge back
    //         for i in 0..unsplit_count {
    //             let child_unsplit = unsplit_buf[i + 1];
    //             let child_split_a = split_a_buf[i + 1];
    //             let child_split_b = split_b.produce_next_node_b(moves_remaining, tables, i);

    //             let remerged = merge(child_split_a, child_split_b, tables);

    //             assert_eq!(
    //                 child_unsplit.edge_group_orient_sym, remerged.edge_group_orient_sym,
    //                 "ego sym mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.edge_group_orient_correct, remerged.edge_group_orient_correct,
    //                 "ego correct mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.corner_perm_correct, remerged.corner_perm_correct,
    //                 "cp correct mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.corner_perm_sym, remerged.corner_perm_sym,
    //                 "cp sym mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.corner_orient_combo, remerged.corner_orient_combo,
    //                 "co raw mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.u_edge_positions, remerged.u_edge_positions,
    //                 "u edge mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.d_edge_positions, remerged.d_edge_positions,
    //                 "d edge mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.e_edge_positions, remerged.e_edge_positions,
    //                 "e edge mismatch at child {}", i
    //             );
    //             assert_eq!(
    //                 child_unsplit.previous_axis, remerged.previous_axis,
    //                 "prev axis mismatch at child {}", i
    //             );

    //             assert_eq!(
    //                 child_unsplit.distance_heuristic(&tables),
    //                 child_split_a.distance_heuristic(),
    //                 "distance heuristic mismatch at child {}: unsplit={}, split_a={}",
    //                 i,
    //                 child_unsplit.distance_heuristic(&tables),
    //                 child_split_a.distance_heuristic()
    //             );
    //         }
    //     }
    // }

    // #[bench]
    // fn phase_1_next(bench: &mut test::Bencher) {
    //     let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
    //     let mut rng = ChaCha8Rng::seed_from_u64(3);
    //     let cube: ReprCube =
    //         rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
    //     let mut phase_1 = Phase1Node::from_cube(cube, &tables);
    //     phase_1.previous_axis = CubePreviousAxis::B;

    //     let mut buf = [phase_1; 16];

    //     bench.iter(|| {
    //         Phase1Node::produce_next_nodes(
    //             &mut buf,
    //             unsafe { NonZeroU8::new_unchecked(30) },
    //             tables,
    //         )
    //     });
    // }

    #[bench]
    fn phase_1_next_a(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let phase_1 = split(Phase1Node::from_cube(cube, &tables), tables).0;

        let mut buf = [phase_1; 16];

        bench.iter(|| {
            SplitPhase1NodeA::produce_next_nodes_a(
                &mut buf,
                unsafe { NonZeroU8::new_unchecked(30) },
                tables,
            )
        });
    }

    #[bench]
    fn phase_1_next_b(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = split(Phase1Node::from_cube(cube, &tables), tables).1;
        phase_1.previous_axis = CubePreviousAxis::B;

        // let mut buf = [phase_1; 16];

        bench.iter(|| {
            phase_1.produce_next_node_b(unsafe { NonZeroU8::new_unchecked(30) }, tables, 0)
        });
    }
}

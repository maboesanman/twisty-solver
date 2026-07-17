use std::{
    num::NonZeroU8,
    simd::{Simd, num::SimdUint},
};

use crate::{
    CornerOrient, CubeMove, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord},
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions,
        },
        search::phase_1_node::Phase1Node,
        tables::{
            lookup_sym_corner_orient::{
                CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable,
            },
            move_edge_positions::MoveEdgePositionsTable,
            move_raw_corner_orient::MoveRawCornerOrientTable,
            move_raw_corner_orient_trits::MoveRawCornerOrientTritsTable,
            move_sym_corner_orient::MoveSymCornerOrientTable,
            move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
            prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table,
        },
    },
    simd_utils::{lut_u8x16x2, lut_u16x16x3},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeA {
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,
    pub corner_perm_sym: CornerPermSymCoord,
    pub corner_orient_sym: CornerOrientSymCoord,
    pub corner_orient_sym_correct: DominoSymmetry,
    pub combined_phase_1_dist_previous_axis: u8,
    pub combined_sym_correct: u8,
}

const SPLIT_A_ZERO: SplitPhase1NodeA = SplitPhase1NodeA {
    edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
    corner_perm_sym: CornerPermSymCoord(0),
    corner_orient_sym: CornerOrientSymCoord(0),
    corner_orient_sym_correct: DominoSymmetry::IDENTITY,
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
pub fn split(node: Phase1Node, tables: &Tables) -> (SplitPhase1NodeA, SplitPhase1NodeB) {
    let Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct,
        corner_perm_correct,
        corner_perm_raw,
        corner_orient_raw,
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = node;
    let move_table: &MoveRawCornerOrientTable = tables.as_ref();
    let prune_table: &PrunePhase1Table = tables.as_ref();
    let lookup: &LookupSymCornerOrientTable = tables.as_ref();

    let corner_orient_adjusted =
        move_table.domino_conjugate(corner_orient_raw, edge_group_orient_correct);

    let dist = prune_table.get_value(edge_group_orient_sym, corner_orient_adjusted);
    let combo = lookup.get_combo_from_raw(corner_orient_raw);
    (
        SplitPhase1NodeA {
            edge_group_orient_sym,
            corner_perm_sym: corner_perm_raw,
            combined_phase_1_dist_previous_axis: (previous_axis as u8 | (dist << 4)),
            combined_sym_correct: (edge_group_orient_correct.0 << 4) | corner_perm_correct.0,
            corner_orient_sym: combo.sym_coord,
            corner_orient_sym_correct: combo.domino_conjugation,
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
pub fn merge(a: SplitPhase1NodeA, b: SplitPhase1NodeB, tables: &Tables) -> Phase1Node {
    let SplitPhase1NodeA {
        edge_group_orient_sym,
        corner_perm_sym: corner_perm_raw,
        combined_phase_1_dist_previous_axis: _,
        combined_sym_correct,
        corner_orient_sym,
        corner_orient_sym_correct,
    } = a;
    let SplitPhase1NodeB {
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = b;
    let lookup: &LookupSymCornerOrientTable = tables.as_ref();
    let corner_orient_raw = lookup.get_raw_from_combo(CornerOrientComboCoord {
        sym_coord: corner_orient_sym,
        domino_conjugation: corner_orient_sym_correct,
    });
    Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct: DominoSymmetry(combined_sym_correct >> 4),
        corner_perm_correct: DominoSymmetry(combined_sym_correct & 0xF),
        corner_perm_raw,
        corner_orient_raw,
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
    #[inline(always)]
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
        let zzco_mv_tbl: &MoveRawCornerOrientTable = tables.as_ref();
        let zzprune_tbl: &PrunePhase1Table = tables.as_ref();
        let lookup: &LookupSymCornerOrientTable = tables.as_ref();

        let co_raw = lookup.get_raw_from_combo(CornerOrientComboCoord {
            sym_coord: start_node.corner_orient_sym,
            domino_conjugation: start_node.corner_orient_sym_correct,
        });

        let phase_1_distance = start_node.combined_phase_1_dist_previous_axis >> 4;

        let distance_options = DISTANCE_OPTIONS_LOOKUP[phase_1_distance as usize];

        let ego_row = ego_mv_tbl.row(start_node.edge_group_orient_sym);
        let cp_row = cp_mv_tbl.row(start_node.corner_perm_sym);
        let co_row = co_mv_tbl.row(start_node.corner_orient_sym);
        let zzco_row = zzco_mv_tbl.row(co_raw);

        let previous_axis =
            unsafe { core::mem::transmute(start_node.combined_phase_1_dist_previous_axis & 0xF) };

        let (mv, num_moves) = CubeMove::new_axis_move_array_raw(previous_axis);

        let ego_i = lut_u8x16x2(
            &SYM_CONJ[(start_node.combined_sym_correct >> 4) as usize],
            mv,
        );
        let cp_i = lut_u8x16x2(
            &SYM_CONJ[(start_node.combined_sym_correct & 0xF) as usize],
            mv,
        );
        let co_i = lut_u8x16x2(
            &SYM_CONJ[start_node.corner_orient_sym_correct.0 as usize],
            mv,
        );
        let ego_c = lut_u16x16x3(ego_row.as_u16_array(), &ego_i);
        let cp_c = lut_u16x16x3(cp_row.as_u16_array(), &cp_i);
        let co_c = lut_u8x16x2(&co_row.as_u8_arrays()[0], &co_i);
        let zzco_c = lut_u16x16x3(zzco_row.as_u16_array(), mv);

        let offset = u128::from_ne_bytes([4u8; 16]);
        let ego_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(ego_i));
        let cp_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(cp_i));
        let co_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(co_i));

        let ego_j = lut_u8x16x2(&ego_row.as_u8_arrays()[1], &ego_i);
        let cp_j = lut_u8x16x2(&cp_row.as_u8_arrays()[1], &cp_i);
        let co_j = lut_u8x16x2(&co_row.as_u8_arrays()[1], &co_i);
        let mut axis_n = lut_u8x16x2(axis_after_table(previous_axis, moves_remaining), mv);

        // --- portable, autovec-reliable from here ---
        let a = u128::from_ne_bytes([start_node.corner_orient_sym_correct.0; 16]);
        let next_syms = u128::from_ne_bytes(co_j);
        let co_syms = DominoSymmetry::bitwise_then_32(a, next_syms);

        let a = u128::from_ne_bytes([start_node.combined_sym_correct; 16]);
        let next_syms = u128::from_ne_bytes(ego_j) << 4 | u128::from_ne_bytes(cp_j);
        let syms = DominoSymmetry::bitwise_then_32(a, next_syms);

        let co_adj = u128::to_ne_bytes(
            DominoSymmetry::bitwise_then_inv_32(
                (syms >> 4) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F,
                co_syms,
            ) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F,
        );
        let co_syms = u128::to_ne_bytes(co_syms);
        let syms = u128::to_ne_bytes(syms);

        for i in 1..=num_moves {
            let ego = EdgeGroupOrientSymCoord(ego_c[i]);
            let combo = CornerOrientComboCoord {
                sym_coord: CornerOrientSymCoord(co_c[i]),
                domino_conjugation: DominoSymmetry(co_adj[i]),
            };
            let test_raw = lookup.get_raw_from_combo(combo);
            assert_eq!(test_raw.0, zzco_c[i]);
            let zzadj = zzco_mv_tbl.domino_conjugate(
                CornerOrientRawCoord(zzco_c[i]),
                DominoSymmetry(syms[i] >> 4),
            );
            let zzdist = zzprune_tbl.get_value(ego, zzadj);
            let d_m3 = prune_tbl.get_value(ego, combo) as usize;
            let dist = distance_options[d_m3];
            assert_eq!(dist, zzdist, "{phase_1_distance} {d_m3} {dist} {zzdist}");
            axis_n[i] = axis_n[i] | (dist << 4)
        }

        let out = combine(ego_c, cp_c, co_c, co_syms, axis_n, syms);

        for i in 1..16 {
            slice[i] = unsafe { core::mem::transmute(out[i]) };
        }

        num_moves
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeAExp {
    pub edge_group_orient_sym: u16,
    pub corner_perm_sym: u16,
    pub corner_orient_sym: u8,
    // cp hi, co lo
    pub combined_sym_correct: u8,
    pub phase_1_dist: u8,
    pub previous_axis: u8,
}

impl SplitPhase1NodeAExp {
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
        let co_mv_tbl: &MoveSymCornerOrientTable = tables.as_ref();
        let cp_mv_tbl: &MoveSymCornerPermAugmentedTable = tables.as_ref();
        let prune_tbl: &PrunePhase1DoubleSymMod3Table = tables.as_ref();

        let ego_row = ego_mv_tbl.row(EdgeGroupOrientSymCoord(start_node.edge_group_orient_sym));
        let co_row = co_mv_tbl.row(CornerOrientSymCoord(start_node.corner_orient_sym));
        let cp_row = cp_mv_tbl.row(CornerPermSymCoord(start_node.corner_perm_sym));

        let previous_axis = unsafe { core::mem::transmute(start_node.previous_axis) };
        let (mv, num_moves) = CubeMove::new_axis_move_array_raw(previous_axis);

        let cmb_s = start_node.combined_sym_correct;
        let offset = u128::from_ne_bytes([4u8; 16]);

        // indices adjusted by current coordinate symmetry
        let ego_i = *mv;
        let co_i = lut_u8x16x2(&SYM_CONJ[(cmb_s & 0xF) as usize], mv);
        let cp_i = lut_u8x16x2(&SYM_CONJ[(cmb_s >> 4) as usize], mv);

        // indices adjusted for delayed position of offset
        let ego_s_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(ego_i));
        let co_s_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(co_i));
        let cp_s_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(cp_i));

        // pre-adjusted symmetries
        let tbl_ego_s_arr = u128::from_ne_bytes(lut_u8x16x2(&ego_row.as_u8_arrays()[1], &ego_s_i));
        let tbl_co_s_arr = u128::from_ne_bytes(lut_u8x16x2(&co_row.as_u8_arrays()[1], &co_s_i));
        let tbl_cp_s_arr = u128::from_ne_bytes(lut_u8x16x2(&cp_row.as_u8_arrays()[1], &cp_s_i));
        
        // packed syms in preparation for combining
        let tbl_ego_ego_s_arr = (tbl_ego_s_arr << 4) | tbl_ego_s_arr;
        let tbl_cp_co_s_arr = (tbl_cp_s_arr << 4) | tbl_co_s_arr;
        let inv_ego_ego_s_arr = DominoSymmetry::bitwise_invert_32(tbl_ego_ego_s_arr);

        // final coordinates
        let new_ego_c_arr = lut_u16x16x3(ego_row.as_u16_array(), &ego_i);
        let new_co_c_arr = lut_u8x16x2(&co_row.as_u8_arrays()[0], &co_i);
        let new_cp_c_arr = lut_u16x16x3(cp_row.as_u16_array(), &cp_i);

        // final packed syms
        let new_cp_co_s_arr = DominoSymmetry::bitwise_then_32(inv_ego_ego_s_arr, tbl_cp_co_s_arr);
        
        let new_co_s_lo_arr =
        u128::to_ne_bytes((tbl_ego_s_arr ^ tbl_co_s_arr) & 0x0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C);
        let dist_byte_addr_arr = ((Simd::from_array(new_ego_c_arr).cast::<u32>() << 2)
        | (Simd::from_array(new_co_s_lo_arr).cast::<u32>())
        | (Simd::from_array(new_co_c_arr).cast::<u32>() << 18)).cast::<usize>().to_array();
        
        let dist_bytes = dist_byte_addr_arr.map(|i| *unsafe { prune_tbl.as_buffer().get_unchecked(i) });
        let new_co_s_hi_arr = Simd::from_array(u128::to_ne_bytes((new_cp_co_s_arr >> 2) & 0x0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C));
        let dist_i = (Simd::from_array(dist_bytes) >> (new_co_s_hi_arr << 1)).to_array();
        let distance_options = DISTANCE_OPTIONS_LOOKUP[start_node.phase_1_dist as usize];
    
        // final dists
        let new_dist = lut_u8x16x2(&distance_options, &dist_i);
        

        let axis_n = lut_u8x16x2(axis_after_table(previous_axis, moves_remaining), mv);
        let out = combine(new_ego_c_arr, new_cp_c_arr, new_co_c_arr, u128::to_ne_bytes(new_cp_co_s_arr), new_dist, axis_n);

        for i in 1..16 {
            slice[i] = unsafe { core::mem::transmute(out.get_unchecked(i)) };
        }
        num_moves
    }
}

#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn combine(
    a: [u16; 16],
    b: [u16; 16],
    c: [u16; 16],
    d: [u8; 16],
    e: [u8; 16],
) -> [SplitPhase1NodeA; 16] {
    let mut out = [SPLIT_A_ZERO; 16];
    for i in 0..16 {
        out[i] = SplitPhase1NodeA {
            edge_group_orient_sym: EdgeGroupOrientSymCoord(a[i]),
            corner_perm_raw: CornerPermSymCoord(b[i]),
            corner_orient_raw: CornerOrientRawCoord(c[i]),
            previous_axis: unsafe { core::mem::transmute::<u8, CubePreviousAxis>(d[i]) },
            combined_sym_correct: e[i],
        };
    }

    out
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn combine(
    a: [u16; 16],
    b: [u16; 16],
    c: [u8; 16],
    d: [u8; 16],
    e: [u8; 16],
    f: [u8; 16],
) -> [[u8; 8]; 16] {
    use core::arch::aarch64::*;
    let mut out = [[0; 8]; 16];

    unsafe {
        let a0 = vld1q_u16(a.as_ptr());
        let a1 = vld1q_u16(a.as_ptr().add(8));
        let b0 = vld1q_u16(b.as_ptr());
        let b1 = vld1q_u16(b.as_ptr().add(8));

        let cv = vld1q_u8(c.as_ptr());
        let dv = vld1q_u8(d.as_ptr());

        let ev = vld1q_u8(e.as_ptr());
        let fv = vld1q_u8(f.as_ptr());

        // Interleave d/e bytes: [d0 e0 d1 e1 ...]. Viewed as u16 lanes on a
        // little-endian target, lane i == d[i] | (e[i] << 8), i.e. byte 0 = d
        // (offset 6 in the struct), byte 1 = e (offset 7).

        let cd0 = vreinterpretq_u16_u8(vzip1q_u8(cv, dv));
        let cd1 = vreinterpretq_u16_u8(vzip2q_u8(cv, dv));

        let ef0 = vreinterpretq_u16_u8(vzip1q_u8(ev, fv));
        let ef1 = vreinterpretq_u16_u8(vzip2q_u8(ev, fv));

        // st4 writes lanes interleaved: a0 b0 c0 de0 a1 b1 c1 de1 ...
        // which is exactly the AoS layout of SplitPhase1NodeA.
        let base = out.as_mut_ptr() as *mut u16;
        vst4q_u16(base, uint16x8x4_t(a0, b0, cd0, ef0));
        vst4q_u16(base.add(32), uint16x8x4_t(a1, b1, cd1, ef1));
    }

    out
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

    #[bench]
    fn phase_1_next(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = Phase1Node::from_cube(cube, &tables);
        phase_1.previous_axis = CubePreviousAxis::B;

        let mut buf = [phase_1; 16];

        bench.iter(|| {
            let _ = Phase1Node::produce_next_nodes(
                &mut buf,
                unsafe { NonZeroU8::new_unchecked(30) },
                tables,
            );
        });
    }
    
    // #[bench]
    // fn phase_1_next_exp(bench: &mut test::Bencher) {
    //     let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
    //     let mut rng = ChaCha8Rng::seed_from_u64(3);
    //     let cube: ReprCube =
    //         rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
    //     let mut phase_1 = Phase1Node::from_cube(cube, &tables);
    //     phase_1.previous_axis = CubePreviousAxis::B;

    //     let mut buf = [phase_1; 16];

    //     bench.iter(|| {
    //         let _ = Phase1Node::produce_next_nodes(
    //             &mut buf,
    //             unsafe { NonZeroU8::new_unchecked(30) },
    //             tables,
    //         );
    //     });
    // }

    // #[test]
    // fn print_table() {
    //     println!("{:?}", DISTANCE_OPTIONS_LOOKUP);
    // }

    #[bench]
    fn phase_1_next_a_exp(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = Phase1Node::from_cube(cube, &tables);
        phase_1.previous_axis = CubePreviousAxis::B;

        let phase_1_a_exp = SplitPhase1NodeAExp {
            edge_group_orient_sym: phase_1.edge_group_orient_sym.0,
            corner_perm_sym: phase_1.corner_perm_raw.0,
            corner_orient_sym: 75,
            combined_sym_correct: 37,
            phase_1_dist: 9,
            previous_axis: CubePreviousAxis::B as u8,
        };

        let mut buf = [phase_1_a_exp; 16];

        bench.iter(|| {
            let _ = SplitPhase1NodeAExp::produce_next_nodes_a(
                &mut buf,
                unsafe { NonZeroU8::new_unchecked(30) },
                tables,
            );
        });
    }

    // #[bench]
    // fn phase_1_next_b(bench: &mut test::Bencher) {
    //     let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
    //     let mut rng = ChaCha8Rng::seed_from_u64(3);
    //     let cube: ReprCube =
    //         rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
    //     let mut phase_1 = split(Phase1Node::from_cube(cube, &tables)).1;
    //     phase_1.previous_axis = CubePreviousAxis::B;

    //     // let mut buf = [phase_1; 16];

    //     bench.iter(|| {
    //         let _ = phase_1.produce_next_node_b(unsafe { NonZeroU8::new_unchecked(30) }, tables, 0);
    //     });
    // }
}

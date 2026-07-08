use std::num::NonZeroU8;

use crate::{
    CubeMove, EdgePerm, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord},
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
        },
        search::phase_1_node::Phase1Node,
        tables::{
            move_edge_positions::MoveEdgePositionsTable,
            move_raw_corner_orient::MoveRawCornerOrientTable,
            move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
        },
    },
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeA {
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,
    pub corner_perm_raw: CornerPermSymCoord,
    pub corner_orient_raw: CornerOrientRawCoord,
    pub previous_axis: CubePreviousAxis,
    pub combined_sym_correct: u8,
}

const SPLIT_A_ZERO: SplitPhase1NodeA = SplitPhase1NodeA {
    edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
    corner_perm_raw: CornerPermSymCoord(0),
    corner_orient_raw: CornerOrientRawCoord(0),
    previous_axis: CubePreviousAxis::U,
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

impl Default for SplitPhase1NodeA {
    fn default() -> Self {
        const {
            Self {
                edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
                corner_perm_raw: CornerPermSymCoord(0),
                corner_orient_raw: CornerOrientRawCoord(0),
                previous_axis: CubePreviousAxis::None,
                combined_sym_correct: 0,
            }
        }
    }
}

impl Default for SplitPhase1NodeB {
    fn default() -> Self {
        const {
            let (u_edge_positions, d_edge_positions, e_edge_positions) =
                split_edge_positions(EdgePerm::SOLVED);
            Self {
                u_edge_positions,
                d_edge_positions,
                e_edge_positions,
                previous_axis: CubePreviousAxis::None,
            }
        }
    }
}

#[inline(always)]
pub fn split(node: Phase1Node) -> (SplitPhase1NodeA, SplitPhase1NodeB) {
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
    (
        SplitPhase1NodeA {
            edge_group_orient_sym,
            corner_perm_raw,
            corner_orient_raw,
            previous_axis,
            combined_sym_correct: (edge_group_orient_correct.0 << 4) | corner_perm_correct.0,
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
pub fn merge(a: SplitPhase1NodeA, b: SplitPhase1NodeB) -> Phase1Node {
    let SplitPhase1NodeA {
        edge_group_orient_sym,
        corner_perm_raw,
        corner_orient_raw,
        previous_axis: _,
        combined_sym_correct,
    } = a;
    let SplitPhase1NodeB {
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = b;
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

#[inline(always)]
fn axis_after_table(prev: CubePreviousAxis, moves_remaining: NonZeroU8) -> &'static [u8; 32] {
    let penultimate = (moves_remaining.get() == 2) as usize;
    &AXIS_AFTER[penultimate][prev as usize]
}

impl SplitPhase1NodeA {
    #[inline(always)]
    pub fn is_domino_reduced(self) -> bool {
        self.corner_orient_raw.0 == 0 && self.edge_group_orient_sym.0 == 0
    }

    #[inline(always)]
    pub fn distance_heuristic(
        self,
        tables: impl AsRef<MoveRawCornerOrientTable> + AsRef<PrunePhase1Table>,
    ) -> u8 {
        let move_table: &MoveRawCornerOrientTable = tables.as_ref();
        let prune_table: &PrunePhase1Table = tables.as_ref();

        let corner_orient_adjusted = move_table.domino_conjugate(
            self.corner_orient_raw,
            DominoSymmetry(self.combined_sym_correct >> 4),
        );

        prune_table.get_value(self.edge_group_orient_sym, corner_orient_adjusted)
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
        let co_mv_tbl: &MoveRawCornerOrientTable = tables.as_ref();

        let ego_row = ego_mv_tbl.row(start_node.edge_group_orient_sym);
        let cp_row = cp_mv_tbl.row(start_node.corner_perm_raw);
        let co_row = co_mv_tbl.row(start_node.corner_orient_raw);

        let (mv, num_moves) = CubeMove::new_axis_move_array_raw(start_node.previous_axis);

        let ego_i = lut_u8x16(
            &SYM_CONJ[(start_node.combined_sym_correct >> 4) as usize],
            mv,
        );
        let cp_i = lut_u8x16(
            &SYM_CONJ[(start_node.combined_sym_correct & 0xF) as usize],
            mv,
        );
        let ego_c = lut_u16x16(ego_row.as_u16_array(), &ego_i);
        let cp_c = lut_u16x16(cp_row.as_u16_array(), &cp_i);
        let co_c = lut_u16x16(co_row.as_u16_array(), mv);

        let offset = u128::from_ne_bytes([4u8; 16]);
        let ego_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(ego_i));
        let cp_i = u128::to_ne_bytes(offset + u128::from_ne_bytes(cp_i));

        let ego_j = lut_u8x16(&ego_row.as_u8_arrays()[1], &ego_i);
        let cp_j = lut_u8x16(&cp_row.as_u8_arrays()[1], &cp_i);
        let axis_n = lut_u8x16(
            axis_after_table(start_node.previous_axis, moves_remaining),
            mv,
        );

        // --- portable, autovec-reliable from here ---
        let a = u128::from_ne_bytes([start_node.combined_sym_correct; 16]);
        let next_syms = u128::from_ne_bytes(ego_j) << 4 | u128::from_ne_bytes(cp_j);
        let syms = u128::to_ne_bytes(DominoSymmetry::then_no_lookup_32(a, next_syms));

        let out = combine(ego_c, cp_c, co_c, axis_n, syms);

        for i in 1..16 {
            slice[i] = out[i];
        }
        num_moves
    }
}

/// 16 parallel byte lookups from a small table.
/// Contract: table.len() >= 32 (pad it), all idx[i] < table.len().
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u8x16(table: &[u8; 32], idx: &[u8; 16]) -> [u8; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u8x16(table: &[u8; 32], idx: &[u8; 16]) -> [u8; 16] {
    use std::arch::aarch64::*;
    unsafe {
        // Load 32 bytes as two 128-bit lanes
        let t = vld1q_u8_x2(table.as_ptr());
        // Load indices
        let i = vld1q_u8(idx.as_ptr());
        // Parallel table lookup
        let res = vqtbl2q_u8(t, i);
        // Store result
        let mut out = [0u8; 16];
        vst1q_u8(out.as_mut_ptr(), res);
        out
    }
}

/// 16 parallel u16 lookups from one 64-byte table.
/// Contract: all idx[i] < 32 relative to `table`'s base.
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u16x16(table: &[u16; 32], idx: &[u8; 16]) -> [u16; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u16x16(table: &[u16; 32], idx: &[u8; 16]) -> [u16; 16] {
    use std::arch::aarch64::*;
    unsafe {
        let t = vld1q_u8_x4(table.as_ptr() as *const u8);

        let i = vld1q_u8(idx.as_ptr());
        let i_lo = vaddq_u8(i, i); // idx * 2
        let i_hi = vaddq_u8(i_lo, vdupq_n_u8(1)); // idx * 2 + 1

        let lo = vqtbl4q_u8(t, i_lo);
        let hi = vqtbl4q_u8(t, i_hi);

        // vzipq_u8 gives two halves: .0 covers idx[0..8], .1 covers idx[8..16]
        let zipped = vzipq_u8(lo, hi);
        let res_lo = vreinterpretq_u16_u8(zipped.0); // u16 results for idx[0..8]
        let res_hi = vreinterpretq_u16_u8(zipped.1); // u16 results for idx[8..16]

        let mut out = [0u16; 16];
        vst1q_u16(out.as_mut_ptr(), res_lo);
        vst1q_u16(out.as_mut_ptr().add(8), res_hi);
        out
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
    c: [u16; 16],
    d: [u8; 16],
    e: [u8; 16],
) -> [SplitPhase1NodeA; 16] {
    use core::arch::aarch64::*;
    let mut out = [SPLIT_A_ZERO; 16];

    unsafe {
        let a0 = vld1q_u16(a.as_ptr());
        let a1 = vld1q_u16(a.as_ptr().add(8));
        let b0 = vld1q_u16(b.as_ptr());
        let b1 = vld1q_u16(b.as_ptr().add(8));
        let c0 = vld1q_u16(c.as_ptr());
        let c1 = vld1q_u16(c.as_ptr().add(8));

        let dv = vld1q_u8(d.as_ptr());
        let ev = vld1q_u8(e.as_ptr());

        // Interleave d/e bytes: [d0 e0 d1 e1 ...]. Viewed as u16 lanes on a
        // little-endian target, lane i == d[i] | (e[i] << 8), i.e. byte 0 = d
        // (offset 6 in the struct), byte 1 = e (offset 7).
        let de0 = vreinterpretq_u16_u8(vzip1q_u8(dv, ev));
        let de1 = vreinterpretq_u16_u8(vzip2q_u8(dv, ev));

        // st4 writes lanes interleaved: a0 b0 c0 de0 a1 b1 c1 de1 ...
        // which is exactly the AoS layout of SplitPhase1NodeA.
        let base = out.as_mut_ptr() as *mut u16;
        vst4q_u16(base, uint16x8x4_t(a0, b0, c0, de0));
        vst4q_u16(base.add(32), uint16x8x4_t(a1, b1, c1, de1));
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

    #[bench]
    fn phase_1_next_a(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = split(Phase1Node::from_cube(cube, &tables)).0;
        phase_1.previous_axis = CubePreviousAxis::B;

        let mut buf = [phase_1; 16];

        bench.iter(|| {
            let _ = SplitPhase1NodeA::produce_next_nodes_a(
                &mut buf,
                unsafe { NonZeroU8::new_unchecked(30) },
                tables,
            );
        });
    }

    #[bench]
    fn phase_1_next_b(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = split(Phase1Node::from_cube(cube, &tables)).1;
        phase_1.previous_axis = CubePreviousAxis::B;

        // let mut buf = [phase_1; 16];

        bench.iter(|| {
            let _ = phase_1.produce_next_node_b(unsafe { NonZeroU8::new_unchecked(30) }, tables, 0);
        });
    }
}

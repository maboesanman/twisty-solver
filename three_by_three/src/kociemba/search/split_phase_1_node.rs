use core::slice;
use std::{intrinsics::simd, mem::offset_of, num::NonZeroU8, ptr::copy_nonoverlapping, simd::{Mask, Simd, cmp::{SimdPartialEq as _, SimdPartialOrd as _}, num::{SimdInt as _, SimdUint}, ptr::{SimdConstPtr as _, SimdMutPtr}}};

use crate::{
    CornerOrient, CubeMove, Tables, cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry}, kociemba::{
        coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord}, partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions,
        }, search::phase_1_node::Phase1Node, tables::{
            lookup_sym_corner_orient::{CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable}, move_edge_positions::MoveEdgePositionsTable, move_raw_corner_orient::MoveRawCornerOrientTable, move_raw_corner_orient_trits::MoveRawCornerOrientTritsTable, move_sym_corner_orient::MoveSymCornerOrientTable, move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable, move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable, prune_phase_1::PrunePhase1Table, prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table
        },
    }, simd_utils::{combine, gather16, lut_u8x16x2, lut_u16x16x3, mask_to_backpointers}
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeA {
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,
    pub corner_perm_sym: CornerPermSymCoord,
    pub corner_orient_sym: CornerOrientSymCoord,
    pub corner_orient_correct: u8,
    pub combined_phase_1_dist_previous_axis: u8,

    /// ego_hi, co_lo
    pub combined_sym_correct: u8,
}

#[derive(Debug, Clone)]
pub struct SplitPhase1NodeAStackStruct {
    pub len: u16,
    // this is a lie, it's actually SoA. we use MaybeUninit<SoATemplatePhase1NodeA>
    // to set the size, but we actually use this as a *mut u8
    buffer: Box<[std::mem::MaybeUninit<SoATemplatePhase1NodeA>]>,
    offsets: Simd<u16, 8>,
    sizes_as_shifts: Simd<u16, 8>,
}

impl SplitPhase1NodeAStackStruct {
    pub fn new(cap: u16, start: SplitPhase1NodeA) -> Self {
        let mut buffer = Box::<[SoATemplatePhase1NodeA]>::new_zeroed_slice(cap as usize);

        let bytes = buffer.as_mut_ptr().cast::<u8>();

        let offsets = Simd::from_array([
            core::mem::offset_of!(SoATemplatePhase1NodeA, edge_group_orient_sym),
            core::mem::offset_of!(SoATemplatePhase1NodeA, corner_perm_sym),
            core::mem::offset_of!(SoATemplatePhase1NodeA, combined_sym_correct),
            core::mem::offset_of!(SoATemplatePhase1NodeA, corner_orient_sym),
            core::mem::offset_of!(SoATemplatePhase1NodeA, corner_orient_correct),
            core::mem::offset_of!(SoATemplatePhase1NodeA, phase_1_dists),
            core::mem::offset_of!(SoATemplatePhase1NodeA, previous_axes),
            core::mem::offset_of!(SoATemplatePhase1NodeA, back_pointers),
        ]).cast() * Simd::splat(cap);
        let sizes_as_shifts = Simd::from_array([
            1, 1, 0, 0, 0, 0, 0, 0
        ]);

        let base_addresses: Simd<*mut u8, 8> = Simd::splat(bytes);
        let ptrs = base_addresses.wrapping_add(offsets.cast()).to_array();

        unsafe {
            *(ptrs[0] as *mut EdgeGroupOrientSymCoord) = start.edge_group_orient_sym;
            *(ptrs[1] as *mut CornerPermSymCoord) = start.corner_perm_sym;
            *(ptrs[2] as *mut u8) = start.combined_sym_correct;
            *(ptrs[3] as *mut CornerOrientSymCoord) = start.corner_orient_sym;
            *(ptrs[4] as *mut u8) = start.corner_orient_correct & 0xF;
            *(ptrs[5] as *mut u8) = start.combined_phase_1_dist_previous_axis >> 4;
            *(ptrs[6] as *mut u8) = start.combined_phase_1_dist_previous_axis & 0xF;
            *(ptrs[7] as *mut u8) = start.corner_orient_correct >> 4;
        }

        Self {
            buffer,
            len: 1,
            offsets,
            sizes_as_shifts,
        }
    }

// #[inline(always)]
//     pub fn get_slice_for_expansion(&mut self, index: usize) -> SplitPhase1NodeABufs<'_> {
//         debug_assert!(index + 15 < self.buffer.len());
//         let base_addr = self.buffer.as_mut_ptr();

//         let base_addresses: Simd<*mut u8, 8> = Simd::splat(base_addr.cast());
//         let ptrs = base_addresses.wrapping_add((self.offsets + (Simd::splat(index) << self.sizes_as_shifts)).cast()).to_array();

//         unsafe {
//             SplitPhase1NodeABufs {
//                 edge_group_orient_sym: &mut *(ptrs[0] as *mut [EdgeGroupOrientSymCoord; 16]),
//                 corner_perm_sym: &mut *(ptrs[1] as *mut [CornerPermSymCoord; 16]),
//                 combined_sym_correct: &mut *(ptrs[2] as *mut [u8; 16]),
//                 corner_orient_sym: &mut *(ptrs[3] as *mut [CornerOrientSymCoord; 16]),
//                 corner_orient_correct: &mut *(ptrs[4] as *mut [DominoSymmetry; 16]),
//                 phase_1_dists: &mut *(ptrs[5] as *mut [u8; 16]),
//                 previous_axes: &mut *(ptrs[6] as *mut [CubePreviousAxis; 16]),
//                 back_pointers: &mut *(ptrs[7] as *mut [u8; 16]),
//             }
//         }
//     }

#[inline(always)]
    pub fn get_aaa(&self, i: u16) -> SplitPhase1NodeA {
        debug_assert!((i as usize) < self.buffer.len());
        let base_addr = self.buffer.as_ptr();

        let base_addresses: Simd<*const u8, 8> = Simd::splat(base_addr.cast());
        let ptrs = base_addresses.wrapping_add((self.offsets + (Simd::splat(i) << self.sizes_as_shifts)).cast()).to_array();

        unsafe {
            let edge_group_orient_sym = *(ptrs[0] as *mut EdgeGroupOrientSymCoord);
            let corner_perm_sym = *(ptrs[1] as *mut CornerPermSymCoord);
            let combined_sym_correct = *(ptrs[2] as *mut u8);
            let corner_orient_sym = *(ptrs[3] as *mut CornerOrientSymCoord);
            let corner_orient_correct = *(ptrs[4] as *mut DominoSymmetry);
            let phase_1_dists = *(ptrs[5] as *mut u8);
            let previous_axes = *(ptrs[6] as *mut CubePreviousAxis);
            let back_pointers = *(ptrs[7] as *mut u8);

            SplitPhase1NodeA {
                edge_group_orient_sym,
                corner_perm_sym,
                corner_orient_sym,
                combined_sym_correct,

                corner_orient_correct: (back_pointers << 4) | corner_orient_correct.0,
                combined_phase_1_dist_previous_axis: (phase_1_dists << 4) | previous_axes as u8,
            }
        }
    }

    #[inline(always)]
    pub fn pop_unchecked_aaa(&mut self) {
        let back_pointer_offset = self.offsets[7] + self.len.wrapping_sub(1);
        let back_pointer = unsafe { *self.buffer.as_ptr().cast::<u8>().add(back_pointer_offset as usize) };
        self.len -= back_pointer as u16;
    }

    #[inline(always)]
    pub fn soa_produce_next_nodes(
        &mut self,
        moves_remaining: NonZeroU8,
        tables: &Tables
    ) -> u16 {
        let base_addr = self.buffer.as_mut_ptr();

        let base_addresses: Simd<*mut u8, 8> = Simd::splat(base_addr.cast());
        let ptrs = base_addresses.wrapping_add((self.offsets + (Simd::splat(self.len - 1) << self.sizes_as_shifts)).cast()).to_array();

        let (
            edge_group_orient_sym,
            corner_perm_sym,
            combined_sym_correct,
            corner_orient_sym,
            corner_orient_correct,
            phase_1_dists,
            previous_axes,
            // back_pointers,
        ) = unsafe {
            (
                *(ptrs[0] as *mut EdgeGroupOrientSymCoord),
                *(ptrs[1] as *mut CornerPermSymCoord),
                *(ptrs[2] as *mut u8),
                *(ptrs[3] as *mut CornerOrientSymCoord),
                *(ptrs[4] as *mut DominoSymmetry),
                *(ptrs[5] as *mut u8),
                *(ptrs[6] as *mut CubePreviousAxis),
                // *(ptrs[7] as *mut u8),
            )
        };

        let ego_mv_tbl: &MoveSymEdgeGroupOrientTable = tables.as_ref();
        let cp_mv_tbl: &MoveSymCornerPermAugmentedTable = tables.as_ref();
        let co_mv_tbl: &MoveSymCornerOrientTable = tables.as_ref();
        let prune_tbl: &PrunePhase1DoubleSymMod3Table = tables.as_ref();

        let ego_row = ego_mv_tbl.row(edge_group_orient_sym);
        let cp_row = cp_mv_tbl.row(corner_perm_sym);
        let co_row = co_mv_tbl.row(corner_orient_sym);

        let previous_axis = previous_axes;

        let (mv, num_moves) = CubeMove::new_axis_move_array_raw(previous_axis);

        let ego_i = lut_u8x16x2(
            unsafe { SYM_CONJ.get_unchecked((combined_sym_correct >> 4) as usize) },
            mv,
        );
        let cp_i = lut_u8x16x2(
            unsafe { SYM_CONJ.get_unchecked((combined_sym_correct & 0xF) as usize) },
            mv,
        );
        let co_i = lut_u8x16x2(
            unsafe { SYM_CONJ.get_unchecked((corner_orient_correct.0) as usize) },
            mv,
        );
        let ego_c = lut_u16x16x3(ego_row.as_u16_array(), &ego_i);
        let cp_c = lut_u16x16x3(cp_row.as_u16_array(), &cp_i);
        let co_c = lut_u8x16x2(&co_row.as_u8_arrays()[0], &co_i);

        let ego_i = Simd::from_array(ego_i) + Simd::splat(4);
        let cp_i = Simd::from_array(cp_i) + Simd::splat(4);
        let co_i = Simd::from_array(co_i) + Simd::splat(4);

        let ego_j = lut_u8x16x2(&ego_row.as_u8_arrays()[1], &ego_i.to_array());
        let cp_j = lut_u8x16x2(&cp_row.as_u8_arrays()[1], &cp_i.to_array());
        let co_j = lut_u8x16x2(&co_row.as_u8_arrays()[1], &co_i.to_array());
        let axis_n = lut_u8x16x2(
            axis_after_table(previous_axis, moves_remaining),
            mv,
        );

        let cmb_syms = {
            let a = Simd::splat(combined_sym_correct);
            let next = Simd::from_array(ego_j) << 4 | Simd::from_array(cp_j);
            DominoSymmetry::bitwise_then_simd(a, next)
        };

        let co_syms = {
            let a = Simd::splat(corner_orient_correct.0);
            let next = Simd::from_array(co_j);
            DominoSymmetry::bitwise_then_simd(a, next)
        };

        let ego_syms = cmb_syms >> 4;
        let ego_syms_inv = DominoSymmetry::bitwise_invert_simd(ego_syms);
        let co_lookup = (DominoSymmetry::bitwise_then_simd(ego_syms_inv, co_syms))
            & Simd::splat(0x0F);
        let co_lookup_lo = ego_syms ^ co_syms;

        let ego_c_simd = Simd::from_array(ego_c);
        let co_c_simd = Simd::from_array(co_c);

        let new_co_s_lo_arr = co_lookup_lo & Simd::splat(0x03);
        let dist_byte_addr_arr = (ego_c_simd.cast::<u32>() << 2)
            | (new_co_s_lo_arr.cast::<u32>())
            | (co_c_simd.cast::<u32>() << 18);

        let dist_bytes = unsafe { gather16(prune_tbl.as_buffer(), dist_byte_addr_arr) };
        let new_co_s_hi_arr = (co_lookup >> 2) & Simd::splat(0x03);

        let dist_i = ((dist_bytes >> (new_co_s_hi_arr << 1)) & Simd::splat(0x03)).to_array();
        let distance_options =
            unsafe { DISTANCE_OPTIONS_LOOKUP.get_unchecked(phase_1_dists as usize) };

        let new_dists = Simd::from_array(lut_u8x16x2(&distance_options, &dist_i));
        let new_prev_axis = Simd::from_array(axis_n);

        let valid = {
            let m = moves_remaining.get();
            let reduced: Mask<i8, 16> = new_dists.simd_eq(Simd::splat(0));
            let le_limit: Mask<i8, 16> = new_dists.simd_le(Simd::splat(m - 1));

            let reduced_ok = (m == 1) || (m > 7);

            let valid = le_limit & (!reduced | Mask::splat(reduced_ok));
            let num_moves_mask = Simd::from_array([0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]).simd_lt(Simd::splat(num_moves as u8));

            valid & num_moves_mask
        };

        let num_moves = 16 - (valid.to_bitmask() as u16).leading_zeros() as usize;

        unsafe {
            let preheat_ego_row = ego_mv_tbl.row(EdgeGroupOrientSymCoord(*ego_c.get_unchecked(num_moves)));
            let preheat_cp_row = cp_mv_tbl.row(CornerPermSymCoord(*cp_c.get_unchecked(num_moves)));
            let preheat_co_row = co_mv_tbl.row(CornerOrientSymCoord(*co_c.get_unchecked(num_moves)));
            core::intrinsics::prefetch_read_data::<_, 3>(preheat_ego_row as *const _);
            core::intrinsics::prefetch_read_data::<_, 3>(preheat_cp_row as *const _);
            core::intrinsics::prefetch_read_data::<_, 3>(preheat_co_row as *const _);
        }

        let back_pointers = mask_to_backpointers(valid);
        let cmb_syms_arr = cmb_syms.to_array();

        unsafe {
            ptrs[0].cast::<u16>().wrapping_add(1).cast::<[u16; 16]>().write_unaligned(ego_c);
            ptrs[1].cast::<u16>().wrapping_add(1).cast::<[u16; 16]>().write_unaligned(cp_c);
            ptrs[2].wrapping_add(1).cast::<[u8; 16]>().write_unaligned(cmb_syms_arr);
            ptrs[3].wrapping_add(1).cast::<[u8; 16]>().write_unaligned(co_c);
            ptrs[4].wrapping_add(1).cast::<[u8; 16]>().write_unaligned(co_syms.to_array());
            ptrs[5].wrapping_add(1).cast::<[u8; 16]>().write_unaligned(new_dists.to_array());
            ptrs[6].wrapping_add(1).cast::<[u8; 16]>().write_unaligned(new_prev_axis.to_array());
            ptrs[7].wrapping_add(1).cast::<[u8; 16]>().write_unaligned(back_pointers.to_array());
        }
        num_moves as u16
    }
}

#[derive(Copy, Clone)]
struct SoATemplatePhase1NodeA {
    edge_group_orient_sym: EdgeGroupOrientSymCoord,
    corner_perm_sym: CornerPermSymCoord,
    /// ego_hi, co_lo
    combined_sym_correct: u8,

    corner_orient_sym: CornerOrientSymCoord,
    corner_orient_correct: DominoSymmetry,
    
    phase_1_dists: u8,
    previous_axes: CubePreviousAxis,
    back_pointers: u8,
}

// pub struct SplitPhase1NodeABufs<'a> {
//     edge_group_orient_sym: &'a mut [EdgeGroupOrientSymCoord; 16],
//     corner_perm_sym: &'a mut [CornerPermSymCoord; 16],
//     /// ego_hi, co_lo
//     combined_sym_correct: &'a mut [u8; 16],

//     corner_orient_sym: &'a mut [CornerOrientSymCoord; 16],
//     corner_orient_correct: &'a mut [DominoSymmetry; 16],
    
//     phase_1_dists: &'a mut [u8; 16],
//     previous_axes: &'a mut [CubePreviousAxis; 16],
//     back_pointers: &'a mut [u8; 16],
// }



const SPLIT_A_ZERO: SplitPhase1NodeA = SplitPhase1NodeA {
    edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
    corner_perm_sym: CornerPermSymCoord(0),
    corner_orient_sym: CornerOrientSymCoord(0),
    corner_orient_correct: 16,
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
            corner_orient_correct: corner_orient_combo.domino_conjugation.0 | 16,
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
    Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct: DominoSymmetry(combined_sym_correct >> 4),
        corner_perm_correct: DominoSymmetry(combined_sym_correct & 0xF),
        corner_perm_sym,
        corner_orient_combo: CornerOrientComboCoord {
            sym_coord: corner_orient_sym,
            domino_conjugation: DominoSymmetry(corner_orient_correct & 0xF),
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

    pub fn back_pointer(
        self
    ) -> u8 {
        self.corner_orient_correct >> 4
    }

    pub fn set_back_pointer(
        &mut self,
        new_value: u8
    ) {
        self.corner_orient_correct = (self.corner_orient_correct & 0x0F) | (new_value << 4)
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
            unsafe { SYM_CONJ.get_unchecked((start_node.corner_orient_correct & 0xF) as usize) },
            mv,
        );
        let ego_c = lut_u16x16x3(ego_row.as_u16_array(), &ego_i);
        let cp_c = lut_u16x16x3(cp_row.as_u16_array(), &cp_i);
        let co_c = lut_u8x16x2(&co_row.as_u8_arrays()[0], &co_i);

        let ego_i = Simd::from_array(ego_i) + Simd::splat(4);
        let cp_i = Simd::from_array(cp_i) + Simd::splat(4);
        let co_i = Simd::from_array(co_i) + Simd::splat(4);

        let ego_j = lut_u8x16x2(&ego_row.as_u8_arrays()[1], &ego_i.to_array());
        let cp_j = lut_u8x16x2(&cp_row.as_u8_arrays()[1], &cp_i.to_array());
        let co_j = lut_u8x16x2(&co_row.as_u8_arrays()[1], &co_i.to_array());
        let axis_n = lut_u8x16x2(
            axis_after_table(previous_axis, moves_remaining),
            mv,
        );

        let cmb_syms = {
            let a = Simd::splat(start_node.combined_sym_correct);
            let next = Simd::from_array(ego_j) << 4 | Simd::from_array(cp_j);
            DominoSymmetry::bitwise_then_simd(a, next)
        };

        let co_syms = {
            let a = Simd::splat(start_node.corner_orient_correct & 0xF);
            let next = Simd::from_array(co_j);
            DominoSymmetry::bitwise_then_simd(a, next)
        };

        let ego_syms = cmb_syms >> 4;
        let ego_syms_inv = DominoSymmetry::bitwise_invert_simd(ego_syms);
        let co_lookup = (DominoSymmetry::bitwise_then_simd(ego_syms_inv, co_syms))
            & Simd::splat(0x0F);
        let co_lookup_lo = ego_syms ^ co_syms;

        let ego_c_simd = Simd::from_array(ego_c);
        let co_c_simd = Simd::from_array(co_c);

        let new_co_s_lo_arr = co_lookup_lo & Simd::splat(0x03);
        let dist_byte_addr_arr = (ego_c_simd.cast::<u32>() << 2)
            | (new_co_s_lo_arr.cast::<u32>())
            | (co_c_simd.cast::<u32>() << 18);

        let dist_bytes = unsafe { gather16(prune_tbl.as_buffer(), dist_byte_addr_arr) };
        let new_co_s_hi_arr = (co_lookup >> 2) & Simd::splat(0x03);

        let dist_i = ((dist_bytes >> (new_co_s_hi_arr << 1)) & Simd::splat(0x03)).to_array();
        let distance_options =
            unsafe { DISTANCE_OPTIONS_LOOKUP.get_unchecked(phase_1_distance as usize) };

        let new_dists = Simd::from_array(lut_u8x16x2(&distance_options, &dist_i));
        let new_prev_axis = Simd::from_array(axis_n);
        let combined_dist_axis = (new_dists << 4) | (new_prev_axis);

        let valid = {
            let m = moves_remaining.get();
            let reduced: Mask<i8, 16> = new_dists.simd_eq(Simd::splat(0));
            let le_limit: Mask<i8, 16> = new_dists
                .simd_le(Simd::splat(m - 1));

            let reduced_ok = (m == 1) || (m > 7);

            let mut valid = le_limit & (!reduced | Mask::splat(reduced_ok));

            valid.set(0, true);
            let num_moves_mask = Simd::from_array([0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]).simd_le(Simd::splat(num_moves as u8));

            valid & num_moves_mask
        };

        let num_moves = 15 - (valid.to_bitmask() as u16).leading_zeros() as usize;

        unsafe {
            let preheat_ego_row = ego_mv_tbl.row(EdgeGroupOrientSymCoord(*ego_c.get_unchecked(num_moves)));
            let preheat_cp_row = cp_mv_tbl.row(CornerPermSymCoord(*cp_c.get_unchecked(num_moves)));
            let preheat_co_row = co_mv_tbl.row(CornerOrientSymCoord(*co_c.get_unchecked(num_moves)));
            core::intrinsics::prefetch_read_data::<_, 3>(preheat_ego_row as *const _);
            core::intrinsics::prefetch_read_data::<_, 3>(preheat_cp_row as *const _);
            core::intrinsics::prefetch_read_data::<_, 3>(preheat_co_row as *const _);
        }

        let back_pointers = mask_to_backpointers(valid);


        let cmb_syms_arr = cmb_syms.to_array();
        let co_syms_arr = (co_syms | (back_pointers << 4)).to_array();

        let out = combine(
            ego_c, cp_c, co_c,
            co_syms_arr,
            combined_dist_axis.to_array(),
            cmb_syms_arr,
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

    // Diagnostic-only differential check (not part of the opaque e2e suite):
    // recomputes the ported "viable" formula using each *stored* child's own
    // distance, then verifies the back_pointer chain (walked from the top,
    // num_moves) visits exactly the set of indices the formula calls valid.
    #[test]
    fn produce_next_nodes_a_backpointer_matches_formula() {
        fn old_viable(dist: u8, m: u8) -> bool {
            // m here is the PARENT's moves_remaining (as passed to produce_next_nodes_a).
            // old code's "moves_remaining" (MR_check) at validity-check time == m - 1.
            let mr_check = m - 1;
            let reduced = dist == 0;
            (mr_check == 0 && reduced) || (!reduced) || (reduced && mr_check > 6)
        }
        fn old_valid(dist: u8, m: u8) -> bool {
            old_viable(dist, m) && dist <= (m - 1)
        }

        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(7);

        let mut mismatches = 0usize;
        let mut total = 0usize;

        for _ in 0..2000 {
            let cube: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
            let phase_1 = Phase1Node::from_cube(cube, &tables);
            let (split_a, _) = split(phase_1, tables);

            for m in [1u8, 2, 3, 6, 7, 8, 15, 30] {
                let moves_remaining = NonZeroU8::new(m).unwrap();
                let mut buf = [split_a; 16];
                let num_moves = SplitPhase1NodeA::produce_next_nodes_a(&mut buf, moves_remaining, tables);

                // Walk the back_pointer chain from the top (num_moves) down to 0,
                // collecting the set of indices the algorithm treats as valid.
                let mut reachable = [false; 16];
                let mut j = num_moves;
                while j > 0 {
                    reachable[j] = true;
                    let bp = buf[j].back_pointer() as usize;
                    assert!(bp >= 1, "back_pointer must be >=1, got {bp} at j={j}");
                    assert!(bp <= j, "back_pointer {bp} at j={j} would underflow the frame");
                    j -= bp;
                }

                for idx in 1..=num_moves {
                    let dist = buf[idx].distance_heuristic();
                    let expected = old_valid(dist, m);
                    let actual = reachable[idx];
                    total += 1;
                    if expected != actual {
                        mismatches += 1;
                        eprintln!(
                            "mismatch: m={m} idx={idx} num_moves={num_moves} dist={dist} expected={expected} actual={actual}"
                        );
                    }
                }
            }
        }

        assert_eq!(mismatches, 0, "{mismatches}/{total} mismatches between formula and back_pointer chain");
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

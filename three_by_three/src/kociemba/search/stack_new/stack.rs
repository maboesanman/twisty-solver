use fearless_simd::{SimdBase, SimdFrom, SimdMask};
use itertools::Itertools;

use crate::{
    CubeMove, ReprCube, Tables,
    cube_ops::cube_sym::CubeSymmetry,
    kociemba::search::stack_new::{
        max_threshold::MaxThreshold,
        partial_stack::PartialStack,
        utils::{
            get_entry, get_entry_mut, get_entry_slice_mut, lut_u8x32, lut_u8x64, mask_to_swizzle,
        },
    },
};

#[derive(Clone, Debug)]
pub struct Stack<'t, const N: usize, M> {
    pub tables: &'t Tables,
    combined_phase_max: M,
    current_position: u16,
    shared: [[Shared; 15]; N],
    e_stack: PartialStack<N>,
    s_stack: PartialStack<N>,
    m_stack: PartialStack<N>,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Shared {
    next_pointer: u16,
    previous_move: u8,
    remaining_moves: u8,
}

impl<'t, const N: usize, M: MaxThreshold> Stack<'t, N, M> {
    pub fn new<S: fearless_simd::Simd>(
        cube: ReprCube,
        mut combined_phase_max: M,
        simd: S,
        tables: &'t Tables,
    ) -> Self {
        let e_cube = cube;
        let s_cube = cube.conjugate(CubeSymmetry(1 << 4));
        let m_cube = cube.conjugate(CubeSymmetry(2 << 4));

        let e_moves = CubeMove::all_iter();
        let s_moves = CubeMove::all_iter().map(|m| m.conjugate(CubeSymmetry(1 << 4)));
        let m_moves = CubeMove::all_iter().map(|m| m.conjugate(CubeSymmetry(2 << 4)));

        let mut e_stack = PartialStack::<N>::new(e_cube, tables);
        let mut s_stack = PartialStack::<N>::new(s_cube, tables);
        let mut m_stack = PartialStack::<N>::new(m_cube, tables);

        let e_init = e_stack.begin_initial_node_expansion(simd, e_moves, tables);
        let s_init = s_stack.begin_initial_node_expansion(simd, s_moves, tables);
        let m_init = m_stack.begin_initial_node_expansion(simd, m_moves, tables);

        let d_max_threshold = combined_phase_max.get();
        let d_min_threshold = N as u8;

        if d_max_threshold == 0 || d_min_threshold == 0 || d_max_threshold < d_min_threshold {
            unimplemented!()
        }

        let collected = CubeMove::all_iter()
            .zip_eq(e_init)
            .zip_eq(s_init)
            .zip_eq(m_init)
            .filter_map(|(((mv, e), s), m)| {
                let max_d = e.1.max(s.1).max(m.1);
                let min_d = e.1.min(s.1).min(m.1);

                let admissable = max_d < d_max_threshold && min_d < d_min_threshold;

                if admissable {
                    Some((mv, e, s, m))
                } else {
                    None
                }
            })
            .collect_vec();

        let e_collected = collected.iter().map(|(_, x, _, _)| *x);
        let s_collected = collected.iter().map(|(_, _, x, _)| *x);
        let m_collected = collected.iter().map(|(_, _, _, x)| *x);

        e_stack.end_initial_node_expansion(e_collected);
        s_stack.end_initial_node_expansion(s_collected);
        m_stack.end_initial_node_expansion(m_collected);

        let mut shared = [[Shared {
            next_pointer: 0,
            previous_move: 0,
            remaining_moves: 0,
        }; 15]; N];

        for (i, c) in collected.iter().enumerate() {
            let slot = get_entry_mut(i, &mut shared);
            slot.next_pointer = 1;
            slot.previous_move = c.0 as u8;
            slot.remaining_moves = (N - 1) as u8;
        }

        Self {
            tables,
            combined_phase_max,
            current_position: collected.len() as u16,
            shared,
            e_stack,
            s_stack,
            m_stack,
        }
    }

    #[inline(always)]
    fn node_expansion<S: fearless_simd::Simd>(&mut self, simd: S) -> i16 {
        use fearless_simd::{u8x16, u8x32, u8x64};
        let current_position = self.current_position as usize;
        let shared = *get_entry(current_position, &self.shared);
        debug_assert_ne!(0, shared.remaining_moves);
        let next_moves_remaining = shared.remaining_moves - 1;

        let previous_move = unsafe { CubeMove::from_u8(shared.previous_move) };

        let (moves, num_moves) = previous_move.new_axis_move_array_raw_new();

        let move_mask = u8x16::simd_from(
            simd,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        )
        .simd_lt(u8x16::splat(simd, num_moves as u8));

        let s_rot = u8x32::<S>::simd_from(simd, ROTATE_ARRAY[0]);
        let m_rot = u8x32::<S>::simd_from(simd, ROTATE_ARRAY[1]);
        let moves_e = u8x16::<S>::simd_from(simd, *moves);
        let moves_s: u8x16<S> = lut_u8x32(simd, s_rot, moves_e);
        let moves_m: u8x16<S> = lut_u8x32(simd, m_rot, moves_e);

        let mut partial_e =
            self.e_stack
                .begin_node_expansion(current_position, simd, moves_e, move_mask, self.tables);
        let mut partial_s =
            self.s_stack
                .begin_node_expansion(current_position, simd, moves_s, move_mask, self.tables);
        let mut partial_m =
            self.m_stack
                .begin_node_expansion(current_position, simd, moves_m, move_mask, self.tables);

        let combined_phase_max = self.combined_phase_max.get();

        let admissible: fearless_simd::mask8x16<S> = {
            let dist_e = *partial_e.get_dist();
            let dist_s = *partial_s.get_dist();
            let dist_m = *partial_m.get_dist();

            let dist_min = simd.min_u8x16(simd.min_u8x16(dist_e, dist_s), dist_m);
            let dist_max = simd.max_u8x16(simd.max_u8x16(dist_e, dist_s), dist_m);

            let min_threshold: u8 = next_moves_remaining;
            let max_threshold: u8 = (combined_phase_max + next_moves_remaining) - N as u8;

            let min = simd.simd_le_u8x16(dist_min, simd.splat_u8x16(min_threshold));
            let max = simd.simd_le_u8x16(dist_max, simd.splat_u8x16(max_threshold));
            min & max
        };

        if admissible.all_false() {
            return -(shared.next_pointer as i16);
        }

        let (swizzle, count) = mask_to_swizzle(simd, admissible);

        partial_e.swizzle(swizzle);
        partial_s.swizzle(swizzle);
        partial_m.swizzle(swizzle);

        let back_pointers_lo = u8x16::splat(simd, 1);
        let back_pointers_hi = u8x16::splat(simd, 0);
        let new_previous_moves = moves_e.swizzle_dyn(swizzle);
        let remaining_moves = simd.splat_u8x16(shared.remaining_moves - 1);

        #[cfg(target_endian = "little")]
        let vectors = [
            back_pointers_lo,
            back_pointers_hi,
            new_previous_moves,
            remaining_moves,
        ];

        #[cfg(target_endian = "big")]
        let vectors = [
            back_pointers_hi,
            back_pointers_lo,
            new_previous_moves,
            remaining_moves,
        ];

        {
            debug_assert_eq!(core::mem::size_of::<Shared>(), 4);
            let out_buf = get_entry_slice_mut(current_position + 1, &mut self.shared);
            let out_ptr: *mut [Shared; 16] = out_buf;
            let out_ptr: *mut [u8; 64] = out_ptr.cast();
            let out_buf: &mut [u8; 64] = unsafe { out_ptr.as_mut_unchecked() };

            simd.store_four_interleaved_u8x16(vectors, out_buf);
        }

        self.e_stack.end_node_expansion(simd, partial_e);
        self.s_stack.end_node_expansion(simd, partial_s);
        self.m_stack.end_node_expansion(simd, partial_m);

        count as i16
    }
}

const ROTATE_ARRAY: [[u8; 32]; 2] = {
    let mut output = [[0; 32]; 2];

    let mut i = 0;
    let mut mv = 0;
    while i < 2 {
        let sym = CubeSymmetry((i + 1) << 4);
        while mv < 18 {
            output[i as usize][mv as usize] = unsafe { CubeMove::from_u8(mv) }.conjugate(sym) as u8;
            mv += 1;
        }
        i += 1;
    }

    output
};

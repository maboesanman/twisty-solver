use std::{borrow::Borrow, mem::MaybeUninit};

use arrayvec::ArrayVec;
use fearless_simd::{SimdBase, SimdFrom, SimdMask};
use itertools::Itertools;

use crate::{
    CubeMove, ReprCube, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::CubeSymmetry},
    kociemba::search::{
        phase_2_node::Phase2Node,
        stack_new::{
            max_threshold::MaxThreshold,
            partial_stack::PartialStack,
            utils::{ConstGenericWorkaround, lut_u8x32, mask_to_swizzle},
        },
    },
};

#[derive(Clone)]
pub struct Stack<'t, const N: usize, const INC: bool, S, M> {
    pub tables: &'t Tables,
    simd: S,
    combined_phase_max: M,
    current_position: i16,
    base_dists: Dists,

    // a workaround since [Dists; 15 * N] is not allowed.
    dists: ConstGenericWorkaround<N, Dists>,
    shared: ConstGenericWorkaround<N, Shared>,
    e_stack: PartialStack<N>,
    s_stack: PartialStack<N>,
    m_stack: PartialStack<N>,
}

impl<'t, const N: usize, const INC: bool, S, M> core::fmt::Debug for Stack<'t, N, INC, S, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stack")
            .field("tables", &self.tables)
            // .field("combined_phase_max", &self.combined_phase_max)
            .field("current_position", &self.current_position)
            .field("base_dists", &self.base_dists)
            .field("dists", &self.dists)
            .field("shared", &self.shared)
            .field("e_stack", &self.e_stack)
            .field("s_stack", &self.s_stack)
            .field("m_stack", &self.m_stack)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct StackOutput<const N: usize> {
    phase_2: Phase2Node,
    moves: ArrayVec<CubeMove, N>,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Shared {
    next_pointer: u16, // 9 bits used
    // 5 bits of move, 3 bits of "if move went from domino reduced to domino reduced"
    // one for each axis
    previous_move_ext: u8, // 5 bits used
    remaining_moves: u8,   // 5 bits used
}

impl Shared {
    #[inline(always)]
    fn get_moves<const N: usize>(items: &[Shared]) -> ArrayVec<CubeMove, N> {
        items
            .into_iter()
            .chunk_by(|p| p.remaining_moves)
            .into_iter()
            .map(|(_, group)| group.last().unwrap())
            .map(|shared| unsafe { CubeMove::from_u8(shared.previous_move_ext & 0b00011111) })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Dists {
    e_dist: u8,
    s_dist: u8,
    m_dist: u8,
    _padding: u8,
}

impl Dists {
    fn new(e_dist: u8, s_dist: u8, m_dist: u8) -> Self {
        Self {
            e_dist,
            s_dist,
            m_dist,
            _padding: 0,
        }
    }

    fn get_mask(self) -> u8 {
        let x: [u8; 4] = unsafe { core::mem::transmute(self) };
        let x = u32::from_ne_bytes(x);

        let has_zero = (x.wrapping_sub(0x01010101)) & !x & 0x80808080;

        let compressed =
            ((has_zero >> 7) | (has_zero >> 14) | (has_zero >> 21) | (has_zero >> 28)) & 0xF;

        (compressed & !(1 << 3)) as u8
    }
}

impl<'t, const N: usize, const INC: bool, S: fearless_simd::Simd, M: MaxThreshold>
    Stack<'t, N, INC, S, M>
{
    pub fn new(
        cube: ReprCube,
        mut combined_phase_max: M,
        simd: S,
        tables: &'t Tables,
    ) -> (Self, ArrayVec<StackOutput<N>, 6>) {
        let mut initial_nodes = const { ArrayVec::new_const() };

        let e_cube = cube;
        let s_cube = cube.conjugate(CubeSymmetry(1 << 4));
        let m_cube = cube.conjugate(CubeSymmetry(2 << 4));

        let (mut e_stack, e_dist) = PartialStack::<N>::new(e_cube, tables);
        let (mut s_stack, s_dist) = PartialStack::<N>::new(s_cube, tables);
        let (mut m_stack, m_dist) = PartialStack::<N>::new(m_cube, tables);

        let base_dists = Dists::new(e_dist, s_dist, m_dist);

        let start_phase_2s = Self::get_phase_2_from_start(&e_stack, &s_stack, &m_stack, base_dists);
        if INC {
            initial_nodes.extend(start_phase_2s.into_iter().map(|phase_2| StackOutput {
                phase_2,
                moves: ArrayVec::new(),
            }));
        } else {
            if N == 0 {
                initial_nodes.extend(start_phase_2s.into_iter().map(|phase_2| StackOutput {
                    phase_2,
                    moves: ArrayVec::new(),
                }));
            }
        }

        let mut shared = ConstGenericWorkaround::<N, Shared>::new();

        let mut dists = ConstGenericWorkaround::<N, Dists>::new();

        if N == 0 {
            return (
                Self {
                    tables,
                    simd,
                    combined_phase_max,
                    current_position: -1,
                    base_dists,
                    dists,
                    shared,
                    e_stack,
                    s_stack,
                    m_stack,
                },
                initial_nodes,
            );
        }

        let e_moves = CubeMove::all_iter();
        let s_moves = CubeMove::all_iter().map(|m| m.conjugate(CubeSymmetry(1 << 4)));
        let m_moves = CubeMove::all_iter().map(|m| m.conjugate(CubeSymmetry(2 << 4)));

        let e_init = e_stack.begin_initial_node_expansion(e_moves, tables);
        let s_init = s_stack.begin_initial_node_expansion(s_moves, tables);
        let m_init = m_stack.begin_initial_node_expansion(m_moves, tables);

        let d_max_threshold = combined_phase_max.get();
        let d_min_threshold = N as u8;

        if d_max_threshold == 0 || d_min_threshold == 0 || d_max_threshold < d_min_threshold {
            unimplemented!()
        }

        let collected = CubeMove::all_iter()
            .zip_eq(e_init)
            .zip_eq(s_init)
            .zip_eq(m_init)
            .filter_map(|(((mv, mut e), mut s), mut m)| {
                e.1 = DISTANCE_OPTIONS_LOOKUP[e_dist as usize][e.1 as usize];
                s.1 = DISTANCE_OPTIONS_LOOKUP[s_dist as usize][s.1 as usize];
                m.1 = DISTANCE_OPTIONS_LOOKUP[m_dist as usize][m.1 as usize];

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

        if collected.is_empty() {
            return (
                Self {
                    tables,
                    simd,
                    combined_phase_max,
                    current_position: -1,
                    base_dists,
                    dists,
                    shared,
                    e_stack,
                    s_stack,
                    m_stack,
                },
                initial_nodes,
            );
        }

        let e_collected = collected.iter().map(|(_, x, _, _)| x.0).collect_vec();
        let s_collected = collected.iter().map(|(_, _, x, _)| x.0).collect_vec();
        let m_collected = collected.iter().map(|(_, _, _, x)| x.0).collect_vec();

        e_stack.end_initial_node_expansion(e_collected);
        s_stack.end_initial_node_expansion(s_collected);
        m_stack.end_initial_node_expansion(m_collected);

        for (i, c) in collected.iter().enumerate() {
            let slot = &mut shared[i];
            slot.next_pointer = 1;
            slot.previous_move_ext = c.0 as u8;
            slot.remaining_moves = (N - 1) as u8;

            let slot = &mut dists[i];
            slot.e_dist = c.1.1;
            slot.s_dist = c.2.1;
            slot.m_dist = c.3.1;
        }

        let current_position = (collected.len() - 1) as i16;
        let slot = dists[collected.len() - 1];

        let first_move = collected.last().unwrap().0;
        let start_phase_2s = Self::get_phase_2_from_first(
            &mut e_stack,
            &mut s_stack,
            &mut m_stack,
            slot,
            first_move as u8,
            tables,
        );
        let outputs = start_phase_2s.into_iter().map(|phase_2| StackOutput {
            phase_2,
            moves: {
                let mut arr = ArrayVec::new();
                arr.push(first_move);
                arr
            },
        });

        if INC {
            initial_nodes.extend(outputs);
        } else {
            if N == 0 {
                initial_nodes.extend(outputs);
            }
        }

        (
            Self {
                tables,
                simd,
                combined_phase_max,
                current_position,
                shared,
                e_stack,
                s_stack,
                m_stack,
                base_dists,
                dists,
            },
            initial_nodes,
        )
    }

    #[inline(always)]
    fn node_expansion(&mut self) -> i16 {
        use fearless_simd::{u8x16, u8x32};
        let current_position = self.current_position as usize;
        let shared = self.shared[current_position];
        if shared.remaining_moves == 0 {
            return -(shared.next_pointer as i16);
        }
        let dists = self.dists[current_position];
        let next_moves_remaining = shared.remaining_moves - 1;

        let previous_move = unsafe { CubeMove::from_u8(shared.previous_move_ext & 0b00011111) };

        let (moves_ext, num_moves) = previous_move.new_axis_move_array_raw_new();

        let move_mask = u8x16::simd_from(
            self.simd,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        )
        .simd_lt(u8x16::splat(self.simd, num_moves as u8));

        let s_rot = u8x32::<S>::simd_from(self.simd, ROTATE_ARRAY[0]);
        let m_rot = u8x32::<S>::simd_from(self.simd, ROTATE_ARRAY[1]);
        let moves_e_ext = u8x16::<S>::simd_from(self.simd, *moves_ext);
        let moves_e = moves_e_ext & u8x16::splat(self.simd, 0b000_11111);
        let moves_s_ext: u8x16<S> = lut_u8x32(self.simd, s_rot, moves_e);
        let moves_m_ext: u8x16<S> = lut_u8x32(self.simd, m_rot, moves_e);
        let moves_s = moves_s_ext & u8x16::splat(self.simd, 0b000_11111);
        let moves_m = moves_m_ext & u8x16::splat(self.simd, 0b000_11111);

        let mut e_begin_result = self.e_stack.begin_node_expansion(
            current_position,
            self.simd,
            moves_e,
            move_mask,
            self.tables,
        );
        let mut s_begin_result = self.s_stack.begin_node_expansion(
            current_position,
            self.simd,
            moves_s,
            move_mask,
            self.tables,
        );
        let mut m_begin_result = self.m_stack.begin_node_expansion(
            current_position,
            self.simd,
            moves_m,
            move_mask,
            self.tables,
        );

        let combined_move_mask = (0b001_11111 & e_begin_result.previous_move_mask)
            | (0b010_11111 & s_begin_result.previous_move_mask)
            | (0b100_11111 & m_begin_result.previous_move_mask);

        let combined_phase_max = self.combined_phase_max.get();

        let dist_e = Self::phase_1_dist_lookup(self.simd, dists.e_dist)
            .swizzle_dyn(e_begin_result.distances_mod_3);
        let dist_s = Self::phase_1_dist_lookup(self.simd, dists.s_dist)
            .swizzle_dyn(s_begin_result.distances_mod_3);
        let dist_m = Self::phase_1_dist_lookup(self.simd, dists.m_dist)
            .swizzle_dyn(s_begin_result.distances_mod_3);

        let admissible: fearless_simd::mask8x16<S> = {
            let dist_min = self
                .simd
                .min_u8x16(self.simd.min_u8x16(dist_e, dist_s), dist_m);
            let dist_max = self
                .simd
                .max_u8x16(self.simd.max_u8x16(dist_e, dist_s), dist_m);

            let min_threshold: u8 = next_moves_remaining;
            let max_threshold: u8 = (combined_phase_max + next_moves_remaining) - N as u8;

            let min = self
                .simd
                .simd_le_u8x16(dist_min, self.simd.splat_u8x16(min_threshold));
            let max = self
                .simd
                .simd_le_u8x16(dist_max, self.simd.splat_u8x16(max_threshold));
            min & max
        };

        if admissible.all_false() {
            return -(shared.next_pointer as i16);
        }

        let (swizzle, count) = mask_to_swizzle(self.simd, admissible);

        e_begin_result.partial_expansion_result.swizzle(swizzle);
        s_begin_result.partial_expansion_result.swizzle(swizzle);
        m_begin_result.partial_expansion_result.swizzle(swizzle);

        let mut back_pointers_lo = u8x16::splat(self.simd, 1);
        let mut back_pointers_hi = u8x16::splat(self.simd, 0);
        let first_backpointer = shared.next_pointer + 1;
        back_pointers_lo[0] = (first_backpointer & 0xFF) as u8;
        back_pointers_hi[0] = (first_backpointer >> 8) as u8;
        let moves = moves_e_ext & self.simd.splat_u8x16(combined_move_mask);
        let new_previous_moves = moves.swizzle_dyn(swizzle);
        let remaining_moves = self.simd.splat_u8x16(shared.remaining_moves - 1);

        // interleaved store of the shared buffer
        {
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

            debug_assert_eq!(core::mem::size_of::<Shared>(), 4);
            let out_buf = self.shared.get_array_mut(current_position + 1);
            let out_ptr: *mut [Shared; 16] = out_buf;
            let out_ptr: *mut [u8; 64] = out_ptr.cast();
            let out_buf: &mut [u8; 64] = unsafe { out_ptr.as_mut_unchecked() };

            self.simd.store_four_interleaved_u8x16(vectors, out_buf);
        }

        // interleaved store of the dists buffer
        {
            let vectors = [dist_e, dist_s, dist_m, u8x16::splat(self.simd, 0)];

            debug_assert_eq!(core::mem::size_of::<Dists>(), 4);
            let out_buf = self.dists.get_array_mut(current_position + 1);
            let out_ptr: *mut [Dists; 16] = out_buf;
            let out_ptr: *mut [u8; 64] = out_ptr.cast();
            let out_buf: &mut [u8; 64] = unsafe { out_ptr.as_mut_unchecked() };

            self.simd.store_four_interleaved_u8x16(vectors, out_buf);
        }

        self.e_stack
            .end_node_expansion(self.simd, e_begin_result.partial_expansion_result);
        self.s_stack
            .end_node_expansion(self.simd, s_begin_result.partial_expansion_result);
        self.m_stack
            .end_node_expansion(self.simd, m_begin_result.partial_expansion_result);

        count as i16
    }

    fn phase_1_dist_lookup(simd: S, dist: u8) -> fearless_simd::u8x16<S> {
        debug_assert!((dist as usize) < DISTANCE_OPTIONS_LOOKUP.len());
        let dist_lookup = unsafe { DISTANCE_OPTIONS_LOOKUP.get_unchecked(dist as usize) };
        fearless_simd::u8x16::simd_from(simd, *dist_lookup)
    }

    #[inline(always)]
    pub fn get_phase_2(&mut self) -> ArrayVec<StackOutput<N>, 3> {
        let dists = &self.dists[self.current_position as usize];
        let mask = dists.get_mask();
        debug_assert!(mask < 8);
        let shared = self.shared[self.current_position as usize];
        let prev_move_ext = shared.previous_move_ext;
        let moves = Shared::get_moves(&Borrow::<[Shared]>::borrow(&self.shared)[..self.current_position as usize]);
        let stack = match mask & (prev_move_ext >> 5) {
            0b000 => return const { ArrayVec::new_const() },
            // single
            0b001 => &mut self.e_stack,
            0b010 => &mut self.s_stack,
            0b100 => &mut self.m_stack,

            // double
            0b011 => {
                return Self::get_phase_2_inner_double(
                    [&mut self.e_stack, &mut self.s_stack],
                    moves,
                    self.tables,
                );
            }
            0b101 => {
                return Self::get_phase_2_inner_double(
                    [&mut self.e_stack, &mut self.m_stack],
                    moves,
                    self.tables,
                );
            }
            0b110 => {
                return Self::get_phase_2_inner_double(
                    [&mut self.s_stack, &mut self.m_stack],
                    moves,
                    self.tables,
                );
            }

            // triple
            0b111 => {
                return Self::get_phase_2_inner_triple(
                    [&mut self.e_stack, &mut self.s_stack, &mut self.m_stack],
                    moves,
                    self.tables,
                );
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        };

        Self::get_phase_2_inner_single(stack, moves, self.tables)
    }

    #[inline(always)]
    fn get_phase_2_inner_single(
        partial_stack: &mut PartialStack<N>,
        moves: ArrayVec<CubeMove, N>,
        tables: &Tables,
    ) -> ArrayVec<StackOutput<N>, 3> {
        let mut output = ArrayVec::new();
        let phase_2 = partial_stack
            .populate_phase_2_cache(moves.iter().map(|mv| mv.into_u8()), tables)
            .resolve();
        let a = StackOutput { phase_2, moves };
        unsafe {
            output.push_unchecked(a);
        }
        output
    }

    #[cold]
    #[inline(never)]
    fn get_phase_2_inner_double(
        partial_stack: [&mut PartialStack<N>; 2],
        moves: ArrayVec<CubeMove, N>,
        tables: &Tables,
    ) -> ArrayVec<StackOutput<N>, 3> {
        let mut output = ArrayVec::new();
        let phase_2 = partial_stack[0]
            .populate_phase_2_cache(moves.iter().map(|mv| mv.into_u8()), tables)
            .resolve();
        let a = StackOutput {
            phase_2,
            moves: moves.clone(),
        };
        let phase_2 = partial_stack[1]
            .populate_phase_2_cache(moves.iter().map(|mv| mv.into_u8()), tables)
            .resolve();
        let b = StackOutput { phase_2, moves };
        unsafe {
            output.push_unchecked(a);
        }
        unsafe {
            output.push_unchecked(b);
        }
        output
    }

    #[cold]
    #[inline(never)]
    fn get_phase_2_inner_triple(
        partial_stack: [&mut PartialStack<N>; 3],
        moves: ArrayVec<CubeMove, N>,
        tables: &Tables,
    ) -> ArrayVec<StackOutput<N>, 3> {
        let mut output = ArrayVec::new();
        let phase_2 = partial_stack[0]
            .populate_phase_2_cache(moves.iter().map(|mv| mv.into_u8()), tables)
            .resolve();
        let a = StackOutput {
            phase_2,
            moves: moves.clone(),
        };
        let phase_2 = partial_stack[1]
            .populate_phase_2_cache(moves.iter().map(|mv| mv.into_u8()), tables)
            .resolve();
        let b = StackOutput {
            phase_2,
            moves: moves.clone(),
        };
        let phase_2 = partial_stack[2]
            .populate_phase_2_cache(moves.iter().map(|mv| mv.into_u8()), tables)
            .resolve();
        let c = StackOutput { phase_2, moves };
        unsafe {
            output.push_unchecked(a);
        }
        unsafe {
            output.push_unchecked(b);
        }
        unsafe {
            output.push_unchecked(c);
        }
        output
    }

    fn get_phase_2_from_start(
        e_stack: &PartialStack<N>,
        s_stack: &PartialStack<N>,
        m_stack: &PartialStack<N>,
        dists: Dists,
    ) -> ArrayVec<Phase2Node, 3> {
        let mut output = const { ArrayVec::new_const() };

        if dists.e_dist == 0 {
            unsafe { output.push_unchecked(e_stack.base_phase_2_coords.resolve()) };
        }

        if dists.s_dist == 0 {
            unsafe { output.push_unchecked(s_stack.base_phase_2_coords.resolve()) };
        }

        if dists.m_dist == 0 {
            unsafe { output.push_unchecked(m_stack.base_phase_2_coords.resolve()) };
        }

        output
    }

    fn get_phase_2_from_first(
        e_stack: &mut PartialStack<N>,
        s_stack: &mut PartialStack<N>,
        m_stack: &mut PartialStack<N>,
        dists: Dists,
        mv: u8,
        tables: &Tables,
    ) -> ArrayVec<Phase2Node, 3> {
        let mut output = const { ArrayVec::new_const() };

        if dists.e_dist == 0 {
            let phase_2 = e_stack.populate_phase_2_cache(Some(mv).into_iter(), tables);
            let phase_2 = phase_2.resolve();
            output.push(phase_2);
        }

        if dists.s_dist == 0 {
            let mv = ROTATE_ARRAY[0][mv as usize];
            let phase_2 = s_stack.populate_phase_2_cache(Some(mv).into_iter(), tables);
            let phase_2 = phase_2.resolve();
            output.push(phase_2);
        }

        if dists.m_dist == 0 {
            let mv = ROTATE_ARRAY[1][mv as usize];
            let phase_2 = m_stack.populate_phase_2_cache(Some(mv).into_iter(), tables);
            let phase_2 = phase_2.resolve();
            output.push(phase_2);
        }

        output
    }

    fn emittable(shared: Shared, dists: Dists) -> bool {
        if INC {
            shared.remaining_moves == 0
        } else {
            dists.e_dist == 0 || dists.s_dist == 0 || dists.m_dist == 0
        }
    }
}

impl<'t, const N: usize, const INC: bool, S: fearless_simd::Simd, M: MaxThreshold> Iterator
    for Stack<'t, N, INC, S, M>
{
    type Item = ArrayVec<StackOutput<N>, 3>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // we assume we have emitted the top already if it's emittable.
            let nudge = self.node_expansion();
            self.current_position += nudge;

            if self.current_position.is_negative() {
                break None;
            }

            if !INC {
                if self.shared[self.current_position as usize].remaining_moves > 0 {
                    continue;
                }
            }

            break Some(self.get_phase_2());
        }
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

const DISTANCE_OPTIONS_LOOKUP: [[u8; 16]; 13] = const {
    let mut out = [[0; 16]; 13];

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

#[cfg(test)]
mod tests {
    use rand::{
        SeedableRng,
        distr::{Distribution, StandardUniform},
    };
    use rand_chacha::ChaCha8Rng;

    use crate::kociemba::search::stack_new::{max_threshold::LocalMaxThreshold, partial_stack};
    use crate::{cube, cube_ops::cube_sym::CubeSymmetry};

    use super::*;

    #[inline(always)]
    fn test_simd() -> impl fearless_simd::Simd {
        // SAFETY: Neon is a baseline feature on aarch64.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            fearless_simd::Neon::assume_supported()
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            fearless_simd::Avx2::assume_supported()
        }
    }

    fn make_stack<const N: usize, const MAX_D: u8>(
        cube: ReprCube,
        tables: &Tables,
    ) -> Stack<'_, N, false, impl fearless_simd::Simd, LocalMaxThreshold> {
        let simd = test_simd();
        let (stack, _) = Stack::new(cube, LocalMaxThreshold(MAX_D), simd, tables);
        println!(
            "constructed stack with {} initial nodes",
            stack.current_position
        );
        stack
    }

    #[test]
    fn explore_new() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        for _ in 0..10 {
            let cube: ReprCube = Distribution::sample(&StandardUniform, &mut rng);
            make_stack::<19, 20>(cube, &tables);
        }

        Ok(())
    }

    /// The distance estimate `Stack` stores for a child whose base cube sits at
    /// true phase-1 distance `base_d` and whose residue mod 3 is `residue`.
    ///
    /// Mirrors DISTANCE_OPTIONS_LOOKUP in partial_stack.rs: among {d-1, d,
    /// d+1} (clamped to [0,12]) the value congruent to `residue` mod 3 wins.
    fn expected_dist(base_d: u8, residue: u8) -> u8 {
        for delta in [-1i16, 0, 1] {
            let new_d = ((base_d as i16 + delta).clamp(0, 12)) as u8;
            if new_d % 3 == residue {
                return new_d;
            }
        }
        unreachable!()
    }

    /// Verify that the initial nodes produced by `Stack::new` match, in the phase-1
    /// domain, what we get from manually applying all 18 moves to a random cube.
    ///
    /// Each of the three frames (e, s, m) sees a partially-conjugated view of the
    /// cube and conjugates its moves by the same symmetry, so all three describe
    /// the very same physical move sequence. We compare `Phase1` structs directly:
    /// both sides are produced in the forward (cube -> Phase1) direction.
    fn verify_stack_initial_nodes(rng: &mut ChaCha8Rng, tables: &Tables) -> anyhow::Result<bool> {
        let sym_s = CubeSymmetry(1 << 4);
        let sym_m = CubeSymmetry(2 << 4);

        for _ in 0..10 {
            let cube: ReprCube = Distribution::sample(&StandardUniform, rng);
            let stack = make_stack::<19, 20>(cube, tables);

            let e_cube = cube;
            let s_cube = cube.conjugate(sym_s);
            let m_cube = cube.conjugate(sym_m);

            // Base (pre-move) phase-1 coordinates + true distances for each frame.
            let e_base_p1 = partial_stack::Phase1::from_cube(e_cube, tables);
            let s_base_p1 = partial_stack::Phase1::from_cube(s_cube, tables);
            let m_base_p1 = partial_stack::Phase1::from_cube(m_cube, tables);

            let e_base_d = e_base_p1.dist_recurse(tables);
            let s_base_d = s_base_p1.dist_recurse(tables);
            let m_base_d = m_base_p1.dist_recurse(tables);

            // Sanity: the stored base nodes match these manually produced ones.
            assert_eq!(e_base_p1, stack.e_stack.base_node());
            assert_eq!(s_base_p1, stack.s_stack.base_node());
            assert_eq!(m_base_p1, stack.m_stack.base_node());

            for (i, mv) in CubeMove::all_iter().enumerate() {
                // --- e frame: base cube is `cube`, move is the canonical one. ---
                let e_child_cube = e_cube.apply_cube_move(mv);
                let e_expected_p1 = partial_stack::Phase1::from_cube(e_child_cube, tables);
                let e_residue = e_expected_p1.dist_mod_3(tables);
                let e_expected_d = expected_dist(e_base_d, e_residue);

                // --- s frame: base cube conjugated by sym_s; move too. ---
                let s_child_cube = s_cube.apply_cube_move(mv.conjugate(sym_s));
                let s_expected_p1 = partial_stack::Phase1::from_cube(s_child_cube, tables);
                let s_residue = s_expected_p1.dist_mod_3(tables);
                let s_expected_d = expected_dist(s_base_d, s_residue);

                // --- m frame: base cube conjugated by sym_m; move too. ---
                let m_child_cube = m_cube.apply_cube_move(mv.conjugate(sym_m));
                let m_expected_p1 = partial_stack::Phase1::from_cube(m_child_cube, tables);
                let m_residue = m_expected_p1.dist_mod_3(tables);
                let m_expected_d = expected_dist(m_base_d, m_residue);

                // Compare against what each axis actually stored in `Stack::new`.
                let e_actual_p1 = stack.e_stack.initial_node(i);
                let s_actual_p1 = stack.s_stack.initial_node(i);
                let m_actual_p1 = stack.m_stack.initial_node(i);

                // Compare in a canonical phase-1 form so equivalent symmetry
                // representatives compare equal (see `Phase1::canonicalize`).
                for (expected, actual) in [
                    (&e_expected_p1.canonicalize(tables), &e_actual_p1),
                    (&s_expected_p1.canonicalize(tables), &s_actual_p1),
                    (&m_expected_p1.canonicalize(tables), &m_actual_p1),
                ] {
                    if *expected != actual.canonicalize(tables) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Compare the initial nodes produced by `Stack::new` for each of the three
    /// frames (e, s, m) against the phase-1 coordinates and distance estimates we
    /// would expect from manually applying all 18 moves to a random cube.
    #[test]
    fn initial_nodes_match_manual_moves() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        assert!(
            verify_stack_initial_nodes(&mut rng, &tables)?,
            "stored initial nodes did not match manually-constructed phase-1 coords"
        );

        Ok(())
    }

    #[test]
    fn explore_node_expansion() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let cube = cube![U F R];
        let simd = test_simd();
        let (stack, already_reduced) =
            Stack::<4, false, _, _>::new(cube, LocalMaxThreshold(10), simd, &tables);

        debug_assert_eq!(already_reduced.len(), 0);

        println!("{stack:#?}");

        for a in stack {
            println!("{a:?}");
        }

        Ok(())
    }

    #[test]
    fn test_stack_new() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let simd = test_simd();

        let (stack, items) =
            Stack::<0, false, _, _>::new(ReprCube::SOLVED, LocalMaxThreshold(20), simd, &tables);
        for i in items {
            println!("{i:?}");
        }
        println!("a");
        let (stack, items) =
            Stack::<0, true, _, _>::new(ReprCube::SOLVED, LocalMaxThreshold(20), simd, &tables);
        for i in items {
            println!("{i:?}");
        }
        println!("b");
        let (stack, items) =
            Stack::<1, false, _, _>::new(ReprCube::SOLVED, LocalMaxThreshold(20), simd, &tables);
        for i in items {
            println!("{i:?}");
        }
        println!("c");
        let (stack, items) =
            Stack::<1, true, _, _>::new(ReprCube::SOLVED, LocalMaxThreshold(20), simd, &tables);
        for i in items {
            println!("{i:?}");
        }
        Ok(())
    }
}

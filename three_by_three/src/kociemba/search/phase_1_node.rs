use std::{hint::unreachable_unchecked, marker::PhantomData, num::NonZeroU8, ptr, simd::{Mask, Simd, cmp::SimdPartialOrd, num::{SimdFloat, SimdUint}, ptr::SimdConstPtr}};

use itertools::Itertools;

use crate::{
    CornerOrient, CornerPerm, CubeMove, EdgeOrient, ReprCube, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientRawCoord, EdgeGroupOrientSymCoord},
            corner_perm_combo_coord::CornerPermComboCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, UEdgePositions, combine_edge_positions,
            split_edge_positions,
        },
    },
};

#[repr(C)]
#[repr(align(4))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Phase1Node {
    // corners
    pub corner_orient_raw: CornerOrientRawCoord,
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,
    
    // edges
    pub corner_perm_sym: CornerPermSymCoord,
    pub u_edge_positions: UEdgePositions,
    pub d_edge_positions: DEdgePositions,
    pub e_edge_positions: EEdgePositions,

    pub edge_group_orient_correct: DominoSymmetry,
    pub corner_perm_correct: DominoSymmetry,
    // bookkeeping
    pub previous_axis: CubePreviousAxis,
}

// a set of nodes which came from the same call to `produce_next_nodes`
pub struct Phase1FrameMetadata<I> {
    pub children: I,
    // pub min_possible_distance: u8,
    pub max_possible_distance: u8,
}

impl Phase1Node {
    pub fn from_cube(cube: ReprCube, tables: &Tables) -> Self {
        let ReprCube {
            corner_perm,
            corner_orient,
            edge_perm,
            edge_orient,
        } = cube;

        let (u_edge_positions, d_edge_positions, e_edge_positions) =
            split_edge_positions(edge_perm);

        let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
        let edge_orient_raw = edge_orient.into_coord();
        let edge_group_orient_raw_coord =
            EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

        let edge_group_orient_combo =
            EdgeGroupOrientComboCoord::from_raw(tables, edge_group_orient_raw_coord);
        let corner_perm_combo = CornerPermComboCoord::from_raw(tables, corner_perm.into_coord());
        let corner_orient_raw = corner_orient.into_coord();

        Self {
            corner_orient_raw,
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis: CubePreviousAxis::None,
            corner_perm_sym: corner_perm_combo.sym_coord,
            edge_group_orient_sym: edge_group_orient_combo.sym_coord,
            corner_perm_correct: corner_perm_combo.domino_conjugation,
            edge_group_orient_correct: edge_group_orient_combo.domino_conjugation,
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let edge_perm = combine_edge_positions(
            self.u_edge_positions,
            self.d_edge_positions,
            self.e_edge_positions,
        );
        let corner_perm_combo = CornerPermComboCoord {
            sym_coord: self.corner_perm_sym,
            domino_conjugation: self.corner_perm_correct
        };
        let corner_perm = CornerPerm::from_coord(corner_perm_combo.into_raw(tables));

        let edge_group_orient_combo = EdgeGroupOrientComboCoord {
            sym_coord: self.edge_group_orient_sym,
            domino_conjugation: self.edge_group_orient_correct
        };
        let edge_orient =
            EdgeOrient::from_coord(edge_group_orient_combo.into_raw(tables).split().1);
        let corner_orient = CornerOrient::from_coord(self.corner_orient_raw);

        ReprCube {
            corner_perm,
            corner_orient,
            edge_perm,
            edge_orient,
        }
    }

    #[inline(always)]
    pub fn distance_heuristic(
        self,
        tables: &Tables,
    ) -> u8 {
        let corner_orient_adjusted = tables.move_raw_corner_orient.domino_conjugate(
            self.corner_orient_raw,
            self.edge_group_orient_correct
        );
        let distance = tables.get_prune_phase_1().get_value(
            self.edge_group_orient_sym,
            corner_orient_adjusted,
        );

        distance
    }

    #[inline(always)]
    pub fn is_domino_reduced(
        self,
    ) -> bool {
        self.corner_orient_raw.0 == 0 && self.edge_group_orient_sym.0 == 0
    }

    #[inline(always)]
    pub fn produce_next_nodes(
        self,
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        tables: &Tables,
    ) -> Option<Phase1FrameMetadata<impl Iterator<Item = Self>>> {
        // TODO: try to do all this stuff as SIMD. seems like a good candidate.

        // Get bounds for the current distance from self to solved, with the restriction
        // that the range must be within the allowed. If the range we have is not a subset of
        // [min_allowed_distance, max_allowed_distance], then we look up the actual distance to solved and
        // return a legal single point range or return None because we're outside the range and must be pruned.
        let max_possible_current_distance = if max_possible_distance > moves_remaining.get() {
            let distance = self.distance_heuristic(tables);

            if distance > moves_remaining.get() {
                return None;
            }

            distance
        } else {
            max_possible_distance
        };

        // prepare the values for feeding the child nodes.
        let max_possible_distance = max_possible_current_distance + 1;

        // perform all new axis moves on all coords
        let move_iter = || CubeMove::new_axis_iter(self.previous_axis, moves_remaining.get() == 1);

        let children = tables
            .move_edge_position
            .apply_all_cube_moves(
                self.u_edge_positions,
                self.d_edge_positions,
                self.e_edge_positions,
                move_iter(),
            )
            .into_iter()
            .zip_eq(move_iter())
            .map(
                move |((u_edge_positions, d_edge_positions, e_edge_positions), cube_move)| {
                    let corner_perm_combo = CornerPermComboCoord {
                        sym_coord: self.corner_perm_sym,
                        domino_conjugation: self.corner_perm_correct,
                    }.apply_cube_move(tables, cube_move);

                    let edge_group_orient_combo = EdgeGroupOrientComboCoord {
                        sym_coord: self.edge_group_orient_sym,
                        domino_conjugation: self.edge_group_orient_correct,
                    }.apply_cube_move(tables, cube_move);

                    Phase1Node {
                        corner_orient_raw: tables
                            .move_raw_corner_orient
                            .apply_cube_move(self.corner_orient_raw, cube_move),
                        u_edge_positions,
                        d_edge_positions,
                        e_edge_positions,
                        previous_axis: self
                            .previous_axis
                            .update_with_new_move(cube_move, moves_remaining.get() - 1),
                        corner_perm_sym: corner_perm_combo.sym_coord,
                        edge_group_orient_sym: edge_group_orient_combo.sym_coord,
                        corner_perm_correct: corner_perm_combo.domino_conjugation,
                        edge_group_orient_correct: edge_group_orient_combo.domino_conjugation,
                    }
                },
            )
            .filter(move |child| {
                // the last cube must be domino reduced, so if moves_remaining is 1,
                // we need to filter for only cubes which are reduced already.
                //
                // additionally, there's an interesting optimization here.
                // there are no domino sequences of 7 moves or less which can be done in fewer moves when treated
                // as a non-domino. this means that if our domino reduction is ever distance 0 at two distinct points
                // within 7 moves, those moves could be replaced by the same number of domino moves.
                // now consider the last position of phase 1, which is distance 0. if we are distance 0 within 7 moves
                // of the final position, that sequence could be replaced by domino moves, which means it will not be shorter
                // than a path already found, because there would exist a solution with a shorter phase 1 ending at the
                // first domino reduction, and staying in domino moves, likely more optimally but never longer.
                let child_is_reduced = child.is_domino_reduced();

                let last_move = moves_remaining.get() == 1;
                if last_move {
                    return child_is_reduced;
                }

                let too_close_to_be_solved = moves_remaining.get() <= 8;
                if too_close_to_be_solved {
                    return !child_is_reduced;
                }

                true
            });

        Some(Phase1FrameMetadata {
            children,
            max_possible_distance,
        })
    }

    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    pub fn produce_next_nodes_simd(
        slice: &mut [Phase1Node; 16],
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        table_offsets: &TableOffsets,
        tables: &Tables,
    ) -> (usize, u8) {

        // Get bounds for the current distance from self to solved, with the restriction
        // that the range must be within the allowed. If the range we have is not a subset of
        // [min_allowed_distance, max_allowed_distance], then we look up the actual distance to solved and
        // return a legal single point range or return None because we're outside the range and must be pruned.
        let max_possible_current_distance = if max_possible_distance > moves_remaining.get() {
            let distance = slice[0].distance_heuristic(tables);

            if distance > moves_remaining.get() {
                return (0, 0);
            }

            distance
        } else {
            max_possible_distance
        };

        // prepare the values for feeding the child nodes.
        let max_possible_distance = max_possible_current_distance + 1;

        let corner_perm_correct = slice[0].corner_perm_correct;
        let edge_group_orient_correct = slice[0].edge_group_orient_correct;
        let base_offsets = table_offsets.node_to_offsets(slice[0]);

        let MoveSimd {
            phantom: _,
            a,
            b,
            count,
            new_prev_moves,
        } = table_offsets.get_simd_resources(slice[0].previous_axis);

        println!("{new_prev_moves:?}");
        let mut count = *count;

        let chunk_a: [[u16; 8]; 8] = {
            let a = a.wrapping_add(base_offsets);
            let a = unsafe { Simd::gather_ptr(a) };
            let a = a.to_array();
            unsafe { core::mem::transmute(a) }
        };

        let chunk_b: [[u16; 8]; 8] = {
            let b = b.wrapping_add(base_offsets);
            let b_mask = const { 
                let mut array = [0; 64];
                let mut i = 0;
                while i < 64 {
                    array[i] = 8 + i / 8;
                    i += 1;
                }
                Simd::from_array(array)
            }.simd_lt(Simd::splat(count));
            let b = unsafe { Simd::gather_select_ptr(b, b_mask, const {
                Simd::from_array([0u16; 64])
            })};
            let b = b.to_array();
            unsafe { core::mem::transmute(b) }
        };

        let dst_u16: &mut [[u16; 8]; 16] = unsafe {
            &mut *(slice.as_mut_ptr() as *mut [[u16; 8]; 16])
        };

        dst_u16[1..9].copy_from_slice(&chunk_a);
        dst_u16[9..16].copy_from_slice(&chunk_b[..7]);

        let dst_u32: &mut [[u32; 4]; 16] = unsafe {
            &mut *(dst_u16.as_mut_ptr() as *mut [[u32; 4]; 16])
        };

        let mut i = 1;
        let required_to_be_reduced = moves_remaining.get() == 1;
        let allowed_to_be_reduced = moves_remaining.get() <= 8;

        while i <= count {
            let is_reduced = dst_u32[i][0] == 0;

            if (allowed_to_be_reduced) || (is_reduced == required_to_be_reduced) {
                i += 1;
                continue;
            }

            // entry at i is invalid → replace it with last live entry
            count -= 1;

            if i != count {
                dst_u32[i] = dst_u32[count];
            }
            // do NOT increment i here; re-check swapped-in entry
        }
        
        let dst_u8: &mut [[u8; 16]; 16] = unsafe {
            &mut *(dst_u32.as_mut_ptr() as *mut [[u8; 16]; 16])
        };

        for i in 1..16 {
            // dst_u8[i][12] = corner_perm_correct.then(DominoSymmetry(dst_u8[i][13])).0;
            // dst_u8[i][13] = edge_group_orient_correct.then(DominoSymmetry(dst_u8[i][15])).0;
            dst_u8[i][14] = new_prev_moves.as_array()[i];
        }

        (count, max_possible_distance)
    }


}

#[derive(Clone)]
struct MoveSimd<'t> {
    phantom: PhantomData<&'t Tables>,

    // base 
    a: Simd<*const u16, 64>,
    b: Simd<*const u16, 64>,
    new_prev_moves: Simd<u8, 16>,
    count: usize,
}

impl<'t> MoveSimd<'t> {
    fn new(prev_axis: CubePreviousAxis, tables: &'t Tables) -> Self {
        let moves = CubeMove::new_axis_iter(prev_axis, false);
        let mut data = [ptr::null::<u16>(); 128];
        let mut new_prev_moves = [0; 16];
        let base_addresses = unsafe { [
            tables.move_raw_corner_orient.as_ptr(),
            tables.move_sym_edge_group_orient.as_ptr(),
            tables.move_sym_corner_perm.as_ptr(),
            tables.move_edge_position.as_ptr(),
            tables.move_edge_position.as_ptr(),
            tables.move_edge_position.as_ptr(),
            tables.move_sym_edge_group_orient.as_ptr().add(1),
            tables.move_sym_corner_perm.as_ptr().add(1),
        ]};
        let move_amount = [
            1,
            2,
            2,
            1,
            1,
            1,
            2,
            2,
        ];
        let mut i = 0;
        let mut count = 0;
        for mv in moves.into_iter() {
            for j in 0..8 {
                data[i] = unsafe { base_addresses[j].add(move_amount[j] * mv.into_index()) };
                i += 1;
            }
            new_prev_moves[count] = prev_axis.update_with_new_move(mv, 100) as u8;
            count += 1;
        }

        println!("{:?}", new_prev_moves);

        let data: [[*const u16; 64]; 2] = unsafe { core::mem::transmute(data)};
        let a = Simd::from_array(data[0]);
        let b = Simd::from_array(data[1]);
        let new_prev_moves = Simd::from_array(new_prev_moves);

        Self {
            phantom: PhantomData,
            a, b, count, new_prev_moves
        }
    }
}

#[derive(Clone)]
struct MoveSimdEnd<'t> {
    phantom: PhantomData<&'t Tables>,

    // base 
    data: Simd<*const u16, 64>,
    new_prev_moves: Simd<u8, 8>,
    count: usize,
}

impl<'t> MoveSimdEnd<'t> {
    fn new(prev_axis: CubePreviousAxis, tables: &'t Tables) -> Self {
        let moves = CubeMove::new_axis_iter(prev_axis, true);
        let mut data = [ptr::null::<u16>(); 64];
        let mut mask = [false; 64];
        let mut new_prev_moves = [0; 8];
        let base_addresses = unsafe { [
            tables.move_raw_corner_orient.as_ptr(),
            tables.move_sym_edge_group_orient.as_ptr(),
            tables.move_sym_corner_perm.as_ptr(),
            tables.move_edge_position.as_ptr(),
            tables.move_edge_position.as_ptr(),
            tables.move_edge_position.as_ptr(),
            tables.move_sym_edge_group_orient.as_ptr().add(1),
            tables.move_sym_corner_perm.as_ptr().add(1),
        ]};
        let move_amount = [
            1,
            2,
            2,
            1,
            1,
            1,
            2,
            2,
        ];
        let mut i = 0;
        let mut count = 0;
        for mv in moves.into_iter() {
            for j in 0..8 {
                data[i] = unsafe { base_addresses[j].add(move_amount[j] * mv.into_index()) };
                mask[i] = true;
                i += 1;
            }
            new_prev_moves[count] = prev_axis.update_with_new_move(mv, 100) as u8;
            count += 1;
        }
        let data = Simd::from_array(data);
        let new_prev_moves = Simd::from_array(new_prev_moves);

        Self {
            phantom: PhantomData,
            data, count,
            new_prev_moves,
        }
    }
}

#[derive(Clone)]
pub struct TableOffsets<'t> {
    phantom: PhantomData<&'t Tables>,

    u: MoveSimd<'t>,
    d_ud: MoveSimd<'t>,
    f: MoveSimd<'t>,
    b_fb: MoveSimd<'t>,
    r: MoveSimd<'t>,
    l_rl: MoveSimd<'t>,

    end_u_d_ud: MoveSimdEnd<'t>,
    end_f: MoveSimdEnd<'t>,
    end_b_fb: MoveSimdEnd<'t>,
    end_r: MoveSimdEnd<'t>,
    end_l_rl: MoveSimdEnd<'t>,
}

unsafe impl<'t> Send for TableOffsets<'t> {}
unsafe impl<'t> Sync for TableOffsets<'t> {}

impl<'t> TableOffsets<'t> {
    pub fn new(tables: &'t Tables) -> Self {
        let u = MoveSimd::new(CubePreviousAxis::U, tables);
        let d_ud = MoveSimd::new(CubePreviousAxis::D, tables);
        let f = MoveSimd::new(CubePreviousAxis::F, tables);
        let b_fb = MoveSimd::new(CubePreviousAxis::B, tables);
        let r = MoveSimd::new(CubePreviousAxis::R, tables);
        let l_rl = MoveSimd::new(CubePreviousAxis::L, tables);

        let end_u_d_ud = MoveSimdEnd::new(CubePreviousAxis::U, tables);
        let end_f = MoveSimdEnd::new(CubePreviousAxis::F, tables);
        let end_b_fb = MoveSimdEnd::new(CubePreviousAxis::B, tables);
        let end_r = MoveSimdEnd::new(CubePreviousAxis::R, tables);
        let end_l_rl = MoveSimdEnd::new(CubePreviousAxis::L, tables);
        Self {
            phantom: PhantomData,
            u,
            d_ud,
            f,
            b_fb,
            r,
            l_rl,
            end_u_d_ud,
            end_f,
            end_b_fb,
            end_r,
            end_l_rl,
        }
    }

    fn get_simd_resources(&self, previous_axis: CubePreviousAxis) -> &MoveSimd<'t> {
        match previous_axis {
            CubePreviousAxis::U => &self.u,
            CubePreviousAxis::D | CubePreviousAxis::UD => &self.d_ud,
            CubePreviousAxis::F => &self.f,
            CubePreviousAxis::B | CubePreviousAxis::FB => &self.b_fb,
            CubePreviousAxis::R => &self.r,
            CubePreviousAxis::L | CubePreviousAxis::RL => &self.l_rl,
            CubePreviousAxis::None => unsafe { unreachable_unchecked() },
        }
    }

    fn get_simd_last_move(&self, previous_axis: CubePreviousAxis) -> &MoveSimdEnd<'t> {
        match previous_axis {
            CubePreviousAxis::U | CubePreviousAxis::D | CubePreviousAxis::UD => &self.end_u_d_ud,
            CubePreviousAxis::F => &self.end_f,
            CubePreviousAxis::B | CubePreviousAxis::FB => &self.end_b_fb,
            CubePreviousAxis::R => &self.end_r,
            CubePreviousAxis::L | CubePreviousAxis::RL => &self.end_l_rl,
            CubePreviousAxis::None => unsafe { unreachable_unchecked() },
        }
    }

    fn node_to_offsets(&self, node: Phase1Node) -> Simd<usize, 64> {
        let coords: [u16; 8] = unsafe { core::mem::transmute(node) };
        let v16 = Simd::<u16, 8>::from_array(coords);
        let v: Simd<usize, 8> = SimdUint::cast(v16);
        
        
        // let row_multiplier_base = ;

        const ROW_MULT: Simd<usize, 8> = Simd::from_array([
            33, // corner orient
            18 * 2, // edge group orient
            18 * 2, // corner perm
            32, // edge pos
            32, // edge pos
            32, // edge pos
            18 * 2, // edge group orient sym
            18 * 2, // corner perm sym
        ]);

        let offsets = (v * ROW_MULT).to_array();
        let offsets: [[usize; 8]; 8] = [offsets; 8];
        let offsets: [usize; 64] = unsafe { core::mem::transmute(offsets) };
        
        Simd::from_array(offsets)
    }
}

    // pub fn new_axis_iter(
    //     prev_axis: CubePreviousAxis,
    //     end_phase_1: bool,
    // ) -> impl IntoIterator<Item = Self> {
    //     use CubeMove::*;

    //     let slice: &[CubeMove] = if end_phase_1 {
    //         match prev_axis {
    //             CubePreviousAxis::U | CubePreviousAxis::D | CubePreviousAxis::UD => {
    //                 &[F1, F3, B1, B3, R1, R3, L1, L3]
    //             }
    //             CubePreviousAxis::F => &[B1, B3, R1, R3, L1, L3],
    //             CubePreviousAxis::B | CubePreviousAxis::FB => &[R1, R3, L1, L3],
    //             CubePreviousAxis::R => &[F1, F3, B1, B3, L1, L3],
    //             CubePreviousAxis::L | CubePreviousAxis::RL => &[F1, F3, B1, B3],

    //             CubePreviousAxis::None => &[
    //                 U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3,
    //             ],
    //         }
    //     } else {
    //         match prev_axis {
    //             CubePreviousAxis::U => {
    //                 &[D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3]
    //             }
    //             CubePreviousAxis::D | CubePreviousAxis::UD => {
    //                 &[F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3]
    //             }
    //             CubePreviousAxis::F => {
    //                 &[U1, U2, U3, D1, D2, D3, B1, B2, B3, R1, R2, R3, L1, L2, L3]
    //             }
    //             CubePreviousAxis::B | CubePreviousAxis::FB => {
    //                 &[U1, U2, U3, D1, D2, D3, R1, R2, R3, L1, L2, L3]
    //             }
    //             CubePreviousAxis::R => {
    //                 &[U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, L1, L2, L3]
    //             }
    //             CubePreviousAxis::L | CubePreviousAxis::RL => {
    //                 &[U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3]
    //             }
    //             CubePreviousAxis::None => &[
    //                 U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3,
    //             ],
    //         }
    //     };

    //     slice.iter().copied()
    // }

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::cube;

    use super::*;
    use std::collections::BTreeSet;
    use std::num::NonZeroU8;

    fn phase1_key(n: &Phase1Node, tables: &Tables) -> [u32; 6] {
        let c = CornerPermComboCoord { sym_coord: n.corner_perm_sym, domino_conjugation: n.corner_perm_correct };
        let e = EdgeGroupOrientComboCoord { sym_coord: n.edge_group_orient_sym, domino_conjugation: n.edge_group_orient_correct };
        [
            n.corner_orient_raw.0 as u32,
            c.into_raw(tables).0 as u32,
            e.into_raw(tables).0,
            n.u_edge_positions.0.0 as u32,
            n.d_edge_positions.0.0 as u32,
            n.e_edge_positions.0.0 as u32,
        ]
    }

    #[test]
    fn phase1_moves_match_cube_moves_single_random() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let mut rng = ChaCha8Rng::seed_from_u64(1);

        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        let moves_remaining = NonZeroU8::new(10).unwrap();

        // ---- Path A: cube -> move -> cube -> phase1 ----
        let mut from_cube = BTreeSet::new();

        for mv in CubeMove::new_axis_iter(CubePreviousAxis::None, false) {
            let moved_cube = cube.apply_move(mv);
            let node = Phase1Node::from_cube(moved_cube, &tables);
            from_cube.insert(phase1_key(&node, &tables));
        }

        // ---- Path B: cube -> phase1 -> move ----
        let node = Phase1Node::from_cube(cube, &tables);

        let frame = node
            .produce_next_nodes(
                /* max_possible_distance = */ 10,
                moves_remaining,
                &tables,
            )
            .expect("root should not be pruned");

        let from_phase1: BTreeSet<_> = frame.children.map(|n| phase1_key(&n, &tables)).collect();

        assert_eq!(
            from_cube, from_phase1,
            "Phase1Node move application does not match ReprCube move application"
        );

        Ok(())
    }

    #[test]
    fn phase1_moves_culled() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let cube = cube![D R2 L];

        let a = Phase1Node::from_cube(cube, tables);

        // [R3, R2, F2, L3, R1]

        let next_moves = a
            .produce_next_nodes(20, NonZeroU8::new(5).unwrap(), tables)
            .unwrap();
        let b = next_moves.children.skip(14).next().unwrap();
        let next_moves = b
            .produce_next_nodes(
                next_moves.max_possible_distance + 1,
                NonZeroU8::new(4).unwrap(),
                tables,
            )
            .unwrap();

        for c in next_moves.children {
            println!("{c:?}")
        }

        Ok(())
    }

    #[test]
    fn new_table_offsets() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let _ = TableOffsets::new(tables);

        Ok(())
    }
    
    fn to_hex_underscored(bytes: &[u16]) -> String {
        // 2 hex chars per byte + underscores
        let mut out = String::with_capacity(bytes.len() * 5);

        for (i, b) in bytes.iter().enumerate() {
            if i != 0 {
                out.push('_');
            }
            use std::fmt::Write;
            write!(out, "{:04X}", b).unwrap();
        }

        out
    }

    fn collect_scalar_children(
        node: Phase1Node,
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        tables: &Tables,
    ) -> (BTreeSet<String>, u8) {
        let frame = node
            .produce_next_nodes(max_possible_distance, moves_remaining, tables)
            .expect("scalar path pruned unexpectedly");

        let keys = frame
            .children
            .map(|n| unsafe { core::mem::transmute::<_, [u16; 8]>(n) })
            .map(|x| to_hex_underscored(&x))
            .collect::<BTreeSet<_>>();

        (keys, frame.max_possible_distance)
    }

    fn collect_simd_children(
        node: Phase1Node,
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        table_offsets: &TableOffsets,
        tables: &Tables,
    ) -> (BTreeSet<String>, u8) {
        // SIMD API requires a 16-wide buffer with node in slot 0
        let mut buf = [node; 16];

        let (count, new_max) = Phase1Node::produce_next_nodes_simd(
            &mut buf,
            max_possible_distance,
            moves_remaining,
            table_offsets,
            tables,
        );

        let keys = buf[1..=count]
            .iter()
            .copied()
            .map(|n| unsafe { core::mem::transmute::<_, [u16; 8]>(n) })
            .map(|x| to_hex_underscored(&x))
            .collect::<BTreeSet<_>>();

        (keys, new_max)
    }

    #[test]
    fn simd_matches_scalar_single_random() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let table_offsets = TableOffsets::new(tables);

        let mut rng = ChaCha8Rng::seed_from_u64(123);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        let mut node = Phase1Node::from_cube(cube, tables);
        node.previous_axis = CubePreviousAxis::B;
        let moves_remaining = NonZeroU8::new(10).unwrap();
        let max_possible_distance = 10;

        let (scalar_keys, scalar_max) =
            collect_scalar_children(node, max_possible_distance, moves_remaining, tables);

        let (simd_keys, simd_max) =
            collect_simd_children(node, max_possible_distance, moves_remaining, &table_offsets, tables);

        assert_eq!(scalar_max, simd_max, "max_possible_distance mismatch");
        assert_eq!(scalar_keys, simd_keys, "SIMD children differ from scalar");

        Ok(())
    }

    #[test]
    fn simd_matches_scalar_last_move_only() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let table_offsets = TableOffsets::new(tables);

        let cube = cube![D R2 L];
        let node = Phase1Node::from_cube(cube, tables);

        let moves_remaining = NonZeroU8::new(1).unwrap();
        let max_possible_distance = 5;

        let (scalar_keys, scalar_max) =
            collect_scalar_children(node, max_possible_distance, moves_remaining, tables);

        let (simd_keys, simd_max) =
            collect_simd_children(node, max_possible_distance, moves_remaining, &table_offsets, tables);

        assert_eq!(scalar_max, simd_max);
        assert_eq!(scalar_keys, simd_keys);

        Ok(())
    }
}

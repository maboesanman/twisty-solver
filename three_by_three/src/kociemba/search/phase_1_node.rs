use std::{
    hint::unreachable_unchecked,
    marker::PhantomData,
    num::NonZeroU8,
    ptr,
    simd::{
        LaneCount, Mask, Simd, SupportedLaneCount, cmp::SimdPartialOrd, num::{SimdFloat, SimdUint}, ptr::SimdConstPtr
    },
};

use itertools::Itertools;

use crate::{
    CornerOrient, CornerPerm, CubeMove, EdgeOrient, ReprCube, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            coords::{
                CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientRawCoord,
                EdgeGroupOrientSymCoord,
            },
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Phase1Node {
    // corners
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,
    
    pub edge_group_orient_correct: u16,
    pub corner_perm_combo: u16,
    
    // edges
    pub corner_orient_raw: CornerOrientRawCoord,
    pub u_edge_positions: UEdgePositions,
    pub d_edge_positions: DEdgePositions,
    pub e_edge_positions: EEdgePositions,
    // bookkeeping
    pub previous_axis: u16,
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
            previous_axis: CubePreviousAxis::None as u8 as u16,
            corner_perm_combo: corner_perm_combo.into_dense(),
            edge_group_orient_sym: edge_group_orient_combo.sym_coord,
            edge_group_orient_correct: edge_group_orient_combo.domino_conjugation.0 as u16,
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let edge_perm = combine_edge_positions(
            self.u_edge_positions,
            self.d_edge_positions,
            self.e_edge_positions,
        );
        let corner_perm_combo = CornerPermComboCoord::from_dense(self.corner_perm_combo);
        let corner_perm = CornerPerm::from_coord(corner_perm_combo.into_raw(tables));

        let edge_group_orient_combo = EdgeGroupOrientComboCoord {
            sym_coord: self.edge_group_orient_sym,
            domino_conjugation: DominoSymmetry(self.edge_group_orient_correct as u8),
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

    #[inline]
    pub fn distance_heuristic(self, tables: &Tables) -> u8 {
        let corner_orient_adjusted = tables
            .move_raw_corner_orient
            .domino_conjugate(self.corner_orient_raw, DominoSymmetry(self.edge_group_orient_correct as u8));
        let distance = tables
            .get_prune_phase_1()
            .get_value(self.edge_group_orient_sym, corner_orient_adjusted);

        distance
    }

    #[inline]
    pub fn is_domino_reduced(self) -> bool {
        self.corner_orient_raw.0 == 0 && self.edge_group_orient_sym.0 == 0
    }

    #[inline]
    pub fn produce_next_nodes(
        self,
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        tables: &Tables,
    ) -> Option<Phase1FrameMetadata<impl Iterator<Item = Self>>> {
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
        let move_iter = || CubeMove::new_axis_iter(unsafe { core::mem::transmute(self.previous_axis as u8)}, moves_remaining.get() == 1);

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
                    let corner_perm_combo = CornerPermComboCoord::from_dense(self.corner_perm_combo)
                    .apply_cube_move(tables, cube_move).into_dense();

                    let edge_group_orient_combo = EdgeGroupOrientComboCoord {
                        sym_coord: self.edge_group_orient_sym,
                        domino_conjugation: DominoSymmetry(self.edge_group_orient_correct as u8),
                    }
                    .apply_cube_move(tables, cube_move);

                    Phase1Node {
                        corner_orient_raw: tables
                            .move_raw_corner_orient
                            .apply_cube_move(self.corner_orient_raw, cube_move),
                        u_edge_positions,
                        d_edge_positions,
                        e_edge_positions,
                        previous_axis: unsafe { core::mem::transmute::<u8, CubePreviousAxis>(self.previous_axis as u8)}
                            .update_with_new_move(cube_move, moves_remaining.get() - 1) as u8 as u16,
                        corner_perm_combo,
                        edge_group_orient_sym: edge_group_orient_combo.sym_coord,
                        edge_group_orient_correct: edge_group_orient_combo.domino_conjugation.0 as u16,
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

    #[inline]
    pub fn produce_next_nodes_simd_compat(
        slice: &mut [Phase1Node; 16],
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        _table_offsets: &TableOffsets, // intentionally unused
        tables: &Tables,
    ) -> (usize, u8) {
        let start_node = slice[0];

        let Some(frame) = start_node.produce_next_nodes(
            max_possible_distance,
            moves_remaining,
            tables,
        ) else {
            return (0, 0);
        };

        let mut count = 0usize;

        for child in frame.children {
            // contract: children are written starting at index 1
            slice[count + 1] = child;
            count += 1;
        }

        (count, frame.max_possible_distance)
    }

    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline]
    pub fn produce_next_nodes_simd(
        slice: &mut [Phase1Node; 16],
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        table_offsets: &TableOffsets,
        tables: &Tables,
    ) -> (usize, u8) {
        let start_node = slice[0];
        // Get bounds for the current distance from self to solved, with the restriction
        // that the range must be within the allowed. If the range we have is not a subset of
        // [min_allowed_distance, max_allowed_distance], then we look up the actual distance to solved and
        // return a legal single point range or return None because we're outside the range and must be pruned.
        let max_possible_current_distance = if max_possible_distance > moves_remaining.get() {
            let distance = start_node.distance_heuristic(tables);

            if distance > moves_remaining.get() {
                return (0, 0);
            }

            distance
        } else {
            max_possible_distance
        };

        // prepare the values for feeding the child nodes.
        let max_possible_distance = max_possible_current_distance + 1;

        let subtable = table_offsets.get_simd_resources(unsafe { core::mem::transmute(start_node.previous_axis as u8)}, moves_remaining);

        let row_starts = subtable.node_to_row_starts(table_offsets, start_node);
        let move_offsets = subtable.node_to_sym_move_offsets(start_node);

        
        let ego_sym_start = DominoSymmetry(start_node.edge_group_orient_correct as u8);
        let cp_sym_start = DominoSymmetry((start_node.corner_perm_combo >> 12) as u8);

        let mut i = 1; // output pointer
        let mut j = 0; // input pointer
        let last_move = moves_remaining.get() == 1;
        let too_close_to_be_reduced = moves_remaining.get() <= 8;

        const CP_MASK: u16 = 0x0FFF;
        const CP_SHIFT: u32 = 12;

        let mut first_valid = true;

        while j < subtable.count {
            let a = (move_offsets.ego_sym_coord[j] << 1) as usize;
            let b = move_offsets.cp_sym_coord[j] as usize;
            let c = move_offsets.raw_coord[j] as usize;
            let offsets = Simd::from_array([a, a, b, c, c, c, c, j]);
            j += 1;
            let source = row_starts.wrapping_add(offsets);
            let out_slot = &mut slice[i];
            unsafe {
                let out = Simd::gather_ptr(source).to_array();
                *out_slot = core::mem::transmute(out);
            }


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
            let child_is_reduced = out_slot.corner_orient_raw.0 == 0 && out_slot.edge_group_orient_sym.0 == 0;

            let should_handle =
                (last_move && child_is_reduced)
                || (!last_move && (!too_close_to_be_reduced || !child_is_reduced));

            if !should_handle {
                continue;
            }

            // if first_valid && moves_remaining.get() != 1 {
            //     let RowStartsBase {
            //         edge_pos,
            //         corner_orient_raw,
            //         edge_group_orient,
            //         corner_combo,
            //     } = table_offsets.row_0_starts;

            //     unsafe {
            //         // let a = edge_group_orient
            //         //     .add(out_slot.edge_group_orient_sym.0 as usize * 18 * 2);
            //         // let b = corner_combo.add((out_slot.corner_perm_combo & 0b0000_1111_1111_1111) as usize * 18);
            //         let c = corner_orient_raw.add((out_slot.corner_orient_raw.0 as usize) << 5);
            //         // let d = edge_pos.add((out_slot.u_edge_positions.0.0 as usize) << 5);
            //         // let e = edge_pos.add((out_slot.d_edge_positions.0.0 as usize) << 5);
            //         // let f = edge_pos.add((out_slot.e_edge_positions.0.0 as usize) << 5);

            //         std::hint::prefetch_read(c, std::hint::Locality::L1);

            //         // for ptr in [a, b, c ,d, e, f] {
            //         //     std::hint::prefetch_read(ptr, std::hint::Locality::L1);
            //         // }
            //     }
            //     first_valid = false;
            // }

            let ego_sym = DominoSymmetry(out_slot.edge_group_orient_correct as u8);
            let ego_sym = ego_sym_start.then(ego_sym);
            out_slot.edge_group_orient_correct = ego_sym.0 as u16;

            let cp_sym = DominoSymmetry((out_slot.corner_perm_combo >> CP_SHIFT) as u8);
            let cp_sym = cp_sym_start.then(cp_sym);
            out_slot.corner_perm_combo = (out_slot.corner_perm_combo & CP_MASK) | ((cp_sym.0 as u16) << CP_SHIFT);

            i += 1;
        }

        // if i > 2 {
        //     // swap slice[i - 1] and slice[1]
        //     unsafe {
        //         let p = slice.as_mut_ptr();
        //         let a = p.add(i - 1);
        //         let b = p.add(1);

        //         std::ptr::swap(a, b);
        //     }
        // }
        let ptr = unsafe { table_offsets.row_0_starts.edge_group_orient.add(slice[i - 1].edge_group_orient_sym.0 as usize * 18 * 2) }; 
            //         //     .add(out_slot.edge_group_orient_sym.0 as usize * 18 * 2); 
        std::hint::prefetch_read(ptr, std::hint::Locality::L1);

        (i - 1, max_possible_distance)
    }
}

#[derive(Clone, Debug)]
struct MoveSimd<const N: usize> {
    // base
    base_move_offsets: [u16; N],
    new_prev_moves: Box<[u16; N]>, // this is pointed into by some of the above pointers.
    count: usize,
}

// struct MoveSimdRef<

impl<const N: usize> MoveSimd<N> where LaneCount<N>: SupportedLaneCount {
    fn new(prev_axis: CubePreviousAxis) -> Self {
        let moves = CubeMove::new_axis_iter(prev_axis, false);
        let mut new_prev_moves = Box::new([0u16; N]);
        let mut count = 0;
        let mut base_move_offsets = [0; N];
        for mv in moves.into_iter() {
            base_move_offsets[count] = mv.into_u8() as u16;
            new_prev_moves[count] = prev_axis.update_with_new_move(mv, 100) as u8 as u16;
            count += 1;
        }

        Self {
            count,
            new_prev_moves,
            base_move_offsets
        }
    }
    #[inline]
    fn node_to_sym_move_offsets(&self, node: Phase1Node) -> Offsets<N> {
        const LOOKUP: [u16; 18 * 16] = {
            let mut table = [0u16; 18 * 16];
            let mut i = 0usize;
            while i < 18 * 16 {
                let mv = unsafe { core::mem::transmute::<u8, CubeMove>((i >> 4) as u8) };
                let sym = DominoSymmetry((i as u8) & 0b1111);
                table[i] = mv.domino_conjugate(sym) as u8 as u16;
                i += 1;
            }
            table
        };

        let offsets: Simd<usize, N> = SimdUint::cast(Simd::from_array(self.base_move_offsets));
        let offsets = offsets << Simd::splat(4);
        let base = Simd::<_, N>::splat(&LOOKUP as *const _ as *const u16);

        unsafe {
            Offsets {
                raw_coord: self.base_move_offsets,
                ego_sym_coord: Simd::gather_ptr(base.wrapping_add(
                    offsets | Simd::splat(node.edge_group_orient_correct as usize),
                ))
                .to_array(),
                cp_sym_coord: Simd::gather_ptr(
                    base.wrapping_add(offsets | Simd::splat((node.corner_perm_combo >> 12) as usize)),
                )
                .to_array(),
            }
        }
    }
    #[inline]
    fn node_to_row_starts(&self, table_offsets: &TableOffsets, node: Phase1Node) -> Simd<*const u16, 8> {
        let RowStartsBase {
            edge_pos,
            corner_orient_raw,
            edge_group_orient,
            corner_combo,
        } = table_offsets.row_0_starts;

        unsafe {
            Simd::<_, 8>::from_array([
                edge_group_orient
                    .add(node.edge_group_orient_sym.0 as usize * 18 * 2),
                edge_group_orient
                    .add(node.edge_group_orient_sym.0 as usize * 18 * 2 + 1),
                corner_combo.add((node.corner_perm_combo & 0b0000_1111_1111_1111) as usize * 18),
                corner_orient_raw.add((node.corner_orient_raw.0 as usize) << 5),
                edge_pos.add((node.u_edge_positions.0.0 as usize) << 5),
                edge_pos.add((node.d_edge_positions.0.0 as usize) << 5),
                edge_pos.add((node.e_edge_positions.0.0 as usize) << 5),
                self.new_prev_moves.as_ptr(),
            ])
        }
    }
}

#[derive(Clone, Debug)]
struct Offsets<const N: usize> {
    raw_coord: [u16; N],
    ego_sym_coord: [u16; N],
    cp_sym_coord: [u16; N],
}

#[repr(C)]
#[derive(Clone, Debug)]
struct RowStartsBase {
    edge_pos: *const u16,
    corner_orient_raw: *const u16,
    edge_group_orient: *const u16,
    corner_combo: *const u16,
}


#[derive(Clone, Debug)]
pub struct TableOffsets<'t> {
    phantom: PhantomData<&'t Tables>,

    row_0_starts: RowStartsBase,

    u: MoveSimd<15>,
    d_ud: MoveSimd<15>,
    f: MoveSimd<15>,
    b_fb: MoveSimd<15>,
    r: MoveSimd<15>,
    l_rl: MoveSimd<15>,

    end_u_d_ud: MoveSimd<15>,
    end_f: MoveSimd<15>,
    end_b_fb: MoveSimd<15>,
    end_r: MoveSimd<15>,
    end_l_rl: MoveSimd<15>,
}

unsafe impl<'t> Send for TableOffsets<'t> {}
unsafe impl<'t> Sync for TableOffsets<'t> {}

impl<'t> TableOffsets<'t> {
    pub fn new(tables: &'t Tables) -> Self {
        let u = MoveSimd::new(CubePreviousAxis::U);
        let d_ud = MoveSimd::new(CubePreviousAxis::D);
        let f = MoveSimd::new(CubePreviousAxis::F);
        let b_fb = MoveSimd::new(CubePreviousAxis::B);
        let r = MoveSimd::new(CubePreviousAxis::R);
        let l_rl = MoveSimd::new(CubePreviousAxis::L);

        let end_u_d_ud = MoveSimd::new(CubePreviousAxis::U);
        let end_f = MoveSimd::new(CubePreviousAxis::F);
        let end_b_fb = MoveSimd::new(CubePreviousAxis::B);
        let end_r = MoveSimd::new(CubePreviousAxis::R);
        let end_l_rl = MoveSimd::new(CubePreviousAxis::L);

        let row_0_starts = unsafe {
            RowStartsBase {
            edge_pos: tables.move_edge_position.as_ptr(),
            corner_orient_raw: tables.move_raw_corner_orient.as_ptr(),
            edge_group_orient: tables.move_sym_edge_group_orient.as_ptr(),
            corner_combo: tables.move_sym_corner_perm.as_ptr(),
        }
        };
        Self {
            phantom: PhantomData,
            row_0_starts,
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

    #[inline]
    fn get_simd_resources(&self, previous_axis: CubePreviousAxis, moves_remaining: NonZeroU8,) -> &MoveSimd<15> {
        if moves_remaining.get() == 1 {
            match previous_axis {
                CubePreviousAxis::U | CubePreviousAxis::D | CubePreviousAxis::UD => &self.end_u_d_ud,
                CubePreviousAxis::F => &self.end_f,
                CubePreviousAxis::B | CubePreviousAxis::FB => &self.end_b_fb,
                CubePreviousAxis::R => &self.end_r,
                CubePreviousAxis::L | CubePreviousAxis::RL => &self.end_l_rl,
                CubePreviousAxis::None => unsafe { unreachable_unchecked() },
            }
        } else {
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
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::cube;
    use crate::kociemba::partial_reprs::edge_positions::EdgePositions;

    use super::*;
    use std::collections::BTreeSet;
    use std::num::NonZeroU8;
    extern crate test;

    fn phase1_key(n: &Phase1Node, tables: &Tables) -> [u32; 6] {
        let e = EdgeGroupOrientComboCoord {
            sym_coord: n.edge_group_orient_sym,
            domino_conjugation: unsafe { core::mem::transmute(n.edge_group_orient_correct as u8)},
        };
        [
            n.corner_orient_raw.0 as u32,
            n.corner_perm_combo as u32,
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
        node.previous_axis = CubePreviousAxis::B as u8 as u16;
        let moves_remaining = NonZeroU8::new(10).unwrap();
        let max_possible_distance = 10;

        let (scalar_keys, scalar_max) =
            collect_scalar_children(node, max_possible_distance, moves_remaining, tables);

        let (simd_keys, simd_max) = collect_simd_children(
            node,
            max_possible_distance,
            moves_remaining,
            &table_offsets,
            tables,
        );

        assert_eq!(scalar_max, simd_max, "max_possible_distance mismatch");
        assert_eq!(scalar_keys, simd_keys, "SIMD children differ from scalar");

        Ok(())
    }

    #[test]
    fn simd_with_cancellation() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let table_offsets = TableOffsets::new(tables);

        let node = Phase1Node {
            previous_axis: CubePreviousAxis::U as u16,
            edge_group_orient_sym: EdgeGroupOrientSymCoord(18910),
            edge_group_orient_correct: 14,
            corner_perm_combo: 12398,
            corner_orient_raw: CornerOrientRawCoord(1550),
            u_edge_positions: UEdgePositions(EdgePositions(5392)),
            d_edge_positions: DEdgePositions(EdgePositions(10634)),
            e_edge_positions: EEdgePositions(EdgePositions(1514)),
        };

        let moves_remaining = NonZeroU8::new(10).unwrap();
        let max_possible_distance = 10;

        let (scalar_keys, scalar_max) =
            collect_scalar_children(node, max_possible_distance, moves_remaining, tables);

        let (simd_keys, simd_max) = collect_simd_children(
            node,
            max_possible_distance,
            moves_remaining,
            &table_offsets,
            tables,
        );

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

        let (simd_keys, simd_max) = collect_simd_children(
            node,
            max_possible_distance,
            moves_remaining,
            &table_offsets,
            tables,
        );

        assert_eq!(scalar_max, simd_max);
        assert_eq!(scalar_keys, simd_keys);

        Ok(())
    }

    #[bench]
    fn simd_micro_incremental_solutions(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let table_offsets = TableOffsets::new(tables);
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube = rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = Phase1Node::from_cube(cube, tables);
        phase_1.previous_axis = CubePreviousAxis::B as u8 as u16;

        let mut buf = [phase_1; 16];

        bench.iter(|| {
            let (count, new_max) = Phase1Node::produce_next_nodes_simd(
                &mut buf,
                20,
                unsafe { NonZeroU8::new_unchecked(30) },
                &table_offsets,
                tables,
            );
        });
    }

    #[bench]
    fn no_simd_micro_incremental_solutions(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let table_offsets = TableOffsets::new(tables);
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let cube: ReprCube = rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = Phase1Node::from_cube(cube, tables);
        phase_1.previous_axis = CubePreviousAxis::B as u8 as u16;

        let mut buf = [phase_1; 16];

        bench.iter(|| {
            let (count, new_max) = Phase1Node::produce_next_nodes_simd_compat(
                &mut buf,
                20,
                unsafe { NonZeroU8::new_unchecked(30) },
                &table_offsets,
                tables,
            );
        });
    }
}

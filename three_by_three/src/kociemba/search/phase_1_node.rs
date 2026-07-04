use std::num::NonZeroU8;

use crate::{
    CornerOrient, CornerPerm, CubeMove, EdgeOrient, EdgePerm, Permutation, ReprCube, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            CornerOrientRawCoord, CornerPermRawCoord, CornerPermSymCoord, EdgeGroupOrientRawCoord,
            EdgeGroupOrientSymCoord, corner_perm_combo_coord::CornerPermComboCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::{
            edge_group::EdgeGroup,
            edge_positions::{
                DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions,
                combine_edge_positions, split_edge_positions,
            },
        },
        tables::{
            lookup_sym_corner_perm::LookupSymCornerPermTable,
            lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable,
            move_edge_positions::MoveEdgePositionsTable,
            move_raw_corner_orient::MoveRawCornerOrientTable,
            move_sym_corner_perm::MoveSymCornerPermTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
        },
    },
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Phase1Node {
    // corners
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,

    pub edge_group_orient_correct: DominoSymmetry,
    pub corner_perm_correct: DominoSymmetry,
    pub corner_perm_raw: CornerPermSymCoord,

    // edges
    pub corner_orient_raw: CornerOrientRawCoord,
    pub u_edge_positions: UEdgePositions,
    pub d_edge_positions: DEdgePositions,
    pub e_edge_positions: EEdgePositions,
    // bookkeeping
    pub previous_axis: CubePreviousAxis,
}

// a set of nodes which came from the same call to `produce_next_nodes`
pub struct Phase1FrameMetadata<I> {
    pub children: I,
}

impl Default for Phase1Node {
    fn default() -> Self {
        const {
            let (u_edge_positions, d_edge_positions, e_edge_positions) =
                split_edge_positions(EdgePerm::SOLVED);
            Self {
                edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
                edge_group_orient_correct: DominoSymmetry::IDENTITY,
                corner_perm_raw: CornerPermSymCoord(0),
                corner_orient_raw: CornerOrientRawCoord(0),
                u_edge_positions,
                d_edge_positions,
                e_edge_positions,
                previous_axis: CubePreviousAxis::None,
                corner_perm_correct: DominoSymmetry::IDENTITY,
            }
        }
    }
}

impl Phase1Node {
    pub(crate) fn from_phase_1_coords(
        edge_group_orient_sym: EdgeGroupOrientSymCoord,
        corner_orient_raw: CornerOrientRawCoord,
        tables: impl AsRef<LookupSymEdgeGroupOrientTable> + AsRef<LookupSymCornerPermTable>,
    ) -> Self {
        let edge_table: &LookupSymEdgeGroupOrientTable = tables.as_ref();
        let edge_group_orient_raw = edge_table.get_rep_from_sym(edge_group_orient_sym);
        let mut cube = ReprCube::SOLVED;

        let (edge_group_raw, edge_orient_raw_coord) = edge_group_orient_raw.split();

        cube.corner_orient = CornerOrient::from_coord(corner_orient_raw);
        cube.edge_orient = EdgeOrient::from_coord(edge_orient_raw_coord);

        let e_edge_pos = EdgePositions::join(
            EdgeGroup::from_coord(edge_group_raw).0,
            Permutation::IDENTITY,
        );
        let (u_edge_pos, d_edge_pos) = e_edge_pos.valid_sibling_pair();

        cube.edge_perm = combine_edge_positions(
            UEdgePositions(u_edge_pos),
            DEdgePositions(d_edge_pos),
            EEdgePositions(e_edge_pos),
        );
        cube.corner_perm =
            CornerPerm::from_coord(CornerPermRawCoord(cube.edge_perm.0.is_odd() as u16));

        // ReprCube::pretty_print(cube);
        // assert!(cube)

        Self::from_cube(cube, tables)
    }

    pub fn from_cube(
        cube: ReprCube,
        tables: impl AsRef<LookupSymEdgeGroupOrientTable> + AsRef<LookupSymCornerPermTable>,
    ) -> Self {
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
            EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);
        let corner_perm_combo = CornerPermComboCoord::from_raw(&tables, corner_perm.into_coord());
        let corner_orient_raw = corner_orient.into_coord();

        Self {
            corner_orient_raw,
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis: CubePreviousAxis::None,
            edge_group_orient_sym: edge_group_orient_combo.sym_coord,
            edge_group_orient_correct: edge_group_orient_combo.domino_conjugation,
            corner_perm_correct: corner_perm_combo.domino_conjugation,
            corner_perm_raw: corner_perm_combo.sym_coord,
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let edge_perm = combine_edge_positions(
            self.u_edge_positions,
            self.d_edge_positions,
            self.e_edge_positions,
        );
        let corner_perm_combo = CornerPermComboCoord {
            sym_coord: self.corner_perm_raw,
            domino_conjugation: self.corner_perm_correct,
        };
        let corner_perm = CornerPerm::from_coord(corner_perm_combo.into_raw(tables));

        let edge_group_orient_combo = EdgeGroupOrientComboCoord {
            sym_coord: self.edge_group_orient_sym,
            domino_conjugation: self.edge_group_orient_correct,
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
}

impl Phase1Node {
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

        let corner_orient_adjusted =
            move_table.domino_conjugate(self.corner_orient_raw, self.edge_group_orient_correct);

        prune_table.get_value(self.edge_group_orient_sym, corner_orient_adjusted)
    }

    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline(always)]
    pub fn produce_next_nodes(
        slice: &mut [Self; 16],
        moves_remaining: NonZeroU8,
        // tables: &(
        //      impl AsRef<MoveEdgePositionsTable>
        //      + AsRef<MoveRawCornerOrientTable>
        //      + AsRef<MoveSymCornerPermTable>
        //      + AsRef<MoveSymEdgeGroupOrientTable>
        //      + AsRef<PrunePhase1Table>
        //  ),
        tables: &Tables
    ) -> usize {
        let start_node = slice[0];
        #[cfg(debug_assertions)]
        let repr_cube = start_node.into_cube(tables);

        let ego_mv_tbl: &MoveSymEdgeGroupOrientTable = tables.as_ref();
        let cp_mv_tbl: &MoveSymCornerPermTable = tables.as_ref();
        let co_mv_tbl: &MoveRawCornerOrientTable = tables.as_ref();
        let ep_mv_tbl: &MoveEdgePositionsTable = tables.as_ref();

        let ego_row = ego_mv_tbl.row(start_node.edge_group_orient_sym);
        let cp_row = cp_mv_tbl.row(start_node.corner_perm_raw);
        let co_row = co_mv_tbl.row(start_node.corner_orient_raw);
        let u_row = ep_mv_tbl.row(start_node.u_edge_positions.0);
        let d_row = ep_mv_tbl.row(start_node.d_edge_positions.0);
        let e_row = ep_mv_tbl.row(start_node.e_edge_positions.0);

        const LOOKUP: [CubeMove; 18 * 16] = {
            let mut table = [CubeMove::U1; 18 * 16];
            let mut i = 0usize;
            while i < 18 * 16 {
                let mv = unsafe { core::mem::transmute::<u8, CubeMove>((i >> 4) as u8) };
                let sym = DominoSymmetry((i as u8) & 0b1111);
                table[i] = mv.domino_conjugate(sym);
                i += 1;
            }
            table
        };

        let (unaltered_move_offsets, num_moves) =
            CubeMove::new_axis_move_array(start_node.previous_axis);
        let ego_move_offsets = unaltered_move_offsets.map(|mv| {
            LOOKUP[(((mv as u8) << 4) | (start_node.edge_group_orient_correct.0)) as usize]
        });
        let cp_move_offsets = unaltered_move_offsets
            .map(|mv| LOOKUP[(((mv as u8) << 4) | (start_node.corner_perm_correct.0)) as usize]);

        for i in 0..num_moves {
            let ego_i = ego_move_offsets[i] as u8 as usize;
            let new_ego_coord = EdgeGroupOrientSymCoord(ego_row.coords[ego_i]);
            let new_ego_correction = DominoSymmetry(ego_row.conjugations[ego_i]);

            let cp_i = cp_move_offsets[i] as u8 as usize;
            let new_cp_coord = CornerPermSymCoord(cp_row.coords[cp_i]);
            let new_cp_correction = DominoSymmetry(cp_row.conjugations[cp_i]);
            // println!("CORRECTIONS: ego-{:x}, cp-{:x}", new_ego_correction.0, new_cp_correction.0);

            let un_i = unaltered_move_offsets[i] as u8 as usize;
            let new_co_coord = CornerOrientRawCoord(co_row.moves[un_i]);
            let new_u_coord = UEdgePositions(EdgePositions(u_row.0[un_i]));
            let new_d_coord = DEdgePositions(EdgePositions(d_row.0[un_i]));
            let new_e_coord = EEdgePositions(EdgePositions(e_row.0[un_i]));

            let new_previous_axis = start_node
                .previous_axis
                .update_with_new_move(unaltered_move_offsets[i], moves_remaining.get() - 1);

            let mv: CubeMove = unsafe { core::mem::transmute(unaltered_move_offsets[i] as u8) };
            
            slice[i + 1] = Phase1Node {
                edge_group_orient_sym: new_ego_coord,
                edge_group_orient_correct: start_node
                .edge_group_orient_correct
                .then(new_ego_correction),
                corner_perm_correct: start_node.corner_perm_correct.then(new_cp_correction),
                corner_perm_raw: new_cp_coord,
                corner_orient_raw: new_co_coord,
                u_edge_positions: new_u_coord,
                d_edge_positions: new_d_coord,
                e_edge_positions: new_e_coord,
                previous_axis: new_previous_axis,
            };
            
            #[cfg(debug_assertions)]
            {
                let x = repr_cube.apply_cube_move(mv);
                let y =  slice[i + 1].into_cube(tables);
                assert_eq!(x, y);
            }

            println!("    mv {}: {}", un_i, slice[i + 1].distance_heuristic(tables));
        }

        num_moves
    }
}

// #[derive(Clone, Debug)]
// struct MoveSimd<const N: usize> {
//     // base
//     base_move_offsets: [u16; N],
//     new_prev_moves: Box<[u16; N]>, // this is pointed into by some of the above pointers.
//     count: usize,
// }

// impl<const N: usize> MoveSimd<N> {
//     fn new(prev_axis: CubePreviousAxis) -> Self {
//         let moves = CubeMove::new_axis_iter(prev_axis, false);
//         let mut new_prev_moves = Box::new([0u16; N]);
//         let mut count = 0;
//         let mut base_move_offsets = [0; N];
//         for mv in moves.into_iter() {
//             base_move_offsets[count] = mv.into_u8() as u16;
//             new_prev_moves[count] = prev_axis.update_with_new_move(mv, 100) as u8 as u16;
//             count += 1;
//         }

//         Self {
//             count,
//             new_prev_moves,
//             base_move_offsets,
//         }
//     }

// #[inline(always)]
// fn node_to_sym_move_offsets(&self, node: Phase1Node) -> Offsets<N> {
//     const LOOKUP: [u16; 18 * 16] = {
//         let mut table = [0u16; 18 * 16];
//         let mut i = 0usize;
//         while i < 18 * 16 {
//             let mv = unsafe { core::mem::transmute::<u8, CubeMove>((i >> 4) as u8) };
//             let sym = DominoSymmetry((i as u8) & 0b1111);
//             table[i] = mv.domino_conjugate(sym) as u8 as u16;
//             i += 1;
//         }
//         table
//     };

//     let offsets: Simd<usize, N> = SimdUint::cast(Simd::from_array(self.base_move_offsets));
//     let offsets = offsets << Simd::splat(4);
//     let base = Simd::<_, N>::splat(&LOOKUP as *const _ as *const u16);

//     unsafe {
//         Offsets {
//             raw_coord: self.base_move_offsets,
//             ego_sym_coord: Simd::gather_ptr(
//                 base.wrapping_add(
//                     offsets | Simd::splat(node.edge_group_orient_correct as usize),
//                 ),
//             )
//             .to_array(),
//             cp_sym_coord: Simd::gather_ptr(
//                 base.wrapping_add(
//                     offsets | Simd::splat((node.corner_perm_combo >> 12) as usize),
//                 ),
//             )
//             .to_array(),
//         }
//     }
// }

//     #[inline(always)]
//     fn node_to_row_starts(
//         &self,
//         table_offsets: &TableOffsets,
//         node: Phase1Node,
//     ) -> Simd<*const u16, 8> {
//         let RowStartsBase {
//             edge_pos,
//             corner_orient_raw,
//             edge_group_orient,
//             corner_combo,
//         } = table_offsets.row_0_starts;

//         unsafe {
//             Simd::<_, 8>::from_array([
//                 edge_group_orient.add(node.edge_group_orient_sym.0 as usize * 18 * 2),
//                 edge_group_orient.add(node.edge_group_orient_sym.0 as usize * 18 * 2 + 1),
//                 corner_combo.add(((node.corner_perm_combo & 0b0000_1111_1111_1111) as usize) << 5),
//                 corner_orient_raw.add((node.corner_orient_raw.0 as usize) << 5),
//                 edge_pos.add((node.u_edge_positions.0.0 as usize) << 5),
//                 edge_pos.add((node.d_edge_positions.0.0 as usize) << 5),
//                 edge_pos.add((node.e_edge_positions.0.0 as usize) << 5),
//                 self.new_prev_moves.as_ptr(),
//             ])
//         }
//     }
// }

// #[derive(Clone, Debug)]
// struct Offsets<const N: usize> {
//     raw_coord: [u16; N],
//     ego_sym_coord: [u16; N],
//     cp_sym_coord: [u16; N],
// }

// #[repr(C)]
// #[derive(Clone, Debug)]
// struct RowStartsBase {
//     edge_pos: *const u16,
//     corner_orient_raw: *const u16,
//     edge_group_orient: *const u16,
//     corner_combo: *const u16,
// }

// #[derive(Clone, Debug)]
// pub struct TableOffsets<'t> {
//     phantom: PhantomData<&'t Tables>,

//     row_0_starts: RowStartsBase,

//     u: MoveSimd<15>,
//     d_ud: MoveSimd<15>,
//     f: MoveSimd<15>,
//     b_fb: MoveSimd<15>,
//     r: MoveSimd<15>,
//     l_rl: MoveSimd<15>,

//     end_u_d_ud: MoveSimd<15>,
//     end_f: MoveSimd<15>,
//     end_b_fb: MoveSimd<15>,
//     end_r: MoveSimd<15>,
//     end_l_rl: MoveSimd<15>,
// }

// unsafe impl<'t> Send for TableOffsets<'t> {}
// unsafe impl<'t> Sync for TableOffsets<'t> {}

// impl<'t> TableOffsets<'t> {
//     pub fn new(
//         tables: &'t (
//                 impl AsRef<MoveEdgePositionsTable>
//                 + AsRef<MoveRawCornerOrientTable>
//                 + AsRef<MoveSymEdgeGroupOrientTable>
//                 + AsRef<MoveSymCornerPermTable>
//             ),
//     ) -> Self {
//         let u = MoveSimd::new(CubePreviousAxis::U);
//         let d_ud = MoveSimd::new(CubePreviousAxis::D);
//         let f = MoveSimd::new(CubePreviousAxis::F);
//         let b_fb = MoveSimd::new(CubePreviousAxis::B);
//         let r = MoveSimd::new(CubePreviousAxis::R);
//         let l_rl = MoveSimd::new(CubePreviousAxis::L);

//         let end_u_d_ud = MoveSimd::new(CubePreviousAxis::U);
//         let end_f = MoveSimd::new(CubePreviousAxis::F);
//         let end_b_fb = MoveSimd::new(CubePreviousAxis::B);
//         let end_r = MoveSimd::new(CubePreviousAxis::R);
//         let end_l_rl = MoveSimd::new(CubePreviousAxis::L);

//         let edge_pos_ref: &'t MoveEdgePositionsTable = tables.as_ref();
//         let corner_orient_raw_ref: &'t MoveRawCornerOrientTable = tables.as_ref();
//         let edge_group_orient_ref: &'t MoveSymEdgeGroupOrientTable = tables.as_ref();
//         let corner_combo_ref: &'t MoveSymCornerPermTable = tables.as_ref();

//         let row_0_starts = unsafe {
//             RowStartsBase {
//                 edge_pos: edge_pos_ref.as_ptr(),
//                 corner_orient_raw: corner_orient_raw_ref.as_ptr(),
//                 edge_group_orient: edge_group_orient_ref.as_ptr(),
//                 corner_combo: corner_combo_ref.as_ptr(),
//             }
//         };
//         Self {
//             phantom: PhantomData,
//             row_0_starts,
//             u,
//             d_ud,
//             f,
//             b_fb,
//             r,
//             l_rl,
//             end_u_d_ud,
//             end_f,
//             end_b_fb,
//             end_r,
//             end_l_rl,
//         }
//     }

//     #[inline(always)]
//     // SAFETY: Can't pass in CubePreviousAxis::None
//     unsafe fn get_simd_resources(
//         &self,
//         previous_axis: CubePreviousAxis,
//         moves_remaining: NonZeroU8,
//     ) -> &MoveSimd<15> {
//         #[cfg(test)]
//         {
//             assert_ne!(previous_axis, CubePreviousAxis::None);
//         }
//         debug_assert_ne!(previous_axis, CubePreviousAxis::None);

//         if moves_remaining.get() == 1 {
//             match previous_axis {
//                 CubePreviousAxis::U | CubePreviousAxis::D | CubePreviousAxis::UD => {
//                     &self.end_u_d_ud
//                 }
//                 CubePreviousAxis::F => &self.end_f,
//                 CubePreviousAxis::B | CubePreviousAxis::FB => &self.end_b_fb,
//                 CubePreviousAxis::R => &self.end_r,
//                 CubePreviousAxis::L | CubePreviousAxis::RL => &self.end_l_rl,
//                 CubePreviousAxis::None => unsafe { unreachable_unchecked() },
//             }
//         } else {
//             match previous_axis {
//                 CubePreviousAxis::U => &self.u,
//                 CubePreviousAxis::D | CubePreviousAxis::UD => &self.d_ud,
//                 CubePreviousAxis::F => &self.f,
//                 CubePreviousAxis::B | CubePreviousAxis::FB => &self.b_fb,
//                 CubePreviousAxis::R => &self.r,
//                 CubePreviousAxis::L | CubePreviousAxis::RL => &self.l_rl,
//                 CubePreviousAxis::None => unsafe { unreachable_unchecked() },
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use rand::SeedableRng;
//     use rand_chacha::ChaCha8Rng;

//     use crate::cube;
//     use crate::kociemba::partial_reprs::edge_positions::EdgePositions;

//     use super::*;
//     use std::collections::BTreeSet;
//     use std::num::NonZeroU8;
//     extern crate test;

//     fn phase1_key(n: &Phase1Node, tables: &Tables) -> [u32; 6] {
//         let e = EdgeGroupOrientComboCoord {
//             sym_coord: n.edge_group_orient_sym,
//             domino_conjugation: unsafe { core::mem::transmute(n.edge_group_orient_correct as u8) },
//         };
//         [
//             n.corner_orient_raw.0 as u32,
//             n.corner_perm_combo as u32,
//             e.into_raw(tables).0,
//             n.u_edge_positions.0.0 as u32,
//             n.d_edge_positions.0.0 as u32,
//             n.e_edge_positions.0.0 as u32,
//         ]
//     }

//     #[test]
//     fn phase1_moves_culled() -> anyhow::Result<()> {
//         let tables = Box::leak(Box::new(Tables::new("tables")?));
//         let cube = cube![D R2 L];

//         let a = Phase1Node::from_cube(cube, &tables);

//         // [R3, R2, F2, L3, R1]

//         let next_moves = a
//             .produce_next_nodes(NonZeroU8::new(5).unwrap(), &tables)
//             .unwrap();
//         let b = next_moves.children.skip(14).next().unwrap();
//         let next_moves = b
//             .produce_next_nodes(NonZeroU8::new(4).unwrap(), &tables)
//             .unwrap();

//         for c in next_moves.children {
//             println!("{c:?}")
//         }

//         Ok(())
//     }

//     #[test]
//     fn new_table_offsets() -> anyhow::Result<()> {
//         let tables = Box::leak(Box::new(Tables::new("tables")?));
//         let _ = TableOffsets::new(tables);

//         Ok(())
//     }

//     fn to_hex_underscored(bytes: &[u16]) -> String {
//         // 2 hex chars per byte + underscores
//         let mut out = String::with_capacity(bytes.len() * 5);

//         for (i, b) in bytes.iter().enumerate() {
//             if i != 0 {
//                 out.push('_');
//             }
//             use std::fmt::Write;
//             write!(out, "{:04X}", b).unwrap();
//         }

//         out
//     }

//     fn collect_scalar_children(
//         node: Phase1Node,
//         moves_remaining: NonZeroU8,
//         tables: &Tables,
//     ) -> BTreeSet<String> {
//         let frame = node
//             .produce_next_nodes(moves_remaining, tables)
//             .expect("scalar path pruned unexpectedly");

//         let keys = frame
//             .children
//             .map(|n| unsafe { core::mem::transmute::<_, [u16; 8]>(n) })
//             .map(|x| to_hex_underscored(&x))
//             .collect::<BTreeSet<_>>();

//         keys
//     }

//     fn collect_simd_children(
//         node: Phase1Node,
//         moves_remaining: NonZeroU8,
//         table_offsets: &TableOffsets,
//     ) -> BTreeSet<String> {
//         // SIMD API requires a 16-wide buffer with node in slot 0
//         let mut buf = [node; 16];

//         let count =
//             Phase1Node::produce_next_nodes::<false>(&mut buf, moves_remaining, table_offsets);

//         let keys = buf[1..=count]
//             .iter()
//             .copied()
//             .map(|n| unsafe { core::mem::transmute::<_, [u16; 8]>(n) })
//             .map(|x| to_hex_underscored(&x))
//             .collect::<BTreeSet<_>>();

//         keys
//     }

//     #[test]
//     fn simd_matches_scalar_single_random() -> anyhow::Result<()> {
//         let tables = Box::leak(Box::new(Tables::new("tables")?));
//         let table_offsets = TableOffsets::new(&tables);

//         let mut rng = ChaCha8Rng::seed_from_u64(123);
//         let cube: ReprCube =
//             rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

//         let mut node = Phase1Node::from_cube(cube, &tables);
//         node.previous_axis = CubePreviousAxis::B as u8 as u16;
//         let moves_remaining = NonZeroU8::new(10).unwrap();

//         let scalar_keys = collect_scalar_children(node, moves_remaining, &tables);

//         let simd_keys = collect_simd_children(node, moves_remaining, &table_offsets);

//         assert_eq!(scalar_keys, simd_keys, "SIMD children differ from scalar");

//         Ok(())
//     }

//     #[test]
//     fn simd_with_cancellation() -> anyhow::Result<()> {
//         let tables = Box::leak(Box::new(Tables::new("tables")?));
//         let table_offsets = TableOffsets::new(tables);

//         let node = Phase1Node {
//             previous_axis: CubePreviousAxis::U as u16,
//             edge_group_orient_sym: EdgeGroupOrientSymCoord(18910),
//             edge_group_orient_correct: 14,
//             corner_perm_combo: 12398,
//             corner_orient_raw: CornerOrientRawCoord(1550),
//             u_edge_positions: UEdgePositions(EdgePositions(5392)),
//             d_edge_positions: DEdgePositions(EdgePositions(10634)),
//             e_edge_positions: EEdgePositions(EdgePositions(1514)),
//         };

//         let moves_remaining = NonZeroU8::new(10).unwrap();

//         let scalar_keys = collect_scalar_children(node, moves_remaining, tables);

//         let simd_keys = collect_simd_children(node, moves_remaining, &table_offsets);

//         assert_eq!(scalar_keys, simd_keys, "SIMD children differ from scalar");

//         Ok(())
//     }

//     #[bench]
//     fn simd_micro_incremental_solutions(bench: &mut test::Bencher) {
//         let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
//         let table_offsets = TableOffsets::new(&tables);
//         let mut rng = ChaCha8Rng::seed_from_u64(3);
//         let cube: ReprCube =
//             rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
//         let mut phase_1 = Phase1Node::from_cube(cube, &tables);
//         phase_1.previous_axis = CubePreviousAxis::B as u8 as u16;

//         let mut buf = [phase_1; 16];

//         bench.iter(|| {
//             let _ = Phase1Node::produce_next_nodes::<false>(
//                 &mut buf,
//                 unsafe { NonZeroU8::new_unchecked(30) },
//                 &table_offsets,
//             );
//         });
//     }
// }

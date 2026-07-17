use std::{
    num::NonZeroU8,
    simd::{Simd, num::SimdUint},
};

use crate::{
    CornerOrient, CubeMove, ReprCube, Tables, cube_ops::{cube_prev_axis::{CubePreviousAxis, CubePreviousFace}, cube_sym::DominoSymmetry}, kociemba::{
        coords::{
            CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientRawCoord, EdgeGroupOrientSymCoord, corner_perm_combo_coord::CornerPermComboCoord, edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        }, partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
        }, search::phase_1_node::Phase1Node, tables::{
            lookup_sym_corner_orient::{
                CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable,
            },
            move_sym_corner_orient::MoveSymCornerOrientTable,
            move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
            prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table,
        },
    }, simd_utils::{combine, lut_u8x16x2, lut_u16x16x3},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JoinedPhase1Node {
    pub search: SearchPhase1Node,
    pub reconstruct: ReconstructPhase1Node,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SearchPhase1Node {
    pub edge_group_orient_sym: u16,
    pub corner_perm_sym: u16,
    pub corner_orient_sym: u8,
    // cp hi, co lo
    pub combined_sym_correct: u8,
    pub phase_1_dist: u8,
    pub previous_face: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReconstructPhase1Node {
    // while this is redundant with the search coord, it makes it easier to re-track the changes to the 
    pub edge_group_orient_sym: u16,
    pub u_edge_positions: UEdgePositions,
    pub d_edge_positions: DEdgePositions,
    pub e_edge_positions: EEdgePositions,
}

impl ReconstructPhase1Node {
    pub fn cube_move(&mut self, mv: CubeMove, tables: &Tables) {
        todo!()
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

impl SearchPhase1Node {
    pub fn from_cube(mut cube: ReprCube, tables: &Tables, mut previous_face: CubePreviousFace) -> Self {
        let corner_orient_table: &LookupSymCornerOrientTable = tables.as_ref();
        let phase_1_prune: &PrunePhase1Table = tables.as_ref();

        {
            let (_u, _d, e_edge_positions) = split_edge_positions(cube.edge_perm);
            let edge_orient_raw = cube.edge_orient.into_coord();

            let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
            let edge_group_orient_raw_coord =
                EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

            let edge_group_orient_combo =
                EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);

            cube = cube.domino_conjugate(edge_group_orient_combo.domino_conjugation);
            previous_face = previous_face.domino_conjugate(edge_group_orient_combo.domino_conjugation);
        };

        let ReprCube { corner_perm, corner_orient, edge_perm, edge_orient } = cube;

        let (_u, _d, e_edge_positions) =
            split_edge_positions(edge_perm);

        let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
        let edge_orient_raw = edge_orient.into_coord();
        let edge_group_orient_raw_coord =
            EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

        let edge_group_orient_combo =
            EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);

        // we rotated the input cube specifically so this would be true.
        debug_assert_eq!(edge_group_orient_combo.domino_conjugation, DominoSymmetry::IDENTITY);

        let corner_perm_combo = CornerPermComboCoord::from_raw(&tables, corner_perm.into_coord());
        let corner_orient_raw = corner_orient.into_coord();
        let corner_orient_combo = corner_orient_table.get_combo_from_raw(corner_orient_raw);

        let combined_sym_correct = (corner_perm_combo.domino_conjugation.0 << 4) | (corner_orient_combo.domino_conjugation.0);

        let phase_1_dist = phase_1_prune.get_value(edge_group_orient_combo.sym_coord, corner_orient_raw);

        Self {
            edge_group_orient_sym: edge_group_orient_combo.sym_coord.0,
            corner_perm_sym: corner_perm_combo.sym_coord.0,
            corner_orient_sym: corner_orient_combo.sym_coord.0,
            combined_sym_correct,
            phase_1_dist,
            previous_face: previous_face.0,
        }
    }

    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline(always)]
    pub fn produce_next_nodes(
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

        let previous_face = CubePreviousFace(start_node.previous_face);
        let mv = unsafe { core::mem::transmute::<&[CubeMove; 16], &[u8; 16]>(previous_face.get_moves()) };

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
            | (Simd::from_array(new_co_c_arr).cast::<u32>() << 18))
            .cast::<usize>()
            .to_array();

        let dist_bytes =
            dist_byte_addr_arr.map(|i| *unsafe { prune_tbl.as_buffer().get_unchecked(i) });
        let new_co_s_hi_arr = Simd::from_array(u128::to_ne_bytes(
            (new_cp_co_s_arr >> 2) & 0x0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C0C,
        ));
        let dist_i = (Simd::from_array(dist_bytes) >> (new_co_s_hi_arr << 1)).to_array();
        let distance_options = DISTANCE_OPTIONS_LOOKUP[start_node.phase_1_dist as usize];

        // final dists
        let new_dist = lut_u8x16x2(&distance_options, &dist_i);

        let initial_prev_faces = unsafe { core::mem::transmute::<&[CubePreviousFace; 16], &[u8; 16]>(previous_face.get_new_prev_axes()) };
        let new_previous_faces = CubePreviousFace::domino_conjugate_16(*initial_prev_faces, u128::to_ne_bytes(tbl_ego_s_arr));

        let out = combine(
            new_ego_c_arr,
            new_cp_c_arr,
            new_co_c_arr,
            u128::to_ne_bytes(new_cp_co_s_arr),
            new_dist,
            new_previous_faces,
        );

        for i in 1..16 {
            unsafe { *slice.get_unchecked_mut(i) = core::mem::transmute(out.get_unchecked(i)) };
        }
        previous_face.get_num_moves() as usize
    }
}

fn phase_1_normalize_cube(cube: ReprCube, tables: &Tables) -> ReprCube {
    let (_, _, e_edge_positions) = split_edge_positions(cube.edge_perm);

    let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
    let edge_orient_raw = cube.edge_orient.into_coord();
    let edge_group_orient_raw_coord =
        EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

    let edge_group_orient_combo =
        EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);

    cube.domino_conjugate(edge_group_orient_combo.domino_conjugation)
}

fn produce_next_nodes_reference(cube: ReprCube, dist: u8, previous_axis: CubePreviousAxis, tables: &Tables) -> Vec<ReprCube> {
    let cube = phase_1_normalize_cube(cube, tables);
    CubeMove::all_iter().map(|mv| phase_1_normalize_cube(cube.apply_cube_move(mv), tables)).collect()
}


#[cfg(test)]
mod test {
    use crate::ReprCube;

    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use std::{hint::black_box, num::NonZeroU8};
    extern crate test;

    #[bench]
    fn phase_1_next_a_new(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let mut phase_1 = Phase1Node::from_cube(cube, &tables);
        phase_1.previous_axis = CubePreviousAxis::B;

        let phase_1_a_exp = SearchPhase1Node {
            edge_group_orient_sym: phase_1.edge_group_orient_sym.0,
            corner_perm_sym: phase_1.corner_perm_raw.0,
            corner_orient_sym: 75,
            combined_sym_correct: 37,
            phase_1_dist: 9,
            previous_face: 4,
        };

        let mut buf = [phase_1_a_exp; 16];

        bench.iter(|| {
            let a = SearchPhase1Node::produce_next_nodes(
                &mut buf,
                unsafe { NonZeroU8::new_unchecked(30) },
                tables,
            );
            black_box(a);
            black_box(&buf);
        });
    }
}

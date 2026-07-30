use std::{
    num::{NonZeroU8, NonZeroUsize},
    simd::{Simd, num::SimdUint},
};

use crate::{
    CornerOrient, CubeMove, ReprCube, Tables,
    cube_ops::{
        cube_prev_axis::{CubePreviousAxis, CubePreviousFace},
        cube_sym::DominoSymmetry,
    },
    kociemba::{
        coords::{
            CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientRawCoord,
            EdgeGroupOrientSymCoord, corner_perm_combo_coord::CornerPermComboCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
        },
        search::phase_1_node::Phase1Node,
        tables::{
            lookup_sym_corner_orient::{
                CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable,
            },
            move_edge_positions::MoveEdgePositionsTable,
            move_sym_corner_orient::MoveSymCornerOrientTable,
            move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
            prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table,
        },
    },
    simd_utils::{combine, gather16, lut_u8x16x2, lut_u16x16x3},
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

pub fn split_cube(
    mut cube: ReprCube,
    tables: &Tables,
    mut previous_face: CubePreviousFace,
) -> (SearchPhase1Node, ReconstructPhase1Node) {
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

    let ReprCube {
        corner_perm,
        corner_orient,
        edge_perm,
        edge_orient,
    } = cube;

    let (u_edge_positions, d_edge_positions, e_edge_positions) = split_edge_positions(edge_perm);

    let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
    let edge_orient_raw = edge_orient.into_coord();
    let edge_group_orient_raw_coord =
        EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

    let edge_group_orient_combo =
        EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);

    // we rotated the input cube specifically so this would be true.
    debug_assert_eq!(
        edge_group_orient_combo.domino_conjugation,
        DominoSymmetry::IDENTITY
    );

    let corner_perm_combo = CornerPermComboCoord::from_raw(&tables, corner_perm.into_coord());
    let corner_orient_raw = corner_orient.into_coord();
    let corner_orient_combo = corner_orient_table.get_combo_from_raw(corner_orient_raw);

    let combined_sym_correct =
        (corner_perm_combo.domino_conjugation.0 << 4) | (corner_orient_combo.domino_conjugation.0);

    let phase_1_dist =
        phase_1_prune.get_value(edge_group_orient_combo.sym_coord, corner_orient_raw);

    let search = SearchPhase1Node {
        edge_group_orient_sym: edge_group_orient_combo.sym_coord.0,
        corner_perm_sym: corner_perm_combo.sym_coord.0,
        corner_orient_sym: corner_orient_combo.sym_coord.0,
        combined_sym_correct,
        phase_1_dist,
        previous_face: previous_face.0,
    };

    let recon = ReconstructPhase1Node {
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
    };

    (search, recon)
}

impl SearchPhase1Node {
    pub fn from_cube(
        mut cube: ReprCube,
        tables: &Tables,
        mut previous_face: CubePreviousFace,
    ) -> Self {
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
            previous_face =
                previous_face.domino_conjugate(edge_group_orient_combo.domino_conjugation);
        };

        let ReprCube {
            corner_perm,
            corner_orient,
            edge_perm,
            edge_orient,
        } = cube;

        let (_u, _d, e_edge_positions) = split_edge_positions(edge_perm);

        let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
        let edge_orient_raw = edge_orient.into_coord();
        let edge_group_orient_raw_coord =
            EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

        let edge_group_orient_combo =
            EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);

        // we rotated the input cube specifically so this would be true.
        debug_assert_eq!(
            edge_group_orient_combo.domino_conjugation,
            DominoSymmetry::IDENTITY
        );

        let corner_perm_combo = CornerPermComboCoord::from_raw(&tables, corner_perm.into_coord());
        let corner_orient_raw = corner_orient.into_coord();
        let corner_orient_combo = corner_orient_table.get_combo_from_raw(corner_orient_raw);

        let combined_sym_correct = (corner_perm_combo.domino_conjugation.0 << 4)
            | (corner_orient_combo.domino_conjugation.0);

        let phase_1_dist =
            phase_1_prune.get_value(edge_group_orient_combo.sym_coord, corner_orient_raw);

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
    #[inline(never)]
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
        let mv = Simd::from_array(*unsafe {
            core::mem::transmute::<&[CubeMove; 16], &[u8; 16]>(previous_face.get_moves())
        });

        let cmb_s = start_node.combined_sym_correct;
        let offset = Simd::splat(4);

        // indices adjusted by current coordinate symmetry
        let ego_i = mv;
        let co_i = Simd::from_array(lut_u8x16x2(
            &SYM_CONJ[(cmb_s & 0xF) as usize],
            &mv.to_array(),
        ));
        let cp_i = Simd::from_array(lut_u8x16x2(
            &SYM_CONJ[(cmb_s >> 4) as usize],
            &mv.to_array(),
        ));

        // indices adjusted for delayed position of offset
        let ego_s_i = offset + ego_i;
        let co_s_i = offset + co_i;
        let cp_s_i = offset + cp_i;

        // pre-adjusted symmetries
        let tbl_ego_s_arr =
            Simd::from_array(lut_u8x16x2(&ego_row.as_u8_arrays()[1], &ego_s_i.to_array()));
        let tbl_co_s_arr =
            Simd::from_array(lut_u8x16x2(&co_row.as_u8_arrays()[1], &co_s_i.to_array()));
        let tbl_cp_s_arr =
            Simd::from_array(lut_u8x16x2(&cp_row.as_u8_arrays()[1], &cp_s_i.to_array()));

        let curr_cp_co_s_arr = Simd::splat(cmb_s);

        // packed syms in preparation for combining
        let tbl_ego_ego_s_arr = (tbl_ego_s_arr << 4) | tbl_ego_s_arr;
        let tbl_cp_co_s_arr = (tbl_cp_s_arr << 4) | tbl_co_s_arr;

        // final coordinates
        let new_ego_c_arr =
            Simd::from_array(lut_u16x16x3(ego_row.as_u16_array(), &ego_i.to_array()));
        let new_co_c_arr =
            Simd::from_array(lut_u8x16x2(&co_row.as_u8_arrays()[0], &co_i.to_array()));
        let new_cp_c_arr = Simd::from_array(lut_u16x16x3(cp_row.as_u16_array(), &cp_i.to_array()));

        // final packed syms
        let new_cp_co_s_arr = DominoSymmetry::bitwise_then_simd(
            DominoSymmetry::bitwise_then_simd(tbl_ego_ego_s_arr, curr_cp_co_s_arr),
            tbl_cp_co_s_arr,
        );

        let new_co_s_lo_arr = (tbl_ego_s_arr ^ tbl_co_s_arr ^ curr_cp_co_s_arr) & Simd::splat(0x03);
        let dist_byte_addr_arr = (new_ego_c_arr.cast::<u32>() << 2)
            | (new_co_s_lo_arr.cast::<u32>())
            | (new_co_c_arr.cast::<u32>() << 18);

        let dist_bytes = unsafe { gather16(prune_tbl.as_buffer(), dist_byte_addr_arr) };
        let new_co_s_hi_arr = (new_cp_co_s_arr >> 2) & Simd::splat(0x03);
        let dist_i = ((dist_bytes >> (new_co_s_hi_arr << 1)) & Simd::splat(0x03)).to_array();
        let distance_options =
            unsafe { DISTANCE_OPTIONS_LOOKUP.get_unchecked(start_node.phase_1_dist as usize) };

        // final dists
        let new_dist = lut_u8x16x2(&distance_options, &dist_i);

        let initial_prev_faces = unsafe {
            core::mem::transmute::<&[CubePreviousFace; 16], &[u8; 16]>(
                previous_face.get_new_prev_axes(),
            )
        };
        let new_previous_faces =
            CubePreviousFace::domino_conjugate_16(*initial_prev_faces, tbl_ego_s_arr.to_array());

        let out = combine(
            new_ego_c_arr.to_array(),
            new_cp_c_arr.to_array(),
            new_co_c_arr.to_array(),
            new_cp_co_s_arr.to_array(),
            new_dist,
            new_previous_faces,
        );

        for i in 1..16 {
            unsafe {
                *slice.get_unchecked_mut(i) =
                    core::mem::transmute::<[u8; 8], Self>(*out.get_unchecked(i))
            };
        }
        previous_face.get_num_moves() as usize
    }
}

impl ReconstructPhase1Node {
    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline(never)]
    pub fn produce_next_node(
        &mut self,
        ego: EdgeGroupOrientSymCoord,
        prev_face: CubePreviousFace,
        move_index: NonZeroUsize,
        tables: &Tables,
    ) {
        let ego_mv_tbl: &MoveSymEdgeGroupOrientTable = tables.as_ref();
        let edge_tbl: &MoveEdgePositionsTable = tables.as_ref();

        let ego_row = ego_mv_tbl.row(ego);
        // println!("{:?}", ego_row.conjugations);
        let u_row_1 = edge_tbl.row(self.u_edge_positions.0);
        let d_row_1 = edge_tbl.row(self.d_edge_positions.0);
        let e_row_1 = edge_tbl.row(self.e_edge_positions.0);
        
        let mv = prev_face.get_moves()[move_index.get()] as u8 as usize;
        for i in 1..16 {
            let mv = prev_face.get_moves()[i] as u8 as usize;
            print!("{}, ", ego_row.conjugations[mv]);
        }
        println!();
        let sym = DominoSymmetry(ego_row.conjugations[mv]);

        let u_adj = u_row_1.0[mv];
        let d_adj = d_row_1.0[mv];
        let e_adj = e_row_1.0[mv];

        let u_row_2 = edge_tbl.row(EdgePositions(u_adj));
        let d_row_2 = edge_tbl.row(EdgePositions(d_adj));
        let e_row_2 = edge_tbl.row(EdgePositions(e_adj));

        self.u_edge_positions = UEdgePositions(EdgePositions(u_row_2.get_conj(sym)));
        self.d_edge_positions = DEdgePositions(EdgePositions(d_row_2.get_conj(sym)));
        self.e_edge_positions = EEdgePositions(EdgePositions(e_row_2.get_conj(sym)));
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

#[cfg(test)]
mod test {
    use crate::ReprCube;

    use super::*;
    use itertools::Itertools as _;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use std::{
        hint::black_box,
        io::{self, Write},
        num::NonZeroU8,
    };
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

    #[test]
    fn neighbors_match() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        let previous_face = CubePreviousFace(0);
        let expected = previous_face
            .get_moves()
            .into_iter()
            .copied()
            .skip(1)
            .take(previous_face.get_num_moves() as usize)
            .map(|mv| {
                SearchPhase1Node::from_cube(
                    cube.apply_cube_move(mv),
                    &tables,
                    previous_face.update_with_new_move(mv),
                )
            })
            .collect_vec();
        let actual = {
            let x = SearchPhase1Node::from_cube(cube, &tables, previous_face);
            let mut buf = [x; 16];
            let count = SearchPhase1Node::produce_next_nodes(
                &mut buf,
                NonZeroU8::new(100).unwrap(),
                &tables,
            );

            for c in buf.iter_mut() {
                c.corner_perm_sym &= 0xFFF;
            }

            buf.into_iter().skip(1).take(count).collect_vec()
        };

        println!();
        print!("ego: ");
        for a in expected.iter() {
            print!(" {:4x}", a.edge_group_orient_sym);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.edge_group_orient_sym);
        }

        println!();
        print!("co:  ");
        for a in expected.iter() {
            print!(" {:4x}", a.corner_orient_sym);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.corner_orient_sym);
        }

        println!();
        print!("cp:  ");
        for a in expected.iter() {
            print!(" {:4x}", a.corner_perm_sym);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.corner_perm_sym);
        }

        println!();
        print!("cor: ");
        for a in expected.iter() {
            print!(" {:4x}", a.combined_sym_correct);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.combined_sym_correct);
        }

        println!();
        print!("dst: ");
        for a in expected.iter() {
            print!(" {:4x}", a.phase_1_dist);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.phase_1_dist);
        }

        println!();
        print!("cpf: ");
        for a in expected.iter() {
            print!(" {:4b}", a.previous_face);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4b}", a.previous_face);
        }
        println!();

        std::io::stdout().flush()?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn neighbors_match_full() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        let previous_face = CubePreviousFace(0);
        let expected = previous_face
            .get_moves()
            .into_iter()
            .copied()
            .skip(1)
            .take(previous_face.get_num_moves() as usize)
            .map(|mv| {
                split_cube(
                    cube.apply_cube_move(mv),
                    &tables,
                    previous_face.update_with_new_move(mv),
                )
            })
            .collect_vec();
        let actual = {
            let (x, y) = split_cube(cube, &tables, previous_face);
            let mut buf = [x; 16];
            let count = SearchPhase1Node::produce_next_nodes(
                &mut buf,
                NonZeroU8::new(100).unwrap(),
                &tables,
            );

            for c in buf.iter_mut() {
                c.corner_perm_sym &= 0xFFF;
            }

            buf[1..16]
                .iter()
                .copied()
                .zip((1..16).into_iter().map(|i| {
                    let mut y = y.clone();
                    y.produce_next_node(
                        EdgeGroupOrientSymCoord(x.edge_group_orient_sym),
                        CubePreviousFace(x.previous_face),
                        NonZeroUsize::new(i).unwrap(),
                        &tables,
                    );
                    y
                }))
                .take(count)
                .collect_vec()
        };

        println!();
        print!("u:   ");
        for a in expected.iter() {
            print!(" {:4x}", a.1.u_edge_positions.0.0);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.1.u_edge_positions.0.0);
        }

        println!();
        print!("d:   ");
        for a in expected.iter() {
            print!(" {:4x}", a.1.d_edge_positions.0.0);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.1.d_edge_positions.0.0);
        }

        println!();
        print!("e:   ");
        for a in expected.iter() {
            print!(" {:4x}", a.1.e_edge_positions.0.0);
        }
        println!();
        print!("     ");
        for a in actual.iter() {
            print!(" {:4x}", a.1.e_edge_positions.0.0);
        }

        // println!();
        // print!("co:  ");
        // for a in expected.iter() {
        //     print!(" {:4x}", a.corner_orient_sym);
        // }
        // println!();
        // print!("     ");
        // for a in actual.iter() {
        //     print!(" {:4x}", a.corner_orient_sym);
        // }

        // println!();
        // print!("cp:  ");
        // for a in expected.iter() {
        //     print!(" {:4x}", a.corner_perm_sym);
        // }
        // println!();
        // print!("     ");
        // for a in actual.iter() {
        //     print!(" {:4x}", a.corner_perm_sym);
        // }

        // println!();
        // print!("cor: ");
        // for a in expected.iter() {
        //     print!(" {:4x}", a.combined_sym_correct);
        // }
        // println!();
        // print!("     ");
        // for a in actual.iter() {
        //     print!(" {:4x}", a.combined_sym_correct);
        // }

        // println!();
        // print!("dst: ");
        // for a in expected.iter() {
        //     print!(" {:4x}", a.phase_1_dist);
        // }
        // println!();
        // print!("     ");
        // for a in actual.iter() {
        //     print!(" {:4x}", a.phase_1_dist);
        // }

        // println!();
        // print!("cpf: ");
        // for a in expected.iter() {
        //     print!(" {:4b}", a.previous_face);
        // }
        // println!();
        // print!("     ");
        // for a in actual.iter() {
        //     print!(" {:4b}", a.previous_face);
        // }
        // println!();

        std::io::stdout().flush()?;

        assert_eq!(expected, actual);

        Ok(())
    }
}

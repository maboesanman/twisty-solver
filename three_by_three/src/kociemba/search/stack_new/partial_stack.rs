use fearless_simd::{Bytes, SimdBase, SimdFrom, SimdWiden, kernel};
use itertools::Itertools;
use pathfinding::utils;

use crate::{
    CubeMove, ReprCube, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{
            CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientRawCoord,
            EdgeGroupOrientSymCoord, corner_perm_combo_coord::CornerPermComboCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::{
            edge_positions::{
                DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
            },
            ud_edge_perm::UDEdgePerm,
        },
        search::{
            phase_2_node::Phase2Node,
            stack_new::utils::{get_entry, get_entry_mut, get_entry_slice_mut},
        },
        tables::{
            lookup_sym_corner_orient::{CornerOrientComboCoord, CornerOrientSymCoord},
            move_edge_positions::MoveEdgePositionsTable,
            move_sym_corner_orient::MoveSymCornerOrientTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
            prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table,
        },
    },
};

#[derive(Clone, Debug)]
pub struct PartialStack<const N: usize> {
    base_cube: ReprCube,
    base_phase_1_coords: Phase1,
    base_phase_1_dist: u8,
    base_phase_2_coords: Phase2,

    phase_1_coords: [[Phase1; 15]; N],
    phase_1_dists: [[u8; 15]; N],

    phase_2_len: u8,
    phase_2_coords: [Phase2; N],
}

impl<const N: usize> PartialStack<N> {
    pub fn new(base_cube: ReprCube, tables: &Tables) -> Self {
        let (base_phase_1_coords, base_phase_2_coords) = produce_phase_1_phase_2(base_cube, tables);
        let base_phase_1_dist = base_phase_1_coords.dist_recurse(tables);

        let phase_1_coords = [[Phase1 {
            ego_c: EdgeGroupOrientSymCoord(0),
            co_c: CornerOrientSymCoord(0),
            combo_s: 0,
        }; 15]; N];

        let phase_1_dists = [[0; 15]; N];

        let phase_2_coords = [Phase2 {
            cp_c: CornerPermComboCoord {
                sym_coord: CornerPermSymCoord(0),
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            u_edge_c: UEdgePositions(EdgePositions(0)),
            d_edge_c: DEdgePositions(EdgePositions(0)),
            e_edge_c: EEdgePositions(EdgePositions(0)),
        }; N];

        Self {
            base_cube,
            base_phase_1_coords,
            base_phase_1_dist,
            base_phase_2_coords,
            phase_1_coords,
            phase_1_dists,
            phase_2_len: 0,
            phase_2_coords,
        }
    }

    fn phase_1_dist_lookup<S: fearless_simd::Simd>(simd: S, dist: u8) -> fearless_simd::u8x16<S> {
        debug_assert!((dist as usize) < DISTANCE_OPTIONS_LOOKUP.len());
        let dist_lookup = unsafe { DISTANCE_OPTIONS_LOOKUP.get_unchecked(dist as usize) };
        fearless_simd::u8x16::simd_from(simd, *dist_lookup)
    }

    pub fn begin_initial_node_expansion<S: fearless_simd::Simd>(
        &self,
        simd: S,
        moves: impl IntoIterator<Item = CubeMove>,
        tables: &Tables,
    ) -> impl IntoIterator<Item = (Phase1, u8)> {
        let phase_1 = self.base_phase_1_coords;
        let phase_1_dist = self.base_phase_1_dist;
        let dist_lookup = Self::phase_1_dist_lookup(simd, phase_1_dist);

        moves.into_iter().map(move |cube_move| {
            let new_phase_1 = phase_1.apply_cube_move(cube_move, tables);
            let d_mod_3 = new_phase_1.dist_mod_3(tables);
            let new_dist = dist_lookup[d_mod_3 as usize];
            (new_phase_1, new_dist)
        })
    }

    pub fn end_initial_node_expansion(&mut self, results: impl IntoIterator<Item = (Phase1, u8)>) {
        for (i, (phase_1, dist)) in results.into_iter().enumerate() {
            *get_entry_mut(i, &mut self.phase_1_coords) = phase_1;
            *get_entry_mut(i, &mut self.phase_1_dists) = dist;
        }
    }

    pub(crate) fn initial_node(&self, index: usize) -> (Phase1, u8) {
        (
            *get_entry(index, &self.phase_1_coords),
            *get_entry(index, &self.phase_1_dists),
        )
    }

    pub(crate) fn base_node(&self) -> (Phase1, u8) {
        (self.base_phase_1_coords, self.base_phase_1_dist)
    }

    ///
    #[inline(always)]
    pub fn begin_node_expansion<S: fearless_simd::Simd>(
        &self,
        current_position: usize,
        simd: S,
        moves: fearless_simd::u8x16<S>,
        move_mask: fearless_simd::mask8x16<S>,
        tables: &Tables,
    ) -> PartialExpansionResult<S> {
        let phase_1 = get_entry(current_position, &self.phase_1_coords);
        let dist = get_entry(current_position, &self.phase_1_dists);
        let dist_lookup = Self::phase_1_dist_lookup(simd, *dist);

        let ego_mv_tbl: &MoveSymEdgeGroupOrientTable = tables.as_ref();
        let co_mv_tbl: &MoveSymCornerOrientTable = tables.as_ref();
        let prune_tbl: &PrunePhase1DoubleSymMod3Table = tables.as_ref();

        let ego_row = ego_mv_tbl.row(phase_1.ego_c).as_fearless_simd(simd);
        let co_row = co_mv_tbl.row(phase_1.co_c).as_fearless_simd(simd);

        let ego_i = {
            let i = (phase_1.combo_s >> 4) as usize;
            debug_assert!(i < SYM_CONJ.len());
            let tbl_arr = *unsafe { SYM_CONJ.get_unchecked(i) };
            let tbl_simd = fearless_simd::u8x32::<S>::simd_from(simd, tbl_arr);
            super::utils::lut_u8x32(simd, tbl_simd, moves)
        };

        let co_i = {
            let i = (phase_1.combo_s & 0xF) as usize;
            debug_assert!(i < SYM_CONJ.len());
            let tbl_arr = *unsafe { SYM_CONJ.get_unchecked(i) };
            let tbl_simd = fearless_simd::u8x32::<S>::simd_from(simd, tbl_arr);
            super::utils::lut_u8x32(simd, tbl_simd, moves)
        };

        let (ego_a, ego_b, ego_s) = {
            let ego_a = ego_i << 1;
            let ego_b = ego_a + simd.splat_u8x16(1);
            let ego_s = ego_i + simd.splat_u8x16(36);

            let ego_a = super::utils::lut_u8x64(simd, ego_row, ego_a);
            let ego_b = super::utils::lut_u8x64(simd, ego_row, ego_b);
            let ego_s = super::utils::lut_u8x64(simd, ego_row, ego_s);

            (ego_a, ego_b, ego_s)
        };

        let (co, co_s) = {
            let co_c = co_i;
            let co_s = co_i + simd.splat_u8x16(36);

            let co_idxs = simd.combine_u8x32(
                simd.combine_u8x16(co_c, simd.splat_u8x16(0)),
                simd.combine_u8x16(co_s, simd.splat_u8x16(0)),
            );
            let abc_ = simd.swizzle_dyn_u8x64(co_row, co_idxs);
            let (a_, c_) = simd.split_u8x64(abc_);
            let (a, _) = simd.split_u8x32(a_);
            let (c, _) = simd.split_u8x32(c_);
            (a, c)
        };

        let combo_s = {
            let a = simd.splat_u8x16(phase_1.combo_s);
            let b = (ego_s << 4) | co_s;
            DominoSymmetry::bitwise_then_fearless_simd(a, b, simd)
        };

        let dist = {
            let mask_2 = fearless_simd::u8x16::splat(simd, 0x03);
            let mask_4 = fearless_simd::u8x16::splat(simd, 0x0F);
            let ego_s_new = combo_s >> 4;
            let ego_syms_inv = DominoSymmetry::bitwise_invert_fearless_simd(ego_s_new, simd);
            let co_lookup =
                DominoSymmetry::bitwise_then_fearless_simd(ego_syms_inv, combo_s, simd) & mask_4;
            let co_lookup_lo = ego_s_new ^ combo_s & mask_2;
            let co_lookup_hi = (co_lookup >> 2) & mask_2;

            let (a, b) = simd.interleave_u8x16(ego_a, ego_b);
            let ab = fearless_simd::u16x16::from_bytes(simd.combine_u8x16(a, b));

            let (c, d) = co.widen();
            let cd = simd.combine_u16x8(c, d);

            #[cfg(target_endian = "big")]
            let (ab, cd) = (cd, ab);

            let (ab, cd) = simd.interleave_u16x16(ab, cd);
            let abcd = fearless_simd::u32x16::from_bytes(simd.combine_u16x16(ab, cd).to_bytes());

            debug_assert_eq!(
                abcd.to_vec(),
                ab.to_vec()
                    .into_iter()
                    .zip(co.to_vec())
                    .map(|(ab, co)| ab as u32 | ((co as u32) << 16))
                    .collect_vec(),
            );

            let table = prune_tbl.as_buffer();
            let word_indices = abcd;
            let byte_indices = co_lookup_lo;

            let dist_bytes = super::utils::gather(simd, table, word_indices, byte_indices);
            let dist_mod_3 = (dist_bytes >> (co_lookup_hi << 1)) & mask_2;

            dist_lookup.swizzle_dyn(dist_mod_3)
        };

        PartialExpansionResult {
            ego_a,
            ego_b,
            co,
            combo_s,
            dist,
            current_position,
        }
    }

    #[inline(always)]
    pub fn end_node_expansion<S: fearless_simd::Simd>(
        &mut self,
        simd: S,
        partial_expansion_result: PartialExpansionResult<S>,
    ) {
        let PartialExpansionResult {
            ego_a,
            ego_b,
            co,
            combo_s,
            dist,
            current_position,
        } = partial_expansion_result;

        let out_pos = current_position + 1;

        // do an interleaved write for the main items
        {
            let vectors = [ego_a, ego_b, co, combo_s];
            debug_assert_eq!(core::mem::size_of::<Phase1>(), 4);
            let out_buf = get_entry_slice_mut(out_pos, &mut self.phase_1_coords);
            let out_ptr: *mut [Phase1; 16] = out_buf;
            let out_ptr: *mut [u8; 64] = out_ptr.cast();
            let out_buf: &mut [u8; 64] = unsafe { out_ptr.as_mut_unchecked() };

            simd.store_four_interleaved_u8x16(vectors, out_buf);
        }

        // do a regular write for the dists
        {
            let out_buf = get_entry_slice_mut(out_pos, &mut self.phase_1_dists);
            dist.store_slice(out_buf);
        }
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

#[derive(Debug)]
pub struct PartialExpansionResult<S: fearless_simd::Simd> {
    // the first 15 entries here are the ones that matter.
    ego_a: fearless_simd::u8x16<S>,
    ego_b: fearless_simd::u8x16<S>,
    co: fearless_simd::u8x16<S>,
    combo_s: fearless_simd::u8x16<S>,
    dist: fearless_simd::u8x16<S>,
    current_position: usize,
}

impl<S: fearless_simd::Simd> PartialExpansionResult<S> {
    #[inline(always)]
    pub fn swizzle(&mut self, indices: fearless_simd::u8x16<S>) {
        self.ego_a.swizzle_dyn(indices);
        self.ego_b.swizzle_dyn(indices);
        self.co.swizzle_dyn(indices);
        self.combo_s.swizzle_dyn(indices);
        self.dist.swizzle_dyn(indices);
    }

    #[inline(always)]
    pub fn get_dist(&self) -> &fearless_simd::u8x16<S> {
        &self.dist
    }

    #[inline(always)]
    pub fn prefetch(&self) {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Phase1 {
    ego_c: EdgeGroupOrientSymCoord,
    co_c: CornerOrientSymCoord,
    combo_s: u8,
}

impl Phase1 {
    fn ego_sym(&self) -> DominoSymmetry {
        DominoSymmetry(self.combo_s >> 4)
    }

    fn ego_combo(&self) -> EdgeGroupOrientComboCoord {
        EdgeGroupOrientComboCoord {
            sym_coord: self.ego_c,
            domino_conjugation: self.ego_sym(),
        }
    }

    fn co_sym(&self) -> DominoSymmetry {
        DominoSymmetry(self.combo_s & 0xF)
    }

    fn co_combo(&self) -> CornerOrientComboCoord {
        CornerOrientComboCoord {
            sym_coord: self.co_c,
            domino_conjugation: self.co_sym(),
        }
    }

    /// Apply a cube move to this subset of coordinates via move tables
    fn apply_cube_move(self, cube_move: CubeMove, tables: &Tables) -> Self {
        let new_ego = self.ego_combo().apply_cube_move(tables, cube_move);
        let new_co = self.co_combo().apply_cube_move(tables, cube_move);

        Self {
            ego_c: new_ego.sym_coord,
            co_c: new_co.sym_coord,
            combo_s: (new_ego.domino_conjugation.0 << 4) | (new_co.domino_conjugation.0 & 0xF),
        }
    }

    /// Reduce a full `ReprCube` down to its phase-1 representation (the
    /// symmetrized edge-group orientation, corner orientation and their domino
    /// conjugations). This projection is destructive: information about the
    /// exact cube that isn't needed for phase 1 (corner permutation, E-slice
    /// details folded into symmetry reduction) is discarded.
    pub(crate) fn from_cube(
        cube: ReprCube,
        tables: &Tables,
    ) -> Self {
        let ReprCube {
            corner_orient,
            edge_perm,
            edge_orient,
            ..
        } = cube;

        let (_, _, e_edge_positions) = split_edge_positions(edge_perm);

        let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
        let edge_orient_raw = edge_orient.into_coord();
        let edge_group_orient_raw_coord =
            EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

        let edge_group_orient_combo =
            EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);
        let corner_orient_combo = CornerOrientComboCoord::from_raw(&tables, corner_orient.into_coord());

        Self {
            ego_c: edge_group_orient_combo.sym_coord,
            co_c: corner_orient_combo.sym_coord,
            combo_s: (edge_group_orient_combo.domino_conjugation.0 << 4)
                | (corner_orient_combo.domino_conjugation.0 & 0xF),
        }
    }

    /// Normalize the stored domino conjugations so that two `Phase1` values which
    /// describe the same physical state compare as equal, regardless of which
    /// (valid) symmetry representative was picked.
    ///
    /// For each of the edge-group-orientation and corner-orientation halves we fix
    /// the current sym coordinate and try every DominoSymmetry; any conjugation
    /// that decodes to the same raw value is a valid representative. We keep the
    /// lowest-valued such symmetry, so both derivations reduce to one canonical form.
    pub(crate) fn canonicalize(&self, tables: &Tables) -> Self {
        let ego_lookup: &crate::kociemba::tables::lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable =
            tables.as_ref();
        let co_lookup: &crate::kociemba::tables::lookup_sym_corner_orient::LookupSymCornerOrientTable =
            tables.as_ref();

        let ego_raw = ego_lookup.get_raw_from_combo(self.ego_combo());
        let co_raw = co_lookup.get_raw_from_combo(self.co_combo());

        let best_ego_domino = DominoSymmetry::all_iter()
            .filter(|&d| {
                ego_lookup
                    .get_raw_from_combo(EdgeGroupOrientComboCoord {
                        sym_coord: self.ego_c,
                        domino_conjugation: d,
                    })
                    == ego_raw
            })
            .min_by_key(|d| d.0)
            .expect("the current conjugation always preserves the raw value");

        let best_co_domino = DominoSymmetry::all_iter()
            .filter(|&d| {
                co_lookup
                    .get_raw_from_combo(CornerOrientComboCoord {
                        sym_coord: self.co_c,
                        domino_conjugation: d,
                    })
                    == co_raw
            })
            .min_by_key(|d| d.0)
            .expect("the current conjugation always preserves the raw value");

        Self {
            ego_c: self.ego_c,
            co_c: self.co_c,
            combo_s: (best_ego_domino.0 << 4) | (best_co_domino.0 & 0xF),
        }
    }

    /// Compute the residue modulo 3 (dist % 3) of HTM distance from this phase 1 representation to phase 1 solved.
    ///
    /// This distance variant uses the mod 3 table, performing a descent to compute, and therefore should
    /// not be called anywhere hot, since it's quite loopy.
    pub(crate) fn dist_mod_3(&self, tables: &Tables) -> u8 {
        let prune_table: &PrunePhase1DoubleSymMod3Table = tables.as_ref();

        let co_combo = CornerOrientComboCoord {
            sym_coord: self.co_c,
            domino_conjugation: self.ego_sym().inverse().then(self.co_sym()),
        };

        prune_table.get_value(self.ego_c, co_combo)
    }

    /// Compute the HTM distance from this phase 1 representation to phase 1 solved.
    ///
    /// This distance variant uses the mod 3 table, performing a descent to compute, and therefore should
    /// not be called anywhere hot, since it's quite loopy.
    pub(crate) fn dist_recurse(&self, tables: &Tables) -> u8 {
        let mut n = 0;
        let mut probe = *self;

        while probe.ego_c.0 != 0 || probe.co_c.0 != 0 {
            n += 1;
            let d_mod_3 = probe.dist_mod_3(tables);
            let next_mod_3 = match d_mod_3 {
                0 => 2,
                1 => 0,
                2 => 1,
                _ => unreachable!(),
            };

            'a: for cube_move in CubeMove::all_iter() {
                let child = probe.apply_cube_move(cube_move, tables);
                if child.dist_mod_3(tables) == next_mod_3 {
                    probe = child;
                    break 'a;
                }
            }
        }

        n
    }

    /// Compute the HTM distance from this phase 1 representation to phase 1 solved.
    ///
    /// This distance variant uses the full distance table, which is larger than it needs to be.
    fn dist_direct(&self, tables: &Tables) -> u8 {
        let prune_table: &PrunePhase1Table = tables.as_ref();

        let co_combo = CornerOrientComboCoord {
            sym_coord: self.co_c,
            domino_conjugation: self.ego_sym().inverse().then(self.co_sym()),
        };
        let co_raw = co_combo.into_raw(tables);

        prune_table.get_value(self.ego_c, co_raw)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Phase2 {
    cp_c: CornerPermComboCoord,
    u_edge_c: UEdgePositions,
    d_edge_c: DEdgePositions,
    e_edge_c: EEdgePositions,
}

impl Phase2 {
    fn apply_cube_move(self, cube_move: CubeMove, tables: &Tables) -> Self {
        let ep_mv_tbl: &MoveEdgePositionsTable = tables.as_ref();

        let u_row = ep_mv_tbl.row(self.u_edge_c.0);
        let d_row = ep_mv_tbl.row(self.d_edge_c.0);
        let e_row = ep_mv_tbl.row(self.e_edge_c.0);

        let new_cp_c = self.cp_c.apply_cube_move(tables, cube_move);

        let mv_i = cube_move as u8 as usize;
        let new_u_c = UEdgePositions(EdgePositions(u_row.0[mv_i]));
        let new_d_c = DEdgePositions(EdgePositions(d_row.0[mv_i]));
        let new_e_c = EEdgePositions(EdgePositions(e_row.0[mv_i]));

        Self {
            cp_c: new_cp_c,
            u_edge_c: new_u_c,
            d_edge_c: new_d_c,
            e_edge_c: new_e_c,
        }
    }

    fn resolve(self, previous_axis: CubePreviousAxis) -> Phase2Node {
        Phase2Node {
            corner_perm_combo: self.cp_c,
            ud_edge_perm_raw: UDEdgePerm(self.u_edge_c, self.d_edge_c).into_coord(),
            e_edge_perm_raw: EEdgePermRawCoord(self.e_edge_c.0.0 as u8),
            previous_axis,
        }
    }
}

pub(crate) fn produce_phase_1_phase_2(base_cube: ReprCube, tables: &Tables) -> (Phase1, Phase2) {
    let ReprCube {
        corner_perm,
        edge_perm,
        ..
    } = base_cube;

    let (u_edge_positions, d_edge_positions, e_edge_positions) = split_edge_positions(edge_perm);

    let corner_perm_combo = CornerPermComboCoord::from_raw(&tables, corner_perm.into_coord());

    let phase_1 = Phase1::from_cube(base_cube, tables);

    let phase_2 = Phase2 {
        cp_c: corner_perm_combo,
        u_edge_c: u_edge_positions,
        d_edge_c: d_edge_positions,
        e_edge_c: e_edge_positions,
    };

    (phase_1, phase_2)
}

#[cfg(test)]
mod test {
    use rand::{
        SeedableRng as _,
        distr::{Distribution, StandardUniform},
    };
    use rand_chacha::ChaCha8Rng;

    use crate::kociemba::tables::{
        lookup_sym_corner_orient::LookupSymCornerOrientTable,
        lookup_sym_corner_perm::LookupSymCornerPermTable,
        lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable,
    };

    use super::*;
    #[test]
    fn test_match_manual_moves() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let ego_lookup: &LookupSymEdgeGroupOrientTable = tables.as_ref();
        let co_lookup: &LookupSymCornerOrientTable = tables.as_ref();
        let cp_lookup: &LookupSymCornerPermTable = tables.as_ref();

        for _ in 0..100 {
            let cube: ReprCube = Distribution::sample(&StandardUniform, &mut rng);
            let (p1, p2) = produce_phase_1_phase_2(cube, &tables);

            for cube_move in CubeMove::all_iter() {
                let cube_moved = cube.apply_cube_move(cube_move);
                let (p1_expected, p2_expected) = produce_phase_1_phase_2(cube_moved, &tables);
                let p1_actual = p1.apply_cube_move(cube_move, &tables);
                let p2_actual = p2.apply_cube_move(cube_move, &tables);

                // ego
                assert_eq!(p1_actual.ego_c, p1_expected.ego_c);
                let ego_exp = ego_lookup.get_raw_from_combo(p1_expected.ego_combo());
                let ego_act = ego_lookup.get_raw_from_combo(p1_actual.ego_combo());
                assert_eq!(ego_exp, ego_act);

                // co
                assert_eq!(p1_actual.co_c, p1_expected.co_c);
                let co_exp = co_lookup.get_raw_from_combo(p1_expected.co_combo());
                let co_act = co_lookup.get_raw_from_combo(p1_actual.co_combo());
                assert_eq!(co_exp, co_act);

                // cp
                assert_eq!(p2_actual.cp_c.sym_coord, p2_expected.cp_c.sym_coord);
                let cp_exp = cp_lookup.get_raw_from_combo(p2_expected.cp_c);
                let cp_act = cp_lookup.get_raw_from_combo(p2_actual.cp_c);
                assert_eq!(cp_exp, cp_act);

                // ep
                assert_eq!(p2_actual.u_edge_c, p2_expected.u_edge_c);
                assert_eq!(p2_actual.d_edge_c, p2_expected.d_edge_c);
                assert_eq!(p2_actual.e_edge_c, p2_expected.e_edge_c);
            }
        }

        Ok(())
    }

    #[test]
    fn test_dists_agree() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        for _ in 0..100 {
            let cube: ReprCube = Distribution::sample(&StandardUniform, &mut rng);
            let (p1, _) = produce_phase_1_phase_2(cube, &tables);

            let direct = p1.dist_direct(&tables);
            let recurse = p1.dist_recurse(&tables);

            assert_eq!(direct, recurse);
        }

        Ok(())
    }

    #[test]
    fn test_begin_node_expansion() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        for _ in 0..100 {
            let cube: ReprCube = Distribution::sample(&StandardUniform, &mut rng);
            let stack = PartialStack::<10>::new(cube, &tables);
        }

        Ok(())
    }
}

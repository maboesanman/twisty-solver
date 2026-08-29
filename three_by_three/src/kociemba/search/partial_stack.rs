use crate::{
    CubeMove, ReprCube, Tables, cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry}, kociemba::{
        coords::{
            CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientRawCoord,
            EdgeGroupOrientSymCoord, corner_perm_combo_coord::CornerPermComboCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        }, partial_reprs::{
            edge_positions::{
                DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
            },
            ud_edge_perm::UDEdgePerm,
        }, search::phase_2_node::Phase2Node, tables::{
            lookup_sym_corner_orient::{CornerOrientComboCoord, CornerOrientSymCoord}, move_edge_positions::MoveEdgePositionsTable, prune_phase_1::PrunePhase1Table, prune_phase_1_double_sym_mod_3::PrunePhase1DoubleSymMod3Table,
        },
    },
};

#[derive(Clone, Debug)]
struct CombinedStack<'t, const N: usize> {
    shared_stack: SharedStack<'t, N>,
    e_stack: PartialStack<N>,
    s_stack: PartialStack<N>,
    m_stack: PartialStack<N>,
}

#[derive(Clone, Debug)]
struct SharedStack<'t, const N: usize> {
    tables: &'t Tables,
    current_position: u16,
    previous_moves: [[u8; 15]; N],
    next_pointers: [[u16; 15]; N],
    parent_indices: [u16; N],
    remaining_moves: [[u8; 15]; N],
}

#[derive(Clone, Debug)]
struct PartialStack<const N: usize> {
    base_cube: ReprCube,
    base_phase_1_coords: Phase1,
    base_phase_1_dist: u8,
    base_phase_2_coords: Phase2,

    phase_1_coords: [[Phase1; 15]; N],
    phase_1_dists: [[u8; 15]; N],

    phase_2_len: u8,
    phase_2_coords: [Phase2; N],
}

#[derive(Clone, Copy, Debug)]
struct Phase1 {
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

    fn apply_cube_move(self, cube_move: CubeMove, tables: &Tables) -> Self {
        let new_ego = self.ego_combo().apply_cube_move(tables, cube_move);
        let new_co = self.co_combo().apply_cube_move(tables, cube_move);

        Self {
            ego_c: new_ego.sym_coord,
            co_c: new_co.sym_coord,
            combo_s: (new_ego.domino_conjugation.0 << 4) | (new_co.domino_conjugation.0 & 0xF),
        }
    }

    fn dist_mod_3(&self, tables: &Tables) -> u8 {
        let prune_table: &PrunePhase1DoubleSymMod3Table = tables.as_ref();

        let co_combo = CornerOrientComboCoord {
            sym_coord: self.co_c,
            domino_conjugation: self.ego_sym().inverse().then(self.co_sym()),
        };

        prune_table.get_value(self.ego_c, co_combo)
    }

    fn dist_recurse(&self, tables: &Tables) -> u8 {
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
                let child = self.apply_cube_move(cube_move, tables);
                if child.dist_mod_3(tables) == next_mod_3 {
                    probe = child;
                    break 'a;
                }
            }
        }

        n
    }

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
struct Phase2 {
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

fn produce_phase_1_phase_2(base_cube: ReprCube, tables: &Tables) -> (Phase1, Phase2) {
    let ReprCube {
        corner_perm,
        corner_orient,
        edge_perm,
        edge_orient,
    } = base_cube;

    let (u_edge_positions, d_edge_positions, e_edge_positions) = split_edge_positions(edge_perm);

    let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
    let edge_orient_raw = edge_orient.into_coord();
    let edge_group_orient_raw_coord =
        EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

    let edge_group_orient_combo =
        EdgeGroupOrientComboCoord::from_raw(&tables, edge_group_orient_raw_coord);
    let corner_perm_combo = CornerPermComboCoord::from_raw(&tables, corner_perm.into_coord());
    let corner_orient_combo = CornerOrientComboCoord::from_raw(&tables, corner_orient.into_coord());

    let ego_c = edge_group_orient_combo.sym_coord;
    let co_c = corner_orient_combo.sym_coord;
    let combo_s = (edge_group_orient_combo.domino_conjugation.0 << 4)
        | (corner_orient_combo.domino_conjugation.0 & 0xF);

    let phase_1 = Phase1 {
        ego_c,
        co_c,
        combo_s,
    };

    let phase_2 = Phase2 {
        cp_c: corner_perm_combo,
        u_edge_c: u_edge_positions,
        d_edge_c: d_edge_positions,
        e_edge_c: e_edge_positions,
    };

    (phase_1, phase_2)
}

impl<const N: usize> PartialStack<N> {
    fn new(base_cube: ReprCube, tables: &Tables) -> Self {
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

    // fn produce_next
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
    fn match_manual_moves() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let ego_lookup: &LookupSymEdgeGroupOrientTable = tables.as_ref();
        let co_lookup: &LookupSymCornerOrientTable = tables.as_ref();
        let cp_lookup: &LookupSymCornerPermTable = tables.as_ref();

        for _ in 0..100 {
            let cube: ReprCube = Distribution::sample(&StandardUniform, &mut rng);
            let (phase_1, phase_2) = produce_phase_1_phase_2(cube, &tables);

            for cube_move in CubeMove::all_iter() {
                let cube_moved = cube.apply_cube_move(cube_move);
                let (phase_1_expected, phase_2_expected) =
                    produce_phase_1_phase_2(cube_moved, &tables);
                let phase_1_actual = phase_1.apply_cube_move(cube_move, &tables);
                let phase_2_actual = phase_2.apply_cube_move(cube_move, &tables);

                // ego
                assert_eq!(phase_1_actual.ego_c, phase_1_expected.ego_c);
                let ego_exp = ego_lookup.get_raw_from_combo(phase_1_expected.ego_combo());
                let ego_act = ego_lookup.get_raw_from_combo(phase_1_actual.ego_combo());
                assert_eq!(ego_exp, ego_act);

                // co
                assert_eq!(phase_1_actual.co_c, phase_1_expected.co_c);
                let co_exp = co_lookup.get_raw_from_combo(phase_1_expected.co_combo());
                let co_act = co_lookup.get_raw_from_combo(phase_1_actual.co_combo());
                assert_eq!(co_exp, co_act);

                // cp
                assert_eq!(
                    phase_2_actual.cp_c.sym_coord,
                    phase_2_expected.cp_c.sym_coord
                );
                let cp_exp = cp_lookup.get_raw_from_combo(phase_2_expected.cp_c);
                let cp_act = cp_lookup.get_raw_from_combo(phase_2_actual.cp_c);
                assert_eq!(cp_exp, cp_act);

                // ep
                assert_eq!(phase_2_actual.u_edge_c, phase_2_expected.u_edge_c);
                assert_eq!(phase_2_actual.d_edge_c, phase_2_expected.d_edge_c);
                assert_eq!(phase_2_actual.e_edge_c, phase_2_expected.e_edge_c);
            }
        }

        Ok(())
    }
}

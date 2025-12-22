use crate::{
    CornerOrient, CornerPerm, EdgeOrient, ReprCube, Tables, cube_ops::{cube_move::DominoMove, cube_prev_axis::CubePreviousAxis}, kociemba::{coords::{
        coords::{EEdgePermRawCoord, UDEdgePermRawCoord},
        corner_perm_combo_coord::CornerPermComboCoord,
    }, partial_reprs::{edge_positions::{EEdgePositions, combine_edge_positions}, ud_edge_perm::UDEdgePerm}, search::phase_1_node::Phase1Node}
};

pub struct Phase2Node {
    pub corner_perm_combo: CornerPermComboCoord,
    pub ud_edge_perm_raw: UDEdgePermRawCoord,
    pub e_edge_perm_raw: EEdgePermRawCoord,
}

impl Phase2Node {
    pub fn from_phase_1_node(node: Phase1Node) -> Self {
        let Phase1Node {
            corner_perm_combo,
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,

            previous_axis: _,
            corner_orient_raw: _,
            edge_group_orient_combo: _,
            skip: _,
        } = node;

        Self {
            corner_perm_combo,
            ud_edge_perm_raw: UDEdgePerm(u_edge_positions, d_edge_positions).into_coord(),
            e_edge_perm_raw: EEdgePermRawCoord(e_edge_positions.0.0 as u8),
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let UDEdgePerm(u_edge_positions, d_edge_positions) = UDEdgePerm::from_coord(self.ud_edge_perm_raw);
        let edge_perm = combine_edge_positions(
            u_edge_positions,
            d_edge_positions,
            EEdgePositions::SOLVED,
        );
        let corner_perm = CornerPerm::from_coord(self.corner_perm_combo.into_raw(tables));

        ReprCube {
            corner_perm,
            corner_orient: CornerOrient::SOLVED,
            edge_perm,
            edge_orient: EdgeOrient::SOLVED,
        }
    }

    pub fn distance_heuristic(
        self,
        tables: &Tables,
    ) -> u8 {
        let ud_edge_perm_adjusted = tables.move_raw_ud_edge_perm.domino_conjugate(
            self.ud_edge_perm_raw,
            self.corner_perm_combo.domino_conjugation,
        );
        tables.get_prune_phase_2().get_value(
            self.corner_perm_combo.sym_coord,
            ud_edge_perm_adjusted,
        )
    }

    pub fn produce_next_nodes(
        self,
        tables: &Tables,
    ) -> impl Iterator<Item = Self> {

        // perform all new axis moves on all coords
        let move_iter = || DominoMove::all_iter();

        move_iter().map(
                move |mv| {
                    Phase2Node {
                        corner_perm_combo: self.corner_perm_combo.apply_cube_move(tables, mv.into()),
                        ud_edge_perm_raw: tables.move_raw_ud_edge_perm.apply_cube_move(self.ud_edge_perm_raw, mv),
                        e_edge_perm_raw: tables.move_raw_e_edge_perm.apply_cube_move(self.e_edge_perm_raw, mv),
                    }
                },
            )
    }
}
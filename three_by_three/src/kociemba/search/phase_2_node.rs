use crate::{
    CornerOrient, CornerPerm, EdgeOrient, ReprCube, Tables,
    cube_ops::{cube_move::DominoMove, cube_prev_axis::CubePreviousAxis},
    kociemba::{
        coords::{
            coords::{EEdgePermRawCoord, UDEdgePermRawCoord},
            corner_perm_combo_coord::CornerPermComboCoord,
        },
        partial_reprs::{
            edge_positions::{EEdgePositions, combine_edge_positions},
            ud_edge_perm::UDEdgePerm,
        },
        search::phase_1_node::Phase1Node,
        tables::{
            move_raw_e_edge_perm::MoveRawEEdgePermTable,
            move_raw_ud_edge_perm::MoveRawUDEdgePermTable,
            move_sym_corner_perm::MoveSymCornerPermTable, prune_phase_2::PrunePhase2Table,
            prune_phase_2_corner_sym::PrunePhase2CornerSymTable,
        },
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Phase2Node {
    pub corner_perm_combo: CornerPermComboCoord,
    pub ud_edge_perm_raw: UDEdgePermRawCoord,
    pub e_edge_perm_raw: EEdgePermRawCoord,
    pub previous_axis: CubePreviousAxis,
}

impl Phase2Node {
    pub fn from_phase_1_node(node: Phase1Node) -> Self {
        let Phase1Node {
            corner_perm_combo,
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis,
            ..
        } = node;

        let corner_perm_combo = CornerPermComboCoord::from_dense(corner_perm_combo);

        Self {
            corner_perm_combo,
            ud_edge_perm_raw: UDEdgePerm(u_edge_positions, d_edge_positions).into_coord(),
            e_edge_perm_raw: EEdgePermRawCoord(e_edge_positions.0.0 as u8),
            previous_axis: unsafe { core::mem::transmute(previous_axis as u8) },
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let UDEdgePerm(u_edge_positions, d_edge_positions) =
            UDEdgePerm::from_coord(self.ud_edge_perm_raw);
        let edge_perm = combine_edge_positions(
            u_edge_positions,
            d_edge_positions,
            EEdgePositions::from_inner(self.e_edge_perm_raw.0 as u16),
        );
        let corner_perm = CornerPerm::from_coord(self.corner_perm_combo.into_raw(tables));

        ReprCube {
            corner_perm,
            corner_orient: CornerOrient::SOLVED,
            edge_perm,
            edge_orient: EdgeOrient::SOLVED,
        }
    }

    pub fn weak_distance_heuristic(self, table: impl AsRef<PrunePhase2CornerSymTable>) -> u8 {
        table.as_ref().get_value(self.corner_perm_combo.sym_coord)
    }

    pub fn distance_heuristic(
        self,
        tables: impl AsRef<MoveRawUDEdgePermTable> + AsRef<PrunePhase2Table>,
    ) -> u8 {
        let move_table: &MoveRawUDEdgePermTable = tables.as_ref();
        let prune_table: &PrunePhase2Table = tables.as_ref();

        let ud_edge_perm_adjusted = move_table.domino_conjugate(
            self.ud_edge_perm_raw,
            self.corner_perm_combo.domino_conjugation,
        );
        prune_table.get_value(self.corner_perm_combo.sym_coord, ud_edge_perm_adjusted)
    }

    pub fn is_solved(self) -> bool {
        self.e_edge_perm_raw.0 == 0
            && self.corner_perm_combo.sym_coord.0 == 0
            && self.ud_edge_perm_raw.0 == 0
    }

    pub fn produce_next_nodes(
        self,
        tables: &(
             impl AsRef<MoveRawUDEdgePermTable>
             + AsRef<MoveRawEEdgePermTable>
             + AsRef<MoveSymCornerPermTable>
         ),
    ) -> impl Iterator<Item = Self> {
        let move_table: &MoveRawUDEdgePermTable = tables.as_ref();
        let prune_table: &MoveRawEEdgePermTable = tables.as_ref();

        DominoMove::new_axis_iter(self.previous_axis)
            .into_iter()
            .map(move |mv| Phase2Node {
                corner_perm_combo: self.corner_perm_combo.apply_cube_move(tables, mv.into()),
                ud_edge_perm_raw: move_table.apply_cube_move(self.ud_edge_perm_raw, mv),
                e_edge_perm_raw: prune_table.apply_cube_move(self.e_edge_perm_raw, mv),
                previous_axis: self.previous_axis.update_with_new_domino_move(mv),
            })
    }
}

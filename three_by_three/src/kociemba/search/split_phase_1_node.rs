use std::num::NonZeroU8;

use crate::{
    CubeMove, EdgePerm, Tables,
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::DominoSymmetry},
    kociemba::{
        coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord},
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
        },
        search::phase_1_node::Phase1Node,
        tables::{
            move_edge_positions::MoveEdgePositionsTable,
            move_raw_corner_orient::MoveRawCornerOrientTable,
            move_sym_corner_perm_augmented::MoveSymCornerPermAugmentedTable,
            move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable,
            prune_phase_1::PrunePhase1Table,
        },
    },
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeA {
    // corners
    pub edge_group_orient_sym: EdgeGroupOrientSymCoord,

    pub edge_group_orient_correct: DominoSymmetry,
    pub corner_perm_correct: DominoSymmetry,
    pub corner_perm_raw: CornerPermSymCoord,

    // edges
    pub corner_orient_raw: CornerOrientRawCoord,
    // bookkeeping
    pub previous_axis: CubePreviousAxis,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SplitPhase1NodeB {
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

impl Default for SplitPhase1NodeA {
    fn default() -> Self {
        const {
            Self {
                edge_group_orient_sym: EdgeGroupOrientSymCoord(0),
                edge_group_orient_correct: DominoSymmetry::IDENTITY,
                corner_perm_raw: CornerPermSymCoord(0),
                corner_orient_raw: CornerOrientRawCoord(0),
                previous_axis: CubePreviousAxis::None,
                corner_perm_correct: DominoSymmetry::IDENTITY,
            }
        }
    }
}

impl Default for SplitPhase1NodeB {
    fn default() -> Self {
        const {
            let (u_edge_positions, d_edge_positions, e_edge_positions) =
                split_edge_positions(EdgePerm::SOLVED);
            Self {
                u_edge_positions,
                d_edge_positions,
                e_edge_positions,
                previous_axis: CubePreviousAxis::None,
            }
        }
    }
}

#[inline(always)]
pub fn split(node: Phase1Node) -> (SplitPhase1NodeA, SplitPhase1NodeB) {
    let Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct,
        corner_perm_correct,
        corner_perm_raw,
        corner_orient_raw,
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = node;
    (
        SplitPhase1NodeA {
            edge_group_orient_sym,
            edge_group_orient_correct,
            corner_perm_correct,
            corner_perm_raw,
            corner_orient_raw,
            previous_axis,
        },
        SplitPhase1NodeB {
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis,
        },
    )
}

#[inline(always)]
pub fn merge(a: SplitPhase1NodeA, b: SplitPhase1NodeB) -> Phase1Node {
    let SplitPhase1NodeA {
        edge_group_orient_sym,
        edge_group_orient_correct,
        corner_perm_correct,
        corner_perm_raw,
        corner_orient_raw,
        previous_axis: _,
    } = a;
    let SplitPhase1NodeB {
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    } = b;
    Phase1Node {
        edge_group_orient_sym,
        edge_group_orient_correct,
        corner_perm_correct,
        corner_perm_raw,
        corner_orient_raw,
        u_edge_positions,
        d_edge_positions,
        e_edge_positions,
        previous_axis,
    }
}

impl SplitPhase1NodeA {
    #[inline(always)]
    pub fn is_domino_reduced(self) -> bool {
        self.corner_orient_raw.0 == 0 && self.edge_group_orient_sym.0 == 0
    }

    // #[inline(always)]
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
    // #[inline(always)]
    pub fn produce_next_nodes_a(
        slice: &mut [Self; 16],
        moves_remaining: NonZeroU8,
        // tables: &(
        //      impl AsRef<MoveEdgePositionsTable>
        //      + AsRef<MoveRawCornerOrientTable>
        //      + AsRef<MoveSymCornerPermTable>
        //      + AsRef<MoveSymEdgeGroupOrientTable>
        //      + AsRef<PrunePhase1Table>
        //  ),
        tables: &Tables,
    ) -> usize {
        let start_node = slice[0];

        let ego_mv_tbl: &MoveSymEdgeGroupOrientTable = tables.as_ref();
        let cp_mv_tbl: &MoveSymCornerPermAugmentedTable = tables.as_ref();
        let co_mv_tbl: &MoveRawCornerOrientTable = tables.as_ref();

        let ego_row = ego_mv_tbl.row(start_node.edge_group_orient_sym);
        let cp_row = cp_mv_tbl.row(start_node.corner_perm_raw);
        let co_row = co_mv_tbl.row(start_node.corner_orient_raw);

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
            LOOKUP[(mv.into_index() << 4) | (start_node.edge_group_orient_correct.0 as usize)]
        });
        let cp_move_offsets = unaltered_move_offsets
            .map(|mv| LOOKUP[(mv.into_index() << 4) | (start_node.corner_perm_correct.0 as usize)]);

        for i in 0..15 {
            let ego_i = ego_move_offsets[i] as u8 as usize;
            let new_ego_coord = EdgeGroupOrientSymCoord(ego_row.coords[ego_i]);
            let new_ego_correction = DominoSymmetry(ego_row.conjugations[ego_i]);

            let cp_i = cp_move_offsets[i] as u8 as usize;
            let new_cp_coord = CornerPermSymCoord(cp_row.coords[cp_i]);
            let new_cp_correction = DominoSymmetry(cp_row.conjugations[cp_i]);

            let un_i = unaltered_move_offsets[i] as u8 as usize;
            let new_co_coord = CornerOrientRawCoord(co_row.moves[un_i]);

            let new_previous_axis = start_node
                .previous_axis
                .update_with_new_move(unaltered_move_offsets[i], moves_remaining.get() - 1);

            slice[i + 1] = SplitPhase1NodeA {
                edge_group_orient_sym: new_ego_coord,
                edge_group_orient_correct: start_node
                    .edge_group_orient_correct
                    .then(new_ego_correction),
                corner_perm_correct: start_node.corner_perm_correct.then(new_cp_correction),
                corner_perm_raw: new_cp_coord,
                corner_orient_raw: new_co_coord,
                previous_axis: new_previous_axis,
            };
        }

        num_moves
    }
}

impl SplitPhase1NodeB {
    /// Places the children from the first item in the array into the remainder of the array in place.
    /// returns the number of new children (which must be placed in the front), and the children's max possible distance.
    #[inline(always)]
    pub fn produce_next_node_b(
        self,
        moves_remaining: NonZeroU8,
        // tables: &(
        //      impl AsRef<MoveEdgePositionsTable>
        //      + AsRef<MoveRawCornerOrientTable>
        //      + AsRef<MoveSymCornerPermTable>
        //      + AsRef<MoveSymEdgeGroupOrientTable>
        //      + AsRef<PrunePhase1Table>
        //  ),
        tables: &Tables,
        i: usize,
    ) -> SplitPhase1NodeB {
        let ep_mv_tbl: &MoveEdgePositionsTable = tables.as_ref();

        let u_row = ep_mv_tbl.row(self.u_edge_positions.0);
        let d_row = ep_mv_tbl.row(self.d_edge_positions.0);
        let e_row = ep_mv_tbl.row(self.e_edge_positions.0);

        let (unaltered_move_offsets, _) = CubeMove::new_axis_move_array(self.previous_axis);

        let un_i = unaltered_move_offsets[i] as u8 as usize;
        let new_u_coord = UEdgePositions(EdgePositions(u_row.0[un_i]));
        let new_d_coord = DEdgePositions(EdgePositions(d_row.0[un_i]));
        let new_e_coord = EEdgePositions(EdgePositions(e_row.0[un_i]));

        let new_previous_axis = self
            .previous_axis
            .update_with_new_move(unaltered_move_offsets[i], moves_remaining.get() - 1);

        SplitPhase1NodeB {
            u_edge_positions: new_u_coord,
            d_edge_positions: new_d_coord,
            e_edge_positions: new_e_coord,
            previous_axis: new_previous_axis,
        }
    }
}

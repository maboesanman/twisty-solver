use crate::{
    ReprCube, Tables, cube_ops::cube_prev_axis::CubePreviousAxis, kociemba::{coords::{
        coords::{EEdgePermRawCoord, UDEdgePermRawCoord},
        corner_perm_combo_coord::CornerPermComboCoord,
    }, search::phase_1_node::Phase1Node}
};

pub struct Phase2Node {
    pub corner_perm_combo: CornerPermComboCoord,
    pub ud_edge_perm_raw: UDEdgePermRawCoord,
    pub e_edge_perm_raw: EEdgePermRawCoord,
    pub previous_axis: CubePreviousAxis,
}

impl Phase2Node {
    pub fn from_phase_1_node(node: Phase1Node, tables: &Tables) -> Self {
        let Phase1Node {
            corner_perm_combo,
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis,

            corner_orient_raw: _,
            edge_group_orient_combo: _,
            skip: _,
        } = node;

        let e_edge_perm_raw = EEdgePermRawCoord(e_edge_positions.0.0 as u8);

        todo!()
        // let ReprCube {
        //     corner_perm,
        //     corner_orient,
        //     edge_perm,
        //     edge_orient,
        // } = cube;

        // let (u_edge_positions, d_edge_positions, e_edge_positions) =
        //     split_edge_positions(edge_perm);

        // let edge_group_raw_coord = e_edge_positions.into_edge_group_raw();
        // let edge_orient_raw = edge_orient.into_coord();
        // let edge_group_orient_raw_coord =
        //     EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw);

        // let edge_group_orient_combo =
        //     EdgeGroupOrientComboCoord::from_raw(tables, edge_group_orient_raw_coord);
        // let corner_perm_combo = CornerPermComboCoord::from_raw(tables, corner_perm.into_coord());
        // let corner_orient_raw = corner_orient.into_coord();

        // Self {
        //     corner_orient_raw,
        //     corner_perm_combo,
        //     edge_group_orient_combo,
        //     u_edge_positions,
        //     d_edge_positions,
        //     e_edge_positions,
        //     previous_axis: CubePreviousAxis::None,
        //     skip: false,
        // }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        // let edge_perm = combine_edge_positions(
        //     self.u_edge_positions,
        //     self.d_edge_positions,
        //     self.e_edge_positions,
        // );
        // let corner_perm = CornerPerm::from_coord(self.corner_perm_combo.into_raw(tables));
        // let edge_orient =
        //     EdgeOrient::from_coord(self.edge_group_orient_combo.into_raw(tables).split().1);
        // let corner_orient = CornerOrient::from_coord(self.corner_orient_raw);

        // ReprCube {
        //     corner_perm,
        //     corner_orient,
        //     edge_perm,
        //     edge_orient,
        // }

        todo!()
    }

    pub fn distance_heuristic(
        self
    ) -> u8 {
        // let corner_orient_adjusted = tables.move_raw_corner_orient.domino_conjugate(
        //     self.corner_orient_raw,
        //     self.edge_group_orient_combo.domino_conjugation,
        // );
        // let distance = tables.get_prune_phase_1().get_value(
        //     self.edge_group_orient_combo.sym_coord,
        //     corner_orient_adjusted,
        // );

        todo!()
    }

    pub fn produce_next_nodes(
        self,
        tables: &Tables,
    ) -> impl Iterator<Item = Self> {

        // perform all new axis moves on all coords
        let move_iter = || CubeMove::new_axis_iter(self.previous_axis);

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
                    Phase1Node {
                        corner_orient_raw: tables
                            .move_raw_corner_orient
                            .apply_cube_move(self.corner_orient_raw, cube_move),
                        corner_perm_combo: self
                            .corner_perm_combo
                            .apply_cube_move(tables, cube_move),
                        edge_group_orient_combo: self
                            .edge_group_orient_combo
                            .apply_cube_move(tables, cube_move),
                        u_edge_positions,
                        d_edge_positions,
                        e_edge_positions,
                        previous_axis: self
                            .previous_axis
                            .update_with_new_move(cube_move, moves_remaining.get() - 1),
                        skip: false,
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
                let child_is_reduced = child.corner_orient_raw.0 == 0
                        && child.edge_group_orient_combo.sym_coord.0 == 0;

                let last_move = moves_remaining.get() == 1;
                if last_move {
                    return child_is_reduced
                }
                
                let too_close_to_be_solved = moves_remaining.get() <= 8;
                if too_close_to_be_solved {
                    return !child_is_reduced
                }

                return true
            });

        Some(Phase1FrameMetadata {
            children,
            max_possible_distance,
        })
    }
}
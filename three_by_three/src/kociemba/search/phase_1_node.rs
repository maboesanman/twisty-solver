use std::num::NonZeroU8;

use itertools::Itertools;

use crate::{
    CubeMove, ReprCube, Tables,
    cube_ops::cube_prev_axis::CubePreviousAxis,
    kociemba::{
        coords::{
            coords::{CornerOrientRawCoord, EdgeGroupOrientRawCoord},
            corner_perm_combo_coord::CornerPermComboCoord,
            edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        },
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, UEdgePositions, split_edge_positions,
        },
    },
};

#[derive(Clone, Copy)]
pub struct Phase1Node {
    // could make this fit in 16 bytes instead of 20

    // corners
    pub corner_orient_raw: CornerOrientRawCoord, // 12 bits
    corner_perm_combo: CornerPermComboCoord,     // 16 bits

    // edges
    pub edge_group_orient_combo: EdgeGroupOrientComboCoord, // 20 bits
    u_edge_positions: UEdgePositions,                       // 14 bits
    d_edge_positions: DEdgePositions,                       // 14 bits
    e_edge_positions: EEdgePositions,                       // 14 bits

    // bookkeeping
    previous_axis: CubePreviousAxis, // 4 bits

    pub skip: bool,
}

// a set of nodes which came from the same call to `produce_next_nodes`
pub struct Frame<I> {
    pub children: I,
    pub min_possible_distance: u8,
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
            corner_perm_combo,
            edge_group_orient_combo,
            u_edge_positions,
            d_edge_positions,
            e_edge_positions,
            previous_axis: CubePreviousAxis::None,
            skip: false,
        }
    }

    pub fn produce_next_nodes(
        self,
        min_possible_distance: u8,
        max_possible_distance: u8,
        moves_remaining: NonZeroU8,
        tables: &Tables,
    ) -> Option<Frame<impl Iterator<Item = Self>>> {
        if self.skip {
            return None
        }
        // there's an interesting optimization here.
        // there are no domino sequences of 7 moves or less which can be done in fewer moves when treated
        // as a non-domino. this means that if our domino reduction is ever distance 0 at two distinct points
        // within 7 moves, those moves could be replaced by the same number of domino moves.
        // now consider the last position of phase 1, which is distance 0. if we are distance 0 within 7 moves
        // of the final position, that sequence could be replaced by domino moves, which means it will not be shorter
        // than a path already found, because it there would exist a solution with a shorter phase 1 ending at the
        // first domino reduction, and staying in domino moves, likely more optimally but never longer.
        let block_domino = moves_remaining.get() < 7;
        let is_domino =
            self.corner_orient_raw.0 == 0 && self.edge_group_orient_combo.sym_coord.0 == 0;
        if block_domino && is_domino {
            return None;
        }

        // Get bounds for the current distance from self to solved, with the restriction
        // that the range must be within the allowed. If the range we have is not a subset of
        // [min_allowed_distance, max_allowed_distance], then we look up the actual distance to solved and
        // return a legal single point range or return None because we're outside the range and must be pruned.
        let (min_possible_current_distance, max_possible_current_distance) = {
            let might_be_to_far = max_possible_distance > moves_remaining.get();

            if might_be_to_far {
                let corner_orient_adjusted = tables.move_raw_corner_orient.domino_conjugate(
                    self.corner_orient_raw,
                    self.edge_group_orient_combo.domino_conjugation,
                );
                let distance = tables.get_prune_phase_1().get_value(
                    self.edge_group_orient_combo.sym_coord,
                    corner_orient_adjusted,
                );
                if distance > moves_remaining.get() {
                    return None;
                }

                (distance, distance)
            } else {
                (min_possible_distance, max_possible_distance)
            }
        };

        // prepare the values for feeding the child nodes.
        let min_possible_distance = min_possible_current_distance.saturating_sub(1);
        let max_possible_distance = max_possible_current_distance + 1;

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
                        previous_axis: self.previous_axis.update_with_new_move(cube_move),
                        skip: false
                    }
                },
            )
            .filter(move |child| {
                moves_remaining.get() != 1
                    || (child.corner_orient_raw.0 == 0
                        && child.edge_group_orient_combo.sym_coord.0 == 0)
            });

        Some(Frame {
            children,
            min_possible_distance,
            max_possible_distance,
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use std::collections::BTreeSet;
    use std::num::NonZeroU8;

    fn phase1_key(n: &Phase1Node, tables: &Tables) -> [u32; 6] {
        [
            n.corner_orient_raw.0 as u32,
            n.corner_perm_combo.into_raw(tables).0 as u32,
            n.edge_group_orient_combo.into_raw(tables).0,
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

        for mv in CubeMove::new_axis_iter(CubePreviousAxis::None) {
            let moved_cube = cube.apply_move(mv);
            let node = Phase1Node::from_cube(moved_cube, &tables);
            from_cube.insert(phase1_key(&node, &tables));
        }

        // ---- Path B: cube -> phase1 -> move ----
        let node = Phase1Node::from_cube(cube, &tables);

        let frame = node
            .produce_next_nodes(
                /* min_possible_distance = */ 0,
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
}

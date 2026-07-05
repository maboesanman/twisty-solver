use arrayvec::ArrayVec;

use crate::{
    Tables,
    kociemba::search::{capped_idastar::idastar_limited, phase_2_node::Phase2Node},
};

/// solve a phase 2 cube.
pub fn solve_domino(
    phase_2_start: Phase2Node,
    tables: &Tables,
    max_moves: u8,
) -> Option<ArrayVec<Phase2Node, 20>> {
    idastar_limited(
        phase_2_start,
        |&cube| cube.produce_next_nodes(tables).map(|c| (c, 1)),
        |&cube| cube.distance_heuristic(tables),
        |&cube| cube.is_solved(),
        max_moves,
    )
    .map(|(solution, _len)| solution)
}

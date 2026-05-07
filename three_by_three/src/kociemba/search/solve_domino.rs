use crate::{
    Tables,
    kociemba::search::{capped_idastar::idastar_limited, phase_2_node::Phase2Node},
};

/// solve a phase 2 cube.
pub fn solve_domino(
    phase_2_start: Phase2Node,
    tables: &Tables,
    max_moves: u8,
) -> Option<Vec<Phase2Node>> {
    // including this fast path showed a 10% performance improvement for the `prove_15_move_cube` benchmark
    if phase_2_start.weak_distance_heuristic(tables) > max_moves {
        return None;
    }

    idastar_limited(
        phase_2_start,
        |&cube| cube.produce_next_nodes(tables).map(|c| (c, 1)),
        |&cube| cube.distance_heuristic(tables),
        |&cube| cube.is_solved(),
        max_moves,
    )
    .map(|(solution, _len)| solution)
}

/// solve a pair of phase 2 starts. these always come in pairs since the last
/// move of a domino reduction is always F/F'/B/B'/R/R'/L/L', and each solution cancels with
/// F2/B2/R2/L2 respectively.
pub fn solve_domino_pair(
    phase_2_start_a: Phase2Node,
    phase_2_start_b: Phase2Node,
    tables: &Tables,
    max_moves: u8,
) -> Option<Vec<Phase2Node>> {
    let phase_2_a_weak_dist = phase_2_start_a.weak_distance_heuristic(tables);

    if phase_2_a_weak_dist + 1 > max_moves {
        return None;
    }

    let solution_a = idastar_limited(
        phase_2_start_a,
        |&cube| cube.produce_next_nodes(tables).map(|c| (c, 1)),
        |&cube| cube.distance_heuristic(tables),
        |&cube| cube.is_solved(),
        max_moves,
    )
    .map(|(solution, _len)| solution);

    let solution_b = idastar_limited(
        phase_2_start_b,
        |&cube| cube.produce_next_nodes(tables).map(|c| (c, 1)),
        |&cube| cube.distance_heuristic(tables),
        |&cube| cube.is_solved(),
        max_moves,
    )
    .map(|(solution, _len)| solution);

    match (solution_a, solution_b) {
        (None, None) => None,
        (None, Some(sol)) => Some(sol),
        (Some(sol), None) => Some(sol),
        (Some(sol_a), Some(sol_b)) => {
            if sol_a.len() < sol_b.len() {
                Some(sol_a)
            } else {
                Some(sol_b)
            }
        }
    }
}

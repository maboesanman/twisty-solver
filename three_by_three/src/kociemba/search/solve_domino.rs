use crate::{
    Tables,
    kociemba::search::{capped_idastar::idastar_limited, phase_2_node::Phase2Node},
};

pub fn solve_domino(
    phase_2_start: Phase2Node,
    tables: &Tables,
    max_moves: u8,
) -> Option<Vec<Phase2Node>> {
    let phase_2_prune = phase_2_start.distance_heuristic(tables);
    if phase_2_prune > max_moves {
        return None;
    }

    let solution = idastar_limited(
        phase_2_start,
        |&cube| {
            cube.produce_next_nodes(tables)
                .into_iter()
                .map(|c| (c, 1))
        },
        |&cube| cube.distance_heuristic(tables),
        |&cube| cube.is_solved(),
        max_moves,
    )
    .map(|(solution, _len)| solution)?;

    debug_assert!(phase_2_prune as usize >= solution.len() - 1);

    Some(solution)
}

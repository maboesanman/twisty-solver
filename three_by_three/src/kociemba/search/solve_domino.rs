use crate::{
    Tables,
    kociemba::{coords::repr_coord::SymReducedReprPhase2, search::capped_idastar::idastar_limited},
};

pub fn solve_domino(
    phase_2_start: SymReducedReprPhase2,
    tables: &Tables,
    max_moves: u8,
) -> Option<Vec<SymReducedReprPhase2>> {
    let phase_2_prune = phase_2_start.prune_distance_phase_2(tables);
    if phase_2_prune > max_moves {
        return None;
    }

    idastar_limited(
        phase_2_start,
        |&cube| {
            cube.full_phase_2_neighbors(tables)
                .into_iter()
                .map(move |c| (c, 1))
        },
        |&cube| cube.prune_distance_phase_2(tables),
        |&cube| cube.is_solved(),
        max_moves,
    )
    .map(|(solution, _len)| solution)
}

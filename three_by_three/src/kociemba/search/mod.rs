mod capped_idastar;
mod domino_reduction_iter;
mod move_resolver;
mod phase_1_node;
mod phase_2_node;
mod solve_domino;
mod solve_with_fixed_len_phase_1;
mod stream_search;

// pub use stream_search::get_incremental_solutions_stream;

// #[cfg(test)]
// mod test {
//     use crate::{Tables, kociemba::coords::repr_coord::SymReducedReprPhase2};

//     #[ignore]
//     #[test]
//     fn domino_optimality_check() -> anyhow::Result<()> {
//         let tables = Box::leak(Box::new(Tables::new("tables")?));

//         let items = pathfinding::directed::bfs::bfs(
//             &SymReducedReprPhase2([0, 0]),
//             |&cube| cube.full_phase_2_neighbors(tables),
//             |&cube| !cube.domino_is_optimal(tables),
//         )
//         .unwrap();

//         println!("MIN_LENGTH_BEFORE_SUB_OPTIMAL: {}", items.len());

//         Ok(())
//     }
// }

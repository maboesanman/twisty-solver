mod capped_idastar;
mod domino_reduction_iter;
mod solve_with_fixed_len_phase_1;
mod stream_search;
mod move_resolver;
mod solve_domino;

pub use stream_search::get_incremental_solutions_stream;

#[cfg(test)]
mod test {
    use pathfinding::directed::idastar;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::{
        ReprCube, Tables,
        kociemba::{
            coords::repr_coord::{SymReducedRepr, SymReducedReprPhase2},
            search::stream_search,
        },
    };

    #[test]
    fn random_phase_2_optimality() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));

        let items = pathfinding::directed::bfs::bfs(
            &SymReducedReprPhase2([0, 0]),
            |&cube| cube.full_phase_2_neighbors(tables),
            |&cube| !cube.domino_is_optimal(tables),
        )
        .unwrap();

        println!("MIN_LENGTH_BEFORE_SUB_OPTIMAL: {}", items.len());

        Ok(())
    }
}

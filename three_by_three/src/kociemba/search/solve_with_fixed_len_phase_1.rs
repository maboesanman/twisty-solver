use std::sync::atomic::{AtomicBool, AtomicU8};

use rayon::iter::ParallelIterator;

use crate::{
    Tables,
    cube_ops::{cube_move::CubeMove, repr_cube::ReprCube},
    kociemba::search::{
        move_resolver::move_resolver_multi_dimension_domino, phase_1_node::TableOffsets,
        solve_domino::solve_domino_pair,
    },
};

/// produce all solutions with phase 1 solutions of length N
pub fn produce_solutions<'t, const N: usize>(
    cube: ReprCube,
    current_best: u8,
    tables: &'t Tables,
    table_offsets: &'t TableOffsets,
) -> impl 't + Iterator<Item = Vec<CubeMove>> {
    let domino_reductions =
        super::domino_reduction_iter::all_domino_reductions::<N>(cube, tables, table_offsets);

    domino_reductions
        .scan(
            current_best,
            |current_best, (phase_1, phase_2_start_a, phase_2_start_b)| {
                let phase_2_max = *current_best - N as u8;

                let Some(phase_2) = solve_domino_pair(
                    phase_2_start_a,
                    phase_2_start_b,
                    tables,
                    phase_2_max,
                    || Some(*current_best - N as u8),
                ) else {
                    return Some(None);
                };

                *current_best = (N + phase_2.len() - 1) as u8;
                Some(Some((phase_1, phase_2)))
            },
        )
        .flatten()
        .map(move |(phase_1, phase_2)| {
            let phase_1 = phase_1.into_iter().map(|node| node.into_cube(tables));
            let phase_2 = phase_2.into_iter().map(|node| node.into_cube(tables));
            move_resolver_multi_dimension_domino(cube, phase_1.chain(phase_2))
        })
}

// Thread-local storage for the last seen value
thread_local! {
    static THREAD_LOCAL_BEST: std::cell::Cell<u8> = const { std::cell::Cell::new(u8::MAX) }
}

/// produce all solutions with phase 1 solutions of length N in parallel
pub fn produce_solutions_par<'a, const N: usize>(
    cube: ReprCube,
    best: &'a AtomicU8,
    tables: &'a Tables,
    table_offsets: &'a TableOffsets,
    cancel: &'a AtomicBool,
) -> impl 'a + ParallelIterator<Item = Vec<CubeMove>> {
    let domino_reductions = super::domino_reduction_iter::all_domino_reductions_par::<N>(
        cube,
        tables,
        table_offsets,
        cancel,
    );

    domino_reductions
        .filter_map(|(phase_1, phase_2_start_a, phase_2_start_b)| {
            let local_best = THREAD_LOCAL_BEST.with(|tl| tl.get());
            let local_phase_2_max = local_best.checked_sub(N as u8)?;

            let phase_2 = solve_domino_pair(
                phase_2_start_a,
                phase_2_start_b,
                tables,
                local_phase_2_max,
                || {
                    let current_best = best.load(std::sync::atomic::Ordering::Relaxed);
                    THREAD_LOCAL_BEST.replace(current_best);
                    Some(current_best.checked_sub(N as u8)?)
                },
            )?;
            let new_path_len = (N + phase_2.len() - 1) as u8;

            // println!("")
            let old = best.fetch_min(new_path_len, std::sync::atomic::Ordering::AcqRel);
            if new_path_len >= old {
                return None;
            }

            Some((phase_1, phase_2))
        })
        .map(move |(phase_1, phase_2)| {
            let phase_1 = phase_1.into_iter().map(|node| node.into_cube(tables));
            let phase_2 = phase_2.into_iter().map(|node| node.into_cube(tables));
            move_resolver_multi_dimension_domino(cube, phase_1.chain(phase_2))
        })
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use crate::cube;

    use super::*;

    #[test]
    fn solve_combined_test_superflip_magic_s() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_offsets = TableOffsets::new(&tables);

        let solutions = produce_solutions::<10>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            u8::MAX,
            &tables,
            &table_offsets,
        );

        for solution in solutions {
            print!("{:02} ", solution.len());
            for m in solution {
                print!("{m} ");
            }
            println!("")
        }

        Ok(())
    }

    #[test]
    fn solve_combined_test_superflip_magic_par() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_offsets = TableOffsets::new(&tables);

        let best = AtomicU8::new(u8::MAX);
        let cancel = AtomicBool::new(false);

        let solutions = produce_solutions_par::<10>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &best,
            &tables,
            &table_offsets,
            &cancel,
        );

        let block = Mutex::new(());

        solutions.for_each(|solution| {
            let lock = block.lock();
            print!("{:02} ", solution.len());
            for m in solution {
                print!("{m} ");
            }
            println!("");
            drop(lock);
        });

        Ok(())
    }
}

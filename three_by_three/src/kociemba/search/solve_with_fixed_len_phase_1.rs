use std::sync::atomic::{AtomicBool, AtomicUsize};

use rayon::iter::ParallelIterator;

use crate::{
    cube_ops::{cube_move::CubeMove, repr_cube::ReprCube},
    kociemba::{
        coords::repr_coord::SymReducedReprPhase2,
        search::{move_resolver::move_resolver_phase_1, solve_domino::solve_domino},
    },
    tables::Tables,
};

#[allow(unused)]
pub fn produce_solutions<const N: usize>(
    cube: ReprCube,
    current_best: usize,
    tables: &Tables,
) -> impl Iterator<Item = Vec<CubeMove>> {
    let domino_reductions =
        super::domino_reduction_iter::all_domino_reductions::<N>(cube, tables);

    domino_reductions
        .scan(current_best, |current_best, (start, phase_1_end)| {
            let phase_2_start = SymReducedReprPhase2([phase_1_end.0[2], phase_1_end.0[3]]);
            let phase_2_max = *current_best - (N + 1);

            let solution = {
                let solution = solve_domino(phase_2_start, tables, phase_2_max as u8)?;
                *current_best = N + solution.len();
                Some((start, end, solution))
            };

            Some(solution)
        })
        .flatten()
        .map(move_resolver_phase_1(cube, tables))
}

pub fn produce_solutions_par<'a, const N: usize>(
    cube: ReprCube,
    best: &'a AtomicUsize,
    tables: &'a Tables,
    cancel: &'a AtomicBool,
) -> impl 'a + ParallelIterator<Item = Vec<CubeMove>> {
    let domino_reductions =
        super::domino_reduction_iter::all_domino_reductions_par::<N>(cube, tables, cancel);

    domino_reductions
        .filter_map(|(start, phase_1_end)| {
            let phase_2_start = SymReducedReprPhase2([phase_1_end.0[2], phase_1_end.0[3]]);
            let curr_best = best.load(std::sync::atomic::Ordering::Relaxed);
            let phase_2_allowed = curr_best.checked_sub(N + 1)? as u8;

            let phase_2_path = solve_domino(phase_2_start, tables, phase_2_allowed)?;
            let new_path_len = N + phase_2_path.len();

            best.compare_exchange(
                curr_best,
                new_path_len,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .ok()?;

            Some((start, end, phase_2_path))
        })
        .map(move_resolver_phase_1(cube, tables))
}

#[cfg(test)]
mod test {
    use std::{sync::Mutex, usize};

    use crate::cube;

    use super::*;

    #[test]
    fn solve_combined_test_superflip_magic() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let solutions = produce_solutions::<10>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            usize::MAX,
            &tables,
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

        let best = AtomicUsize::new(usize::MAX);
        let cancel = AtomicBool::new(false);

        let solutions = produce_solutions_par::<10>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &best,
            &tables,
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

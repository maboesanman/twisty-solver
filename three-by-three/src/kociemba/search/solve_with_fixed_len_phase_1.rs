use std::sync::atomic::{AtomicBool, AtomicUsize};

use rayon::iter::ParallelIterator;

use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry, repr_cube::ReprCube},
    kociemba::search::capped_idastar::idastar_limited,
    tables::Tables,
};

pub fn produce_solutions<const N: usize, const S: bool>(
    cube: ReprCube,
    current_best: usize,
    tables: &Tables,
) -> impl Iterator<Item = Vec<CubeMove>> {
    let domino_reductions = super::domino_reduction_iter::all_domino_reductions::<N, S>(cube, tables);

    domino_reductions
        .scan(current_best, |current_best, (start, end)| {
            let phase_2_start = end.last().copied().unwrap_or(start[1]);
            let phase_2_prune = phase_2_start.prune_distance_phase_2(tables);
            let phase_2_allowed = (*current_best - (N + 1)) as u8;
            if phase_2_prune > phase_2_allowed {
                return Some(None);
            }
            let (phase_2_path, phase_2_len) = match idastar_limited(
                phase_2_start,
                |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
                |&cube| cube.prune_distance_phase_2(tables),
                |&cube| cube.is_solved(),
                phase_2_allowed,
            ) {
                Some(path) => path,
                None => return Some(None),
            };

            let new_path_len = (N + 1) + phase_2_len as usize;

            if new_path_len < *current_best {
                *current_best = new_path_len;
                Some(Some((start, end, phase_2_path)))
            } else {
                Some(None)
            }
        })
        .flatten()
        .map(move |(a, b, c)| {
            let mut moves = vec![];
            let mut last = cube;

            for solve_c in a[1..]
                .iter()
                .chain(b.iter())
                .chain(c[1..].iter())
                .map(|c| c.into_cube(tables))
            {
                let (_, l, mv) = match CubeMove::all_iter()
                    .flat_map(|mv| {
                        let next = last.apply_cube_move(mv);
                        DominoSymmetry::all_iter()
                            .map(move |s| (next.domino_conjugate(s), next, mv))
                    })
                    .find(|(c, _, _)| *c == solve_c)
                {
                    Some(a) => a,
                    None => panic!(),
                };

                last = l;
                moves.push(mv);
            }

            moves
        })
}

pub fn produce_solutions_par<'a, const N: usize, const S: bool>(
    cube: ReprCube,
    best: &'a AtomicUsize,
    tables: &'a Tables,
    cancel: &'a AtomicBool,
) -> impl 'a + ParallelIterator<Item = Vec<CubeMove>> {
    let domino_reductions =
        super::domino_reduction_iter::all_domino_reductions_par::<N, S>(cube, tables, cancel);

    domino_reductions
        .filter_map(|(start, end)| {
            let phase_2_start = end.last().copied().unwrap_or(start[1]);
            let phase_2_prune = phase_2_start.prune_distance_phase_2(tables);
            let curr_best = best.load(std::sync::atomic::Ordering::Relaxed);
            let phase_2_allowed = match curr_best.checked_sub(N + 1) {
                Some(x) => x as u8,
                None => return None,
            };
            if phase_2_prune > phase_2_allowed {
                return None;
            }
            let (phase_2_path, phase_2_len) = match idastar_limited(
                phase_2_start,
                |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
                |&cube| cube.prune_distance_phase_2(tables),
                |&cube| cube.is_solved(),
                phase_2_allowed,
            ) {
                Some(path) => path,
                None => return None,
            };

            let new_path_len = (N + 1) + phase_2_len as usize;

            if curr_best <= new_path_len {
                return None;
            }

            match best.compare_exchange(
                curr_best,
                new_path_len,
                std::sync::atomic::Ordering::Relaxed, // or Release if publishing data
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => Some((start, end, phase_2_path)), // we won the race
                Err(_) => None, // lost, someone else wrote a smaller value
            }
        })
        .map(move |(a, b, c)| {
            let mut moves = vec![];
            let mut last = cube;

            for solve_c in a[1..]
                .iter()
                .chain(b.iter())
                .chain(c[1..].iter())
                .map(|c| c.into_cube(tables))
            {
                let (_, l, mv) = match CubeMove::all_iter()
                    .flat_map(|mv| {
                        let next = last.apply_cube_move(mv);
                        DominoSymmetry::all_iter()
                            .map(move |s| (next.domino_conjugate(s), next, mv))
                    })
                    .find(|(c, _, _)| *c == solve_c)
                {
                    Some(a) => a,
                    None => panic!(),
                };

                last = l;
                moves.push(mv);
            }

            moves
        })
}

#[cfg(test)]
mod test {
    use std::{sync::Mutex, usize};

    use crate::cube;

    use super::*;

    #[test]
    fn solve_combined_test_superflip_magic() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let solutions = produce_solutions::<9, false>(
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

        let solutions = produce_solutions_par::<10, true>(
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

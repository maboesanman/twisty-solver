use std::sync::atomic::AtomicU8;

use rayon::iter::ParallelIterator;

use crate::{
    Tables,
    cube_ops::{cube_move::CubeMove, repr_cube::ReprCube},
    kociemba::search::{
        move_resolver::move_resolver_multi_dimension_domino, solve_domino::solve_domino,
    },
};

/// produce all solutions with phase 1 solutions of length N
pub fn produce_solutions<'t, const N: usize>(
    cube: ReprCube,
    current_best: &'t mut u8,
    target: u8,
    tables: &'t Tables,
    axes: &[u8],
) -> impl 't + Iterator<Item = Vec<CubeMove>> {
    let domino_reductions = super::domino_reduction_iter::all_domino_reductions::<N>(
        cube,
        tables,
        axes,
        current_best as *const _,
        target,
    );

    let mut_ptr_best = current_best as *mut u8;
    domino_reductions
        .scan((), move |_, (phase_1, phase_2_start)| {
            let phase_2_max = unsafe { *mut_ptr_best } - N as u8;

            let Some(phase_2) = solve_domino(phase_2_start, tables, phase_2_max) else {
                return Some(None);
            };

            unsafe { *mut_ptr_best = (N + phase_2.len() - 1) as u8 };
            Some(Some((phase_1, phase_2)))
        })
        .flatten()
        .map(move |(phase_1, phase_2)| {
            let phase_1 = phase_1.into_iter().map(|node| node.into_cube(tables));
            let phase_2 = phase_2.into_iter().map(|node| node.into_cube(tables));
            move_resolver_multi_dimension_domino(cube, phase_1.chain(phase_2))
        })
}

// // Thread-local storage for the last seen value
// thread_local! {
//     static THREAD_LOCAL_BEST: std::cell::Cell<u8> = const { std::cell::Cell::new(u8::MAX) }
// }

/// produce all solutions with phase 1 solutions of length N in parallel
pub fn produce_solutions_par<'a, const N: usize>(
    cube: ReprCube,
    best: &'a AtomicU8,
    target: u8,
    tables: &'a Tables,
    axes: &[u8],
) -> impl 'a + ParallelIterator<Item = Vec<CubeMove>> {
    let domino_reductions = super::domino_reduction_iter::all_domino_reductions_par::<N>(
        cube, tables, axes, best, target,
    );

    domino_reductions
        .filter_map(|(phase_1, phase_2_start)| {
            let phase_2_max = {
                let current_best = best.load(std::sync::atomic::Ordering::Relaxed);
                // THREAD_LOCAL_BEST.replace(current_best);
                current_best.checked_sub(N as u8)
            }?;

            let phase_2 = solve_domino(phase_2_start, tables, phase_2_max)?;
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
        let mut best = u8::MAX;
        let solutions = produce_solutions::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &mut best,
            0,
            &tables,
            &[0],
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

        let best = AtomicU8::new(u8::MAX);

        let solutions = produce_solutions_par::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &best,
            0,
            &tables,
            &[0],
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

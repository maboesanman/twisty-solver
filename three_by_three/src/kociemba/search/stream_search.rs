use std::{
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8},
    },
    thread::JoinHandle,
};

use flume::{Sender, r#async::RecvStream};
use futures::{StreamExt, future};
use futures_core::Stream;
use rayon::iter::ParallelIterator;

use crate::{
    cube_ops::{cube_move::CubeMove, repr_cube::ReprCube},
    kociemba::{
        search::solve_with_fixed_len_phase_1::{produce_solutions, produce_solutions_par},
        tables::Tables,
    },
};

fn solver_thread_single(
    cube: ReprCube,
    tables: &Tables,
    send: Sender<Vec<CubeMove>>,
    target: u8,
    seed_best: u8,
) {
    let mut best = seed_best;

    if cube == ReprCube::SOLVED {
        let _ = send.send(Vec::new());
        return;
    }

    if best <= 1 {
        return;
    }
    produce_solutions::<1>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 2 + 1 {
        return;
    }
    produce_solutions::<2>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 3 + 1 {
        return;
    }
    produce_solutions::<3>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 4 + 1 {
        return;
    }
    produce_solutions::<4>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 5 + 1 {
        return;
    }
    produce_solutions::<5>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 6 + 1 {
        return;
    }
    produce_solutions::<6>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 7 + 1 {
        return;
    }
    produce_solutions::<7>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 8 + 1 {
        return;
    }
    produce_solutions::<8>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 9 + 1 {
        return;
    }
    produce_solutions::<9>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 10 + 1 {
        return;
    }
    produce_solutions::<10>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 11 + 1 {
        return;
    }
    produce_solutions::<11>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 12 + 1 {
        return;
    }
    produce_solutions::<12>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 13 + 1 {
        return;
    }
    produce_solutions::<13>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 14 + 1 {
        return;
    }
    produce_solutions::<14>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 15 + 1 {
        return;
    }
    produce_solutions::<15>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 16 + 1 {
        return;
    }
    produce_solutions::<16>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 17 + 1 {
        return;
    }
    produce_solutions::<17>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 18 + 1 {
        return;
    }
    produce_solutions::<18>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 19 + 1 {
        return;
    }
    produce_solutions::<19>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if best <= 20 + 1 {
        return;
    }
    produce_solutions::<20>(cube, &mut best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });
}

fn solver_thread_parallel(
    cube: ReprCube,
    tables: &Tables,
    send: Sender<Vec<CubeMove>>,
    target: u8,
    seed_best: u8,
) {
    let mut best = AtomicU8::new(seed_best);

    if cube == ReprCube::SOLVED {
        let _ = send.send(Vec::new());
        return;
    }

    if *(best.get_mut()) <= 1 {
        return;
    }
    produce_solutions_par::<1>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 2 + 1 {
        return;
    }
    produce_solutions_par::<2>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 3 + 1 {
        return;
    }
    produce_solutions_par::<3>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 4 + 1 {
        return;
    }
    produce_solutions_par::<4>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 5 + 1 {
        return;
    }
    produce_solutions_par::<5>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 6 + 1 {
        return;
    }
    produce_solutions_par::<6>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 7 + 1 {
        return;
    }
    produce_solutions_par::<7>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 8 + 1 {
        return;
    }
    produce_solutions_par::<8>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 9 + 1 {
        return;
    }
    produce_solutions_par::<9>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 10 + 1 {
        return;
    }
    produce_solutions_par::<10>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 11 + 1 {
        return;
    }
    produce_solutions_par::<11>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 12 + 1 {
        return;
    }
    produce_solutions_par::<12>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 13 + 1 {
        return;
    }
    produce_solutions_par::<13>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 14 + 1 {
        return;
    }
    produce_solutions_par::<14>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 15 + 1 {
        return;
    }
    produce_solutions_par::<15>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 16 + 1 {
        return;
    }
    produce_solutions_par::<16>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 17 + 1 {
        return;
    }
    produce_solutions_par::<17>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 18 + 1 {
        return;
    }
    produce_solutions_par::<18>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 19 + 1 {
        return;
    }
    produce_solutions_par::<19>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 20 + 1 {
        return;
    }
    produce_solutions_par::<20>(cube, &best, target, tables, &[0, 1, 2]).for_each(|solution| {
        let _ = send.send(solution);
    });
}

pub fn get_incremental_solutions_stream(
    cube: ReprCube,
    tables: &'static Tables,
    max_moves: Option<u8>,
    target: u8,
    parallel: bool,
) -> impl Stream<Item = Vec<CubeMove>> {
    let (send, recv) = flume::unbounded();
    let cancel = Arc::new(AtomicBool::new(false));
    let join_handle = std::thread::spawn(move || {
        let send = send;
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            if parallel {
                solver_thread_parallel(
                    cube,
                    tables,
                    send,
                    target,
                    max_moves.unwrap_or(u8::MAX).saturating_add(1),
                );
            } else {
                solver_thread_single(
                    cube,
                    tables,
                    send,
                    target,
                    max_moves.unwrap_or(u8::MAX).saturating_add(1),
                );
            }
        }));

        if let Err(err) = result {
            eprintln!("[worker] panicked: {:?}", err);
            std::panic::resume_unwind(err);
        }
    });

    ImprovingSolutionStream {
        recv: recv.into_stream(),
        cancel,
        join_handle: Some(join_handle),
    }
    .scan(usize::MAX, |best_len, solution: Vec<CubeMove>| {
        let emit = if solution.len() < *best_len {
            *best_len = solution.len();
            Some(solution)
        } else {
            None
        };
        future::ready(Some(emit))
    })
    .filter_map(future::ready)
}

struct ImprovingSolutionStream<'a> {
    recv: RecvStream<'a, Vec<CubeMove>>,
    cancel: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl<'a> Drop for ImprovingSolutionStream<'a> {
    fn drop(&mut self) {
        self.cancel
            .store(true, std::sync::atomic::Ordering::Release);
        self.join_handle.take().unwrap().join().unwrap();
    }
}

impl<'a> Stream for ImprovingSolutionStream<'a> {
    type Item = Vec<CubeMove>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().recv) }.poll_next(cx)
    }
}

pub fn solve_direct(cube: ReprCube, tables: &Tables) -> Vec<CubeMove> {
    let phase_1 = super::phase_1_node::Phase1Node::from_cube(cube, tables);
    let domino_dist = phase_1.distance_heuristic(tables);

    match domino_dist {
        0 => super::solve_with_fixed_len_phase_1::produce_solutions::<0>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        1 => super::solve_with_fixed_len_phase_1::produce_solutions::<1>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        2 => super::solve_with_fixed_len_phase_1::produce_solutions::<2>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        3 => super::solve_with_fixed_len_phase_1::produce_solutions::<3>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        4 => super::solve_with_fixed_len_phase_1::produce_solutions::<4>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        5 => super::solve_with_fixed_len_phase_1::produce_solutions::<5>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        6 => super::solve_with_fixed_len_phase_1::produce_solutions::<6>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        7 => super::solve_with_fixed_len_phase_1::produce_solutions::<7>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        8 => super::solve_with_fixed_len_phase_1::produce_solutions::<8>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        9 => super::solve_with_fixed_len_phase_1::produce_solutions::<9>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        10 => super::solve_with_fixed_len_phase_1::produce_solutions::<10>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        11 => super::solve_with_fixed_len_phase_1::produce_solutions::<11>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        12 => super::solve_with_fixed_len_phase_1::produce_solutions::<12>(
            cube,
            &mut 255,
            255,
            tables,
            &[0],
        )
        .next()
        .unwrap(),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    extern crate test;

    use super::*;
    use crate::cube;

    #[test]
    fn test_stream_superflip() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));

        let mut stream = get_incremental_solutions_stream(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            tables,
            Some(20),
            20,
            true,
        );

        let future = stream.next();

        let solution = futures::executor::block_on(future).unwrap();
        print!("{:02} ", solution.len());
        for m in solution {
            print!("{m} ");
        }
        println!("");

        Ok(())
    }

    #[test]
    fn test_stream_simple() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));

        let stream = get_incremental_solutions_stream(
            cube![R U Rp Up R U Rp Up R U Rp Up ],
            tables,
            Some(20),
            20,
            false,
        );

        for solution in futures::executor::block_on_stream(stream) {
            print!("{:02} ", solution.len());
            for m in solution {
                print!("{m} ");
            }
            println!("");
        }

        Ok(())
    }

    #[test]
    fn test_stream_random() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let mut rng = ChaCha8Rng::seed_from_u64(1);

        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        let cube = cube
            .apply_cube_move(CubeMove::D2)
            .apply_cube_move(CubeMove::F3);
        cube.pretty_print();
        let stream = get_incremental_solutions_stream(cube, tables, None, 0, true);

        for solution in futures::executor::block_on_stream(stream) {
            print!("{:02} ", solution.len());
            for m in solution.into_iter().rev() {
                let m = match m {
                    CubeMove::U1 => CubeMove::U3,
                    CubeMove::U2 => CubeMove::U2,
                    CubeMove::U3 => CubeMove::U1,
                    CubeMove::D1 => CubeMove::D3,
                    CubeMove::D2 => CubeMove::D2,
                    CubeMove::D3 => CubeMove::D1,
                    CubeMove::F1 => CubeMove::F3,
                    CubeMove::F2 => CubeMove::F2,
                    CubeMove::F3 => CubeMove::F1,
                    CubeMove::B1 => CubeMove::B3,
                    CubeMove::B2 => CubeMove::B2,
                    CubeMove::B3 => CubeMove::B1,
                    CubeMove::R1 => CubeMove::R3,
                    CubeMove::R2 => CubeMove::R2,
                    CubeMove::R3 => CubeMove::R1,
                    CubeMove::L1 => CubeMove::L3,
                    CubeMove::L2 => CubeMove::L2,
                    CubeMove::L3 => CubeMove::L1,
                };
                print!("{m} ");
            }
            println!("");
        }

        Ok(())
    }

    #[test]
    fn gen_scrambles() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        for _ in 0..100 {
            let cube: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

            cube.pretty_print();

            let mut stream = get_incremental_solutions_stream(cube, tables, Some(20), 20, false);
            let future = stream.next();
            // assert!(futures::executor::block_on(future).is_some());

            let solution = futures::executor::block_on(future).unwrap();
            {
                print!("{:02} ", solution.len());
                for m in solution.into_iter().rev() {
                    let m = match m {
                        CubeMove::U1 => CubeMove::U3,
                        CubeMove::U2 => CubeMove::U2,
                        CubeMove::U3 => CubeMove::U1,
                        CubeMove::D1 => CubeMove::D3,
                        CubeMove::D2 => CubeMove::D2,
                        CubeMove::D3 => CubeMove::D1,
                        CubeMove::F1 => CubeMove::F3,
                        CubeMove::F2 => CubeMove::F2,
                        CubeMove::F3 => CubeMove::F1,
                        CubeMove::B1 => CubeMove::B3,
                        CubeMove::B2 => CubeMove::B2,
                        CubeMove::B3 => CubeMove::B1,
                        CubeMove::R1 => CubeMove::R3,
                        CubeMove::R2 => CubeMove::R2,
                        CubeMove::R3 => CubeMove::R1,
                        CubeMove::L1 => CubeMove::L3,
                        CubeMove::L2 => CubeMove::L2,
                        CubeMove::L3 => CubeMove::L1,
                    };
                    print!("{m} ");
                }
                println!("");
            }
        }

        Ok(())
    }

    #[bench]
    fn solve_a_cube_in_20_moves(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let cubes: Vec<ReprCube> = (0..10000)
            .into_iter()
            .map(|_| rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng))
            .collect_vec();

        let mut i = 0;

        bench.iter(|| {
            let cube = cubes.get(i % 10000).unwrap();
            i += 1;
            let mut stream = get_incremental_solutions_stream(*cube, tables, Some(20), 20, true);
            let future = stream.next();
            let solution = futures::executor::block_on(future).unwrap();
            test::black_box(solution);
        });
    }

    #[bench]
    fn solve_a_cube_in_20_moves_single(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let cubes: Vec<ReprCube> = (0..10000)
            .into_iter()
            .map(|_| rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng))
            .collect_vec();

        let mut i = 0;

        bench.iter(|| {
            let cube = cubes.get(i % 10000).unwrap();
            i += 1;
            let mut stream = get_incremental_solutions_stream(*cube, tables, Some(20), 20, false);
            let future = stream.next();
            let solution = futures::executor::block_on(future).unwrap();
            test::black_box(solution);
        });
    }

    #[bench]
    fn solve_a_cube_in_21_moves(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let cubes: Vec<ReprCube> = (0..10000)
            .into_iter()
            .map(|_| rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng))
            .collect_vec();

        let mut i = 0;

        bench.iter(|| {
            let cube = cubes.get(i % 10000).unwrap();
            i += 1;
            let mut stream = get_incremental_solutions_stream(*cube, tables, Some(21), 21, true);
            let future = stream.next();
            let solution = futures::executor::block_on(future).unwrap();
            test::black_box(solution);
        });
    }

    #[bench]
    fn solve_a_cube_in_22_moves(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let cubes: Vec<ReprCube> = (0..10000)
            .into_iter()
            .map(|_| rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng))
            .collect_vec();

        let mut i = 0;

        bench.iter(|| {
            let cube = cubes.get(i % 10000).unwrap();
            i += 1;
            let mut stream = get_incremental_solutions_stream(*cube, tables, Some(22), 22, true);
            let future = stream.next();
            let solution = futures::executor::block_on(future).unwrap();
            test::black_box(solution);
        });
    }

    #[bench]
    fn solve_a_cube_in_any_moves(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let cubes: Vec<ReprCube> = (0..10000)
            .into_iter()
            .map(|_| rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng))
            .collect_vec();

        let mut i = 0;

        bench.iter(|| {
            let cube = cubes.get(i % 10000).unwrap();
            i += 1;
            let solution = solve_direct(*cube, tables);
            test::black_box(solution);
        });
    }

    #[test]
    fn solve_a_cube_in_any_moves_test() {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        (0..100)
            .into_iter()
            .map(|_| rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng))
            .skip(4)
            .for_each(|cube: ReprCube| {
                cube.pretty_print();
                let solution = solve_direct(cube, tables);
                for m in solution.into_iter().rev() {
                    let m = match m {
                        CubeMove::U1 => CubeMove::U3,
                        CubeMove::U2 => CubeMove::U2,
                        CubeMove::U3 => CubeMove::U1,
                        CubeMove::D1 => CubeMove::D3,
                        CubeMove::D2 => CubeMove::D2,
                        CubeMove::D3 => CubeMove::D1,
                        CubeMove::F1 => CubeMove::F3,
                        CubeMove::F2 => CubeMove::F2,
                        CubeMove::F3 => CubeMove::F1,
                        CubeMove::B1 => CubeMove::B3,
                        CubeMove::B2 => CubeMove::B2,
                        CubeMove::B3 => CubeMove::B1,
                        CubeMove::R1 => CubeMove::R3,
                        CubeMove::R2 => CubeMove::R2,
                        CubeMove::R3 => CubeMove::R1,
                        CubeMove::L1 => CubeMove::L3,
                        CubeMove::L2 => CubeMove::L2,
                        CubeMove::L3 => CubeMove::L1,
                    };
                    print!("{m} ");
                }
                println!();
            });
    }

    #[bench]
    fn bench_20_move_superflip(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));

        let cube = cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2];
        bench.iter(|| {
            let mut stream = get_incremental_solutions_stream(cube, tables, Some(20), 20, true);
            let future = stream.next();
            let solution = futures::executor::block_on(future).unwrap();
            test::black_box(solution);
        });
    }

    #[bench]
    fn prove_15_move_cube(bench: &mut test::Bencher) {
        let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
        let mut rng = ChaCha8Rng::seed_from_u64(1);

        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        let cube = cube
            .apply_cube_move(CubeMove::D2)
            .apply_cube_move(CubeMove::F3);

        bench.iter(|| {
            let stream = get_incremental_solutions_stream(cube, tables, None, 0, true);
            let opt_solution = futures::executor::block_on_stream(stream).last();
            test::black_box(opt_solution);
        });
    }

    #[test]
    fn test_already_solved() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));

        let mut stream =
            get_incremental_solutions_stream(ReprCube::SOLVED, tables, Some(20), 0, true);
        let future = stream.next();
        let solution = futures::executor::block_on(future).unwrap();
        assert_eq!(solution.len(), 0);

        Ok(())
    }
}

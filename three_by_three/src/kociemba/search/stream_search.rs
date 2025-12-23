use std::{
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize},
    },
    thread::JoinHandle,
    usize,
};

use flume::{Sender, r#async::RecvStream};
use futures::{StreamExt, future};
use futures_core::Stream;
use rayon::iter::ParallelIterator;

use crate::{
    cube_ops::{cube_move::CubeMove, repr_cube::ReprCube},
    kociemba::{
        search::{
            move_resolver, solve_domino::solve_domino,
            solve_with_fixed_len_phase_1::produce_solutions_par,
        },
    },
    kociemba::tables::Tables,
};

fn solver_thread(
    cube: ReprCube,
    tables: &Tables,
    send: Sender<Vec<CubeMove>>,
    cancel: Arc<AtomicBool>,
    seed_best: usize,
) {
    let mut best = AtomicUsize::new(seed_best);

    if cube == ReprCube::SOLVED {
        let _ = send.send(Vec::new());
        return;
    }
    
    if *(best.get_mut()) <= 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<1>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 2 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<2>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 3 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<3>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 4 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<4>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 5 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<5>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 6 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<6>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 7 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<7>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 8 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<8>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 9 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<9>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 10 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<10>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 11 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<11>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 12 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<12>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 13 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<13>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 14 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<14>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 15 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<15>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 16 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<16>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 17 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<17>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 18 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<18>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 19 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<19>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });

    if *(best.get_mut()) <= 20 + 1 {
        return;
    }
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return;
    }
    produce_solutions_par::<20>(cube, &best, tables, &cancel).for_each(|solution| {
        let _ = send.send(solution);
    });
}

pub fn get_incremental_solutions_stream(
    cube: ReprCube,
    tables: &'static Tables,
    max_moves: Option<usize>,
) -> impl Stream<Item = Vec<CubeMove>> {
    let (send, recv) = flume::unbounded();
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();
    let join_handle = std::thread::spawn(move || {
        let send = send;
        let cancel = cancel_clone;
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            solver_thread(
                cube,
                tables,
                send,
                cancel,
                max_moves.unwrap_or(usize::MAX).saturating_add(1),
            );
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

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::{
        CornerOrient, EdgeOrient, EdgePerm, cube, kociemba::partial_reprs::edge_group::EdgeGroup,
    };

    #[test]
    fn test_stream_superflip() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));

        let mut stream = get_incremental_solutions_stream(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            tables,
            Some(20),
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

        let cube = cube.apply_cube_move(CubeMove::D2);
        cube.pretty_print();
        let stream = get_incremental_solutions_stream(
            cube,
            tables,
            Some(20),
        );

        for solution in futures::executor::block_on_stream(stream) 
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

            let mut stream = get_incremental_solutions_stream(cube, tables, Some(20));
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

    // #[test]
    // fn gen_domino_scrambles() -> anyhow::Result<()> {
    //     let tables = Box::leak(Box::new(Tables::new("tables")?));
    //     let mut rng = ChaCha8Rng::seed_from_u64(1);

    //     for _ in 0..100 {
    //         let mut cube: ReprCube =
    //             rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

    //         cube.edge_orient = EdgeOrient::SOLVED;
    //         cube.corner_orient = CornerOrient::SOLVED;
    //         cube.edge_perm = {
    //             let (_, ud, e) = EdgePerm::split(cube.edge_perm);
    //             EdgePerm::join(EdgeGroup::SOLVED, ud, e)
    //         };

    //         cube.pretty_print();

    //         let mut stream = get_incremental_solutions_stream(cube, tables, None);
    //         let future = stream.next();
    //         // assert!(futures::executor::block_on(future).is_some());

    //         let solution = futures::executor::block_on(future).unwrap();
    //         {
    //             print!("{:02} ", solution.len());
    //             for m in solution.into_iter().rev() {
    //                 let m = match m {
    //                     CubeMove::U1 => CubeMove::U3,
    //                     CubeMove::U2 => CubeMove::U2,
    //                     CubeMove::U3 => CubeMove::U1,
    //                     CubeMove::D1 => CubeMove::D3,
    //                     CubeMove::D2 => CubeMove::D2,
    //                     CubeMove::D3 => CubeMove::D1,
    //                     CubeMove::F1 => CubeMove::F3,
    //                     CubeMove::F2 => CubeMove::F2,
    //                     CubeMove::F3 => CubeMove::F1,
    //                     CubeMove::B1 => CubeMove::B3,
    //                     CubeMove::B2 => CubeMove::B2,
    //                     CubeMove::B3 => CubeMove::B1,
    //                     CubeMove::R1 => CubeMove::R3,
    //                     CubeMove::R2 => CubeMove::R2,
    //                     CubeMove::R3 => CubeMove::R1,
    //                     CubeMove::L1 => CubeMove::L3,
    //                     CubeMove::L2 => CubeMove::L2,
    //                     CubeMove::L3 => CubeMove::L1,
    //                 };
    //                 print!("{m} ");
    //             }
    //             println!("");
    //         }
    //     }

    //     Ok(())
    // }

    #[test]
    fn test_already_solved() -> anyhow::Result<()> {
        let tables = Box::leak(Box::new(Tables::new("tables")?));

        let mut stream = get_incremental_solutions_stream(ReprCube::SOLVED, tables, Some(20));
        let future = stream.next();
        let solution = futures::executor::block_on(future).unwrap();
        assert_eq!(solution.len(), 0);

        Ok(())
    }
}

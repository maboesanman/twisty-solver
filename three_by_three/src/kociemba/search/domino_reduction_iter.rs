use std::{num::NonZeroU8, sync::atomic::AtomicU8};

use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator, ParallelIterator,
    plumbing::{UnindexedConsumer, UnindexedProducer, bridge_unindexed},
};

use crate::{
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::CubeSymmetry, repr_cube::ReprCube},
    kociemba::{
        search::{
            phase_1_node::Phase1Node,
            phase_2_node::Phase2Node,
            split_phase_1_node::{SplitPhase1NodeA, merge, split},
        },
        tables::Tables,
    },
};

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
pub fn all_domino_reductions<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    axes: &'_ [u8],
    best: *const u8,
    target: u8,
) -> impl 'a + Iterator<Item = ([Phase1Node; N], Phase2Node)> {
    Stack::<_, _>::new(cube, tables, best, target, axes)
        .into_iter()
        .flatten()
}

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
pub fn all_domino_reductions_par<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    axes: &'_ [u8],
    best: &'a AtomicU8,
    target: u8,
) -> impl 'a + ParallelIterator<Item = ([Phase1Node; N], Phase2Node)> {
    Stack::<'a, N, _>::new(cube, tables, best, target, axes)
        .into_par_iter()
        .flatten()
}

// N is number of moves
// this doesn't work if the cube is already domino reduced.
#[derive(Debug, Clone)]
struct Stack<'t, const N: usize, Best> {
    tables: &'t Tables,

    best: Best,
    target: u8,

    frame_metadata: [FrameMetadata; N],
    start: Phase1Node,
    frame_data: Vec<SplitPhase1NodeA>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameMetadata {
    // the offset into the frame_data buffer that this begins.
    // note that this can never be empty
    start: u16,
    correct: u8,
}

pub trait BestTrait: Copy {
    fn get_best(self) -> u8;
}

impl BestTrait for &AtomicU8 {
    fn get_best(self) -> u8 {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl BestTrait for &u8 {
    fn get_best(self) -> u8 {
        *self
    }
}

impl<'a> BestTrait for *const u8 {
    fn get_best(self) -> u8 {
        unsafe { *self }
    }
}

impl<'t, const N: usize, Best: Clone> Stack<'t, N, Best> {
    pub fn new(
        cube: ReprCube,
        tables: &'t Tables,
        best: Best,
        target: u8,
        axes: &[u8],
    ) -> Vec<Self> {
        assert!(axes.iter().all_unique() & axes.iter().all(|a| *a < 3));

        let nodes = axes
            .iter()
            .map(|x| Phase1Node::from_cube(cube.conjugate(CubeSymmetry(x << 4)), tables));

        if N == 0 {
            nodes
                .filter(|n| n.is_domino_reduced())
                .map(|start| Self::new_inner(start, tables, best.clone(), target))
                .collect_vec()
        } else {
            nodes
                .flat_map(|n| {
                    let mut o = n;
                    o.previous_axis = CubePreviousAxis::NoneAlt;
                    [
                        Self::new_inner(n, tables, best.clone(), target),
                        Self::new_inner(o, tables, best.clone(), target),
                    ]
                })
                .collect_vec()
        }
    }

    fn new_inner(start: Phase1Node, tables: &'t Tables, best: Best, target: u8) -> Self {
        let mut frame_data = Vec::with_capacity(15 * N + 1);
        let (split_start, _) = split(start);
        frame_data.push(split_start);

        let mut stack = Self {
            tables,
            best,
            target,
            frame_data,
            frame_metadata: [FrameMetadata {
                start: 0,
                correct: 0,
            }; _],
            start,
        };

        stack.fill_recurse(0);

        stack
    }

    /// ensure the top frame is non-empty and the top item is not too far away from reduced to reach in the remaining moves.
    /// this should be called to clean up AFTER THE ITEM WAS POPPED OR AFTER AN UNKNOWN NUMBER OF ITEMS HAVE POPULATED THE NEW FRAME
    #[inline(always)]
    fn drop_recurse(&mut self, i: &mut usize) -> bool {
        debug_assert!(*i > 0);

        let mut len = self.frame_data.len() as u16;
        let frame_metadata = self.frame_metadata.as_ptr();

        loop {
            unsafe {
                let top_frame_empty = (*frame_metadata.add(*i - 1)).start == len;

                if top_frame_empty {
                    len -= 1;

                    if *i == 1 {
                        self.frame_data.set_len(0);
                        return false;
                    }
                    *i -= 1;
                    continue;
                }

                let moves_remaining = (N - *i) as u8;
                let top = *self.frame_data.as_ptr().add((len - 1) as usize);

                let reduced = top.is_domino_reduced();
                let viable = (moves_remaining == 0 && reduced)
                    || (!reduced)
                    || (reduced && moves_remaining > 6);

                if viable {
                    let distance = (*self.frame_data.as_ptr().add((len - 1) as usize))
                        .distance_heuristic(self.tables);

                    if distance <= moves_remaining {
                        break;
                    }
                }

                len -= 1;
            }
        }

        unsafe {
            self.frame_data.set_len(len as usize);
        }

        true
    }

    #[inline(always)]
    fn fill_recurse(&mut self, i: usize) {
        let mut i = i;
        while i < N {
            self.frame_metadata[i] = FrameMetadata {
                start: self.frame_data.len() as u16,
                correct: 0,
            };

            let last_data = unsafe { self.frame_data.last_mut().unwrap_unchecked() };

            let slice = unsafe { &mut *(last_data as *mut SplitPhase1NodeA).cast_array::<16>() };

            let moves_remaining = unsafe { NonZeroU8::new_unchecked((N - i) as u8) };

            let added = SplitPhase1NodeA::produce_next_nodes_a(slice, moves_remaining, self.tables);

            unsafe {
                self.frame_data.set_len(self.frame_data.len() + added);
            }

            i += 1;

            if !self.drop_recurse(&mut i) {
                return;
            }
        }
    }
}

impl<'t, const N: usize, Best: Clone + BestTrait> Iterator for Stack<'t, N, Best> {
    type Item = ([Phase1Node; N], Phase2Node);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let phase_1_tail = self.frame_data.pop()?;
            let corner_dist = (phase_1_tail.corner_perm_raw.0 >> 12) as u8;

            let best = self.best.get_best();
            if best <= self.target {
                return None;
            }
            if best < N as u8 + corner_dist {
                let mut i = N;
                if self.drop_recurse(&mut i) {
                    self.fill_recurse(i);
                };
                continue;
            }

            let (head, tail) = {
                let mut head_out = [self.start; N];
                let mut out = split(self.start).1;
                let mut tail_out = self.start;

                for i in 0..N {
                    let moves_remaining = unsafe { NonZeroU8::new_unchecked((N - i) as u8) };
                    let frame_i_remaining = self
                        .frame_metadata
                        .get(i + 1)
                        .map(|m| m.start as usize)
                        .unwrap_or(self.frame_data.len() + 1)
                        - self.frame_metadata[i].start as usize;

                    let j = (frame_i_remaining + self.frame_metadata[i].correct as usize) - 1;
                    out = out.produce_next_node_b(moves_remaining, self.tables, j);
                    if i + 1 < N {
                        head_out[i + 1] = merge(
                            self.frame_data[self.frame_metadata[i + 1].start as usize - 1],
                            out,
                        );
                    } else {
                        tail_out = merge(phase_1_tail, out);
                    }
                }

                (head_out, tail_out)
            };

            let phase_2_head = Phase2Node::from_phase_1_node(tail);

            let mut i = N;
            if self.drop_recurse(&mut i) {
                self.fill_recurse(i);
            };

            break Some((head, phase_2_head));
        }
    }
}

impl<'t, const N: usize, Best: Clone + Send + BestTrait> UnindexedProducer for Stack<'t, N, Best> {
    type Item = <Self as Iterator>::Item;

    fn split(mut self) -> (Self, Option<Self>) {
        loop {
            // we've already been exhausted. nothing to split
            if self.frame_data.is_empty() {
                return (self, None);
            }

            let mut i = 0;
            let (frame_start, frame_end) = loop {
                if i + 2 > N {
                    return (self, None);
                }
                let start = if i == 0 {
                    0
                } else {
                    unsafe { *self.frame_metadata.get_unchecked(i - 1) }.start
                };
                let end = self.frame_metadata[i].start;

                if end - start > 1 {
                    break (start as usize, end as usize);
                }

                i += 1;
            };

            // frame i has more than 1 item in it. let's produce a new frame metadata and data.

            // if it's odd, the last one in the frame is alreay potentially partially expanded so we
            // give more items to the latter half (round down)
            let frame_split = (frame_start + frame_end) / 2;

            let mut new_frame_data = Vec::with_capacity(15 * N + 1);
            new_frame_data.extend_from_slice(&self.frame_data[0..frame_split]);

            self.frame_data.drain(frame_start..frame_split);

            for x in 0..N {
                if self.frame_metadata[x].start > frame_start as u16 {
                    self.frame_metadata[x].start -= (frame_split - frame_start) as u16;
                }
            }

            let mut new_stack = Stack {
                tables: self.tables,
                best: self.best,
                frame_metadata: self.frame_metadata,
                frame_data: new_frame_data,
                target: self.target,
                start: self.start,
            };

            self.frame_metadata[i - 1].correct += (frame_split - frame_start) as u8;

            new_stack.fill_recurse(i);

            if !new_stack.frame_data.is_empty() {
                break (self, Some(new_stack));
            };
        }
    }

    fn fold_with<F>(self, mut folder: F) -> F
    where
        F: rayon::iter::plumbing::Folder<Self::Item>,
    {
        let best = self.best;
        let target = self.target;
        for item in self {
            folder = folder.consume(item);
            if folder.full() || best.get_best() <= target {
                break;
            }
        }
        folder
    }
}

impl<'t, const N: usize, Best: Clone + Send + BestTrait> ParallelIterator for Stack<'t, N, Best> {
    type Item = ([Phase1Node; N], Phase2Node);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::{cube, kociemba::search::move_resolver::move_resolver_multi_dimension_domino};

    use super::*;

    #[test]
    fn domino_reduce_empty() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let cube = cube![R U Rp Up];
        let best = &20;
        let stack = all_domino_reductions::<0>(cube, &tables, &[0, 1, 2], best, 0).collect_vec();

        println!("{stack:#?}");

        Ok(())
    }

    #[test]
    fn domino_reduce_test_iter_2() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_ref = &tables;
        let cube = cube![R U Rp Up];
        // let cube = cube![D R2 L];
        let res = move |path: &[Phase1Node], last: &Phase2Node| {
            let cubes = path
                .into_iter()
                .map(|x| x.into_cube(table_ref))
                .chain(Some(last.into_cube(table_ref)));

            move_resolver_multi_dimension_domino(cube, cubes)
        };
        let best = (&20u8) as *const _;
        let stack = all_domino_reductions::<2>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<3>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<4>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<5>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });

        Ok(())
    }

    #[test]
    fn domino_reduce_test_iter_2_par() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let table_ref = &tables;
        let cube = cube![R U Rp Up];
        // let cube = cube![D R2 L];
        let res = move |path: &[Phase1Node], last: &Phase2Node| {
            let cubes = path
                .into_iter()
                .map(|x| x.into_cube(table_ref))
                .chain(Some(last.into_cube(table_ref)));

            move_resolver_multi_dimension_domino(cube, cubes)
        };
        let best = &AtomicU8::new(20);
        let stack = all_domino_reductions_par::<2>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions_par::<3>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions_par::<4>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions_par::<5>(cube, &tables, &[0, 1, 2], best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });

        Ok(())
    }

    #[test]
    fn domino_reduce_test_split() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let cancel = AtomicU8::new(100);
        let table_ref = &tables;
        let cube = cube![R U Rp Up];
        // let cube = cube![D R2 L];
        let res = move |path: &[Phase1Node], last: &Phase2Node| {
            let cubes = path
                .into_iter()
                .map(|x| x.into_cube(table_ref))
                .chain(Some(last.into_cube(table_ref)));

            move_resolver_multi_dimension_domino(cube, cubes)
        };
        let stack = Stack::<5, _>::new(cube, &tables, &cancel, 0, &[0, 1, 2])
            .into_iter()
            .filter_map(|x| {
                let old = x.clone();
                let (a, b) = x.split();
                let b = b?;

                println!("B1: {:?}", old.frame_metadata);
                println!("B2: {:?}", a.frame_metadata);
                println!("B3: {:?}", b.frame_metadata);
                old.split();
                Some([a, b])
            })
            .flatten();

        stack.into_iter().flatten().for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2_single() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let best = &200;
        let stack = all_domino_reductions::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &[0],
            best,
            0,
        );

        println!("{:?}", stack.count());

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2_par() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let cancel = AtomicU8::new(200);

        let stack = all_domino_reductions_par::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &[0],
            &cancel,
            0,
        );

        println!("{:?}", stack.count());

        Ok(())
    }

    #[test]
    fn domino_reduction_length_chart() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let cancel = AtomicU8::new(100);

        println!(
            "0: {}",
            all_domino_reductions_par::<0>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "1: {}",
            all_domino_reductions_par::<1>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "2: {}",
            all_domino_reductions_par::<2>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "3: {}",
            all_domino_reductions_par::<3>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "4: {}",
            all_domino_reductions_par::<4>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "5: {}",
            all_domino_reductions_par::<5>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "6: {}",
            all_domino_reductions_par::<6>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "7: {}",
            all_domino_reductions_par::<7>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "8: {}",
            all_domino_reductions_par::<8>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "9: {}",
            all_domino_reductions_par::<9>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "10: {}",
            all_domino_reductions_par::<10>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "11: {}",
            all_domino_reductions_par::<11>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );
        println!(
            "12: {}",
            all_domino_reductions_par::<12>(cube, &tables, &[0, 1, 2], &cancel, 0).count()
        );

        Ok(())
    }
    // {(12, 19), (1, 16), (5, 20), (3, 11), (2, 9), (4, 16), (11, 20), (2, 15), (6, 15), (8, 12), (8, 19), (5, 11), (2, 16), (4, 15), (7, 14), (10, 13), (3, 15), (9, 10), (8, 8), (10, 19), (8, 13), (3, 8), (2, 3), (10, 10), (10, 14), (8, 18), (7, 20), (1, 14), (7, 16), (8, 15), (6, 14), (6, 13), (0, 6), (2, 8), (5, 9), (9, 16), (4, 18), (2, 20), (5, 18), (1, 20), (5, 13), (7, 12), (0, 1), (12, 14), (3, 10), (4, 7), (2, 10), (9, 17), (4, 20), (3, 13), (3, 18), (2, 6), (1, 17), (1, 9), (2, 5), (2, 19), (7, 17), (7, 13), (7, 19), (7, 11), (12, 18), (9, 18), (11, 18), (4, 17), (6, 18), (6, 7), (3, 9), (8, 10), (10, 15), (9, 20), (4, 5), (0, 16), (0, 7), (3, 3), (0, 11), (4, 19), (0, 13), (1, 10), (6, 6), (3, 20), (12, 16), (1, 12), (11, 12), (6, 8), (9, 15), (9, 14), (5, 6), (6, 19), (10, 20), (2, 12), (3, 16), (10, 17), (9, 9), (8, 11), (1, 4), (10, 18), (9, 13), (0, 9), (7, 18), (5, 10), (12, 15), (1, 15), (11, 14), (12, 12), (11, 19), (3, 17), (0, 12), (4, 6), (6, 9), (3, 19), (4, 4), (6, 12), (1, 8), (1, 11), (0, 5), (6, 17), (5, 12), (0, 8), (12, 20), (0, 17), (3, 6), (12, 17), (2, 18), (5, 8), (6, 11), (5, 17), (2, 13), (11, 17), (4, 13), (8, 20), (3, 14), (9, 19), (5, 19), (6, 16), (11, 11), (0, 0), (6, 20), (11, 16), (7, 7), (0, 15), (5, 16), (7, 8), (0, 10), (10, 16), (5, 7), (7, 10), (0, 14), (2, 4), (8, 9), (12, 13), (0, 20), (10, 12), (4, 10), (5, 15), (4, 9), (1, 19), (9, 12), (2, 2), (11, 13), (7, 15), (6, 10), (3, 12), (0, 18), (1, 1), (8, 14), (10, 11), (4, 12), (4, 14), (4, 8), (5, 14), (2, 7), (1, 18), (3, 5), (7, 9), (3, 4), (2, 17), (2, 11), (2, 14), (5, 5), (1, 7), (8, 17), (8, 16), (11, 15), (3, 7), (0, 19), (4, 11), (1, 13), (1, 6), (1, 5), (9, 11)}
}

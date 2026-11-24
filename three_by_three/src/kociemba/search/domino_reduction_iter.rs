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
            split_phase_1_node::{
                SplitPhase1NodeA, SplitPhase1NodeAStackStruct, SplitPhase1NodeB, merge, split,
            },
        },
        tables::Tables,
    },
};

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
pub fn all_domino_reductions<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    best: *const u8,
    target: u8,
) -> impl 'a + Iterator<Item = ([Phase1Node; N], Phase2Node)> {
    Stack::<_, _>::new(cube, tables, best, target)
}

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
pub fn all_domino_reductions_par<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    best: &'a AtomicU8,
    target: u8,
) -> impl 'a + ParallelIterator<Item = ([Phase1Node; N], Phase2Node)> {
    Stack::<'a, N, _>::new(cube, tables, best, target)
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
    start_r1: Phase1Node,
    start_r2: Phase1Node,
    /// Precomputed SplitPhase1NodeB for self.start, so next_unpruned
    /// doesn't need to re-split on every call.
    start_split_b: SplitPhase1NodeB,
    start_split_b_r1: SplitPhase1NodeB,
    start_split_b_r2: SplitPhase1NodeB,
    frame_data: SplitPhase1NodeAStackStruct,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameMetadata {
    // the offset into the frame_data buffer that this begins.
    // note that this can never be empty
    start: u16,
}

pub trait BestTrait: Copy {
    fn get_best(self) -> u8;
}

impl BestTrait for *const u8 {
    fn get_best(self) -> u8 {
        unsafe { *self }
    }
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

impl<'t, const N: usize, Best: Clone> Stack<'t, N, Best> {
    pub fn new(cube: ReprCube, tables: &'t Tables, best: Best, target: u8) -> Self {
        todo!()
        // let [base, r1, r2] = [0, 1, 2]
        //     .map(|x| Phase1Node::from_cube(cube.conjugate(CubeSymmetry(x << 4)), tables));

        // let (split_start, start_split_b) = split(base, tables);
        // let (split_start_r1, start_split_b_r1) = split(r1, tables);
        // let (split_start_r2, start_split_b_r2) = split(r1, tables);
        // let frame_data = SplitPhase1NodeAStackStruct::new((15 * N + 1) as u16, split_start);

        // let mut stack = Self {
        //     tables,
        //     best,
        //     target,
        //     frame_data,
        //     frame_metadata: [FrameMetadata {
        //         start: 0,
        //     }; _],
        //     start,
        //     start_split_b,
        // };

        // stack.fill_recurse(0);

        // stack

        // if N == 0 {
        //     todo!()
        //     // nodes
        //     //     .filter(|n| n.is_domino_reduced())
        //     //     .map(|start| Self::new_inner(start, tables, best.clone(), target))
        //     //     .collect_vec()
        // } else {
        //     todo!()
        //     // nodes
        //     //     .flat_map(|n| {
        //     //         let mut o = n;
        //     //         o.previous_axis = CubePreviousAxis::NoneAlt;
        //     //         [
        //     //             Self::new_inner(n, tables, best.clone(), target),
        //     //             Self::new_inner(o, tables, best.clone(), target),
        //     //         ]
        //     //     })
        //     //     .collect_vec()
        // }
    }

    fn new_inner(start: Phase1Node, tables: &'t Tables, best: Best, target: u8) -> Self {
        todo!()
    }

    /// ensure the top frame is non-empty and the top item is not too far away from reduced to reach in the remaining moves.
    /// this should be called to clean up AFTER THE ITEM WAS POPPED OR AFTER AN UNKNOWN NUMBER OF ITEMS HAVE POPULATED THE NEW FRAME
    #[inline(always)]
    fn drop_recurse(&mut self, i: &mut usize) -> bool {
        debug_assert!(*i > 0);

        let frame_metadata = self.frame_metadata.as_ptr();

        unsafe {
            loop {
                let top_frame_empty = (*frame_metadata.add(*i - 1)).start == self.frame_data.len;

                if !top_frame_empty {
                    break;
                }

                if *i == 1 {
                    self.frame_data.len = 0;
                    return false;
                }
                *i -= 1;

                self.frame_data.pop_unchecked_aaa();
            }
        }

        true
    }

    #[inline(always)]
    fn fill_recurse(&mut self, i: usize) {
        let mut i = i;
        while i < N {
            self.frame_metadata[i] = FrameMetadata {
                start: self.frame_data.len as u16,
            };

            let moves_remaining = unsafe { NonZeroU8::new_unchecked((N - i) as u8) };
            let added = self
                .frame_data
                .soa_produce_next_nodes(moves_remaining, self.tables);
            self.frame_data.len += added;

            i += 1;

            if !self.drop_recurse(&mut i) {
                return;
            }
        }
    }

    #[inline(never)]
    fn next_unpruned(
        &mut self,
        phase_1_tail: SplitPhase1NodeA,
        pre_pop_len: u16,
    ) -> ([Phase1Node; N], Phase2Node) {
        let (head, tail) = {
            let mut head_out = [self.start; N];
            let mut out = self.start_split_b;
            let mut tail_out = self.start;

            for i in 0..N {
                let moves_remaining = unsafe { NonZeroU8::new_unchecked((N - i) as u8) };
                let frame_i_remaining = self
                    .frame_metadata
                    .get(i + 1)
                    .map(|m| m.start as usize)
                    .unwrap_or(pre_pop_len as usize)
                    - self.frame_metadata[i].start as usize;

                let j = (frame_i_remaining as usize) - 1;
                out = out.produce_next_node_b(moves_remaining, self.tables, j);
                if i + 1 < N {
                    head_out[i + 1] = merge(
                        self.frame_data
                            .get_aaa(self.frame_metadata[i + 1].start - 1),
                        out,
                        self.tables,
                    );
                } else {
                    tail_out = merge(phase_1_tail, out, self.tables);
                }
            }

            (head_out, tail_out)
        };

        let phase_2_head = Phase2Node::from_phase_1_node(tail);

        let mut i = N;
        if self.drop_recurse(&mut i) {
            self.fill_recurse(i);
        };

        (head, phase_2_head)
    }
}

impl<'t, const N: usize, Best: Clone + BestTrait> Iterator for Stack<'t, N, Best> {
    type Item = ([Phase1Node; N], Phase2Node);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let phase_1_tail = self.frame_data.get_aaa(self.frame_data.len.checked_sub(1)?);
            let pre_pop_len = self.frame_data.len;
            self.frame_data.pop_unchecked_aaa();

            let corner_dist = (phase_1_tail.corner_perm_sym.0 >> 12) as u8;

            let best = self.best.get_best();

            if core::hint::unlikely(best <= self.target) {
                return None;
            }
            if core::hint::likely(best < N as u8 + corner_dist) {
                let mut i = N;
                if self.drop_recurse(&mut i) {
                    self.fill_recurse(i);
                };
                continue;
            }

            break Some(self.next_unpruned(phase_1_tail, pre_pop_len));
        }
    }
}

impl<'t, const N: usize, Best: Clone + Send + BestTrait> UnindexedProducer for Stack<'t, N, Best> {
    type Item = <Self as Iterator>::Item;

    fn split(mut self) -> (Self, Option<Self>) {
        todo!()
        // loop {
        //     // we've already been exhausted. nothing to split
        //     if self.frame_data.len == 0 {
        //         return (self, None);
        //     }

        //     let mut i = 1;
        //     let new_top = loop {
        //         if i + 2 > N || i > 4 {
        //             return (self, None);
        //         }

        //         let start = unsafe { *self.frame_metadata.get_unchecked(i - 1) }.start;
        //         let end = self.frame_metadata[i].start - 1;

        //         if start == end {
        //             i += 1;
        //             continue;
        //         }
        //         let end_frame = unsafe { self.frame_data.get_unchecked_mut(end as usize) };
        //         let penultimate = end - (end_frame.back_pointer() as u16);

        //         if penultimate != start - 1 {
        //             let mut i = penultimate;
        //             let mut j = end;
        //             let mut k;
        //             loop {
        //                 let i_backpointer = unsafe { self.frame_data.get_unchecked(i as usize) }.back_pointer();
        //                 k = i - (i_backpointer as u16);
        //                 let j_mut = unsafe { self.frame_data.get_unchecked_mut(j as usize) };
        //                 j_mut.set_back_pointer(j_mut.back_pointer() + i_backpointer);
        //                 if k == start - 1 {
        //                     break
        //                 }
        //                 j = i;
        //                 i = k;
        //             }
        //             // this means the frame is splittable

        //             // need to tweak end's backpointer to point directly to start
        //             // need to copy the stack data up to penultimate as the new stack.
        //             break penultimate;
        //         }

        //         i += 1;
        //     };

        //     let mut new_frame_data = Vec::with_capacity(15 * N + 1);
        //     new_frame_data.extend_from_slice(&self.frame_data[0..=(new_top as usize)]);

        //     let mut new_stack = Stack {
        //         tables: self.tables,
        //         best: self.best,
        //         frame_metadata: self.frame_metadata,
        //         frame_data: new_frame_data,
        //         target: self.target,
        //         start: self.start,
        //         start_split_b: self.start_split_b,
        //     };

        //     new_stack.fill_recurse(i);

        //     if !new_stack.frame_data.is_empty() {
        //         break (self, Some(new_stack));
        //     };
        // }
    }

    fn fold_with<F>(mut self, mut folder: F) -> F
    where
        F: rayon::iter::plumbing::Folder<Self::Item>,
    {
        while let Some(item) = self.next() {
            folder = folder.consume(item);
            if folder.full() || self.best.get_best() <= self.target {
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

    // Diagnostic-only differential check (not part of the opaque e2e suite):
    // a simple, non-flattened recursive brute force (no back_pointer bookkeeping,
    // no frame_data flattening) that ports the exact same "viable" formula
    // (already independently verified against produce_next_nodes_a's back_pointer
    // chain in split_phase_1_node.rs), compared against Stack's iterator output.
    fn old_valid(dist: u8, m: u8) -> bool {
        let mr_check = m - 1;
        let reduced = dist == 0;
        ((mr_check == 0 && reduced) || (!reduced) || (reduced && mr_check > 6)) && dist <= mr_check
    }

    #[test]
    fn domino_reduce_empty() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let cube = cube![R U Rp Up];
        let best = &20;
        let stack = all_domino_reductions::<0>(cube, &tables, best, 0).collect_vec();

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
        let stack = all_domino_reductions::<2>(cube, &tables, best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<3>(cube, &tables, best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<4>(cube, &tables, best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<5>(cube, &tables, best, 0);
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
        let stack = all_domino_reductions_par::<2>(cube, &tables, best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions_par::<3>(cube, &tables, best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions_par::<4>(cube, &tables, best, 0);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions_par::<5>(cube, &tables, best, 0);
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
        let stack = Some(Stack::<5, _>::new(cube, &tables, &cancel, 0))
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
            &cancel,
            0,
        );

        println!("{:?}", stack.count());

        Ok(())
    }

    fn check_par_seq_identical<const N: usize>(cube: ReprCube, tables: &Tables) -> usize {
        let best: u8 = 255;
        let seq_results: Vec<_> =
            all_domino_reductions::<N>(cube, tables, &best as *const _, 0).collect();

        let par_results: Vec<_> =
            all_domino_reductions_par::<N>(cube, tables, &AtomicU8::new(255), 0).collect();

        assert_eq!(
            seq_results.len(),
            par_results.len(),
            "N={N}: sequential produced {}, parallel produced {}",
            seq_results.len(),
            par_results.len()
        );

        let seq_len = seq_results.len();
        let seq_set: std::collections::HashSet<_> = seq_results.into_iter().collect();
        let par_set: std::collections::HashSet<_> = par_results.into_iter().collect();

        assert_eq!(
            seq_set, par_set,
            "N={N}: sequential and parallel sets differ"
        );

        seq_len
    }

    #[test]
    fn par_and_seq_produce_identical_set() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let cubes = [
            cube![R U R U R U],
            cube![R2 U2 R2 U2],
            cube![R U2 R U2 R2],
            cube![R L U D F B],
            cube![R U F R U F],
            cube![R2 U2 F2 R2 U2],
        ];

        let mut total = 0;
        for cube in cubes {
            for &n in &[6, 7, 8] {
                let count = match n {
                    6 => check_par_seq_identical::<6>(cube, &tables),
                    7 => check_par_seq_identical::<7>(cube, &tables),
                    8 => check_par_seq_identical::<8>(cube, &tables),
                    _ => unreachable!(),
                };
                assert!(
                    count > 0,
                    "N={n} produced 0 results for cube={cube:?} — trivial case"
                );
                total += count;
            }
        }

        assert!(total > 0, "No non-trivial comparisons were performed");

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
            all_domino_reductions_par::<0>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "1: {}",
            all_domino_reductions_par::<1>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "2: {}",
            all_domino_reductions_par::<2>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "3: {}",
            all_domino_reductions_par::<3>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "4: {}",
            all_domino_reductions_par::<4>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "5: {}",
            all_domino_reductions_par::<5>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "6: {}",
            all_domino_reductions_par::<6>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "7: {}",
            all_domino_reductions_par::<7>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "8: {}",
            all_domino_reductions_par::<8>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "9: {}",
            all_domino_reductions_par::<9>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "10: {}",
            all_domino_reductions_par::<10>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "11: {}",
            all_domino_reductions_par::<11>(cube, &tables, &cancel, 0).count()
        );
        println!(
            "12: {}",
            all_domino_reductions_par::<12>(cube, &tables, &cancel, 0).count()
        );

        Ok(())
    }
}

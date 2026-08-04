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
            split_phase_1_node::{SplitPhase1NodeA, SplitPhase1NodeB, merge, split},
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
    /// Precomputed SplitPhase1NodeB for self.start, so next_unpruned
    /// doesn't need to re-split on every call.
    start_split_b: SplitPhase1NodeB,
    frame_data: Vec<SplitPhase1NodeA>,
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
        let (split_start, start_split_b) = split(start, tables);
        frame_data.push(split_start);

        let mut stack = Self {
            tables,
            best,
            target,
            frame_data,
            frame_metadata: [FrameMetadata {
                start: 0,
            }; _],
            start,
            start_split_b,
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

        unsafe {
            loop {
                let top_frame_empty = (*frame_metadata.add(*i - 1)).start == len;

                if !top_frame_empty {
                    break;
                }

                if *i == 1 {
                    self.frame_data.set_len(0);
                    return false;
                }
                *i -= 1;

                let top = *self.frame_data.as_ptr().add((len - 1) as usize);
                len -= top.back_pointer() as u16;
            }
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
                        self.frame_data[self.frame_metadata[i + 1].start as usize - 1],
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
            let phase_1_tail = self.frame_data.last()?.clone();
            let pre_pop_len = self.frame_data.len() as u16;
            unsafe {
                self.frame_data
                    .set_len(self.frame_data.len() - phase_1_tail.back_pointer() as usize);
            }

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
        loop {
            // we've already been exhausted. nothing to split
            if self.frame_data.is_empty() {
                return (self, None);
            }

            let mut i = 1;
            let new_top = loop {
                if i + 2 > N {
                    return (self, None);
                }

                let start = unsafe { *self.frame_metadata.get_unchecked(i - 1) }.start;
                let end = self.frame_metadata[i].start - 1;
                
                if start == end {
                    i += 1;
                    continue;
                }
                let end_frame = unsafe { self.frame_data.get_unchecked_mut(end as usize) };
                let penultimate = end - (end_frame.back_pointer() as u16);

                if penultimate != start - 1 {
                    // this means the frame is splittable

                    // need to tweak end's backpointer to point directly to start
                    // need to copy the stack data up to penultimate as the new stack.
                    end_frame.set_back_pointer((end + 1 - start) as u8);
                    break penultimate;
                }

                i += 1;
            };

            let mut new_frame_data = Vec::with_capacity(15 * N + 1);
            new_frame_data.extend_from_slice(&self.frame_data[0..=(new_top as usize)]);

            let mut new_stack = Stack {
                tables: self.tables,
                best: self.best,
                frame_metadata: self.frame_metadata,
                frame_data: new_frame_data,
                target: self.target,
                start: self.start,
                start_split_b: self.start_split_b,
            };

            new_stack.fill_recurse(i);

            if !new_stack.frame_data.is_empty() {
                break (self, Some(new_stack));
            };
        }
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

    fn brute_recurse(
        node: Phase1Node,
        tables: &Tables,
        remaining: usize,
        path: &mut Vec<Phase1Node>,
        results: &mut Vec<(Vec<Phase1Node>, Phase2Node)>,
    ) {
        let (split_a, split_b) = split(node, tables);
        let moves_remaining = NonZeroU8::new(remaining as u8).unwrap();
        let mut buf = [split_a; 16];
        let num_moves = SplitPhase1NodeA::produce_next_nodes_a(&mut buf, moves_remaining, tables);
        for j in 1..=num_moves {
            let dist = buf[j].distance_heuristic();
            if !old_valid(dist, remaining as u8) {
                continue;
            }
            let child_b = split_b.produce_next_node_b(moves_remaining, tables, j - 1);
            let child = merge(buf[j], child_b, tables);
            path.push(node);
            if remaining == 1 {
                results.push((path.clone(), Phase2Node::from_phase_1_node(child)));
            } else {
                brute_recurse(child, tables, remaining - 1, path, results);
            }
            path.pop();
        }
    }

    fn brute_force<const N: usize>(
        cube: ReprCube,
        tables: &Tables,
        axes: &[u8],
    ) -> Vec<(Vec<Phase1Node>, Phase2Node)> {
        let mut results = Vec::new();
        for &ax in axes {
            let base = Phase1Node::from_cube(cube.conjugate(CubeSymmetry(ax << 4)), tables);
            if N == 0 {
                if base.is_domino_reduced() {
                    results.push((vec![], Phase2Node::from_phase_1_node(base)));
                }
                continue;
            }
            let mut alt = base;
            alt.previous_axis = CubePreviousAxis::NoneAlt;
            for start in [base, alt] {
                let mut path = Vec::new();
                brute_recurse(start, tables, N, &mut path, &mut results);
            }
        }
        results
    }

    fn check_matches_brute_force<const N: usize>(cube: ReprCube, tables: &Tables) {
        let axes = &[0u8, 1, 2];
        let best = &255u8 as *const u8;
        let stack_results = all_domino_reductions::<N>(cube, tables, axes, best, 0).collect_vec();
        let brute_results = brute_force::<N>(cube, tables, axes);
        assert_eq!(
            stack_results.len(),
            brute_results.len(),
            "N={N}: Stack produced {} results, brute force produced {}",
            stack_results.len(),
            brute_results.len()
        );

        use std::collections::HashSet;
        let key = |path: &[Phase1Node], last: &Phase2Node| format!("{path:?}|{last:?}");
        let stack_set: HashSet<String> = stack_results.iter().map(|(p, l)| key(p, l)).collect();
        let brute_set: HashSet<String> = brute_results.iter().map(|(p, l)| key(p, l)).collect();
        assert_eq!(stack_set, brute_set, "N={N}: content mismatch");
    }

    #[test]
    fn domino_reduce_matches_brute_force() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let cube = cube![R U Rp Up];

        check_matches_brute_force::<0>(cube, &tables);
        check_matches_brute_force::<1>(cube, &tables);
        check_matches_brute_force::<2>(cube, &tables);
        check_matches_brute_force::<3>(cube, &tables);
        check_matches_brute_force::<4>(cube, &tables);
        check_matches_brute_force::<5>(cube, &tables);

        let mut rng = ChaCha8Rng::seed_from_u64(11);
        for _ in 0..5 {
            let cube2: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
            check_matches_brute_force::<0>(cube2, &tables);
            check_matches_brute_force::<1>(cube2, &tables);
            check_matches_brute_force::<2>(cube2, &tables);
            check_matches_brute_force::<3>(cube2, &tables);
        }

        Ok(())
    }

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
}

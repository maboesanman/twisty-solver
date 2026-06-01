use std::{
    num::{NonZeroU8, NonZeroUsize},
    sync::atomic::AtomicBool,
};

use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator, ParallelIterator,
    plumbing::{UnindexedConsumer, UnindexedProducer, bridge_unindexed},
};

use crate::{
    cube_ops::{cube_sym::CubeSymmetry, repr_cube::ReprCube},
    kociemba::{
        coords::{CornerOrientRawCoord, EdgeGroupOrientSymCoord},
        search::{
            phase_1_node::{Phase1Node, TableOffsets},
            phase_2_node::Phase2Node,
        },
        tables::Tables,
    },
};

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
///
/// if S is false, the move sequence is not allowed to be domino reduced at any time before the final state.
/// if S is true, only the second-to-last state is prevented from being domino reduced
pub fn all_domino_reductions<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    table_offsets: &'a TableOffsets<'a>,
) -> impl Iterator<Item = ([Phase1Node; N], Phase2Node, Phase2Node)> {
    Stack::<_, _>::new(cube, tables, table_offsets, ())
        .into_iter()
        .flatten()
}

pub fn any_domino_reductions_const<const N: usize>(
    edge_group_orient_sym: EdgeGroupOrientSymCoord,
    corner_orient_raw: CornerOrientRawCoord,
    tables: &Tables,
    table_offsets: &TableOffsets,
) -> u8 {
    let cube = Phase1Node::from_phase_1_coords(edge_group_orient_sym, corner_orient_raw, tables);
    Stack::<N, _>::new_inner(cube, tables, table_offsets, ())
        .take(255)
        .count() as u8
}

pub fn any_domino_reductions(
    edge_group_orient_sym: EdgeGroupOrientSymCoord,
    corner_orient_raw: CornerOrientRawCoord,
    tables: &Tables,
    table_offsets: &TableOffsets,
    n: u8,
) -> u8 {
    match n {
        0 => any_domino_reductions_const::<0>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        1 => any_domino_reductions_const::<1>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        2 => any_domino_reductions_const::<2>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        3 => any_domino_reductions_const::<3>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        4 => any_domino_reductions_const::<4>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        5 => any_domino_reductions_const::<5>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        6 => any_domino_reductions_const::<6>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        7 => any_domino_reductions_const::<7>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        8 => any_domino_reductions_const::<8>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        9 => any_domino_reductions_const::<9>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        10 => any_domino_reductions_const::<10>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        11 => any_domino_reductions_const::<11>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        12 => any_domino_reductions_const::<12>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        13 => any_domino_reductions_const::<13>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        14 => any_domino_reductions_const::<14>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        15 => any_domino_reductions_const::<15>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        16 => any_domino_reductions_const::<16>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        17 => any_domino_reductions_const::<17>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        18 => any_domino_reductions_const::<18>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        19 => any_domino_reductions_const::<19>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        20 => any_domino_reductions_const::<20>(
            edge_group_orient_sym,
            corner_orient_raw,
            tables,
            table_offsets,
        ),
        _ => unreachable!(),
    }
}

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
///
/// if S is false, the move sequence is not allowed to be domino reduced at any time before the final state.
/// if S is true, only the second-to-last state is prevented from being domino reduced
pub fn all_domino_reductions_par<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    table_offsets: &'a TableOffsets<'a>,
    cancel: &'a AtomicBool,
) -> impl 'a + ParallelIterator<Item = ([Phase1Node; N], Phase2Node, Phase2Node)> {
    Stack::<'a, N, &'a AtomicBool>::new(cube, tables, table_offsets, cancel)
        .into_par_iter()
        .flatten()
}

// N is number of moves
// this doesn't work if the cube is already domino reduced.
#[derive(Debug, Clone)]
struct Stack<'t, const N: usize, C> {
    tables: &'t Tables,
    table_offsets: &'t TableOffsets<'t>,

    cancel: C,

    frame_metadata: [FrameMetadata; N],

    // 3, 18, 15 * (N - 1) at most. when N=20, 306 at most
    frame_data: Vec<Phase1Node>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameMetadata {
    // the offset into the frame_data buffer that this begins.
    // note that this can never be empty
    start: u16,

    // the bounds on the actual distance we know right now.
    max_distance: u8,
}

impl FrameMetadata {
    const fn default_const() -> Self {
        Self {
            start: 0,
            max_distance: 20,
        }
    }

    #[cold]
    #[inline(never)]
    fn cold_default() -> FrameMetadata {
        FrameMetadata::default()
    }
}

impl Default for FrameMetadata {
    fn default() -> Self {
        Self::default_const()
    }
}

impl<'t, const N: usize, C: Clone> Stack<'t, N, C> {
    const FRAME_DATA_CAP: usize = 3 + 15 * N;

    pub fn new(
        cube: ReprCube,
        tables: &'t Tables,
        table_offsets: &'t TableOffsets,
        cancel: C,
    ) -> Vec<Self> {
        let mut options =
            [0, 1, 2].map(|x| Phase1Node::from_cube(cube.conjugate(CubeSymmetry(x << 4)), tables));
        options.sort_by_key(|n| n.distance_heuristic(tables));
        options
            .into_iter()
            .map(|node| Self::new_inner(node, tables, table_offsets, cancel.clone()))
            .collect_vec()
    }

    fn new_inner(
        start: Phase1Node,
        tables: &'t Tables,
        table_offsets: &'t TableOffsets,
        cancel: C,
    ) -> Self {
        let starts = Some(start);
        let mut frame_data = if N == 0 {
            starts
                .into_iter()
                .filter(|n| n.is_domino_reduced())
                .collect_vec()
        } else {
            starts.into_iter().collect_vec()
        };

        frame_data.reserve_exact(Self::FRAME_DATA_CAP);

        let mut stack = Self {
            tables,
            table_offsets,
            cancel,
            frame_data,
            frame_metadata: [const { FrameMetadata::default_const() }; _],
        };

        stack.fill_recurse_no_simd(0);

        stack
    }

    /// drop the frame at the top (belonging to frame i)
    #[inline(always)]
    fn drop_recurse(&mut self, i: &mut usize) -> bool {
        debug_assert!(*i > 0);

        // let initial_i = *i;

        let mut len = self.frame_data.len() as u16;
        let frame_metadata = self.frame_metadata.as_ptr();

        while unsafe { (*frame_metadata.add(*i - 1)).start } == len {
            len -= 1;
            if *i == 1 {
                unsafe {
                    self.frame_data.set_len(0);
                }
                return false;
            }
            *i -= 1;
        }

        unsafe {
            self.frame_data.set_len(len as usize);
        }

        // After k while-loop iterations, we've popped k ancestors. The deepest empty
        // frame (initial_i) has max_distance M. Going up k hops, the sibling frame's
        // nodes have distance ≤ M + k = M + (initial_i + 1 - *i).

        // let smaller_max = self.frame_metadata[initial_i - 1].max_distance + (initial_i - *i) as u8;
        // if self.frame_metadata[*i - 1].max_distance > smaller_max {
        //     self.frame_metadata[*i - 1].max_distance = smaller_max
        // }

        true
    }

    // THIS IS THE HOTTEST OF HOT LOOPS. MOST OF THE ALGORITHM IS SPENT IN THIS FUNCTION
    // 39 % of program runtime is spent in fill_recurse's implementation (not counting other function calls)
    // recurse into frames

    // ASSUMES frame_data is non-empty

    fn fill_recurse_no_simd(&mut self, i: usize) {
        let mut i = i;
        while i < N {
            let last_data = self.frame_data.last().unwrap();
            let last_frame = self.get_frame_metadata_i(i);

            // 34 % of program runtime is spent in produce_next_nodes
            let incoming = last_data.produce_next_nodes(
                last_frame.max_distance,
                unsafe { NonZeroU8::new_unchecked((N - i) as u8) },
                self.tables,
            );

            self.frame_metadata[i].start = self.frame_data.len() as u16;

            if let Some(incoming) = incoming {
                self.frame_metadata[i].max_distance = incoming.max_possible_distance;
                let dst = self.frame_data.as_mut_ptr();
                let mut len = self.frame_data.len();
                unsafe {
                    let mut out = dst.add(len);

                    for item in incoming.children {
                        // SAFETY: caller guarantees capacity
                        std::ptr::write(out, item);
                        out = out.add(1);
                        len += 1;
                    }

                    self.frame_data.set_len(len);
                }
            }

            i += 1;

            if !self.drop_recurse(&mut i) {
                return;
            }
        }
    }

    #[inline(always)]
    fn fill_recurse_simd(&mut self, i: NonZeroUsize) {
        let mut i = i.get();
        while i < N {
            let len = self.frame_data.len();
            self.frame_metadata[i].start = len as u16;
            let last_data = unsafe { self.frame_data.last_mut().unwrap_unchecked() };
            let last_frame = unsafe { *self.frame_metadata.get_unchecked(i - 1) };

            let slice = unsafe { &mut *(last_data as *mut Phase1Node).cast_array::<16>() };

            let (added, new_max_dist) = Phase1Node::produce_next_nodes_simd::<true>(
                slice,
                last_frame.max_distance,
                unsafe { NonZeroU8::new_unchecked((N - i) as u8) },
                self.table_offsets,
                self.tables,
            );

            self.frame_metadata[i].max_distance = new_max_dist;
            unsafe {
                self.frame_data.set_len(len + added);
            }

            i += 1;

            if !self.drop_recurse(&mut i) {
                return;
            }
        }
    }

    fn get_frame_metadata_i(&self, i: usize) -> FrameMetadata {
        if std::hint::likely(i != 0) {
            unsafe { *self.frame_metadata.get_unchecked(i - 1) }
        } else {
            FrameMetadata::cold_default()
        }
    }
}

impl<'t, const N: usize, C: Clone> Iterator for Stack<'t, N, C> {
    type Item = ([Phase1Node; N], Phase2Node, Phase2Node);

    fn next(&mut self) -> Option<Self::Item> {
        let phase_1_tail_a = self.frame_data.pop()?;
        let phase_1_tail_b = self.frame_data.pop().unwrap_or(phase_1_tail_a);
        let phase_2_head_a = Phase2Node::from_phase_1_node(phase_1_tail_a);
        let phase_2_head_b = Phase2Node::from_phase_1_node(phase_1_tail_b);
        let head = self
            .frame_metadata
            .map(|m| self.frame_data[m.start as usize - 1]);

        let mut i = N;
        if self.drop_recurse(&mut i) {
            // TODO: Think about if this is valid under the assumption that there is only one root node.
            self.fill_recurse_simd(unsafe { NonZeroUsize::new_unchecked(i) });
        };

        Some((head, phase_2_head_a, phase_2_head_b))
    }
}

impl<'t, const N: usize> UnindexedProducer for Stack<'t, N, &'t AtomicBool> {
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
                let start = self.get_frame_metadata_i(i).start;
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

            let mut new_frame_data = Vec::with_capacity(Self::FRAME_DATA_CAP);
            new_frame_data.extend_from_slice(&self.frame_data[0..frame_split]);

            for _ in 0..frame_split - frame_start {
                self.frame_data.remove(frame_start);
            }
            for x in 0..N {
                if self.frame_metadata[x].start > frame_start as u16 {
                    self.frame_metadata[x].start -= (frame_split - frame_start) as u16;
                }
            }

            let mut new_stack = Stack {
                tables: self.tables,
                table_offsets: self.table_offsets,
                cancel: self.cancel,
                frame_metadata: self.frame_metadata,
                frame_data: new_frame_data,
            };

            match NonZeroUsize::new(i) {
                Some(i) => new_stack.fill_recurse_simd(i),
                None => new_stack.fill_recurse_no_simd(i),
            }

            if !new_stack.frame_data.is_empty() {
                break (self, Some(new_stack));
            };
        }
    }

    fn fold_with<F>(self, mut folder: F) -> F
    where
        F: rayon::iter::plumbing::Folder<Self::Item>,
    {
        let cancel = self.cancel;
        for item in self {
            folder = folder.consume(item);
            if folder.full() || cancel.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
        }
        folder
    }
}

impl<'t, const N: usize> ParallelIterator for Stack<'t, N, &'t AtomicBool> {
    type Item = ([Phase1Node; N], Phase2Node, Phase2Node);

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
        let table_offsets = TableOffsets::new(&tables);
        let cube = cube![R U Rp Up];
        let stack = all_domino_reductions::<0>(cube, &tables, &table_offsets).collect_vec();

        println!("{stack:#?}");

        Ok(())
    }

    #[test]
    fn domino_reduce_test_iter_2() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_ref = &tables;
        let table_offsets = TableOffsets::new(&tables);
        let cube = cube![R U Rp Up];
        // let cube = cube![D R2 L];
        let res = move |path: &[Phase1Node], last: &Phase2Node| {
            let cubes = path
                .into_iter()
                .map(|x| x.into_cube(table_ref))
                .chain(Some(last.into_cube(table_ref)));

            move_resolver_multi_dimension_domino(cube, cubes)
        };
        let stack = all_domino_reductions::<2>(cube, &tables, &table_offsets);
        stack.for_each(|(path, last_a, last_b)| {
            println!("{:?} {:?}", res(&path, &last_a), res(&path, &last_b));
        });
        let stack = all_domino_reductions::<3>(cube, &tables, &table_offsets);
        stack.for_each(|(path, last_a, last_b)| {
            println!("{:?} {:?}", res(&path, &last_a), res(&path, &last_b));
        });
        let stack = all_domino_reductions::<4>(cube, &tables, &table_offsets);
        stack.for_each(|(path, last_a, last_b)| {
            println!("{:?} {:?}", res(&path, &last_a), res(&path, &last_b));
        });
        let stack = all_domino_reductions::<5>(cube, &tables, &table_offsets);
        stack.for_each(|(path, last_a, last_b)| {
            println!("{:?} {:?}", res(&path, &last_a), res(&path, &last_b));
        });

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2_single() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_offsets = TableOffsets::new(&tables);

        let stack = all_domino_reductions::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &table_offsets,
        );

        println!("{:?}", stack.count());

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2_par() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_offsets = TableOffsets::new(&tables);

        let cancel = AtomicBool::new(false);

        let stack = all_domino_reductions_par::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &table_offsets,
            &cancel,
        );

        println!("{:?}", stack.count());

        Ok(())
    }

    #[test]
    fn domino_reduction_length_chart() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_offsets = TableOffsets::new(&tables);

        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        let cancel = AtomicBool::new(false);

        println!(
            "0: {}",
            all_domino_reductions_par::<0>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "1: {}",
            all_domino_reductions_par::<1>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "2: {}",
            all_domino_reductions_par::<2>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "3: {}",
            all_domino_reductions_par::<3>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "4: {}",
            all_domino_reductions_par::<4>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "5: {}",
            all_domino_reductions_par::<5>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "6: {}",
            all_domino_reductions_par::<6>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "7: {}",
            all_domino_reductions_par::<7>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "8: {}",
            all_domino_reductions_par::<8>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "9: {}",
            all_domino_reductions_par::<9>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "10: {}",
            all_domino_reductions_par::<10>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "11: {}",
            all_domino_reductions_par::<11>(cube, &tables, &table_offsets, &cancel).count()
        );
        println!(
            "12: {}",
            all_domino_reductions_par::<12>(cube, &tables, &table_offsets, &cancel).count()
        );

        Ok(())
    }
    // {(12, 19), (1, 16), (5, 20), (3, 11), (2, 9), (4, 16), (11, 20), (2, 15), (6, 15), (8, 12), (8, 19), (5, 11), (2, 16), (4, 15), (7, 14), (10, 13), (3, 15), (9, 10), (8, 8), (10, 19), (8, 13), (3, 8), (2, 3), (10, 10), (10, 14), (8, 18), (7, 20), (1, 14), (7, 16), (8, 15), (6, 14), (6, 13), (0, 6), (2, 8), (5, 9), (9, 16), (4, 18), (2, 20), (5, 18), (1, 20), (5, 13), (7, 12), (0, 1), (12, 14), (3, 10), (4, 7), (2, 10), (9, 17), (4, 20), (3, 13), (3, 18), (2, 6), (1, 17), (1, 9), (2, 5), (2, 19), (7, 17), (7, 13), (7, 19), (7, 11), (12, 18), (9, 18), (11, 18), (4, 17), (6, 18), (6, 7), (3, 9), (8, 10), (10, 15), (9, 20), (4, 5), (0, 16), (0, 7), (3, 3), (0, 11), (4, 19), (0, 13), (1, 10), (6, 6), (3, 20), (12, 16), (1, 12), (11, 12), (6, 8), (9, 15), (9, 14), (5, 6), (6, 19), (10, 20), (2, 12), (3, 16), (10, 17), (9, 9), (8, 11), (1, 4), (10, 18), (9, 13), (0, 9), (7, 18), (5, 10), (12, 15), (1, 15), (11, 14), (12, 12), (11, 19), (3, 17), (0, 12), (4, 6), (6, 9), (3, 19), (4, 4), (6, 12), (1, 8), (1, 11), (0, 5), (6, 17), (5, 12), (0, 8), (12, 20), (0, 17), (3, 6), (12, 17), (2, 18), (5, 8), (6, 11), (5, 17), (2, 13), (11, 17), (4, 13), (8, 20), (3, 14), (9, 19), (5, 19), (6, 16), (11, 11), (0, 0), (6, 20), (11, 16), (7, 7), (0, 15), (5, 16), (7, 8), (0, 10), (10, 16), (5, 7), (7, 10), (0, 14), (2, 4), (8, 9), (12, 13), (0, 20), (10, 12), (4, 10), (5, 15), (4, 9), (1, 19), (9, 12), (2, 2), (11, 13), (7, 15), (6, 10), (3, 12), (0, 18), (1, 1), (8, 14), (10, 11), (4, 12), (4, 14), (4, 8), (5, 14), (2, 7), (1, 18), (3, 5), (7, 9), (3, 4), (2, 17), (2, 11), (2, 14), (5, 5), (1, 7), (8, 17), (8, 16), (11, 15), (3, 7), (0, 19), (4, 11), (1, 13), (1, 6), (1, 5), (9, 11)}
}

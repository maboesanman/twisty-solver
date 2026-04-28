use std::{
    num::{NonZeroU8, NonZeroUsize},
    sync::{Arc, atomic::AtomicBool},
};

use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator, ParallelIterator,
    plumbing::{UnindexedConsumer, UnindexedProducer, bridge_unindexed},
};

use crate::{
    cube_ops::{cube_sym::CubeSymmetry, repr_cube::ReprCube},
    kociemba::{
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
pub fn all_domino_reductions<const N: usize>(
    cube: ReprCube,
    tables: &Tables,
) -> impl Iterator<Item = ([Phase1Node; N], Phase2Node)> {
    Stack::<_, _>::new(cube, tables, ()).into_iter().flatten()
}

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
///
/// if S is false, the move sequence is not allowed to be domino reduced at any time before the final state.
/// if S is true, only the second-to-last state is prevented from being domino reduced
pub fn all_domino_reductions_par<'a, const N: usize>(
    cube: ReprCube,
    tables: &'a Tables,
    cancel: &'a AtomicBool,
) -> impl 'a + ParallelIterator<Item = ([Phase1Node; N], Phase2Node)> {
    Stack::<'a, N, &'a AtomicBool>::new(cube, tables, cancel)
        .into_par_iter()
        .flatten()
}

// N is number of moves
// this doesn't work if the cube is already domino reduced.
#[derive(Debug, Clone)]
struct Stack<'t, const N: usize, C> {
    tables: &'t Tables,
    table_offsets: Arc<TableOffsets<'t>>,

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

    pub fn new(cube: ReprCube, tables: &'t Tables, cancel: C) -> Vec<Self> {
        let mut options =
            [0, 1, 2].map(|x| Phase1Node::from_cube(cube.conjugate(CubeSymmetry(x << 4)), tables));
        options.sort_by_key(|n| n.distance_heuristic(tables));
        options
            .into_iter()
            .map(|node| Self::new_inner(node, tables, cancel.clone()))
            .collect_vec()
    }

    fn new_inner(start: Phase1Node, tables: &'t Tables, cancel: C) -> Self {
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
            table_offsets: Arc::new(TableOffsets::new(tables)),
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
                &self.table_offsets,
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
    type Item = ([Phase1Node; N], Phase2Node);

    fn next(&mut self) -> Option<Self::Item> {
        let phase_1_tail = self.frame_data.pop()?;
        let phase_2_head = Phase2Node::from_phase_1_node(phase_1_tail);
        let head = self
            .frame_metadata
            .map(|m| self.frame_data[m.start as usize - 1]);

        let mut i = N;
        if self.drop_recurse(&mut i) {
            // TODO: Think about if this is valid under the assumption that there is only one root node.
            self.fill_recurse_simd(unsafe { NonZeroUsize::new_unchecked(i) });
        };

        Some((head, phase_2_head))
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
                table_offsets: self.table_offsets.clone(),
                cancel: self.cancel,
                frame_metadata: self.frame_metadata,
                frame_data: new_frame_data,
            };

            new_stack.fill_recurse_no_simd(i);

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
        let stack = all_domino_reductions::<0>(cube, &tables).collect_vec();

        println!("{stack:#?}");

        Ok(())
    }

    #[test]
    fn domino_reduce_test_iter_2() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_ref = &tables;
        let cube = cube![R U Rp Up];
        // let cube = cube![D R2 L];
        let stack = all_domino_reductions::<3>(cube, &tables);
        let res = move |path: &[Phase1Node], last: &Phase2Node| {
            let cubes = path
                .into_iter()
                .map(|x| x.into_cube(table_ref))
                .chain(Some(last.into_cube(table_ref)));

            move_resolver_multi_dimension_domino(cube, cubes)
        };
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<2>(cube, &tables);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<3>(cube, &tables);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<4>(cube, &tables);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });
        let stack = all_domino_reductions::<5>(cube, &tables);
        stack.for_each(|(path, last)| {
            println!("{:?}", res(&path, &last));
        });

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2_single() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let stack = all_domino_reductions::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
        );

        println!("{:?}", stack.count());

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2_par() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let cancel = AtomicBool::new(false);

        let stack = all_domino_reductions_par::<11>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &cancel,
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
        let cancel = AtomicBool::new(false);

        println!(
            "0: {}",
            all_domino_reductions_par::<0>(cube, &tables, &cancel).count()
        );
        println!(
            "1: {}",
            all_domino_reductions_par::<1>(cube, &tables, &cancel).count()
        );
        println!(
            "2: {}",
            all_domino_reductions_par::<2>(cube, &tables, &cancel).count()
        );
        println!(
            "3: {}",
            all_domino_reductions_par::<3>(cube, &tables, &cancel).count()
        );
        println!(
            "4: {}",
            all_domino_reductions_par::<4>(cube, &tables, &cancel).count()
        );
        println!(
            "5: {}",
            all_domino_reductions_par::<5>(cube, &tables, &cancel).count()
        );
        println!(
            "6: {}",
            all_domino_reductions_par::<6>(cube, &tables, &cancel).count()
        );
        println!(
            "7: {}",
            all_domino_reductions_par::<7>(cube, &tables, &cancel).count()
        );
        println!(
            "8: {}",
            all_domino_reductions_par::<8>(cube, &tables, &cancel).count()
        );
        println!(
            "9: {}",
            all_domino_reductions_par::<9>(cube, &tables, &cancel).count()
        );
        println!(
            "10: {}",
            all_domino_reductions_par::<10>(cube, &tables, &cancel).count()
        );
        println!(
            "11: {}",
            all_domino_reductions_par::<11>(cube, &tables, &cancel).count()
        );
        println!(
            "12: {}",
            all_domino_reductions_par::<12>(cube, &tables, &cancel).count()
        );

        Ok(())
    }
}

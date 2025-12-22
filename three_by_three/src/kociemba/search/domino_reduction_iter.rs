use std::{num::NonZeroU8, sync::atomic::AtomicBool};

use arrayvec::ArrayVec;
use itertools::Itertools;
use rayon::iter::{
    ParallelIterator,
    plumbing::{UnindexedConsumer, UnindexedProducer, bridge_unindexed},
};

use crate::{
    cube_ops::{cube_prev_axis::CubePreviousAxis, cube_sym::CubeSymmetry, repr_cube::ReprCube},
    kociemba::search::phase_1_node::{Phase1FrameMetadata, Phase1Node},
    kociemba::tables::Tables,
};

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
///
/// if S is false, the move sequence is not allowed to be domino reduced at any time before the final state.
/// if S is true, only the second-to-last state is prevented from being domino reduced
pub fn all_domino_reductions<const N: usize>(
    cube: ReprCube,
    tables: &Tables,
) -> impl Iterator<Item = ([Phase1Node; N], Phase1Node)> {
    Stack::<_, _>::new(cube, tables, ())
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
) -> impl 'a + ParallelIterator<Item = ([Phase1Node; N], Phase1Node)> {
    Stack::<_, _>::new(cube, tables, cancel)
}

// N is number of moves
// this doesn't work if the cube is already domino reduced.
#[derive(Clone)]
struct Stack<'t, const N: usize, C> {
    tables: &'t Tables,
    cancel: C,

    frame_metadata: [FrameMetadata; N],

    // 3, 18, 15 * (N - 1) at most. when N=20, 306 at most
    frame_data: Vec<Phase1Node>,
}

#[derive(Clone, Copy, Debug)]
pub struct FrameMetadata {
    // the offset into the frame_data buffer that this begins.
    // note that this can never be empty
    start: u16,

    // the bounds on the actual distance we know right now.
    min_distance: u8,
    max_distance: u8,
}

impl FrameMetadata {
    const fn default_const() -> Self {
        Self {
            start: 0,
            min_distance: 0,
            max_distance: 20,
        }
    }
}

impl Default for FrameMetadata {
    fn default() -> Self {
        Self::default_const()
    }
}

impl<'t, const N: usize, C> Stack<'t, N, C> {
    const FRAME_DATA_CAP: usize = 3 + 15 * N;

    pub fn new(cube: ReprCube, tables: &'t Tables, cancel: C) -> Self {
        let options = (0..2)
            .map(|x| Phase1Node::from_cube(cube.conjugate(CubeSymmetry(x << 4)), tables))
            .collect();
        // let options = vec![Phase1Node::from_cube(cube, tables)];
        Self::new_from_frame_0(options, tables, cancel)
    }

    fn new_from_frame_0(starts: Vec<Phase1Node>, tables: &'t Tables, cancel: C) -> Self {
        let mut frame_data = starts;
        frame_data.reserve_exact(Self::FRAME_DATA_CAP);

        let mut stack = Self {
            tables,
            cancel,
            frame_data,
            frame_metadata: [const { FrameMetadata::default_const() }; _],
        };

        stack.fill_recurse(0);

        stack
    }

    /// drop the frame at the top (belonging to frame i)
    fn drop_recurse(&mut self, i: &mut usize) -> Option<()> {
        while self.get_frame_metadata_i(*i).start == self.frame_data.len() as u16 {
            self.frame_data.pop()?;
            *i -= 1;
        }

        Some(())
    }

    // recurse into frames
    fn fill_recurse(&mut self, i: usize) -> Option<()> {
        let mut i = i;
        while i < N {
            let last_data = self.frame_data.last()?;
            let last_frame = self.get_frame_metadata_i(i);

            let incoming = last_data.produce_next_nodes(
                last_frame.max_distance,
                unsafe { NonZeroU8::new_unchecked((N - i) as u8) },
                self.tables,
            );

            self.frame_metadata[i].start = self.frame_data.len() as u16;

            if let Some(incoming) = incoming {
                self.frame_metadata[i].max_distance = incoming.max_possible_distance;
                self.frame_data.extend(incoming.children);
            }

            i += 1;

            self.drop_recurse(&mut i);
        }

        Some(())
    }

    pub fn pretty_print(&self) {
        println!("=== STACK STATE ===");

        let default = FrameMetadata::default();

        println!("{:#?}", self.frame_metadata);
        println!(
            "{:#?}",
            self.frame_data
                .iter()
                .map(|n| format!(
                    "{}-{}",
                    n.edge_group_orient_combo.sym_coord.0, n.corner_orient_raw.0
                ))
                .collect_vec()
        );
    }

    fn get_frame_metadata_i(&self, i: usize) -> FrameMetadata {
        i.checked_sub(1)
            .map(|j| self.frame_metadata[j])
            .unwrap_or_default()
    }
}

impl<'t, const N: usize, C> Iterator for Stack<'t, N, C> {
    type Item = ([Phase1Node; N], Phase1Node);

    fn next(&mut self) -> Option<Self::Item> {
        let tail = self.frame_data.pop()?;
        let head = self
            .frame_metadata
            .map(|m| self.frame_data[m.start as usize - 1]);

        let mut i = N;
        self.drop_recurse(&mut i);
        self.fill_recurse(i);

        Some((head, tail))
    }
}

impl<'t, const N: usize> UnindexedProducer for Stack<'t, N, &'t AtomicBool> {
    type Item = <Self as Iterator>::Item;

    fn split(mut self) -> (Self, Option<Self>) {
        // we've already been exhausted. nothing to split
        if self.frame_data.is_empty() {
            return (self, None);
        }

        let mut i = 0;
        let (frame_start, frame_end) = loop {
            if i > N {
                return (self, None);
            }
            let mut start = self.get_frame_metadata_i(i).start;
            while self.frame_data[start as usize].skip {
                start += 1;
            }
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

        for node in self.frame_data[frame_start..frame_split].iter_mut() {
            node.skip = true;
        }

        let mut new_stack = Stack {
            tables: self.tables,
            cancel: self.cancel,
            frame_metadata: self.frame_metadata.clone(),
            frame_data: new_frame_data,
        };

        match new_stack.fill_recurse(i) {
            Some(()) => (self, Some(new_stack)),
            None => (self, None),
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
    type Item = ([Phase1Node; N], Phase1Node);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}

#[cfg(test)]
mod test {

    use crate::{cube, kociemba::search::move_resolver::{move_resolver, move_resolver_multi_dimension_domino}};

    use super::*;

    #[test]
    fn domino_reduce_test_iter_2() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table_ref = &tables;
        let cube = cube![R U Rp Up];
        // let cube = cube![D R2 L];
        let stack = all_domino_reductions::<1>(cube, &tables);
        let res = move |path: &[Phase1Node], last: &Phase1Node| {
            let cubes = path
                .into_iter()
                .chain(Some(last))
                .map(|x| x.into_cube(table_ref));

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

        let stack = all_domino_reductions::<10>(
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

        let stack = all_domino_reductions_par::<14>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &cancel,
        );

        println!("{:?}", stack.count());

        Ok(())
    }
}

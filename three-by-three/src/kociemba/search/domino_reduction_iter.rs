use std::sync::atomic::AtomicBool;

use arrayvec::ArrayVec;
use rayon::iter::{
    ParallelIterator,
    plumbing::{UnindexedConsumer, UnindexedProducer, bridge_unindexed},
};

use crate::{
    cube_ops::{cube_move::CubeMove, repr_cube::ReprCube},
    kociemba::coords::repr_coord::SymReducedRepr,
    tables::Tables,
};

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
///
/// if S is false, the move sequence is not allowed to be domino reduced at any time before the final state.
/// if S is true, only the second-to-last state is prevented from being domino reduced
pub fn all_domino_reductions<const N: usize, const S: bool>(
    cube: ReprCube,
    tables: &Tables,
) -> impl Iterator<Item = ([SymReducedRepr; 2], [SymReducedRepr; N])> {
    Stack::<_, S, _>::new(cube, tables, ())
}

/// returns all sequences of sym cubes which correspond with a
/// domino reduction of exactly N + 1 moves.
///
/// if S is false, the move sequence is not allowed to be domino reduced at any time before the final state.
/// if S is true, only the second-to-last state is prevented from being domino reduced
pub fn all_domino_reductions_par<'a, const N: usize, const S: bool>(
    cube: ReprCube,
    tables: &'a Tables,
    cancel: &'a AtomicBool,
) -> impl 'a + ParallelIterator<Item = ([SymReducedRepr; 2], [SymReducedRepr; N])> {
    Stack::<_, S, _>::new(cube, tables, cancel)
}

// N is number of moves - 1
// this doesn't work if the cube is already domino reduced.
#[derive(Clone)]
struct Stack<'t, const N: usize, const S: bool, C> {
    tables: &'t Tables,
    cancel: C,

    // one for each S_URF3 symmetry of the starting position
    frame_0: StackFrame<3, ()>,

    // the 18 possible moves from the starting position
    frame_1: StackFrame<18, CubeMove>,

    // the 15 possible moves from the previous position, because you can't turn the face you just turned.
    frames_after: [StackFrame<15, CubeMove>; N],
}

impl<'t, const N: usize, const S: bool, C> std::fmt::Debug for Stack<'t, N, S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Stack");
        dbg.field("frame_0", &self.frame_0);
        dbg.field("frame_1", &self.frame_1);
        for (i, frame) in self.frames_after.iter().enumerate() {
            dbg.field(&format!("frame_{}", i + 2), frame);
        }
        dbg.finish()
    }
}

#[derive(Clone, Debug, Default)]
struct StackFrame<const N: usize, M: Copy> {
    next_cubes: ArrayVec<NextCubes<M>, N>,
}

impl<const N: usize, M: Copy> StackFrame<N, M> {
    fn split(&mut self) -> Option<Self> {
        if self.next_cubes.len() <= 1 {
            return None;
        }

        let split_index = self.next_cubes.len().div_ceil(2);

        let mut new = self.next_cubes[split_index..].iter().copied().collect();
        self.next_cubes.truncate(split_index);

        core::mem::swap(&mut new, &mut self.next_cubes);

        Some(StackFrame { next_cubes: new })
    }
}

#[derive(Clone, Copy, Debug)]
struct NextCubes<M: Copy> {
    cube: SymReducedRepr,
    previous_move: M,
    domino_distance: u8,
}

impl<'t, const N: usize, const S: bool, C> Stack<'t, N, S, C> {
    pub fn new(cube: ReprCube, tables: &'t Tables, cancel: C) -> Self {
        let base = SymReducedRepr::from_cube(cube, tables);
        Self::new_from_frame_0([base].into_iter().collect(), tables, cancel)
    }

    fn new_from_frame_0(
        frame_0: ArrayVec<SymReducedRepr, 3>,
        tables: &'t Tables,
        cancel: C,
    ) -> Self {
        let next_cubes = frame_0
            .into_iter()
            .map(|cube| {
                let domino_distance = cube.prune_distance_phase_1(tables);

                NextCubes {
                    cube,
                    previous_move: (),
                    domino_distance,
                }
            })
            .collect();

        let frame_0 = StackFrame { next_cubes };

        let mut stack = Self {
            tables,
            cancel,
            frame_0,
            frame_1: StackFrame {
                next_cubes: ArrayVec::new(),
            },
            frames_after: [const {
                StackFrame {
                    next_cubes: ArrayVec::new_const(),
                }
            }; _],
        };

        stack.fill_empty_frames();

        stack
    }

    fn cube_child_filter(
        parent_dist: u8,
        parent_moves_remaining: u8,
        iter: impl IntoIterator<Item = (SymReducedRepr, CubeMove)>,
        tables: &Tables,
    ) -> impl Iterator<Item = NextCubes<CubeMove>> {
        let min_d = match parent_moves_remaining - 1 {
            0 => 0,
            1 => 1,
            _ => if S {
                parent_dist.saturating_sub(1)
            } else {
                parent_dist.saturating_sub(2) + 1
            }
        };
        let max_d = (parent_dist + 1).min(parent_moves_remaining - 1);

        iter.into_iter().filter_map(move |(cube, previous_move)| {
            let domino_distance = cube.prune_distance_phase_1(tables);

            if !(min_d..=max_d).contains(&domino_distance) {
                return None;
            }

            Some(NextCubes {
                cube,
                previous_move,
                domino_distance,
            })
        })
    }

    /// returns true if the parent has more siblings (didn't just pop the last item in the list)
    fn pop_parent_after_m(&mut self, i: usize) -> Option<bool> {
        let new_len = match i.checked_sub(1).map(|x| x.checked_sub(1)) {
            None => {
                self.frame_0.next_cubes.pop()?;
                self.frame_0.next_cubes.len()
            }
            Some(None) => {
                self.frame_1.next_cubes.pop()?;
                self.frame_1.next_cubes.len()
            }
            Some(Some(i)) => {
                if i > N {
                    panic!()
                }
                let frame = self.frames_after.get_mut(i)?;
                frame.next_cubes.pop()?;
                frame.next_cubes.len()
            }
        };
        Some(new_len > 0)
    }

    fn pop_parent_after_m_complete(&mut self, i: &mut usize) {
        loop {
            if self.pop_parent_after_m(*i).unwrap() {
                break;
            }

            if *i == 0 {
                break;
            }
            *i -= 1;
        }
    }

    /// returns true if any items were written to m
    fn set_frame_after_m_overwrite(&mut self, i: usize) -> bool {
        match i.checked_sub(1).map(|x| x.checked_sub(1)) {
            Some(x) => {
                let (parent, child_frame) = match x {
                    Some(i) => {
                        if i > N {
                            panic!()
                        }
                        let [prev, next] =
                            unsafe { self.frames_after.get_disjoint_unchecked_mut([i, i + 1]) };
                        (prev.next_cubes.last().unwrap(), next)
                    }
                    None => (
                        self.frame_1.next_cubes.last().unwrap(),
                        &mut self.frames_after[0],
                    ),
                };

                let prev_d = parent.domino_distance;
                let prev_m_remaining = (N + 1 - i) as u8;

                child_frame.next_cubes.truncate(0);
                child_frame.next_cubes.extend(Self::cube_child_filter(
                    prev_d,
                    prev_m_remaining,
                    parent
                        .cube
                        .partial_phase_1_neighbors(self.tables, parent.previous_move),
                    self.tables,
                ));
                !child_frame.next_cubes.is_empty()
            }
            None => {
                let parent = self.frame_0.next_cubes.last().unwrap();

                let prev_d = parent.domino_distance;
                let prev_m_remaining = (N + 1 - i) as u8;

                self.frame_1.next_cubes.truncate(0);
                self.frame_1.next_cubes.extend(Self::cube_child_filter(
                    prev_d,
                    prev_m_remaining,
                    parent.cube.full_phase_1_neighbors(self.tables),
                    self.tables,
                ));
                !self.frame_1.next_cubes.is_empty()
            }
        }
    }

    fn fill_empty_frames(&mut self) {
        let mut i = match self
            .frames_after
            .iter()
            .enumerate()
            .rev()
            .find(|(_, f)| !f.next_cubes.is_empty())
        {
            Some((i, _)) => i + 2,
            None => {
                if self.frame_1.next_cubes.is_empty() {
                    if self.frame_0.next_cubes.is_empty() {
                        return;
                    }
                    0
                } else {
                    1
                }
            }
        };

        while i < N + 1 {
            // fill the frames, and note if there were any items that were written.
            let any_items_written = self.set_frame_after_m_overwrite(i);
            if any_items_written {
                i += 1;
                continue;
            }

            self.pop_parent_after_m_complete(&mut i);

            if self.frame_0.next_cubes.is_empty() {
                return;
            }
        }
    }
}

impl<'t, const N: usize, const S: bool, C> Iterator for Stack<'t, N, S, C> {
    type Item = ([SymReducedRepr; 2], [SymReducedRepr; N]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame_0.next_cubes.is_empty() {
            return None;
        }
        let return_value_head = [
            self.frame_0.next_cubes.last().unwrap().cube,
            self.frame_1.next_cubes.last().unwrap().cube,
        ];
        let return_value_tail: [SymReducedRepr; N] =
            std::array::from_fn(|i| self.frames_after[i].next_cubes.last().unwrap().cube);

        let mut i = N + 1;
        self.pop_parent_after_m_complete(&mut i);
        self.fill_empty_frames();

        Some((return_value_head, return_value_tail))
    }
}

impl<'t, const N: usize, const S: bool> UnindexedProducer for Stack<'t, N, S, &'t AtomicBool> {
    type Item = <Self as Iterator>::Item;

    fn split(mut self) -> (Self, Option<Self>) {
        if let Some(other) = self.frame_0.split() {
            let mut new = Self {
                cancel: self.cancel,
                tables: self.tables,
                frame_0: other,
                frame_1: StackFrame {
                    next_cubes: ArrayVec::new(),
                },
                frames_after: [const {
                    StackFrame {
                        next_cubes: ArrayVec::new_const(),
                    }
                }; _],
            };
            new.fill_empty_frames();
            return (self, Some(new));
        }

        if let Some(other) = self.frame_1.split() {
            let mut new = Self {
                cancel: self.cancel,
                tables: self.tables,
                frame_0: self.frame_0.clone(),
                frame_1: other,
                frames_after: [const {
                    StackFrame {
                        next_cubes: ArrayVec::new_const(),
                    }
                }; _],
            };
            new.fill_empty_frames();
            return (self, Some(new));
        }

        for (i, frame) in self.frames_after.iter_mut().enumerate() {
            if let Some(other) = frame.split() {
                let mut new = Self {
                    cancel: self.cancel,
                    tables: self.tables,
                    frame_0: self.frame_0.clone(),
                    frame_1: self.frame_1.clone(),
                    frames_after: [const {
                        StackFrame {
                            next_cubes: ArrayVec::new_const(),
                        }
                    }; _],
                };
                for j in 0..i {
                    new.frames_after[j] = self.frames_after[j].clone();
                }
                new.frames_after[i] = other;
                new.fill_empty_frames();
                return (self, Some(new));
            }
        }

        (self, None)
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

impl<'t, const N: usize, const S: bool> ParallelIterator for Stack<'t, N, S, &'t AtomicBool> {
    type Item = ([SymReducedRepr; 2], [SymReducedRepr; N]);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}

#[cfg(test)]
mod test {

    use crate::cube;

    use super::*;

    #[test]
    fn domino_reduce_test_iter_2() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let stack = all_domino_reductions::<5, false>(cube![R U Rp Up], &tables);

        println!("{:?}", stack.count());
        // loop {
        //     if stack.next().is_none() {
        //         break
        //     };
        //     println!("{stack:?}");
        //     // println!("{x:?} ");
        // }

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip_2() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let stack = all_domino_reductions::<9, false>(
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

        let stack = all_domino_reductions_par::<9, false>(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
            &cancel,
        );

        println!("{:?}", stack.count());

        Ok(())
    }
}

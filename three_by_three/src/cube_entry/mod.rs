use std::collections::HashSet;

use arrayvec::ArrayVec;
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    U,
    D,
    F,
    B,
    R,
    L,
}

/// the colors of each corner, starting with U/D and going clockwise
const CORNER_COLORS: [[Color; 3]; 8] = [
    [Color::U, Color::R, Color::F],
    [Color::U, Color::F, Color::L],
    [Color::U, Color::B, Color::R],
    [Color::U, Color::L, Color::B],
    [Color::D, Color::F, Color::R],
    [Color::D, Color::L, Color::F],
    [Color::D, Color::R, Color::B],
    [Color::D, Color::B, Color::L],
];

/// the colors of each edge, starting with the "oriented" face (U/D, or F/B if there a U/D available)
const EDGE_COLORS: [[Color; 2]; 12] = [
    [Color::U, Color::F],
    [Color::U, Color::B],
    [Color::U, Color::R],
    [Color::U, Color::L],
    [Color::D, Color::F],
    [Color::D, Color::B],
    [Color::D, Color::R],
    [Color::D, Color::L],
    [Color::F, Color::R],
    [Color::F, Color::L],
    [Color::B, Color::R],
    [Color::B, Color::L],
];

/// the indices in the color array of each corner slot, starting with U/D and going clockwise
const CORNER_INDICES: [[usize; 3]; 8] = [
    [7, 24, 18],
    [5, 16, 10],
    [2, 32, 26],
    [0, 8, 34],
    [42, 23, 29],
    [40, 15, 21],
    [47, 31, 37],
    [45, 39, 13],
];

/// the indices in the color array of each edge slot, starting with the "oriented" face (U/D, or F/B if there a U/D available)
const EDGE_INDICES: [[usize; 2]; 12] = [
    [6, 17],
    [1, 33],
    [4, 25],
    [3, 9],
    [41, 22],
    [46, 38],
    [44, 30],
    [43, 13],
    [20, 27],
    [19, 12],
    [35, 28],
    [36, 11],
];

/// the colors that have been locked in, as an array
pub struct CubeEntry([Option<Color>; 48]);

impl Default for CubeEntry {
    fn default() -> Self {
        Self([const { None }; 48])
    }
}

impl CubeEntry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_options(&self, i: usize) -> Option<ArrayVec<Color, 6>> {
        let mut corner_slot_options: [ArrayVec<(usize, u8), 24>; 8] = [const { ArrayVec::new_const() }; 8];

        for (slot_i, slot) in CORNER_INDICES.iter().enumerate() {
            let slot = slot.map(|x| self.0[x]);
            for (corner, colors) in CORNER_COLORS.iter().enumerate() {
                let mut colors = colors.clone();
                (0..3).into_iter().filter(|orient| {
                    colors.rotate_left(1);
                    slot.iter().zip(colors).all(|(slot, piece)| {
                        match slot {
                            Some(c) => piece == *c,
                            None => true,
                        }
                    })
                }).for_each(|orient| {
                    corner_slot_options[slot_i].push((corner, orient))
                });
            }
        }
        hone(&mut corner_slot_options);
        println!("{corner_slot_options:?}");

        let mut edge_slot_options: [ArrayVec<(usize, u8), 24>; 8] = [const { ArrayVec::new_const() }; 8];

        for (slot_i, slot) in EDGE_INDICES.iter().enumerate() {
            let slot = slot.map(|x| self.0[x]);
            for (edge, colors) in EDGE_COLORS.iter().enumerate() {
                let mut colors = colors.clone();
                (0..2).into_iter().filter(|orient| {
                    colors.rotate_left(1);
                    slot.iter().zip(colors).all(|(slot, piece)| {
                        match slot {
                            Some(c) => piece == *c,
                            None => true,
                        }
                    })
                }).for_each(|orient| {
                    edge_slot_options[slot_i].push((edge, orient))
                });
            }
        }
        hone(&mut edge_slot_options);
        println!("{edge_slot_options:?}");

        todo!()
    }

    pub fn set_color(&mut self, i: usize, color: Color) -> Result<bool, ()> {
        todo!()
    }

    pub fn is_complete(&self) -> bool {
        todo!()
    }
}

fn hone<const N: usize>(slot_options: &mut [ArrayVec<(usize, u8), 24>; N]) {
    let mut focus_columns = const {
        let mut indices = [0; N];
        let mut i = 0;
        while i < N {
            indices[i] = i;
            i += 1;
        }
        indices
    };

    hone_inner(slot_options, &mut focus_columns);
}

fn hone_inner<const N: usize>(slot_options: &mut [ArrayVec<(usize, u8), 24>; N], focus_columns: &mut [usize]) {
    if focus_columns.len() == 1 {
        return;
    }
    let naked_group = (1..focus_columns.len()).flat_map(|k| focus_columns.iter().copied().combinations(k)).filter_map(|slots| {
        let set = slots.iter().flat_map(|x| (&slot_options[*x]).into_iter().map(|(corner, _orient)| *corner)).collect::<HashSet<_>>();
        
        if set.len() == slots.len() {
            Some((slots, set))
        } else {
            None
        }
    }).next();

    println!("{naked_group:?}");

    let (naked_group, to_remove) = match naked_group {
        Some(x) => x,
        None => return,
    };

    focus_columns.sort_by_key(|x| !naked_group.contains(x));
    let i = focus_columns.partition_point(|x| naked_group.contains(x));
    let (left, right) = focus_columns.split_at_mut(i);

    for i in right.iter() {
        slot_options[*i].retain(|(x, _)| !to_remove.contains(x));
    }

    hone_inner(slot_options, left);
    hone_inner(slot_options, right);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_entry() {
        let mut entry = CubeEntry::new();

        entry.0[16] = Some(Color::F);
        entry.0[18] = Some(Color::R);
        entry.0[24] = Some(Color::B);
        entry.0[29] = Some(Color::R);
        entry.0[31] = Some(Color::R);
        entry.0[37] = Some(Color::B);

        entry.0[17] = Some(Color::U);
        entry.0[19] = Some(Color::U);
        entry.0[20] = Some(Color::U);
        entry.0[22] = Some(Color::U);
        

        entry.get_options(15);

    }
}
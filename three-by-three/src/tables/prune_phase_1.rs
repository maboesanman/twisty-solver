use bitvec::field::BitField;
use bitvec::view::BitView;
use num_integer::Integer;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering, fence};

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::cube_ops::cube_move::CubeMove;
use crate::cube_ops::cube_sym::DominoSymmetry;
use crate::kociemba::coords::coords::{CornerOrientRawCoord, EdgeGroupOrientSymCoord};
use crate::kociemba::coords::edge_group_orient_combo_coord::EdgeGroupOrientComboCoord;
use crate::tables::Tables;
use crate::tables::lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;

use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_ENTRY_COUNT: usize = 64430 * 2187;
const WORKING_TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 2;
const TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT * 3 / 8 + 1;
const FILE_CHECKSUM: u32 = 1885600379;

static PRUNE_TABLE_SHORTCUTS: phf::Map<u32, u8> = phf::phf_map! {
    0 => 0,
    41357406 => 1,
    32141373 | 39257866 | 48556106 | 118414733 | 130390793 => 2,
    408255 | 433764 | 4930976 | 5516352 | 6566193 | 16395231 | 16652328 | 20703858 | 20995503 | 23625519 | 26541627 | 32141454 | 34471516 | 37279792 | 37280838 | 44006902 | 50935562 | 50936775 | 53245466 | 54292445 | 55554938 | 55555476 | 66751874 | 83011895 | 91470877 | 96019853 | 100376358 | 100498089 | 109731859 | 109885026 | 113839775 | 113865272 | 113935017 | 113944740 | 117845915 | 118408244 | 122901128 | 126698705 | 130712282 | 131124248 | 131124491 | 132874388 | 135604169 | 139608511 => 3,
    7667 | 7694 | 7747 | 7907 | 7987 | 8014 | 8387 | 8414 | 8494 | 8654 | 8707 | 8734 | 27665 | 104647 | 203057 | 205297 | 240241 | 266048 | 266480 | 364228 | 364415 | 364948 | 365188 | 375812 | 375908 | 401594 | 405797 | 405805 | 405961 | 405962 | 406013 | 406016 | 406021 | 406024 | 406447 | 406448 | 406453 | 406454 | 406502 | 406508 | 406669 | 406672 | 4737001 | 8795353 | 11112304 | 11114059 | 11295526 | 11704063 | 23081477 | 25359691 | 25381561 | 36885593 | 37156796 | 37921766 | 38014100 | 38139251 | 39133817 | 39133924 | 40371740 | 40957802 | 41259608 | 41264062 | 41268356 | 41312149 | 41653348 | 41697008 | 50690165 | 125642178 | 126080064 | 127558983 | 127998570 | 130397688 | 130631697 | 130761946 | 130991601 | 131004019 | 131004381 | 131004703 | 131011264 | 131049414 | 131049417 | 131049433 | 131049436 | 131049604 | 131049624 | 131049927 | 131050066 | 131050188 | 131050305 | 131050630 | 131050650 | 131050818 | 131050840 | 131069629 | 131069749 | 131069871 | 131069991 | 131080051 | 131080923 | 131080942 | 131091483 | 131113353 | 131113747 | 131115556 | 131115918 | 131117211 | 131120337 | 131120659 | 131121585 | 131121586 | 131121588 | 131121604 | 131121606 | 131121630 | 131121750 | 131121774 | 131121775 | 131122072 | 131122098 | 131122237 | 131122314 | 131122359 | 131122476 | 131122800 | 131122821 | 131123011 | 135843340 | 139075704 | 139662030 => 12,
};

struct WorkingTable<'a>(&'a [AtomicU8]);

impl<'a> WorkingTable<'a> {
    fn visited(&self, i: usize) -> bool {
        let j = i % 2;
        let i = i / 2;

        let atomic = &self.0[i];

        let shift = j * 4;
        let mask = 0b1111 << shift;

        atomic.load(Ordering::Relaxed) & mask != 0
    }

    fn visited_at_level(&self, i: usize, level: u8) -> bool {
        let j = i % 2;
        let i = i / 2;

        let atomic = &self.0[i];

        let shift = j * 4;
        let mask = 0b1111 << shift;

        let expected_residue = (level + 1) << shift;

        atomic.load(Ordering::Relaxed) & mask == expected_residue
    }

    /// write to the table. returns true if write was successful and the moves from here should be handled.
    fn write(&self, i: usize, level: u8) -> bool {
        let j = i % 2;
        let i = i / 2;

        let atomic = &self.0[i];

        let shift = j * 4;
        let mask = 0b1111 << shift;
        let bits = (level + 1) << shift;

        // try once: if the slot is still 00, set it to `bits`, else bail
        atomic
            .fetch_update(
                Ordering::AcqRel,  // on success: Acquire+Release
                Ordering::Acquire, // on failure: Acquire
                |old| {
                    if old & mask != 0 {
                        None // someone else already wrote non-zero
                    } else {
                        Some(old | bits) // set the two bits
                    }
                },
            )
            .is_ok()
    }

    fn read(&self, i: usize) -> u8 {
        let j = i % 2;
        let i = i / 2;

        let atomic = &self.0[i];
        let value = atomic.load(Ordering::Relaxed);

        let shift = j * 4;
        ((value >> shift) & 0b1111) - 1
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PartialPhase1 {
    pub edge_group_orient_combo_coord: EdgeGroupOrientComboCoord,
    pub corner_orient_raw_coord: CornerOrientRawCoord,
}

impl PartialPhase1 {
    pub fn from_index(index: usize) -> Self {
        let (a, b) = index.div_rem(&2187);
        Self {
            edge_group_orient_combo_coord: EdgeGroupOrientComboCoord {
                sym_coord: EdgeGroupOrientSymCoord(a as u16),
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            corner_orient_raw_coord: CornerOrientRawCoord(b as u16),
        }
    }

    pub fn into_index(self) -> usize {
        debug_assert_eq!(
            self.edge_group_orient_combo_coord.domino_conjugation,
            DominoSymmetry::IDENTITY
        );
        let a = self.edge_group_orient_combo_coord.sym_coord.0;
        let b = self.corner_orient_raw_coord.0;

        (a as usize) * 2187 + (b as usize)
    }

    pub fn apply_cube_move(self, tables: &Tables, cube_move: CubeMove) -> Self {
        let edge_group_orient_combo_coord = self
            .edge_group_orient_combo_coord
            .apply_cube_move(tables, cube_move);

        let corner_orient_raw_coord = tables
            .move_raw_corner_orient
            .apply_cube_move(self.corner_orient_raw_coord, cube_move);

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
        }
    }

    pub fn domino_conjugate(self, tables: &Tables, sym: DominoSymmetry) -> Self {
        if sym == DominoSymmetry::IDENTITY {
            return self;
        }

        let edge_group_orient_combo_coord =
            self.edge_group_orient_combo_coord.domino_conjugate(sym);

        let corner_orient_raw_coord = tables
            .move_raw_corner_orient
            .domino_conjugate(self.corner_orient_raw_coord, sym);

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
        }
    }

    pub fn normalize(self, tables: &Tables) -> impl IntoIterator<Item = Self> {
        let rep = self.domino_conjugate(
            tables,
            self.edge_group_orient_combo_coord.domino_conjugation,
        );

        LookupSymEdgeGroupOrientTable::get_all_stabilizing_conjugations(
            rep.edge_group_orient_combo_coord.sym_coord,
        )
        .into_iter()
        .map(move |sym| PartialPhase1 {
            edge_group_orient_combo_coord: rep.edge_group_orient_combo_coord,
            corner_orient_raw_coord: tables
                .move_raw_corner_orient
                .domino_conjugate(rep.corner_orient_raw_coord, sym),
        })
    }

    pub fn single_normalize(self, tables: &Tables) -> Self {
        self.domino_conjugate(
            tables,
            self.edge_group_orient_combo_coord.domino_conjugation,
        )
    }
}

pub fn top_down_adjacent(index: usize, tables: &Tables) -> impl IntoIterator<Item = usize> {
    let start = PartialPhase1::from_index(index);

    CubeMove::all_iter()
        .flat_map(move |cube_move| start.apply_cube_move(tables, cube_move).normalize(tables))
        .map(PartialPhase1::into_index)
}

pub fn bottom_up_adjacent(index: usize, tables: &Tables) -> impl IntoIterator<Item = usize> {
    let start = PartialPhase1::from_index(index);

    CubeMove::all_iter()
        .map(move |cube_move| {
            start
                .apply_cube_move(tables, cube_move)
                .single_normalize(tables)
        })
        .map(PartialPhase1::into_index)
}

pub struct PrunePhase1Table(Mmap);

impl PrunePhase1Table {
    pub fn get_value(
        &self,
        edge_group_orient_sym_coord: EdgeGroupOrientSymCoord,
        corner_orient_raw_coord: CornerOrientRawCoord,
    ) -> u8 {
        let partial = PartialPhase1 {
            edge_group_orient_combo_coord: EdgeGroupOrientComboCoord {
                sym_coord: edge_group_orient_sym_coord,
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            corner_orient_raw_coord,
        };
        let i = partial.into_index();

        PRUNE_TABLE_SHORTCUTS
            .get(&(i as u32))
            .copied()
            .unwrap_or_else(|| {
                let bits = self.0.view_bits::<bitvec::order::Lsb0>();

                let start = i * 3;
                let chunk = &bits[start..start + 3];
                4 + (chunk.load_le::<u8>() & 0b111)
            })
    }

    fn generate(buffer: &mut [u8], tables: &Tables) {
        let mut working_buffer = vec![0u8; WORKING_TABLE_SIZE_BYTES];

        let atom = unsafe { as_atomic_u8_slice(&mut working_buffer) };
        let working = WorkingTable(atom);

        let special_cases = [0, 1, 2, 3, 12];

        let mut shortcut_map = HashMap::new();

        // initial state
        let root = 0;

        working.write(root, 0);

        let mut frontier = vec![root];
        let mut frontier_level = 0u8; // real level, not mod-3
        let mut total_visited = 1;

        while !frontier.is_empty() {
            if special_cases.contains(&frontier_level) {
                shortcut_map.insert(frontier_level, frontier.clone());
            }
            let next_level = frontier_level + 1;
            println!("level: {:?} frontier: {:?}", frontier_level, frontier.len());
            let unvisited = TABLE_ENTRY_COUNT - total_visited;
            let _use_bottom_up = frontier.len() * /* degree of graph */ 18 > unvisited; // cheap heuristic

            // TODO: fix top down search so this is dramatically more efficient
            let use_bottom_up = true;

            let next = if !use_bottom_up {
                /* ---------- top-down ---------- */
                frontier
                    .par_iter()
                    .flat_map_iter(|&v| top_down_adjacent(v, tables))
                    .filter_map(|nbr| {
                        if working.write(nbr, next_level) {
                            Some(nbr)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                /* ---------- bottom-up ---------- */
                (0..TABLE_ENTRY_COUNT)
                    .into_par_iter()
                    .filter_map(|v| {
                        if working.visited(v) {
                            return None; // already discovered
                        }
                        for nbr in bottom_up_adjacent(v, tables) {
                            if working.visited_at_level(nbr, frontier_level) {
                                if working.write(v, next_level) {
                                    return Some(v);
                                } else {
                                    return None;
                                }
                            }
                        }
                        None
                    })
                    .collect()
            };

            frontier = next;

            fence(Ordering::SeqCst);
            frontier_level += 1;
            total_visited += frontier.len();
        }

        // let mut out_string = "static PRUNE_TABLE_SHORTCUTS: phf::Map<u32, u8> = phf::phf_map! {\n".to_string();
        // for (k, v) in shortcut_map {
        //     out_string.push_str(&format!("    {} => {},\n", v.into_iter().map(|x| format!("{x}")).join(" | "), k));
        // }
        // out_string.push_str("};");

        // println!("{out_string}");

        let bits = buffer.view_bits_mut::<bitvec::order::Lsb0>();

        let mut set = |i: usize, val: u8| {
            assert!(val < 8);
            let start = i * 3;
            bits[start..start + 3].store_le::<u8>(val);
        };

        for i in 0..TABLE_ENTRY_COUNT {
            let x = working.read(i);
            (set)(i, x.clamp(4, 11) - 4);
        }

        println!("{:?}", frontier.len());
        println!("{frontier_level:?}");
    }

    pub fn load<P: AsRef<Path>>(path: P, tables: &Tables) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, tables)
        })
        .map(Self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate() -> anyhow::Result<()> {
        let _tables = Tables::new("tables")?;

        Ok(())
    }
}

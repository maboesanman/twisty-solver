use bitvec::field::BitField;
use bitvec::view::BitView;
use num_integer::Integer;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering, fence};

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::cube_ops::coords::{CornerPermSymCoord, UDEdgePermRawCoord};
use crate::cube_ops::corner_perm_combo_coord::CornerPermComboCoord;
use crate::cube_ops::cube_move::DominoMove;
use crate::cube_ops::cube_sym::DominoSymmetry;
use crate::tables::Tables;
use crate::tables::lookup_sym_corner_perm::LookupSymCornerPermTable;

use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_ENTRY_COUNT: usize = 2768 * 40320;
const WORKING_TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT;
const TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 2;
const FILE_CHECKSUM: u32 = 796939987;

static PRUNE_TABLE_SHORTCUTS: phf::Map<u32, u8> = phf::phf_map! {
    241926 | 282257 | 12902505 => 1,
    0 => 0,
    7177047 | 11733190 | 12418671 | 12781540 | 32742948 | 83393416 | 83433727 | 83474062 | 106208046 | 110799952 => 2,
};

struct WorkingTable<'a>(&'a [AtomicU8]);

impl<'a> WorkingTable<'a> {
    fn visited(&self, i: usize) -> bool {
        let atomic = &self.0[i];
        atomic.load(Ordering::Relaxed) != 0
    }

    fn visited_at_level(&self, i: usize, level: u8) -> bool {
        let atomic = &self.0[i];
        let expected_residue = level + 1;

        atomic.load(Ordering::Relaxed) == expected_residue
    }

    /// write to the table. returns true if write was successful and the moves from here should be handled.
    fn write(&self, i: usize, level: u8) -> bool {
        let atomic = &self.0[i];
        let bits = level + 1;

        // try once: if the slot is still 00, set it to `bits`, else bail
        atomic
            .fetch_update(
                Ordering::AcqRel,  // on success: Acquire+Release
                Ordering::Acquire, // on failure: Acquire
                |old| {
                    if old != 0 {
                        None // someone else already wrote non-zero
                    } else {
                        Some(bits) // set the two bits
                    }
                },
            )
            .is_ok()
    }

    fn read(&self, i: usize) -> u8 {
        let atomic = &self.0[i];
        let value = atomic.load(Ordering::Relaxed);

        value - 1
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PartialPhase2 {
    pub corner_perm_combo_coord: CornerPermComboCoord,
    pub ud_edge_perm_raw_coord: UDEdgePermRawCoord,
}

impl PartialPhase2 {
    pub fn from_index(index: usize) -> Self {
        let (a, b) = index.div_rem(&40320);
        Self {
            corner_perm_combo_coord: CornerPermComboCoord {
                sym_coord: CornerPermSymCoord(a as u16),
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            ud_edge_perm_raw_coord: UDEdgePermRawCoord(b as u16),
        }
    }

    pub fn into_index(self) -> usize {
        debug_assert_eq!(
            self.corner_perm_combo_coord.domino_conjugation,
            DominoSymmetry::IDENTITY
        );
        let a = self.corner_perm_combo_coord.sym_coord.0;
        let b = self.ud_edge_perm_raw_coord.0;

        (a as usize) * 40320 + (b as usize)
    }

    pub fn apply_domino_move(self, tables: &Tables, domino_move: DominoMove) -> Self {
        let corner_perm_combo_coord = self
            .corner_perm_combo_coord
            .apply_cube_move(tables, domino_move.into());

        let ud_edge_perm_raw_coord = tables
            .grouped_edge_moves
            .update_edge_perm_phase_2_partial_domino_move(domino_move, self.ud_edge_perm_raw_coord);

        Self {
            corner_perm_combo_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn domino_conjugate(self, tables: &Tables, sym: DominoSymmetry) -> Self {
        if sym == DominoSymmetry::IDENTITY {
            return self;
        }

        let corner_perm_combo_coord = self.corner_perm_combo_coord.domino_conjugate(sym);

        let ud_edge_perm_raw_coord = tables
            .grouped_edge_moves
            .update_edge_perm_phase_2_partial_domino_symmetry(sym, self.ud_edge_perm_raw_coord);

        Self {
            corner_perm_combo_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn normalize(self, tables: &Tables) -> impl IntoIterator<Item = Self> {
        let rep = self.domino_conjugate(tables, self.corner_perm_combo_coord.domino_conjugation);

        LookupSymCornerPermTable::get_all_stabilizing_conjugations(
            rep.corner_perm_combo_coord.sym_coord,
        )
        .into_iter()
        .map(move |sym| PartialPhase2 {
            corner_perm_combo_coord: rep.corner_perm_combo_coord,
            ud_edge_perm_raw_coord: tables
                .grouped_edge_moves
                .update_edge_perm_phase_2_partial_domino_symmetry(sym, rep.ud_edge_perm_raw_coord),
        })
    }

    pub fn single_normalize(self, tables: &Tables) -> Self {
        self.domino_conjugate(tables, self.corner_perm_combo_coord.domino_conjugation)
    }
}

pub fn top_down_adjacent(index: usize, tables: &Tables) -> impl IntoIterator<Item = usize> {
    let start = PartialPhase2::from_index(index);

    DominoMove::all_iter()
        .flat_map(move |cube_move| start.apply_domino_move(tables, cube_move).normalize(tables))
        .map(PartialPhase2::into_index)
}

pub fn bottom_up_adjacent(index: usize, tables: &Tables) -> impl IntoIterator<Item = usize> {
    let start = PartialPhase2::from_index(index);

    DominoMove::all_iter()
        .map(move |cube_move| {
            start
                .apply_domino_move(tables, cube_move)
                .single_normalize(tables)
        })
        .map(PartialPhase2::into_index)
}

pub struct PrunePhase2Table(Mmap);

impl PrunePhase2Table {
    pub fn get_value(
        &self,
        corner_perm_combo_coord: CornerPermSymCoord,
        ud_edge_perm_raw_coord: UDEdgePermRawCoord,
    ) -> u8 {
        let partial = PartialPhase2 {
            corner_perm_combo_coord: CornerPermComboCoord {
                sym_coord: corner_perm_combo_coord,
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            ud_edge_perm_raw_coord,
        };
        let i = partial.into_index();
        PRUNE_TABLE_SHORTCUTS
            .get(&(i as u32))
            .copied()
            .unwrap_or_else(|| {
                let bits = self.0.view_bits::<bitvec::order::Lsb0>();

                let start = i * 4;
                let chunk = &bits[start..start + 4];
                4 + (chunk.load_le::<u8>() & 0b1111)
            })
    }

    fn generate(buffer: &mut [u8], tables: &Tables) {
        let mut working_buffer = vec![0u8; WORKING_TABLE_SIZE_BYTES];

        let atom = unsafe { as_atomic_u8_slice(&mut working_buffer) };
        let working = WorkingTable(atom);

        let special_cases = [0, 1, 2];

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
            let _use_bottom_up = frontier.len() * /* degree of graph */ 10 > unvisited; // cheap heuristic

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
            assert!(val < 16);
            let start = i * 4;
            bits[start..start + 4].store_le::<u8>(val);
        };

        for i in 0..TABLE_ENTRY_COUNT {
            let x = working.read(i);
            (set)(i, x.clamp(3, 18) - 3);
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

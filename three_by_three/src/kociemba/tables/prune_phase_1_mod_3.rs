use bitvec::field::BitField;
use bitvec::view::BitView;
use num_integer::Integer;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU8, Ordering, fence};

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::cube_ops::cube_move::CubeMove;
use crate::cube_ops::cube_sym::DominoSymmetry;
use crate::kociemba::coords::coords::{CornerOrientRawCoord, EdgeGroupOrientSymCoord};
use crate::kociemba::coords::edge_group_orient_combo_coord::EdgeGroupOrientComboCoord;
use crate::kociemba::tables::Tables;
use crate::kociemba::tables::prune_phase_1::{bottom_up_adjacent, top_down_adjacent};

use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_ENTRY_COUNT: usize = 64430 * 2187;
const WORKING_TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 2;
const TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 5;
const FILE_CHECKSUM: u32 = 22471410;

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

pub struct PrunePhase1Mod3Table(Mmap);

impl PrunePhase1Mod3Table {

    #[inline(always)]
    pub fn get_value(
        &self,
        edge_group_orient_sym_coord: EdgeGroupOrientSymCoord,
        corner_orient_raw_coord: CornerOrientRawCoord,
    ) -> u8 {
        let a = edge_group_orient_sym_coord.0;
        let b = corner_orient_raw_coord.0;
        self.get_value_inner((a as usize) * 2187 + (b as usize))
    }

    fn get_value_inner(
        &self,
        i: usize,
    ) -> u8 {
        let byte = self.0[i / 5];
        let divisor = [81u8, 27, 9, 3, 1][i % 5];
        (byte / divisor) % 3
    }

    pub fn get_distance(
        edge_group_orient_sym_coord: EdgeGroupOrientSymCoord,
        corner_orient_raw_coord: CornerOrientRawCoord,
        tables: &Tables,
    ) -> u8 {
        let a = edge_group_orient_sym_coord.0;
        let b = corner_orient_raw_coord.0;
        let mut i = (a as usize) * 2187 + (b as usize);

        let table = tables.get_prune_phase_1_mod_3();

        let mut d = 0;
        let mut current = table.get_value_inner(i);
        while i != 0 {
            current = (current + 2) % 3;
            let next = bottom_up_adjacent(i, tables)
                .into_iter()
                .find(|&i| table.get_value_inner(i) == current);

            i = next.unwrap();
            d += 1;
        }

        d
    }

    fn generate(buffer: &mut [u8], tables: &Tables) {
        let mut working_buffer = vec![0u8; WORKING_TABLE_SIZE_BYTES];

        let atom = unsafe { as_atomic_u8_slice(&mut working_buffer) };
        let working = WorkingTable(atom);

        // initial state
        let root = 0;

        working.write(root, 0);

        let mut frontier = vec![root];
        let mut frontier_level = 0u8; // real level, not mod-3

        while !frontier.is_empty() {
            let next_level = frontier_level + 1;
            println!("level: {:?} frontier: {:?}", frontier_level, frontier.len());

            // we tested all thresholds to determine this is the fastest on my laptop (very scientific)
            let use_bottom_up = frontier_level > 6;

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
        }

        // let mut histogram = BTreeMap::<u8, u64>::new();

        for i in 0..TABLE_SIZE_BYTES {
            let start = i * 5;
            let a = working.read(start);
            let b = working.read(start + 1);
            let c = working.read(start + 2);
            let d = working.read(start + 3);
            let e = working.read(start + 4);

            // let min = a.min(b.min(c.min(d.min(e))));

            // *histogram.entry(min).or_default() += 1;

            let mut acc = 0;

            acc += a % 3;
            acc *= 3;
            acc += b % 3;
            acc *= 3;
            acc += c % 3;
            acc *= 3;
            acc += d % 3;
            acc *= 3;
            acc += e % 3;
            
            buffer[i] = acc;
        }

        // println!("{histogram:?}");
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
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand::Rng;
    use super::*;

    const SAMPLE_COUNT: usize = 1000;
    const SEED: u64 = 42;

    #[test]
    fn generate() -> anyhow::Result<()> {
        let _tables = Tables::new("tables")?;

        Ok(())
    }

    // /// get_distance should reconstruct the exact phase-1 pruning value.
    // #[test]
    // fn get_distance_matches_prune_phase_1() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let p1 = tables.get_prune_phase_1();
    //     let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    //     for _ in 0..SAMPLE_COUNT {
    //         let i = rng.random_range(0..TABLE_ENTRY_COUNT);
    //         let sym = EdgeGroupOrientSymCoord((i / 2187) as u16);
    //         let raw = CornerOrientRawCoord((i % 2187) as u16);
    //         let expected = p1.get_value(sym, raw);
    //         let got = PrunePhase1Mod3Table::get_distance(sym, raw, &tables);
    //         assert_eq!(expected, got, "index {i}");
    //     }
    //     Ok(())
    // }

    // /// get_value for the mod-3 table should equal the phase-1 value mod 3.
    // #[test]
    // fn get_value_matches_prune_phase_1_mod_3() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let p1 = tables.get_prune_phase_1();
    //     let p1m3 = tables.get_prune_phase_1_mod_3();
    //     let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    //     for _ in 0..SAMPLE_COUNT {
    //         let i = rng.random_range(0..TABLE_ENTRY_COUNT);
    //         let sym = EdgeGroupOrientSymCoord((i / 2187) as u16);
    //         let raw = CornerOrientRawCoord((i % 2187) as u16);
    //         let expected = p1.get_value(sym, raw) % 3;
    //         let got = p1m3.get_value(sym, raw);
    //         assert_eq!(expected, got, "index {i}");
    //     }
    //     Ok(())
    // }
}

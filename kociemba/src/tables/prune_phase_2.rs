use rand::distr::Distribution;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering, fence};

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::cube_ops::repr_coord::SymReducedPhase2PartialRepr;
use crate::tables::Tables;

use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_SIZE_BYTES: usize = (2768 * 40320) / 4;
const FILE_CHECKSUM: u32 = 2553198974;

struct WorkingTable<'a>(&'a [AtomicU8]);

impl<'a> WorkingTable<'a> {
    fn visited(&self, coords: SymReducedPhase2PartialRepr) -> bool {
        let i = coords.into_pruning_index();

        let j = i % 4;
        let i = i / 4;

        let atomic = &self.0[i];

        let shift = j * 2;
        let mask = 0b11 << shift;

        atomic.load(Ordering::Relaxed) & mask != 0
    }

    fn visited_at_level_residue(
        &self,
        coords: SymReducedPhase2PartialRepr,
        level_residue: u8,
    ) -> bool {
        let i = coords.into_pruning_index();

        let j = i % 4;
        let i = i / 4;

        let atomic = &self.0[i];

        let shift = j * 2;
        let mask = 0b11 << shift;

        let expected_residue = level_residue << shift;

        atomic.load(Ordering::Relaxed) & mask == expected_residue
    }

    /// write to the table. returns true if write was successful and the moves from here should be handled.
    fn write(&self, coords: SymReducedPhase2PartialRepr, level_residue: u8) -> bool {
        let i = coords.into_pruning_index();

        let j = i % 4;
        let i = i / 4;

        let atomic = &self.0[i];

        let shift = j * 2;
        let mask = 0b11 << shift;
        let bits = level_residue << shift;

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
}

pub struct PrunePhase2Table(Mmap);

impl PrunePhase2Table {
    pub fn get_value(&self, pruning_index: usize) -> u8 {
        let i = pruning_index;

        let j = i % 4;
        let i = i / 4;

        let byte = self.0[i];

        let shift = j * 2;

        ((byte >> shift) & 0b11) - 1
    }

    fn generate(buffer: &mut [u8], tables: &Tables) {
        let atom = unsafe { as_atomic_u8_slice(buffer) };
        let working = WorkingTable(atom);

        // initial state
        let root = SymReducedPhase2PartialRepr::SOLVED;

        working.write(root, 1);

        let mut frontier = vec![root];
        let mut frontier_level = 0u8; // real level, not mod-3
        let mut total_visited = 1;

        while !frontier.is_empty() {
            let frontier_residue = (frontier_level % 3) + 1;
            let next_residue = ((frontier_level + 1) % 3) + 1;
            println!("level: {:?} frontier: {:?}", frontier_level, frontier.len());
            let unvisited = 2768 * 40320 - total_visited;
            let use_bottom_up = frontier.len() * /* degree of graph */ 10 > unvisited; // cheap heuristic

            let use_bottom_up = true;

            let next = if !use_bottom_up {
                /* ---------- top-down ---------- */
                frontier
                    .par_iter()
                    .flat_map_iter(|&v| tables.phase_2_partial_adjacent(v))
                    .filter_map(|nbr| {
                        if working.write(nbr, next_residue) {
                            Some(nbr)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                /* ---------- bottom-up ---------- */
                (0..(2768 * 40320))
                    .into_par_iter()
                    .map(SymReducedPhase2PartialRepr::from_pruning_index)
                    .filter_map(|v| {
                        if working.visited(v) {
                            return None; // already discovered
                        }
                        for nbr in tables.phase_2_partial_adjacent(v) {
                            if working.visited_at_level_residue(nbr, frontier_residue) {
                                if working.write(v, next_residue) {
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

        // println!("{:?}", frontier.len());
        // println!("{frontier_level:?}");
    }

    pub fn load<P: AsRef<Path>>(path: P, tables: &Tables) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, tables)
        })
        .map(Self)
    }
}

// #[cfg(test)]
// mod test {

//     use crate::tables::lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;

//     use super::*;
//     // use crate::tables::{lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, move_raw_corner_orient::MoveRawCornerOrientTable, move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable};

//     #[test]
//     fn generate() {
//         let lookup_sym_edge_group_orient = LookupSymEdgeGroupOrientTable::load(
//             "edge_group_orient_sym_lookup_table.dat",
//         ).unwrap();

//         let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
//             "edge_group_orient_sym_move_table.dat",
//             &lookup_sym_edge_group_orient,
//         ).unwrap();

//         let move_raw_corner_orient = MoveRawCornerOrientTable::load("corner_orient_move_table.dat").unwrap();

//         let move_sym_edge_group_orient_ref = &move_sym_edge_group_orient;
//         let move_raw_corner_orient_ref = &move_raw_corner_orient;

//         let prune_phase_1 = PrunePhase2Table::load("phase_1_prune_table.dat", move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).unwrap();

//     }
// }

// #[test]
// fn test_neighbors() -> anyhow::Result<()> {
//     let phase_1_move_edge_raw_table =
//         crate::tables::move_raw_edge_group_flip::load("edge_group_and_orient_move_table.dat")?;
//     let phase_1_move_corner_raw_table =
//         crate::tables::move_raw_corner_orient::load("corner_orient_move_table.dat")?;
//     let phase_1_lookup_edge_sym_table = crate::tables::lookup_sym_edge_group_flip::load(
//         "phase_1_edge_sym_lookup_table.dat",
//         &phase_1_move_edge_raw_table,
//     )?;
//     let phase_1_move_edge_sym_table = crate::tables::move_sym_edge_group_flip::load(
//         "phase_1_edge_sym_move_table.dat",
//         &phase_1_lookup_edge_sym_table,
//         &phase_1_move_edge_raw_table,
//     )?;

//     use rand::{Rng, SeedableRng};
//     let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

//     'main: for _ in 0..1000 {
//         let cube: crate::repr_cubie::ReprCube = rand::distr::StandardUniform.sample(&mut rng);

//         let eo = crate::coords::RawEdgeOrientCoord::from_cubie(cube);
//         let eg = crate::coords::RawEdgeGroupCoord::from_cubie(cube);
//         let co = RawCornerOrientCoord::from_cubie(cube);

//         let (sym_start, transform) =
//             phase_1_lookup_edge_sym_table.get_sym_from_raw(&phase_1_move_edge_raw_table, eg, eo);

//         let raw_start = phase_1_move_corner_raw_table.conjugate_by_transform(co, transform);

//         let sym_neighbors: Vec<_> = neighbors((sym_start, raw_start), &phase_1_move_edge_sym_table, &phase_1_move_corner_raw_table).collect();

//         let raw_then_sym: Vec<_> = Move::all_iter().map(|mv| cube.then(mv.into())).map(|cube| {
//             let eo = crate::coords::RawEdgeOrientCoord::from_cubie(cube);
//             let eg = crate::coords::RawEdgeGroupCoord::from_cubie(cube);
//             let co = RawCornerOrientCoord::from_cubie(cube);

//             let (sym_start, transform) =
//                 phase_1_lookup_edge_sym_table.get_sym_from_raw(&phase_1_move_edge_raw_table, eg, eo);

//             let raw_start = phase_1_move_corner_raw_table.conjugate_by_transform(co, transform);
//             (sym_start, raw_start)
//         }).collect();

//         for a in sym_neighbors.iter() {
//             if !raw_then_sym.contains(a) {
//                 println!("sym_neighbors: {:?}", &itertools::Itertools::collect_vec(sym_neighbors.iter().map(|(a,b)|(a.inner(),b.inner()))));
//                 println!("raw_neighbors: {:?}", &itertools::Itertools::collect_vec(raw_then_sym.iter().map(|(a,b)|(a.inner(),b.inner()))));
//                 println!();
//                 continue 'main;
//             }
//             assert!(raw_then_sym.contains(a))
//         }
//     }

//     Ok(())
// }

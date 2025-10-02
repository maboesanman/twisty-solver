use rand::distr::Distribution;
use rayon::prelude::*;

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::cube_ops::coords::{CornerPermSymCoord, UDEdgePermRawCoord};
use crate::cube_ops::repr_coord::SymReducedPhase2PartialRepr;
use crate::tables::Tables;

use super::table_loader::load_table;

const TABLE_SIZE_BYTES: usize = 2768;
const FILE_CHECKSUM: u32 = 2553198974;

struct WorkingTable<'a>(&'a mut [u8]);

impl<'a> WorkingTable<'a> {
    fn visited(&mut self, coords: CornerPermSymCoord) -> bool {
        let i = coords.0 as usize;

        self.0[i] != 0 || i == 0
    }

    fn visited_at_level(&mut self, coords: CornerPermSymCoord, level: u8) -> bool {
        let i = coords.0 as usize;

        self.0[i] == level
    }

    /// write to the table. returns true if write was successful and the moves from here should be handled.
    fn write(&mut self, coords: CornerPermSymCoord, level: u8) -> bool {
        let i = coords.0 as usize;

        if i == 0 {
            return false;
        }

        let result = self.0[i] == 0;

        self.0[i] = level;

        result
    }
}

pub struct PrunePhaseCornerTable(Mmap);

impl PrunePhaseCornerTable {
    pub fn get_value(&self, pruning_index: usize) -> u8 {
        let i = pruning_index;
        self.0[i]
    }

    fn generate(buffer: &mut [u8], tables: &Tables) {
        let mut working = WorkingTable(buffer);

        // initial state
        let root = SymReducedPhase2PartialRepr::SOLVED;

        working.write(root.get_corner_perm_sym_coord(), 1);

        let mut frontier = vec![root.get_corner_perm_sym_coord()];
        let mut frontier_level = 0u8;

        while !frontier.is_empty() {
            let next_level = frontier_level + 1;
            println!("level: {:?} frontier: {:?}", frontier_level, frontier.len());

            /* ---------- bottom-up ---------- */
            frontier = (0..2768)
                .map(CornerPermSymCoord)
                .filter_map(|v| {
                    if working.visited(v) {
                        return None; // already discovered
                    }
                    for nbr in tables.phase_2_partial_adjacent(
                        SymReducedPhase2PartialRepr::from_coords(v, UDEdgePermRawCoord(0)),
                    ) {
                        if working.visited_at_level(nbr.get_corner_perm_sym_coord(), frontier_level)
                        {
                            if working.write(v, next_level) {
                                return Some(v);
                            } else {
                                return None;
                            }
                        }
                    }
                    None
                })
                .collect();
            frontier_level += 1;
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

//         let prune_phase_1 = PrunePhaseCornerTable::load("phase_1_prune_table.dat", move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).unwrap();

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

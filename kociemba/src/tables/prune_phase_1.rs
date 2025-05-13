use rand::distr::Distribution;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::coords::{RawCornerOrientCoord, SymEdgeGroupFlipCoord};
use crate::moves::Move;

use super::move_raw_corner_orient::MoveRawCornerOrientTable;
use super::move_sym_edge_group_flip::MoveSymEdgeGroupFlipTable;
use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_SIZE_BYTES: usize = (64430 * 2187) / 4 + 1;
const FILE_CHECKSUM: u32 = 204444714;

struct WorkingTable<'a>(&'a [AtomicU8]);

impl<'a> WorkingTable<'a> {
    fn visited(&self, coords: (SymEdgeGroupFlipCoord, RawCornerOrientCoord)) -> bool {
        let i = coords.0.inner() as usize * 2187 + coords.1.inner() as usize;

        let j = i % 4;
        let i = i / 4;

        let atomic = &self.0[i];

        let shift = j * 2;
        let mask = 0b11 << shift;

        atomic.load(Ordering::Relaxed) & mask != 0
    }

    /// write to the table. returns true if write was successful and the moves from here should be handled.
    fn write(&self, coords: (SymEdgeGroupFlipCoord, RawCornerOrientCoord), val: u8) -> bool {
        let i = coords.0.inner() as usize * 2187 + coords.1.inner() as usize;

        let j = i % 4;
        let i = i / 4;

        let atomic = &self.0[i];

        let shift = j * 2;
        let mask = 0b11 << shift;
        let bits = (((val % 3) + 1) & 0b11) << shift;

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

pub struct PrunePhase1Table(Mmap);

impl PrunePhase1Table {
    pub fn get_value(&self, coords: (SymEdgeGroupFlipCoord, RawCornerOrientCoord)) -> u8 {
        let i = coords.0.inner() as usize * 2187 + coords.1.inner() as usize;

        let j = i % 4;
        let i = i / 4;

        let byte = self.0[i];

        let shift = j * 2;

        ((byte >> shift) & 0b11) - 1
    }

    fn generate(
        buffer: &mut [u8],
        edge_table: &MoveSymEdgeGroupFlipTable,
        corner_table: &MoveRawCornerOrientTable,
    ) {
        let atom = unsafe { as_atomic_u8_slice(buffer) };
        let working = WorkingTable(atom);

        // initial state
        let root = (0.into(), 0.into());
        working.write(root, 0);
        let mut frontier = vec![root];
        let mut level = 0u8; // real level, not mod-3

        while !frontier.is_empty() {
            // let unvisited = atom.len() - frontier.len();
            // let use_bottom_up =
            //     frontier.len() * /* degree of graph */ 18 > unvisited; // cheap heuristic

            let next: Vec<_> = {
                //if !use_bottom_up {
                /* ---------- top-down ---------- */
                frontier
                    .iter()
                    .flat_map(|&v| neighbors(v, edge_table, corner_table))
                    .filter_map(|nbr| {
                        if working.write(nbr, level + 1) {
                            Some(nbr)
                        } else {
                            None
                        }
                    })
                    .collect()
            };
            // } else {
            //     /* ---------- bottom-up ---------- */
            //     itertools::iproduct!(0..64430, 0..2187)
            //         .par_bridge()
            //         .map(|(i, j)| (Phase1EdgeSymCoord::from(i), CornerOrientCoord::from(j)))
            //         .filter_map(|v| {
            //             if working.visited(v) {
            //                 return None;                // already discovered
            //             }
            //             for n in neighbors(v, edge_table, corner_table) {
            //                 if working.visited(n) {

            //                 }
            //             }
            //             None
            //         })
            //         .collect()
            // };

            frontier = next;
            level += 1;
        }

        println!("{:?}", frontier.len());
        println!("{level:?}");
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        edge_table: &MoveSymEdgeGroupFlipTable,
        corner_table: &MoveRawCornerOrientTable,
    ) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, edge_table, corner_table)
        })
        .map(Self)
    }
}

fn neighbors(
    coords: (SymEdgeGroupFlipCoord, RawCornerOrientCoord),
    edge_table: &MoveSymEdgeGroupFlipTable,
    corner_table: &MoveRawCornerOrientTable,
) -> impl Iterator<Item = (SymEdgeGroupFlipCoord, RawCornerOrientCoord)> {
    Move::all_iter().map(move |mv| {
        let (new_sym_coord, transform) = edge_table.apply_move(coords.0, mv);
        let new_raw_coord = corner_table.apply_move_and_transform(coords.1, mv, transform);
        (new_sym_coord, new_raw_coord)
    })
}

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

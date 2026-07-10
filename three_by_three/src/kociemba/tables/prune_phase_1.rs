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
use crate::kociemba::coords::edge_group_orient_combo_coord::EdgeGroupOrientComboCoord;
use crate::kociemba::coords::{CornerOrientRawCoord, EdgeGroupOrientSymCoord};
use crate::kociemba::tables::lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable;
use crate::kociemba::tables::move_raw_corner_orient::MoveRawCornerOrientTable;
use crate::kociemba::tables::move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable;

use super::table_loader::{as_atomic_u8_slice, load_table};

const TABLE_ENTRY_COUNT: usize = 64430 * 2187;
const WORKING_TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 2;
pub(crate) const TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 2;
const FILE_CHECKSUM: u32 = 3898141313;

static PRUNE_TABLE_SHORTCUTS: phf::Map<u32, u8> = phf::phf_map! {
    0 => 0,
    41356582 => 1,
    32140645 | 39257386 | 130391033 | 118414805 | 48557730 => 2,
    20995743 | 26541963 | 23624471 | 4931120 | 16394159 | 433108 | 131125875 | 32140654 | 131125872 | 83011823 | 55554676 | 113839847 | 16652472 | 44008038 | 54294237 | 37280376 | 407351 | 34472628 | 66753642 | 100497361 | 50935727 | 113934289 | 135604825 | 130712522 | 117846155 | 113865928 | 113943284 | 109884058 | 91470317 | 100375214 | 126699361 | 139608511 | 6565465 | 118408172 | 5515696 | 132875428 | 109731931 | 122902752 | 53246674 | 50937186 | 55556146 | 96019925 | 37280014 | 20702714 => 3,
    7667 | 7694 | 7747 | 7774 | 7934 | 7987 | 8414 | 8467 | 8627 | 8654 | 8707 | 8734 | 28321 | 103991 | 203297 | 204737 | 239585 | 266704 | 266720 | 364148 | 364388 | 364468 | 364975 | 375836 | 376124 | 402154 | 405773 | 405781 | 405797 | 405805 | 405959 | 405961 | 406016 | 406024 | 406429 | 406453 | 406508 | 406526 | 406670 | 406672 | 406688 | 406690 | 4735961 | 8795113 | 11113331 | 11113344 | 11294870 | 11703823 | 23081237 | 25359451 | 25381321 | 36885281 | 37157036 | 37922326 | 38014340 | 38141019 | 39133364 | 39134057 | 40371908 | 40958042 | 41259848 | 41263502 | 41268596 | 41311589 | 41652788 | 41697248 | 50689925 | 125640970 | 126078368 | 127559367 | 127998954 | 130397688 | 130631697 | 130761466 | 130990777 | 131003749 | 131004223 | 131004411 | 131010784 | 131049414 | 131049433 | 131049447 | 131049466 | 131049604 | 131049657 | 131049826 | 131049864 | 131050150 | 131050188 | 131050305 | 131050410 | 131050548 | 131050600 | 131069287 | 131069359 | 131070021 | 131070093 | 131080051 | 131080923 | 131080942 | 131091243 | 131113113 | 131113747 | 131115286 | 131115948 | 131117211 | 131119849 | 131120323 | 131121585 | 131121586 | 131121592 | 131121604 | 131121618 | 131121630 | 131121774 | 131121775 | 131121828 | 131121846 | 131121990 | 131121997 | 131122314 | 131122320 | 131122359 | 131122476 | 131122581 | 131122771 | 135844452 | 139075704 | 139662270 => 12,
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
        let (e, c) = index.div_rem(&2187);
        Self {
            edge_group_orient_combo_coord: EdgeGroupOrientComboCoord {
                sym_coord: EdgeGroupOrientSymCoord(e as u16),
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            corner_orient_raw_coord: CornerOrientRawCoord(c as u16),
        }
    }

    pub fn from_index_exhaustive(
        index: usize,
        tables: &(impl AsRef<LookupSymEdgeGroupOrientTable> + AsRef<MoveRawCornerOrientTable>),
    ) -> impl IntoIterator<Item = Self> {
        let sym_lookup: &LookupSymEdgeGroupOrientTable = tables.as_ref();
        let move_corner: &MoveRawCornerOrientTable = tables.as_ref();

        let base = Self::from_index(index);

        sym_lookup
            .get_all_stabilizing_conjugations(base.edge_group_orient_combo_coord.sym_coord)
            .into_iter()
            .map(move |sym| Self {
                edge_group_orient_combo_coord: base.edge_group_orient_combo_coord,
                corner_orient_raw_coord: move_corner
                    .domino_conjugate(base.corner_orient_raw_coord, sym),
            })
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

    pub fn apply_cube_move(
        self,
        tables: &(impl AsRef<MoveRawCornerOrientTable> + AsRef<MoveSymEdgeGroupOrientTable>),
        cube_move: CubeMove,
    ) -> Self {
        let move_corner: &MoveRawCornerOrientTable = tables.as_ref();

        let edge_group_orient_combo_coord = self
            .edge_group_orient_combo_coord
            .apply_cube_move(tables, cube_move);

        let corner_orient_raw_coord =
            move_corner.apply_cube_move(self.corner_orient_raw_coord, cube_move);

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
        }
    }

    pub fn domino_conjugate(
        self,
        tables: impl AsRef<MoveRawCornerOrientTable>,
        sym: DominoSymmetry,
    ) -> Self {
        let move_corner: &MoveRawCornerOrientTable = tables.as_ref();
        if sym == DominoSymmetry::IDENTITY {
            return self;
        }

        let edge_group_orient_combo_coord =
            self.edge_group_orient_combo_coord.domino_conjugate(sym);

        let corner_orient_raw_coord =
            move_corner.domino_conjugate(self.corner_orient_raw_coord, sym);

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
        }
    }

    pub fn normalize(
        self,
        tables: &(impl AsRef<LookupSymEdgeGroupOrientTable> + AsRef<MoveRawCornerOrientTable>),
    ) -> impl IntoIterator<Item = Self> {
        let sym_lookup: &LookupSymEdgeGroupOrientTable = tables.as_ref();
        let move_corner: &MoveRawCornerOrientTable = tables.as_ref();

        let rep = self.domino_conjugate(
            tables,
            self.edge_group_orient_combo_coord.domino_conjugation,
        );

        sym_lookup
            .get_all_stabilizing_conjugations(rep.edge_group_orient_combo_coord.sym_coord)
            .into_iter()
            .map(move |sym| PartialPhase1 {
                edge_group_orient_combo_coord: rep.edge_group_orient_combo_coord,
                corner_orient_raw_coord: move_corner
                    .domino_conjugate(rep.corner_orient_raw_coord, sym),
            })
    }

    pub fn single_normalize(self, tables: impl AsRef<MoveRawCornerOrientTable>) -> Self {
        self.domino_conjugate(
            tables,
            self.edge_group_orient_combo_coord.domino_conjugation,
        )
    }
}

pub fn top_down_adjacent(
    index: usize,
    tables: &(
         impl AsRef<LookupSymEdgeGroupOrientTable>
         + AsRef<MoveRawCornerOrientTable>
         + AsRef<MoveSymEdgeGroupOrientTable>
     ),
) -> impl IntoIterator<Item = usize> {
    let starts = PartialPhase1::from_index_exhaustive(index, tables);
    starts.into_iter().flat_map(move |start| {
        CubeMove::all_iter()
            .flat_map(move |cube_move| start.apply_cube_move(tables, cube_move).normalize(tables))
            .map(PartialPhase1::into_index)
    })
}

pub fn bottom_up_adjacent(
    index: usize,
    tables: &(
         impl AsRef<LookupSymEdgeGroupOrientTable>
         + AsRef<MoveRawCornerOrientTable>
         + AsRef<MoveSymEdgeGroupOrientTable>
     ),
) -> impl IntoIterator<Item = usize> {
    let start = PartialPhase1::from_index(index);

    CubeMove::all_iter()
        .map(move |cube_move| {
            start
                .apply_cube_move(tables, cube_move)
                .single_normalize(tables)
        })
        .map(PartialPhase1::into_index)
}

pub fn equivalent(
    index: usize,
    tables: &(impl AsRef<LookupSymEdgeGroupOrientTable> + AsRef<MoveRawCornerOrientTable>),
) -> impl IntoIterator<Item = usize> {
    PartialPhase1::from_index_exhaustive(index, tables)
        .into_iter()
        .map(PartialPhase1::into_index)
        .filter(move |x| *x != index)
}

pub fn adjacent(
    index: usize,
    tables: &(
         impl AsRef<LookupSymEdgeGroupOrientTable>
         + AsRef<MoveRawCornerOrientTable>
         + AsRef<MoveSymEdgeGroupOrientTable>
     ),
) -> impl IntoIterator<Item = usize> {
    let starts = PartialPhase1::from_index_exhaustive(index, tables);

    starts
        .into_iter()
        .flat_map(move |start| {
            CubeMove::all_iter().flat_map(move |cube_move| {
                start.apply_cube_move(tables, cube_move).normalize(tables)
            })
        })
        .map(PartialPhase1::into_index)
}

pub struct PrunePhase1Table {
    buffer: [u8],
}

impl PrunePhase1Table {
    #[inline(always)]
    pub fn get_value(
        &self,
        edge_group_orient_sym_coord: EdgeGroupOrientSymCoord,
        corner_orient_raw_coord: CornerOrientRawCoord,
    ) -> u8 {
        let i =
            (edge_group_orient_sym_coord.0 as usize) * 2187 + corner_orient_raw_coord.0 as usize;

        // assert!(i < 64430 * 2187, "{} {}", edge_group_orient_sym_coord.0, corner_orient_raw_coord.0);
        let byte = self.buffer[i >> 1];
        let shift = (i & 1) << 2;

        (byte >> shift) & 0b1111
    }
}

impl PrunePhase1Table {
    fn generate(
        buffer: &mut [u8],
        tables: &(
             impl Send
             + Sync
             + AsRef<LookupSymEdgeGroupOrientTable>
             + AsRef<MoveRawCornerOrientTable>
             + AsRef<MoveSymEdgeGroupOrientTable>
         ),
    ) {
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

        while !frontier.is_empty() {
            if special_cases.contains(&frontier_level) {
                shortcut_map.insert(frontier_level, frontier.clone());
            }
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

        let mut out_string =
            "static PRUNE_TABLE_SHORTCUTS: phf::Map<u32, u8> = phf::phf_map! {\n".to_string();
        for (k, v) in shortcut_map.iter() {
            out_string.push_str(&format!(
                "    {} => {},\n",
                itertools::Itertools::join(&mut v.iter().map(|x| format!("{x}")), " | "),
                k
            ));
        }
        out_string.push_str("};");

        for (k, v) in shortcut_map
            .into_iter()
            .flat_map(|(k, v)| v.into_iter().map(move |v| (k, v)))
        {
            if PRUNE_TABLE_SHORTCUTS.get(&(v as u32)).copied() != Some(k) {
                println!("{out_string}");
                panic!();
            }
        }

        let bits = buffer.view_bits_mut::<bitvec::order::Lsb0>();

        let mut set = |i: usize, val: u8| {
            assert!(val < 16);
            let start = i * 4;
            bits[start..start + 4].store_le::<u8>(val);
        };

        for i in 0..TABLE_ENTRY_COUNT {
            let x = working.read(i);
            (set)(i, x);
        }
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        tables: &(
             impl Send
             + Sync
             + AsRef<LookupSymEdgeGroupOrientTable>
             + AsRef<MoveRawCornerOrientTable>
             + AsRef<MoveSymEdgeGroupOrientTable>
         ),
    ) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, tables)
        })
    }

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

// fn generate_column_permutations(tables: &Tables) -> (Box<[u16; 64430]>, Box<[u16; 2187]>) {
//     let mut next_row = 1;
//     let mut next_col = 1;

//     let mut edges = Box::new([u16::MAX; 64430]);
//     let mut corners = Box::new([u16::MAX; 2187]);

//     edges[0] = 0;
//     corners[0] = 0;

//     let table = tables.get_prune_phase_1();

//     for i in 1u8..=12 {
//         for index in 1..(64430 * 2187) {
//             let byte = table.0[index >> 1];
//             let shift = (i & 1) << 2;
//             let dist = (byte >> shift) & 0b1111;
//             if dist == i {
//                 let (e, c) = index.div_rem(&2187);
//                 if edges[e] == u16::MAX {
//                     edges[e] = next_row;
//                     next_row += 1;
//                 }
//                 if corners[c] == u16::MAX {
//                     corners[c] = next_col;
//                     next_col += 1;
//                 }
//             }
//         }
//     }

//     (edges, corners)
// }

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use itertools::Itertools;

    use crate::Tables;

    use super::*;

    fn find_single_edge_preserving(
        edge_coord: u16,
        corner_coord: u16,
        tables: &(
             impl Send
             + Sync
             + AsRef<LookupSymEdgeGroupOrientTable>
             + AsRef<MoveRawCornerOrientTable>
             + AsRef<MoveSymEdgeGroupOrientTable>
         ),
    ) -> Vec<u16> {
        let expand = move |corner: u16| {
            let index = edge_coord as usize * 2187 + corner as usize;
            let eq = equivalent(index, tables);
            let adj = adjacent(index, tables);
            eq.into_iter()
                .chain(adj)
                .filter(move |idx| (*idx / 2187) as u16 == edge_coord)
                .map(|idx| (idx % 2187) as u16)
        };

        let start = corner_coord;

        let mut visited = std::collections::HashSet::new();
        let mut frontier = Vec::new();

        if visited.insert(start) {
            frontier.push(start);
        }

        while !frontier.is_empty() {
            let mut next_frontier = Vec::new();
            for c in frontier {
                for nbr in expand(c) {
                    if visited.insert(nbr) {
                        next_frontier.push(nbr);
                    }
                }
            }
            frontier = next_frontier;
        }

        let mut result: Vec<u16> = visited.into_iter().collect();
        result.sort_unstable();
        result.dedup();
        result
    }

    fn find_edge_preserving(
        edge_coord: EdgeGroupOrientSymCoord,
        tables: &(
             impl Send
             + Sync
             + AsRef<LookupSymEdgeGroupOrientTable>
             + AsRef<MoveRawCornerOrientTable>
             + AsRef<MoveSymEdgeGroupOrientTable>
         ),
    ) -> Vec<Vec<u16>> {
        let mut todo: BTreeSet<u16> = (0..2187u16).rev().collect();
        let mut result = Vec::new();

        while let Some(first) = todo.pop_first() {
            let set = find_single_edge_preserving(edge_coord.0, first, tables);

            if set.len() == 1 {
                continue;
            }
            // println!("{set:?}");
            for c in &set {
                todo.remove(&c);
            }
            result.push(set);
        }

        result
    }

    // #[test]
    fn try_edge_preserving() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        // let center = EdgeGroupOrientSymCoord(30);

        // let result = find_edge_preserving(center, &tables);

        let result: Vec<_> = (0..64430)
            .into_par_iter()
            .map(|x| {
                find_edge_preserving(
                    EdgeGroupOrientSymCoord(x),
                    &tables.prune_pre_tables.moves_pre_table,
                )
            })
            .collect();

        let sets_iter = result.into_iter().flatten();

        let mut edges: HashMap<(u16, u16), usize> = HashMap::new();

        for set in sets_iter {
            for (&a, &b) in set.iter().tuple_combinations() {
                *edges.entry((a, b)).or_default() += 1;
            }
        }

        println!("len: {}", edges.len());

        Ok(())
    }

    // #[test]
    // fn check_out_generate_column_permutations() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     generate_column_permutations(&tables);

    //     Ok(())
    // }
}

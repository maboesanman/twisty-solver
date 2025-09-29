use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use itertools::Itertools;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::cube_ops::{
    coords::{
        CornerPermRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupRawCoord,
        UDEdgePermRawCoord,
    },
    cube_move::{CubeMove, DominoMove},
    cube_sym::DominoSymmetry,
    partial_reprs::{
        corner_perm::CornerPerm,
        e_edge_perm::{self, EEdgePerm},
        edge_group::EdgeGroup,
        edge_perm::EdgePerm,
        ud_edge_perm::{self, UDEdgePerm},
    },
};

use super::table_loader::{
    as_u16_slice, as_u16_slice_mut, collect_unique_sorted_parallel, load_table,
};

const RESTRICTED_TABLE_SIZE_BYTES: usize = 495 * 33 * 3 + 1; // group * move/nontrivial_domino_sym * size(reduced_perm_entry)
const RESTRICTED_FILE_CHECKSUM: u32 = 1135746172;

const UD_TABLE_SIZE_BYTES: usize = 40320 * 1123 * 2; // restricted_perm * ud_edge_perm * size(ud_edge_perm)
const UD_FILE_CHECKSUM: u32 = 908484496;

const E_TABLE_SIZE_BYTES: usize = 24 * 135 * 1; // e_perm * e_perm * size(e_perm)
const E_FILE_CHECKSUM: u32 = 3668178995;

#[derive(Copy, Clone, Debug)]
// only 10 bits. 978 possible values
#[repr(transparent)]
struct RestrictedUDEdgePermCoord(u16);

impl RestrictedUDEdgePermCoord {
    fn phase_2_from_domino_move(mv: DominoMove) -> Self {
        Self((mv as u8) as u16)
    }
    fn phase_2_from_domino_symmetry(mv: DominoSymmetry) -> Self {
        Self((mv.0) as u16)
    }
}

// ud: 10.1 (1123)
// e:   7.1 (135)

//  16  8  8  16
// [ud][e][e][ud]
// ud(i) = 3 * (i >> 1) + 2 * (i & 1) <- index into u16 array
// e(i) = 3 * i + (((!i) & 1) << 1) <- index into u8 array

#[derive(Copy, Clone, Debug)]
// only 11 bits. 1123 possible values
#[repr(transparent)]
struct RestrictedEEdgePermCoord(u8);

impl RestrictedEEdgePermCoord {
    fn phase_2_from_domino_move(mv: DominoMove) -> Self {
        Self(mv as u8 + 9)
    }
    fn phase_2_from_domino_symmetry(mv: DominoSymmetry) -> Self {
        Self(mv.0 + 9)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
struct ReducedPermEntry(u16);

impl ReducedPermEntry {
    pub fn new(ud: RestrictedUDEdgePermCoord, e: RestrictedEEdgePermCoord) -> Self {
        Self(((e.0 as u16) << 10) | ud.0)
    }

    pub fn get_ud(self) -> RestrictedUDEdgePermCoord {
        RestrictedUDEdgePermCoord(self.0 & (0b0000001111111111))
    }

    pub fn get_e(self) -> RestrictedEEdgePermCoord {
        RestrictedEEdgePermCoord((self.0 >> 10) as u8)
    }
}

// 978 possible permutations on

/// This is a trio of tables that implements the functions (edge_group, move/sym, ud_edge_perm) -> ud_edge_perm.
/// it does this via a function (edge_group, move/sym) -> restricted_ud_edge_perm, and a function (ud_edge_perm, restricted_ud_edge_perm) -> ud_edge_perm.
///
/// same for e, but the first tables are combined.
pub struct GroupedEdgeMovesTable {
    restricted_perm_lookup: Mmap,
    ud_edge_perm_mult: Mmap,
    e_edge_perm_mult: Mmap,
}

impl GroupedEdgeMovesTable {
    fn update_edge_perms_shared(
        &self,
        grouping: EdgeGroupRawCoord,
        sub_i: usize,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let i = grouping.0 as usize * 33 + sub_i;
        let ud_i = Self::get_restriction_table_index_ud(i);
        let e_i = Self::get_restriction_table_index_e(i);

        let u8_slice: &[u8] = &self.restricted_perm_lookup;
        let u16_slice = as_u16_slice(u8_slice);

        let e_intermediate = u8_slice[e_i] as usize;
        let ud_intermediate = u16_slice[ud_i] as usize;

        let e = self.e_edge_perm_mult[135 * e_edge_perm.0 as usize + e_intermediate];
        let ud =
            as_u16_slice(&self.ud_edge_perm_mult)[1123 * ud_edge_perm.0 as usize + ud_intermediate];

        (UDEdgePermRawCoord(ud), EEdgePermRawCoord(e))
    }

    pub fn update_edge_perms_cube_move(
        &self,
        grouping: EdgeGroupRawCoord,
        cube_move: CubeMove,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let sub_i = cube_move.into_index();
        self.update_edge_perms_shared(grouping, sub_i, ud_edge_perm, e_edge_perm)
    }

    pub fn update_edge_perms_domino_conjugate(
        &self,
        grouping: EdgeGroupRawCoord,
        domino_symmetry: DominoSymmetry,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let sub_i = match domino_symmetry.into_index().checked_sub(1) {
            Some(i) => i + 18,
            None => return (ud_edge_perm, e_edge_perm),
        };
        self.update_edge_perms_shared(grouping, sub_i, ud_edge_perm, e_edge_perm)
    }

    fn update_edge_perms_phase_2_shared(
        &self,
        sub_i: usize,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let e = self.e_edge_perm_mult[135 * e_edge_perm.0 as usize + sub_i];
        let ud = as_u16_slice(&self.ud_edge_perm_mult)[1123 * ud_edge_perm.0 as usize + sub_i];

        (UDEdgePermRawCoord(ud), EEdgePermRawCoord(e))
    }

    pub fn update_edge_perm_phase_2_domino_move(
        &self,
        domino_move: DominoMove,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let sub_i = CubeMove::from(domino_move).into_index();
        self.update_edge_perms_phase_2_shared(sub_i, ud_edge_perm, e_edge_perm)
    }

    pub fn update_edge_perm_phase_2_domino_symmetry(
        &self,
        domino_symmetry: DominoSymmetry,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let sub_i = match domino_symmetry.into_index().checked_sub(1) {
            Some(i) => i + 18,
            None => return (ud_edge_perm, e_edge_perm),
        };
        self.update_edge_perms_phase_2_shared(sub_i, ud_edge_perm, e_edge_perm)
    }

    fn update_edge_perms_phase_2_partial_shared(
        &self,
        sub_i: usize,
        ud_edge_perm: UDEdgePermRawCoord,
    ) -> UDEdgePermRawCoord {
        let ud = as_u16_slice(&self.ud_edge_perm_mult)[1123 * ud_edge_perm.0 as usize + sub_i];

        UDEdgePermRawCoord(ud)
    }

    pub fn update_edge_perm_phase_2_partial_domino_move(
        &self,
        domino_move: DominoMove,
        ud_edge_perm: UDEdgePermRawCoord,
    ) -> UDEdgePermRawCoord {
        let sub_i = CubeMove::from(domino_move).into_index();
        self.update_edge_perms_phase_2_partial_shared(sub_i, ud_edge_perm)
    }

    pub fn update_edge_perm_phase_2_partial_domino_symmetry(
        &self,
        domino_symmetry: DominoSymmetry,
        ud_edge_perm: UDEdgePermRawCoord,
    ) -> UDEdgePermRawCoord {
        let sub_i = match domino_symmetry.into_index().checked_sub(1) {
            Some(i) => i + 18,
            None => return ud_edge_perm,
        };
        self.update_edge_perms_phase_2_partial_shared(sub_i, ud_edge_perm)
    }

    #[inline]
    fn get_restriction_table_index_ud(i: usize) -> usize {
        3 * (i >> 1) + 2 * (i & 1)
    }

    #[inline]
    fn get_restriction_table_index_e(i: usize) -> usize {
        3 * i + (((!i) & 1) << 1)
    }

    fn compute_unique_ud_table_columns() -> (
        BTreeMap<Arc<[u16; 40320]>, (u16, Vec<usize>)>,
        Vec<Arc<[u16; 40320]>>,
    ) {
        let mut ud_cols: BTreeMap<Arc<[u16; 40320]>, (u16, Vec<usize>)> = BTreeMap::new();
        let mut ud_vec: Vec<Arc<[u16; 40320]>> = Vec::new();

        // cube moves
        for g in 0..495 {
            let group = EdgeGroup::from_coord(EdgeGroupRawCoord(g as u16));
            for m in CubeMove::all_iter() {
                let mut col: Box<[u16; 40320]> =
                    vec![0u16; 40320].into_boxed_slice().try_into().unwrap();

                col.par_iter_mut().enumerate().for_each(|(ud, slot)| {
                    let ud = UDEdgePerm::from_coord(UDEdgePermRawCoord(ud as u16));
                    let full_edge_perm = EdgePerm::join(group, ud, EEdgePerm::SOLVED);
                    let (_new_g, new_ud, _new_e) = full_edge_perm.apply_cube_move(m).split();

                    *slot = new_ud.into_coord().0;
                });

                match ud_cols.entry(col.into()) {
                    std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                        let key = ud_vec.len() as u16;
                        ud_vec.push(vacant_entry.key().clone());
                        vacant_entry.insert((key, Vec::new()))
                    }
                    std::collections::btree_map::Entry::Occupied(occupied_entry) => {
                        occupied_entry.into_mut()
                    }
                }
                .1
                .push(g * 33 + m.into_index());
            }

            for s in DominoSymmetry::nontrivial_iter() {
                let mut col: Box<[u16; 40320]> =
                    vec![0u16; 40320].into_boxed_slice().try_into().unwrap();

                col.par_iter_mut().enumerate().for_each(|(ud, slot)| {
                    let ud = UDEdgePerm::from_coord(UDEdgePermRawCoord(ud as u16));
                    let full_edge_perm = EdgePerm::join(group, ud, EEdgePerm::SOLVED);
                    let (_new_g, new_ud, _new_e) = full_edge_perm.domino_conjugate(s).split();

                    *slot = new_ud.into_coord().0;
                });

                match ud_cols.entry(col.into()) {
                    std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                        let key = ud_vec.len() as u16;
                        ud_vec.push(vacant_entry.key().clone());
                        vacant_entry.insert((key, Vec::new()))
                    }
                    std::collections::btree_map::Entry::Occupied(occupied_entry) => {
                        occupied_entry.into_mut()
                    }
                }
                .1
                .push(g * 33 + s.into_index() + 17);
            }
        }

        (ud_cols, ud_vec)
    }

    fn compute_unique_e_table_columns() -> (
        BTreeMap<Arc<[u8; 24]>, (u8, Vec<usize>)>,
        Vec<Arc<[u8; 24]>>,
    ) {
        let mut e_cols: BTreeMap<Arc<[u8; 24]>, (u8, Vec<usize>)> = BTreeMap::new();
        let mut e_vec: Vec<Arc<[u8; 24]>> = Vec::new();

        // cube moves
        for g in 0..495 {
            let group = EdgeGroup::from_coord(EdgeGroupRawCoord(g as u16));
            for m in CubeMove::all_iter() {
                let mut col: Box<[u8; 24]> = vec![0u8; 24].into_boxed_slice().try_into().unwrap();

                col.par_iter_mut().enumerate().for_each(|(e, slot)| {
                    let e = EEdgePerm::from_coord(EEdgePermRawCoord(e as u8));
                    let full_edge_perm = EdgePerm::join(group, UDEdgePerm::SOLVED, e);
                    let (_new_g, _new_ud, new_e) = full_edge_perm.apply_cube_move(m).split();

                    *slot = new_e.into_coord().0;
                });

                match e_cols.entry(col.into()) {
                    std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                        let key = e_vec.len() as u8;
                        e_vec.push(vacant_entry.key().clone());
                        vacant_entry.insert((key, Vec::new()))
                    }
                    std::collections::btree_map::Entry::Occupied(occupied_entry) => {
                        occupied_entry.into_mut()
                    }
                }
                .1
                .push(g * 33 + m.into_index());
            }

            for s in DominoSymmetry::nontrivial_iter() {
                let mut col: Box<[u8; 24]> = vec![0u8; 24].into_boxed_slice().try_into().unwrap();

                col.par_iter_mut().enumerate().for_each(|(e, slot)| {
                    let e = EEdgePerm::from_coord(EEdgePermRawCoord(e as u8));
                    let full_edge_perm = EdgePerm::join(group, UDEdgePerm::SOLVED, e);
                    let (_new_g, _new_ud, new_e) = full_edge_perm.domino_conjugate(s).split();

                    *slot = new_e.into_coord().0;
                });

                match e_cols.entry(col.into()) {
                    std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                        let key = e_vec.len() as u8;
                        e_vec.push(vacant_entry.key().clone());
                        vacant_entry.insert((key, Vec::new()))
                    }
                    std::collections::btree_map::Entry::Occupied(occupied_entry) => {
                        occupied_entry.into_mut()
                    }
                }
                .1
                .push(g * 33 + s.into_index() + 17);
            }
        }

        (e_cols, e_vec)
    }

    fn generate_restricted(
        ud_set: BTreeMap<Arc<[u16; 40320]>, (u16, Vec<usize>)>,
        e_set: BTreeMap<Arc<[u8; 24]>, (u8, Vec<usize>)>,
        restricted_buffer: &mut [u8],
    ) {
        let restricted_buffer_len = restricted_buffer.len();
        let restricted_buffer_u16_len = restricted_buffer.len() >> 1;
        let start_u8 = restricted_buffer.as_mut_ptr();
        debug_assert_eq!((start_u8 as usize) & 1, 0, "buffer not u16-aligned");
        let start_u16: *mut u16 = start_u8.cast();

        ud_set.into_iter().for_each(|(_, (rep, indices))| {
            for i in indices {
                let i = Self::get_restriction_table_index_ud(i);
                debug_assert!(i < restricted_buffer_u16_len);
                unsafe { *start_u16.add(i) = rep };
            }
        });

        e_set.into_iter().for_each(|(_, (rep, indices))| {
            for i in indices {
                let i = Self::get_restriction_table_index_e(i);
                debug_assert!(i < restricted_buffer_len);
                unsafe { *start_u8.add(i) = rep };
            }
        });
    }

    fn generate_ud(ud_set: Vec<Arc<[u16; 40320]>>, ud_edge_buffer: &mut [u8]) {
        let start_u8 = ud_edge_buffer.as_mut_ptr();
        debug_assert_eq!((start_u8 as usize) & 1, 0, "buffer not u16-aligned");
        let start_u16: *mut u16 = start_u8.cast();
        let start_u16_addr = start_u16 as usize;

        let stride = ud_set.len();

        ud_set.into_par_iter().enumerate().for_each(|(i, col)| {
            let start_u16 = start_u16_addr as *mut u16;
            for (in_perm, out_perm) in col.iter().copied().enumerate() {
                let table_i = (in_perm as usize) * stride + i;
                unsafe { *start_u16.add(table_i) = out_perm };
            }
        });
    }

    fn generate_e(e_set: Vec<Arc<[u8; 24]>>, e_edge_buffer: &mut [u8]) {
        let start_u8_addr = e_edge_buffer.as_mut_ptr() as usize;

        let stride = e_set.len();

        e_set.into_par_iter().enumerate().for_each(|(i, col)| {
            let start_u8 = start_u8_addr as *mut u8;
            for (in_perm, out_perm) in col.iter().copied().enumerate() {
                let table_i = (in_perm as usize) * stride + i;
                unsafe { *start_u8.add(table_i) = out_perm };
            }
        });
    }

    pub fn load<P: AsRef<Path>>(restricted_path: P, ud_path: P, e_path: P) -> Result<Self> {
        let ud_set = RefCell::new(None);
        let ud_vec = RefCell::new(None);
        let e_set = RefCell::new(None);
        let e_vec = RefCell::new(None);

        let populate_ud = || {
            let (set, vec) = Self::compute_unique_ud_table_columns();
            ud_set.replace(Some(set));
            ud_vec.replace(Some(vec));
        };

        let populate_e = || {
            let (set, vec) = Self::compute_unique_e_table_columns();
            e_set.replace(Some(set));
            e_vec.replace(Some(vec));
        };

        Ok(Self {
            restricted_perm_lookup: load_table(
                restricted_path,
                RESTRICTED_TABLE_SIZE_BYTES,
                RESTRICTED_FILE_CHECKSUM,
                |buf| {
                    populate_ud();
                    populate_e();
                    Self::generate_restricted(ud_set.take().unwrap(), e_set.take().unwrap(), buf)
                },
            )?,
            ud_edge_perm_mult: load_table(ud_path, UD_TABLE_SIZE_BYTES, UD_FILE_CHECKSUM, |buf| {
                if ud_set.borrow().is_none() {
                    populate_ud();
                }
                Self::generate_ud(ud_vec.take().unwrap(), buf)
            })?,
            e_edge_perm_mult: load_table(e_path, E_TABLE_SIZE_BYTES, E_FILE_CHECKSUM, |buf| {
                if e_set.borrow().is_none() {
                    populate_e();
                }
                Self::generate_e(e_vec.take().unwrap(), buf)
            })?,
        })
    }
}

#[test]
fn test() -> Result<()> {
    // let corner_table = MoveRawCornerPermTable::load("corner_perm_move_table.dat")?;
    let _ = GroupedEdgeMovesTable::load(
        "grouped_edge_move_restricted_table.dat",
        "grouped_edge_move_ud_table.dat",
        "grouped_edge_move_e_table.dat",
    )?;

    Ok(())
}

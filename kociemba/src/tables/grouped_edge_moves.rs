use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
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
    cube_move::CubeMove,
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

const RESTRICTED_TABLE_SIZE_BYTES: usize = 495 * 33 * 2; // group * move/nontrivial_domino_sym * size(reduced_perm_entry)
const RESTRICTED_FILE_CHECKSUM: u32 = 2998989242;

const UD_TABLE_SIZE_BYTES: usize = 40320 * 978 * 2; // restricted_perm * ud_edge_perm * size(ud_edge_perm)
const UD_FILE_CHECKSUM: u32 = 530288661;

const E_TABLE_SIZE_BYTES: usize = 24 * 24 * 1; // e_perm * e_perm * size(e_perm)
const E_FILE_CHECKSUM: u32 = 250397246;

#[derive(Copy, Clone, Debug)]
// only 10 bits. 978 possible values
#[repr(transparent)]
pub struct RestrictedUDEdgePermCoord(u16);

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
struct ReducedPermEntry(u16);

impl ReducedPermEntry {
    pub fn new(ud: RestrictedUDEdgePermCoord, e: EEdgePermRawCoord) -> Self {
        Self(((e.0 as u16) << 10) | ud.0)
    }

    pub fn get_ud(self) -> RestrictedUDEdgePermCoord {
        RestrictedUDEdgePermCoord(self.0 & (0b0000001111111111))
    }

    pub fn get_e(self) -> EEdgePermRawCoord {
        EEdgePermRawCoord((self.0 >> 10) as u8)
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
    pub fn update_edge_perms_cube_move(
        &self,
        grouping: EdgeGroupRawCoord,
        cube_move: CubeMove,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let entry = self.restricted_slice(grouping)[cube_move.into_index()];
        let ud = self.ud_then(ud_edge_perm, entry.get_ud());
        let e = self.e_then(e_edge_perm, entry.get_e());
        (ud, e)
    }

    pub fn update_edge_perms_domino_conjugate(
        &self,
        grouping: EdgeGroupRawCoord,
        domino_symmetry: DominoSymmetry,
        ud_edge_perm: UDEdgePermRawCoord,
        e_edge_perm: EEdgePermRawCoord,
    ) -> (UDEdgePermRawCoord, EEdgePermRawCoord) {
        let i = match domino_symmetry.into_index().checked_sub(1) {
            Some(i) => i + 18,
            None => return (ud_edge_perm, e_edge_perm),
        };
        let entry = self.restricted_slice(grouping)[i];
        let ud = self.ud_then(ud_edge_perm, entry.get_ud());
        let e = self.e_then(e_edge_perm, entry.get_e());
        (ud, e)
    }

    fn restricted_slice(
        &self,
        grouping: EdgeGroupRawCoord,
    ) -> &[ReducedPermEntry; 33] {
        let i = (grouping.0 * 33) as usize;
        let slice = as_u16_slice(&self.restricted_perm_lookup);
        let ptr = unsafe { slice.as_ptr().add(i) as *const [u16; 33] };
        let ptr = ptr.cast();
        unsafe { &*ptr }
    }

    fn ud_then(
        &self,
        a: UDEdgePermRawCoord,
        b: RestrictedUDEdgePermCoord,
    ) -> UDEdgePermRawCoord {
        let i = (a.0 as usize) * 978 + (b.0 as usize);
        UDEdgePermRawCoord(as_u16_slice(&self.ud_edge_perm_mult)[i])
    }

    fn e_then(
        &self,
        a: EEdgePermRawCoord,
        b: EEdgePermRawCoord,
    ) -> EEdgePermRawCoord {
        let i = (a.0 as usize) * 24 + (b.0 as usize);
        EEdgePermRawCoord(self.e_edge_perm_mult[i])
    }

    fn compute_ud_set() -> Vec<u16> {
        let mut ud_set = BTreeSet::new();

        for g in 0..495 {
            let group = EdgeGroup::from_coord(EdgeGroupRawCoord(g));
            let joined = EdgePerm::join(group, UDEdgePerm::SOLVED, EEdgePerm::SOLVED);
            for mv in CubeMove::all_iter() {
                let (_, ud, _) = joined.apply_cube_move(mv).split();

                ud_set.insert(ud.into_coord().0);
            }

            for conj in DominoSymmetry::all_iter() {
                let (_, ud, _) = joined.domino_conjugate(conj).split();

                ud_set.insert(ud.into_coord().0);
            }
        }

        let ud_set = ud_set.into_iter().collect_vec();
        assert_eq!(ud_set.len(), 978);
        ud_set
    }

    fn generate_restricted(ud_set: &[u16], restricted_buffer: &mut [u8]) {
        as_u16_slice_mut(restricted_buffer)
            .par_chunks_mut(33)
            .enumerate()
            .for_each(|(g, chunk)| {
                let group = EdgeGroup::from_coord(EdgeGroupRawCoord(g as u16));
                let joined = EdgePerm::join(group, UDEdgePerm::SOLVED, EEdgePerm::SOLVED);
                for (i, perm) in CubeMove::all_iter()
                    .map(|mv| joined.apply_cube_move(mv))
                    .chain(
                        DominoSymmetry::nontrivial_iter().map(|sym| joined.domino_conjugate(sym)),
                    )
                    .enumerate()
                {
                    let (_, ud, e) = perm.split();

                    let ud = RestrictedUDEdgePermCoord(
                        ud_set.binary_search(&ud.into_coord().0).unwrap() as u16,
                    );
                    let e = e.into_coord();

                    chunk[i] = ReducedPermEntry::new(ud, e).0;
                }
            });
    }

    fn generate_ud(ud_set: &[u16], ud_edge_buffer: &mut [u8]) {
        as_u16_slice_mut(ud_edge_buffer)
            .par_chunks_mut(978)
            .enumerate()
            .for_each(|(ud_perm, chunk)| {
                let ud_perm_base = UDEdgePerm::from_coord(UDEdgePermRawCoord(ud_perm as u16));
                for (ud_perm_next, out) in ud_set.iter().zip(chunk) {
                    let ud_perm_next = UDEdgePerm::from_coord(UDEdgePermRawCoord(*ud_perm_next));
                    let ud_perm_new = ud_perm_base.then(ud_perm_next);
                    *out = ud_perm_new.into_coord().0;
                }
            });
    }

    fn generate_e(e_edge_buffer: &mut [u8]) {
        e_edge_buffer
            .par_chunks_mut(24)
            .enumerate()
            .for_each(|(e_perm, chunk)| {
                let e_perm_base = EEdgePerm::from_coord(EEdgePermRawCoord(e_perm as u8));
                for i in 0..24 {
                    let e_perm_next = EEdgePerm::from_coord(EEdgePermRawCoord(i));
                    let e_perm_new = e_perm_base.then(e_perm_next);
                    chunk[i as usize] = e_perm_new.into_coord().0;
                }
            });
    }

    pub fn load<P: AsRef<Path>>(restricted_path: P, ud_path: P, e_path: P) -> Result<Self> {
        let ud_set = Self::compute_ud_set();

        Ok(Self {
            restricted_perm_lookup: load_table(
                restricted_path,
                RESTRICTED_TABLE_SIZE_BYTES,
                RESTRICTED_FILE_CHECKSUM,
                |buf| Self::generate_restricted(&ud_set, buf),
            )?,
            ud_edge_perm_mult: load_table(
                ud_path,
                UD_TABLE_SIZE_BYTES,
                UD_FILE_CHECKSUM,
                |buf| Self::generate_ud(&ud_set, buf),
            )?,
            e_edge_perm_mult: load_table(
                e_path,
                E_TABLE_SIZE_BYTES,
                E_FILE_CHECKSUM,
                |buf| Self::generate_e(buf),
            )?,
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

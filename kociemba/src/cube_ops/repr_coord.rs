use std::num::{NonZeroU32, NonZeroU64};

use crate::cube_ops::{coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord, EdgeOrientRawCoord}, cube_sym::DominoSymmetry, partial_reprs::{edge_orient::EdgeOrient, edge_perm::EdgePerm}, repr_cube::ReprCube};


// AAAA_BBBBBBBBBBBB_CCCCCCCCCCCCCCCC___DDDDD_EEEEEEEEEEEEEEEEEEEEEEEEEEE
// 4    12           16                 5     27
// 
// A: correction symmetry for corner_perm_sym_coord
// B: corner_orient_coord
// C: edge_group_orient_sym_coord
// D: e_edge_perm_coord
// E: ud_edge_perm * corner_perm_sym_coord
// 
// notes:
// B and C == 0 => phase 1 solved. at which point we can apply A inverse to D and truncate. to DE

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SymReducedPhase1Repr(u64);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SymReducedPhase2Repr(u64);


impl SymReducedPhase1Repr {

}
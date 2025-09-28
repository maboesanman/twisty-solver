use std::num::{NonZeroU32, NonZeroU64};

use num_integer::Integer;
use pathfinding::num_traits::Euclid;

use crate::{cube_ops::{coords::{CornerOrientRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientSymCoord, EdgeOrientRawCoord, UDEdgePermRawCoord}, cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::{edge_orient::EdgeOrient, edge_perm::EdgePerm}, repr_cube::ReprCube}, tables};


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
pub struct SymReducedPhase1Repr(pub u64);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SymReducedPhase2Repr(pub u32);


impl SymReducedPhase1Repr {
    pub fn get_corner_orient_coord(self) -> CornerOrientRawCoord {
        CornerOrientRawCoord(((self.0 >> 48) & 0b111111111111) as u16)
    }

    pub fn get_edge_group_orient_sym_coord(self) -> EdgeGroupOrientSymCoord {
        EdgeGroupOrientSymCoord((self.0 >> 32) as u16)
    }

    pub fn e_edge_perm_coord(self) -> EEdgePermRawCoord {
        EEdgePermRawCoord(((self.0 >> 27) & 0b11111) as u8)
    }

    pub fn ud_edge_and_corner_perm_coords(self) -> (UDEdgePermRawCoord, CornerPermSymCoord) {
        let (a, b) = (self.0 & 0b_111_111_111_111_111_111_111_111_111).div_rem(&2768);
        (UDEdgePermRawCoord(a as u16), CornerPermSymCoord(b as u16))
    }

    pub fn corner_perm_sym_correction(self) -> DominoSymmetry {
        DominoSymmetry((self.0 >> 60) as u8)
    }

    pub fn from_coords(
        corner_orient: CornerOrientRawCoord,
        edge_group_orient: EdgeGroupOrientSymCoord,
        e_edge_perm: EEdgePermRawCoord,
        ud_edge_perm: UDEdgePermRawCoord,
        corner_perm: CornerPermSymCoord,
        corner_perm_correction: DominoSymmetry,
    ) -> Self {
        let mut val = 0;
        
        val |= (corner_perm_correction.0 as u64) << 60;
        val |= (corner_orient.0 as u64) << 48;
        val |= (edge_group_orient.0 as u64) << 32;
        val |= (e_edge_perm.0 as u64) << 27;
        val |= (ud_edge_perm.0 as u64) * 2768 + (corner_perm.0 as u64);

        Self(val)
    }
}

impl SymReducedPhase2Repr {
    pub fn e_edge_perm_coord(self) -> EEdgePermRawCoord {
        EEdgePermRawCoord(((self.0 >> 27) & 0b11111) as u8)
    }

    pub fn ud_edge_and_corner_perm_coords(self) -> (UDEdgePermRawCoord, CornerPermSymCoord) {
        let (a, b) = (self.0 & 0b_111_111_111_111_111_111_111_111_111).div_rem(&2768);
        (UDEdgePermRawCoord(a as u16), CornerPermSymCoord(b as u16))
    }

    pub fn from_coords(
        e_edge_perm: EEdgePermRawCoord,
        ud_edge_perm: UDEdgePermRawCoord,
        corner_perm: CornerPermSymCoord,
    ) -> Self {
        let mut val = 0;

        val |= (e_edge_perm.0 as u32) << 27;
        val |= (ud_edge_perm.0 as u32) * 2768 + (corner_perm.0 as u32);

        Self(val)
    }
}


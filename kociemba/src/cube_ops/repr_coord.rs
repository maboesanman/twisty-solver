use num_integer::Integer;

use crate::cube_ops::{
    coords::{
        CornerOrientRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientSymCoord,
        UDEdgePermRawCoord,
    },
    cube_sym::DominoSymmetry,
};

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
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

    pub fn into_pruning_index(self) -> usize {
        let corner = ((self.0 >> 48) & 0xFFF) as usize;
        let edge = ((self.0 >> 32) & 0x0000_0000_0000_FFFF) as usize;
        edge * 2187 + corner
    }
}

impl SymReducedPhase2Repr {
    pub const SOLVED: Self = { Self(0) };

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

    pub fn into_pruning_index(self) -> usize {
        (self.0 & 0b_111_111_111_111_111_111_111_111_111) as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SymReducedPhase1PartialRepr {
    edge_group_orient: EdgeGroupOrientSymCoord,
    corner_orient: CornerOrientRawCoord,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SymReducedPhase2PartialRepr {
    corner_perm: CornerPermSymCoord,
    ud_edge_perm: UDEdgePermRawCoord,
}

impl SymReducedPhase1PartialRepr {
    pub const SOLVED: Self = Self {
        edge_group_orient: EdgeGroupOrientSymCoord(0),
        corner_orient: CornerOrientRawCoord(0),
    };

    pub fn get_corner_orient_coord(self) -> CornerOrientRawCoord {
        self.corner_orient
    }

    pub fn get_edge_group_orient_sym_coord(self) -> EdgeGroupOrientSymCoord {
        self.edge_group_orient
    }

    pub fn from_coords(
        edge_group_orient: EdgeGroupOrientSymCoord,
        corner_orient: CornerOrientRawCoord,
    ) -> Self {
        Self {
            edge_group_orient,
            corner_orient,
        }
    }

    pub fn from_pruning_index(index: usize) -> Self {
        let (div, rem) = index.div_rem(&2187);
        Self::from_coords(
            EdgeGroupOrientSymCoord(div as u16),
            CornerOrientRawCoord(rem as u16),
        )
    }

    pub fn into_pruning_index(self) -> usize {
        (self.edge_group_orient.0 as usize) * 2187 + (self.corner_orient.0 as usize)
    }
}

impl SymReducedPhase2PartialRepr {
    pub const SOLVED: Self = Self {
        corner_perm: CornerPermSymCoord(0),
        ud_edge_perm: UDEdgePermRawCoord(0),
    };

    pub fn get_ud_edge_perm_coord(self) -> UDEdgePermRawCoord {
        self.ud_edge_perm
    }

    pub fn get_corner_perm_sym_coord(self) -> CornerPermSymCoord {
        self.corner_perm
    }

    pub fn from_coords(corner_perm: CornerPermSymCoord, ud_edge_perm: UDEdgePermRawCoord) -> Self {
        Self {
            corner_perm,
            ud_edge_perm,
        }
    }

    pub fn from_pruning_index(index: usize) -> Self {
        let (div, rem) = index.div_rem(&40320);
        Self::from_coords(
            CornerPermSymCoord(div as u16),
            UDEdgePermRawCoord(rem as u16),
        )
    }

    pub fn into_pruning_index(self) -> usize {
        (self.corner_perm.0 as usize) * 40320 + (self.ud_edge_perm.0 as usize)
    }
}

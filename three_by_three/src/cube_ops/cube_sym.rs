use std::simd::{Simd, num::SimdUint};

use crate::{
    cube, cube_ops::partial_reprs::corner_orient, kociemba::{coords::CornerOrientRawCoord, partial_reprs::{edge_group::EdgeGroup, edge_positions::split_edge_positions}}, simd_utils::{lut_u8x16x2, lut_u16x16x3}
};

use super::{
    cube_move::CubeMove,
    partial_reprs::{
        corner_orient::CornerOrient, corner_perm::CornerPerm, edge_orient::EdgeOrient,
        edge_perm::EdgePerm,
    },
    repr_cube::ReprCube,
};

pub const S_URF3_1_CORNER_PERM: CornerPerm = cube![R Lp F Bp U Dp R Lp].corner_perm;
pub const S_URF3_1_EDGE_PERM: EdgePerm = cube![R Lp F Bp U Dp R Lp].edge_perm;
// pub const S_URF3_1_CORNER_ORIENT_CORRECT: CornerOrient = cube![R Lp F Bp U Dp R Lp].corner_orient;
// pub const S_URF3_1_EDGE_ORIENT_CORRECT: EdgeOrient = cube![R Lp F Bp U Dp R Lp].edge_orient;

pub const S_URF3_1_CUBE: ReprCube = cube![R Lp F Bp U Dp R Lp];

pub const S_F2_CORNER_PERM: CornerPerm = cube![R2 L2 F Bp U2 D2 F Bp].corner_perm;
pub const S_F2_EDGE_PERM: EdgePerm = cube![R2 L2 F Bp U2 D2 F Bp].edge_perm;

pub const S_LR_CORNER_PERM: CornerPerm =
    cube![U Dp F2 U F2 L2 U2 B2 R2 F2 L2 U2 F2 L2 Dp R2].corner_perm;
pub const S_LR_EDGE_PERM: EdgePerm = cube![U2 F2 D2 U2 F2 U2 F2 B2].edge_perm;

pub const S_U4_1_CORNER_PERM: CornerPerm = cube![B2 F2 D2 L2 R2 D B2 F2 U2 L2 R2 Up].corner_perm;
pub const S_U4_1_EDGE_PERM: EdgePerm =
    cube![D B2 U B2 F2 R2 U2 L2 Dp R2 F2 Dp L D2 B2 Dp F L2 D F Up Rp].edge_perm;
pub const S_U4_1_EDGE_ORIENT_CORRECT: EdgeOrient =
    EdgeOrient::const_from_array([0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1]);

// transform is 0b_00_dd_c_bb_a
// S_LR^a ( S_U4^b ( S_F2^c ( S_URF3^d ) ) )
// (perform these right to left)
// conjugation is then

// M
// S_LR^-a -> S_U4^-b -> S_F2^-c -> S_URF3^-d -> M -> S_URF3^d -> S_F2^c -> S_U4^b -> S_LR^a

const CORNER_PERM_LOOKUP: [CornerPerm; 48] = {
    let mut table = [CornerPerm::SOLVED; 48];
    let mut i = 0usize;
    while i < 48 {
        let a = i & 0b000001;
        let b = (i & 0b000110) >> 1;
        let c = (i & 0b001000) >> 3;
        let d = (i & 0b110000) >> 4;

        let mut j = 0;
        while j < d {
            table[i] = table[i].then(S_URF3_1_CORNER_PERM);
            j += 1;
        }

        if c == 1 {
            table[i] = table[i].then(S_F2_CORNER_PERM);
        }

        let mut j = 0;
        while j < b {
            table[i] = table[i].then(S_U4_1_CORNER_PERM);
            j += 1;
        }

        if a == 1 {
            table[i] = table[i].then(S_LR_CORNER_PERM);
        }

        i += 1;
    }

    table
};

pub const EDGE_PERM_LOOKUP: [EdgePerm; 48] = {
    let mut table = [EdgePerm::SOLVED; 48];
    let mut i = 0usize;
    while i < 48 {
        let a = i & 0b000001;
        let b = (i & 0b000110) >> 1;
        let c = (i & 0b001000) >> 3;
        let d = (i & 0b110000) >> 4;

        let mut j = 0;
        while j < d {
            table[i] = table[i].then(S_URF3_1_EDGE_PERM);
            j += 1;
        }

        if c == 1 {
            table[i] = table[i].then(S_F2_EDGE_PERM);
        }

        let mut j = 0;
        while j < b {
            table[i] = table[i].then(S_U4_1_EDGE_PERM);
            j += 1;
        }

        if a == 1 {
            table[i] = table[i].then(S_LR_EDGE_PERM);
        }

        i += 1;
    }

    table
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CubeSymmetry(pub u8);

/// only the 4 low bits of the transform described above. we omit the S_URF3 element and its consequent symmetries
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct DominoSymmetry(pub u8);

impl DominoSymmetry {
    pub const IDENTITY: Self = DominoSymmetry(0);

    #[must_use]
    pub const fn to_cube_symmetry(self) -> CubeSymmetry {
        CubeSymmetry(self.0)
    }

    #[must_use]
    pub fn into_index(self) -> usize {
        self.0 as usize
    }
}

impl From<DominoSymmetry> for CubeSymmetry {
    fn from(value: DominoSymmetry) -> Self {
        CubeSymmetry(value.0)
    }
}

impl TryFrom<CubeSymmetry> for DominoSymmetry {
    type Error = CubeSymmetry;

    fn try_from(value: CubeSymmetry) -> Result<Self, Self::Error> {
        if value.0 < 16 {
            Ok(DominoSymmetry(value.0))
        } else {
            Err(value)
        }
    }
}

impl CubeSymmetry {
    #[must_use]
    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0..48).map(Self)
    }

    #[must_use]
    pub fn nontrivial_iter() -> impl Iterator<Item = Self> {
        (1..48).map(Self)
    }
}

impl DominoSymmetry {
    #[must_use]
    pub const fn then(self, other: Self) -> Self {
        const TRANSFORM_COMPOSE_LOOKUP: [DominoSymmetry; 256] = {
            let reference_cube = cube![R U D2];

            let mut output = [DominoSymmetry(0); 256];

            let mut t1 = 0u8;
            while t1 < 16 {
                let conj_1 = reference_cube
                    .corner_perm
                    .domino_conjugate(DominoSymmetry(t1));
                let mut t2 = 0u8;
                while t2 < 16 {
                    let conj_2 = conj_1.domino_conjugate(DominoSymmetry(t2));
                    let i = ((t1 << 4) + t2) as usize;
                    let mut t3 = 0;
                    'deep: while t3 < 16 {
                        let conj_3 = reference_cube
                            .corner_perm
                            .domino_conjugate(DominoSymmetry(t3));
                        if conj_2.const_eq(conj_3) {
                            output[i] = DominoSymmetry(t3);
                            break 'deep;
                        }
                        t3 += 1;
                    }
                    t2 += 1;
                }
                t1 += 1;
            }

            output
        };

        TRANSFORM_COMPOSE_LOOKUP[((self.0 << 4) + other.0) as usize]
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then(self, other: Self) -> Self {
        let a = self.0;
        let b = other.0;
        let as1 = a << 1;
        let corr_mask = 0x4;
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        Self(a ^ b ^ corr)
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_2(a: u8, b: u8) -> u8 {
        let as1 = a << 1;
        let corr_mask = 0x44;
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_4(a: u16, b: u16) -> u16 {
        let as1 = a << 1;
        let corr_mask = 0x4444;
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_8(a: u32, b: u32) -> u32 {
        let as1 = a << 1;
        let corr_mask = 0x44444444;
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_16(a: u64, b: u64) -> u64 {
        let as1 = a << 1;
        let corr_mask = 0x4444444444444444;
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_32(a: u128, b: u128) -> u128 {
        let as1 = a << 1;
        let corr_mask = 0x44444444444444444444444444444444;
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub fn bitwise_then_simd<const N: usize>(a: Simd<u8, N>, b: Simd<u8, N>) -> Simd<u8, N> {
        let as1 = a << 1;
        let corr_mask = Simd::splat(0x44);
        let corr = (((b << 1) & (as1 ^ (a << 2))) ^ ((b >> 1) & as1)) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_invert(self) -> Self {
        let a = self.0;
        let corr_mask = 0x4;
        let corr = ((a << 1) & !((a << 2) ^ (a >> 1))) & corr_mask;
        Self(a ^ corr)
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_invert_32(a: u128) -> u128 {
        let corr_mask = 0x44444444444444444444444444444444;
        let corr = ((a << 1) & !((a << 2) ^ (a >> 1))) & corr_mask;
        a ^ corr
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_inv(self, other: Self) -> Self {
        let a = self.0;
        let b = other.0;
        let corr_mask = 0x4;
        let as1 = a << 1;             // a1 at bit 2
        let corr = (((b << 1) & !(as1 ^ (a << 2) ^ (b << 2) ^ (b >> 1)))
                ^ (as1 & (b >> 1))) & corr_mask;
        Self(a ^ b ^ corr)
    }

    #[must_use]
    #[inline(always)]
    pub const fn bitwise_then_inv_32(a: u128, b: u128) -> u128 {
        let corr_mask = 0x44444444444444444444444444444444;
        let as1 = a << 1;             // a1 at bit 2
        let corr = (((b << 1) & !(as1 ^ (a << 2) ^ (b << 2) ^ (b >> 1)))
                ^ (as1 & (b >> 1))) & corr_mask;
        a ^ b ^ corr
    }

    #[must_use]
    #[inline(always)]
    pub fn conjugate_corner_orient(
        self,
        dense: u16,
    ) -> u16 {
        let t = TBL[self.0 as usize];
        let mut coord = dense;

        // Expand a single flag bit to a u16 mask: 0x0000 or 0xFFFF
        let mask = |bit: u8| (((t >> bit) & 1) as i16).wrapping_neg() as u16;

        // 1. L: swap fields 1<->2 and 5<->6 (bit0<->bit1 of indices)
        let s = (coord & 0xC3C3) | ((coord & 0x0C0C) << 2) | ((coord >> 2) & 0x0C0C);
        coord ^= (coord ^ s) & mask(3);

        // 2. c' bit 0: XOR index with 1
        let s = ((coord & 0x3333) << 2) | ((coord >> 2) & 0x3333);
        coord ^= (coord ^ s) & mask(0);

        // 3. c' bit 1: XOR index with 2
        let s = ((coord & 0x0F0F) << 4) | ((coord >> 4) & 0x0F0F);
        coord ^= (coord ^ s) & mask(1);

        // 4. c' bit 2: XOR index with 4
        let s = (coord << 8) | (coord >> 8);
        coord ^= (coord ^ s) & mask(2);

        // 5. Conditional mirror: swaps the two bits within each 2-bit field
        let mirror_mask = ((self.0 & 1) as i16).wrapping_neg() as u16;
        let mirrored = ((coord & 0x5555) << 1) | ((coord >> 1) & 0x5555);
        coord ^= (coord ^ mirrored) & mirror_mask;

        let coord = coord & 0x3FFF; // omit the highest two_bits

        // pairs of 2-bit digits -> one 4-bit group, value = hi*3 + lo (0..=8)
        let coord = ((coord >> 2) & 0x3333) * 3
            + (coord & 0x3333);

        // pairs of 4-bit groups -> 8-bit group, value = hi*9 + lo (0..=80)
        let coord = ((coord >> 4) & 0x0F0F) * 9
            + (coord & 0x0F0F);

        // two 8-bit groups -> final, hi*81 + lo (0..=6560)
        let coord = (coord >> 8) * 81 + (coord & 0x00FF);

        coord
    }

    #[must_use]
    #[inline(always)]
    pub fn conjugate_corner_orient_16(
        syms: [u8; 16],
        dense_coords: [u16; 16],
    ) -> [u16; 16] {
        type U8  = Simd<u8, 16>;
        type U16 = Simd<u16, 16>;
        
        let t = U16::from_array(lut_u16x16x3(&TBL, &syms));
        let mut coord = U16::from_array(dense_coords);

        // Expand bit `n` of `t` to 0x0000 or 0xFFFF per lane.
        // Arithmetic shift right on signed i16 sign-extends the LSB.
        let m0 = ((t >> 0) & U16::splat(1)).wrapping_neg();
        let m1 = ((t >> 1) & U16::splat(1)).wrapping_neg();
        let m2 = ((t >> 2) & U16::splat(1)).wrapping_neg();
        let m3 = ((t >> 3) & U16::splat(1)).wrapping_neg();

        // 1. L: swap fields 1<->2 and 5<->6 (bit0<->bit1 of indices)
        let s = (coord & U16::splat(0xC3C3)) | ((coord & U16::splat(0x0C0C)) << 2) | ((coord >> 2) & U16::splat(0x0C0C));
        coord ^= (coord ^ s) & m3;

        // 2. c' bit 0: XOR index with 1
        let s = ((coord & U16::splat(0x3333)) << 2) | ((coord >> 2) & U16::splat(0x3333));
        coord ^= (coord ^ s) & m0;

        // 3. c' bit 1: XOR index with 2
        let s = ((coord & U16::splat(0x0F0F)) << 4) | ((coord >> 4) & U16::splat(0x0F0F));
        coord ^= (coord ^ s) & m1;

        // 4. c' bit 2: XOR index with 4
        let s = (coord << 8) | (coord >> 8);
        coord ^= (coord ^ s) & m2;

        // 5. Conditional mirror: swaps the two bits within each 2-bit field
        let wide_syms: U16 = U8::from_array(syms).cast();
        let mirror_mask = (wide_syms & U16::splat(1)).wrapping_neg();
        let mirrored = ((coord & U16::splat(0x5555)) << 1) | ((coord >> 1) & U16::splat(0x5555));
        coord ^= (coord ^ mirrored) & mirror_mask;

        let v = coord & Simd::splat(0x3FFF); // omit the highest two_bits

        // pairs of 2-bit digits -> one 4-bit group, value = hi*3 + lo (0..=8)
        let v = ((v >> 2) & Simd::splat(0x3333)) * Simd::splat(3)
            + (v & Simd::splat(0x3333));

        // pairs of 4-bit groups -> 8-bit group, value = hi*9 + lo (0..=80)
        let v = ((v >> 4) & Simd::splat(0x0F0F)) * Simd::splat(9)
            + (v & Simd::splat(0x0F0F));

        // two 8-bit groups -> final, hi*81 + lo (0..=6560)
        let v = (v >> 8) * Simd::splat(81) + (v & Simd::splat(0x00FF));

        v.to_array()
    }

    #[must_use]
    pub const fn inverse(self) -> Self {
        const TRANSFORM_INVERT_LOOKUP: [DominoSymmetry; 16] = {
            let reference_cube = cube![R U D2];

            let mut output = [DominoSymmetry(0); 16];

            let mut t1 = 0u8;
            while t1 < 16 {
                let conj_1 = reference_cube
                    .corner_perm
                    .domino_conjugate(DominoSymmetry(t1));
                let mut t2 = 0u8;
                while t2 < 16 {
                    let conj_2 = conj_1.domino_conjugate(DominoSymmetry(t2));
                    if conj_2.const_eq(reference_cube.corner_perm) {
                        output[t1 as usize] = DominoSymmetry(t2);
                    }
                    t2 += 1;
                }
                t1 += 1;
            }

            output
        };

        TRANSFORM_INVERT_LOOKUP[self.0 as usize]
    }

    #[must_use]
    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0..16).map(Self)
    }

    #[must_use]
    pub fn nontrivial_iter() -> impl Iterator<Item = Self> {
        (1..16).map(Self)
    }
}

impl CornerPerm {
    #[must_use]
    pub const fn conjugate_perms(self, sym: CubeSymmetry) -> (Self, Self) {
        let perm = CORNER_PERM_LOOKUP[sym.0 as usize];
        let inv_perm = perm.inverse();
        (perm, inv_perm)
    }

    #[must_use]
    pub const fn conjugate(self, sym: CubeSymmetry) -> Self {
        let (perm, inv_perm) = self.conjugate_perms(sym);
        inv_perm.then(self).then(perm)
    }

    #[must_use]
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        self.conjugate(sym.to_cube_symmetry())
    }
}

impl EdgePerm {
    #[must_use]
    pub const fn conjugate_perms(self, sym: CubeSymmetry) -> (Self, Self) {
        let perm = EDGE_PERM_LOOKUP[sym.0 as usize];
        let inv_perm = perm.inverse();
        (perm, inv_perm)
    }

    #[must_use]
    pub const fn conjugate(self, sym: CubeSymmetry) -> Self {
        let (perm, inv_perm) = self.conjugate_perms(sym);
        inv_perm.then(self).then(perm)
    }

    #[must_use]
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        self.conjugate(sym.to_cube_symmetry())
    }
}

impl CornerOrient {
    #[must_use]
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = CORNER_PERM_LOOKUP[sym.0 as usize];
        let permuted = self.permute(perm);
        if sym.0 % 2 == 1 {
            permuted.mirror()
        } else {
            permuted
        }
    }
}

impl ReprCube {
    #[must_use]
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        Self {
            corner_perm: self.corner_perm.domino_conjugate(sym),
            corner_orient: self.corner_orient.domino_conjugate(sym),
            edge_perm: self.edge_perm.domino_conjugate(sym),
            edge_orient: crate::kociemba::partial_reprs::edge_group_orient::EdgeGroupOrient(
                EdgeGroup(split_edge_positions(self.edge_perm).2.0.split().0),
                self.edge_orient,
            )
            .domino_conjugate(sym)
            .1,
        }
    }

    #[must_use]
    pub const fn conjugate(self, sym: CubeSymmetry) -> Self {
        let domino_component = DominoSymmetry(sym.0 & 0xF);
        let domino_conjugated = self.domino_conjugate(domino_component);
        match sym.0 >> 4 {
            0 => domino_conjugated,
            1 => S_URF3_1_CUBE
                .then(domino_conjugated)
                .then(S_URF3_1_CUBE)
                .then(S_URF3_1_CUBE),
            2 => S_URF3_1_CUBE
                .then(S_URF3_1_CUBE)
                .then(domino_conjugated)
                .then(S_URF3_1_CUBE),
            _ => unreachable!(),
        }
    }
}

impl CubeMove {
    pub const fn conjugate(self, _sym: CubeSymmetry) -> Self {
        unimplemented!()
    }

    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        const TABLE: [CubeMove; 18 * 16] = const {
            let mut move_reference = [CornerPerm::SOLVED; 18];

            let mut i = 0;
            while i < 18 {
                let mv: CubeMove = unsafe { core::mem::transmute(i as u8) };
                move_reference[i] = mv.into_corner_perm();
                i += 1;
            }

            let mut val = [CubeMove::U1; 18 * 16];
            let mut i = 0;
            while i < 18 {
                let mut j = 0;
                let mv: CubeMove = unsafe { core::mem::transmute(i as u8) };
                while j < 16 {
                    let sym: DominoSymmetry = unsafe { core::mem::transmute(j as u8) };
                    val[i * 16 + j] = {
                        let perm = mv.into_corner_perm().domino_conjugate(sym);
                        let mut k = 0;
                        'k: while k < 18 {
                            let mut l = 0;
                            let mut equal = true;
                            'l: while l < 8 {
                                if move_reference[k].0.0[l] != perm.0.0[l] {
                                    equal = false;
                                    break 'l;
                                }
                                l += 1;
                            }
                            if equal {
                                break 'k;
                            }

                            k += 1;
                        }

                        unsafe { core::mem::transmute::<u8, CubeMove>(k as u8) }
                    };
                    j += 1;
                }
                i += 1;
            }

            val
        };
        TABLE[self.into_index() * 16 + (sym.0 as usize)]
    }
}
/// TBL encodes (swap_flag << 3) | c' where c' = L(c) for the GATHER convention.
const TBL: [u16; 24] = [
    0,  1,  9,  8,  3,  2, 10, 11,
    5,  4, 15, 14,  6,  7, 12, 13,
    0, 0, 0, 0, 0, 0, 0, 0,
];

// /// Branchless permutation of 2-bit orientation fields according to symmetry `sym` (0..15).
// /// Implements gather convention: new[i] = old[p[i]] (matches your apply_to/invert logic).
// pub fn permute(sym: u8, mut coord: u16) -> u16 {
//     debug_assert!(sym < 16);
//     let t = TBL[sym as usize];

//     // Expand a single flag bit to a u16 mask: 0x0000 or 0xFFFF
//     let mask = |bit: u8| (((t >> bit) & 1) as i16).wrapping_neg() as u16;

//     // 1. L: swap fields 1<->2 and 5<->6 (bit0<->bit1 of indices)
//     let s = (coord & 0xC3C3) | ((coord & 0x0C0C) << 2) | ((coord >> 2) & 0x0C0C);
//     coord ^= (coord ^ s) & mask(3);

//     // 2. c' bit 0: XOR index with 1 (swap adjacent 2-bit fields)
//     let s = ((coord & 0x3333) << 2) | ((coord >> 2) & 0x3333);
//     coord ^= (coord ^ s) & mask(0);

//     // 3. c' bit 1: XOR index with 2 (swap adjacent 4-bit fields)
//     let s = ((coord & 0x0F0F) << 4) | ((coord >> 4) & 0x0F0F);
//     coord ^= (coord ^ s) & mask(1);

//     // 4. c' bit 2: XOR index with 4 (swap halves)
//     let s = (coord << 8) | (coord >> 8);
//     coord ^= (coord ^ s) & mask(2);

//     coord
// }



#[test]
fn check_no_duplicates_for_corner_perm_repr() {
    let cube = cube![R U D2];

    let mut set = std::collections::HashSet::new();

    for sym in CubeSymmetry::all_iter() {
        assert!(set.insert(cube.corner_perm.conjugate(sym)))
    }
}

#[test]
fn print_inv() {
    for i in 0..16 {
        let s = DominoSymmetry(i);
        let j = s.inverse().0;

        println!("{:x} {:x}", i, j)
    }
}

#[test]
fn random_cubes_with_conjugation() {
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for _ in 0..100 {
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        for a in DominoSymmetry::all_iter() {
            for b in DominoSymmetry::all_iter() {
                assert_eq!(
                    cube.domino_conjugate(a).domino_conjugate(b),
                    cube.domino_conjugate(a.then(b))
                )
            }
        }
    }
}

#[test]
fn test_long_apply() {
    let c = cube![R U2 Dp B Dp];

    let mut c2 = ReprCube::SOLVED;
    c2 = c2.then(c);
    let mut count = 1;
    while c2 != ReprCube::SOLVED {
        count += 1;
        c2 = c2.then(c);
    }
    assert_eq!(count, 1260);
}

#[test]
fn then_and_then_no_lookup_agree() {
    for a in 0u8..16 {
        for b in 0u8..16 {
            let sym_a = DominoSymmetry(a);
            let sym_b = DominoSymmetry(b);

            assert_eq!(
                sym_a.then(sym_b),
                sym_a.bitwise_then(sym_b),
                "mismatch at ({a}, {b})",
            );
        }
    }
}

#[test]
fn nolookup_conjugate_matches() {
    for s in DominoSymmetry::all_iter() {
        for c in 0..2187u16 {
            let co = CornerOrient::from_coord(CornerOrientRawCoord(c));
            let reference = co.domino_conjugate(s).into_coord().0;
            let test = DominoSymmetry::conjugate_corner_orient(s, co.into_dense());

            assert_eq!(reference, test);
        }
    }
}

#[test]
fn nolookup_conjugate_matches_16() {
    for s in DominoSymmetry::all_iter() {
        for c in 0..2187u16 {
            let co = CornerOrient::from_coord(CornerOrientRawCoord(c));
            let reference = co.domino_conjugate(s).into_coord().0;
            let test = DominoSymmetry::conjugate_corner_orient_16([s.0; 16], [co.into_dense(); 16]);

            assert_eq!([reference; 16], test);
        }
    }
}

#[test]
fn basic_sequence() {
    let cube = cube![R U D2 U L F B L];

    for sym in DominoSymmetry::all_iter() {
        cube.domino_conjugate(sym).pretty_print();
        println!()
    }
}

#[test]
fn print_corner_sym() {
    for i in DominoSymmetry::all_iter() {
        println!("{:?}", CORNER_PERM_LOOKUP[i.0 as usize]);
    }
}


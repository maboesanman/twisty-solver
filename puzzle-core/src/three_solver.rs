use std::{
    ops::{Add, Rem},
    simd::{Mask, Simd},
};

pub struct CubieRepr {
    // see corner resident enum for ordering
    corner_perm: [CornerResident; 8],
    corner_orient: [CornerOrient; 8],

    // see edge resident enum for ordering
    // only first 12 are used
    edge_perm: [EdgeResident; 12],
    edge_orient: [EdgeOrient; 12],
}

// consts for performing moves on CubieRepr

const fn compose<const N: usize>(base: &[usize; N], next: &[usize; N]) -> [usize; N] {
    let mut x = [0; N];
    let mut i = 0;
    loop {
        x[i] = base[next[i]];
        if i == N - 1 {
            break;
        }
        i += 1;
    }
    x
}

// corner permutations
const U1_CORNER_INDEX: [usize; 8] = [2, 0, 3, 1, 4, 5, 6, 7];
const D1_CORNER_INDEX: [usize; 8] = [0, 1, 2, 3, 5, 7, 4, 6];
const F1_CORNER_INDEX: [usize; 8] = [1, 5, 2, 3, 0, 4, 6, 7];
const B1_CORNER_INDEX: [usize; 8] = [0, 1, 6, 2, 4, 5, 7, 3];
const R1_CORNER_INDEX: [usize; 8] = [4, 1, 0, 3, 6, 5, 2, 7];
const L1_CORNER_INDEX: [usize; 8] = [0, 3, 2, 7, 4, 1, 6, 5];

// edge permutations
const U1_EDGE_INDEX: [usize; 16] = [2, 3, 1, 0, 4, 5, 6, 7, 8, 9, 10, 11, 15, 15, 15, 15];
const D1_EDGE_INDEX: [usize; 16] = [0, 1, 2, 3, 7, 6, 4, 5, 8, 9, 10, 11, 15, 15, 15, 15];
const F1_EDGE_INDEX: [usize; 16] = [9, 1, 2, 3, 8, 5, 6, 7, 0, 4, 10, 11, 15, 15, 15, 15];
const B1_EDGE_INDEX: [usize; 16] = [0, 10, 2, 3, 4, 11, 6, 7, 8, 9, 5, 1, 15, 15, 15, 15];
const R1_EDGE_INDEX: [usize; 16] = [0, 1, 8, 3, 4, 5, 10, 7, 6, 9, 2, 11, 15, 15, 15, 15];
const L1_EDGE_INDEX: [usize; 16] = [0, 1, 2, 11, 4, 5, 6, 9, 8, 3, 10, 7, 15, 15, 15, 15];

// corner orientation corrections (added after permuting)
const F_CORNER_ROT: [u8; 8] = [1, 2, 0, 0, 2, 1, 0, 0];
const B_CORNER_ROT: [u8; 8] = [0, 0, 2, 1, 0, 0, 1, 2];
const R_CORNER_ROT: [u8; 8] = [2, 0, 1, 0, 1, 0, 2, 0];
const L_CORNER_ROT: [u8; 8] = [0, 1, 0, 2, 0, 2, 0, 1];

// edge orientation corrections (added after permuting)
const F_EDGE_FLIP: [u8; 16] = [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0];
const B_EDGE_FLIP: [u8; 16] = [0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0];

// consts for reducing orientation modulo
const SPLAT_3: [u8; 8] = [3; 8];
const SPLAT_2: [u8; 16] = [2; 16];
const EDGE_MASK: [bool; 16] = [
    true, true, true, true, true, true, true, true, true, true, true, true, false, false, false,
    false,
];
const EDGE_OR: [u8; 16] = [0; 16];

const U2_CORNER_INDEX: [usize; 8] = compose(&U1_CORNER_INDEX, &U1_CORNER_INDEX);
const U3_CORNER_INDEX: [usize; 8] = compose(&U2_CORNER_INDEX, &U1_CORNER_INDEX);
const D2_CORNER_INDEX: [usize; 8] = compose(&D1_CORNER_INDEX, &D1_CORNER_INDEX);
const D3_CORNER_INDEX: [usize; 8] = compose(&D2_CORNER_INDEX, &D1_CORNER_INDEX);
const F2_CORNER_INDEX: [usize; 8] = compose(&F1_CORNER_INDEX, &F1_CORNER_INDEX);
const F3_CORNER_INDEX: [usize; 8] = compose(&F2_CORNER_INDEX, &F1_CORNER_INDEX);
const B2_CORNER_INDEX: [usize; 8] = compose(&B1_CORNER_INDEX, &B1_CORNER_INDEX);
const B3_CORNER_INDEX: [usize; 8] = compose(&B2_CORNER_INDEX, &B1_CORNER_INDEX);
const R2_CORNER_INDEX: [usize; 8] = compose(&R1_CORNER_INDEX, &R1_CORNER_INDEX);
const R3_CORNER_INDEX: [usize; 8] = compose(&R2_CORNER_INDEX, &R1_CORNER_INDEX);
const L2_CORNER_INDEX: [usize; 8] = compose(&L1_CORNER_INDEX, &L1_CORNER_INDEX);
const L3_CORNER_INDEX: [usize; 8] = compose(&L2_CORNER_INDEX, &L1_CORNER_INDEX);

const U2_EDGE_INDEX: [usize; 16] = compose(&U1_EDGE_INDEX, &U1_EDGE_INDEX);
const U3_EDGE_INDEX: [usize; 16] = compose(&U2_EDGE_INDEX, &U1_EDGE_INDEX);
const D2_EDGE_INDEX: [usize; 16] = compose(&D1_EDGE_INDEX, &D1_EDGE_INDEX);
const D3_EDGE_INDEX: [usize; 16] = compose(&D2_EDGE_INDEX, &D1_EDGE_INDEX);
const F2_EDGE_INDEX: [usize; 16] = compose(&F1_EDGE_INDEX, &F1_EDGE_INDEX);
const F3_EDGE_INDEX: [usize; 16] = compose(&F2_EDGE_INDEX, &F1_EDGE_INDEX);
const B2_EDGE_INDEX: [usize; 16] = compose(&B1_EDGE_INDEX, &B1_EDGE_INDEX);
const B3_EDGE_INDEX: [usize; 16] = compose(&B2_EDGE_INDEX, &B1_EDGE_INDEX);
const R2_EDGE_INDEX: [usize; 16] = compose(&R1_EDGE_INDEX, &R1_EDGE_INDEX);
const R3_EDGE_INDEX: [usize; 16] = compose(&R2_EDGE_INDEX, &R1_EDGE_INDEX);
const L2_EDGE_INDEX: [usize; 16] = compose(&L1_EDGE_INDEX, &L1_EDGE_INDEX);
const L3_EDGE_INDEX: [usize; 16] = compose(&L2_EDGE_INDEX, &L1_EDGE_INDEX);

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum CornerResident {
    UFR = 0,
    UFL = 1,
    UBR = 2,
    UBL = 3,
    DFR = 4,
    DFL = 5,
    DBR = 6,
    DBL = 7,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum CornerOrient {
    Solved = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum EdgeResident {
    UF = 0,
    UB = 1,
    UR = 2,
    UL = 3,
    DF = 4,
    DB = 5,
    DR = 6,
    DL = 7,
    FR = 8,
    FL = 9,
    BR = 10,
    BL = 11,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum EdgeOrient {
    Solved = 0,
    Unsolved = 1,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum Phase1Move {
    U1,
    U2,
    U3,
    D1,
    D2,
    D3,
    F1,
    F2,
    F3,
    B1,
    B2,
    B3,
    R1,
    R2,
    R3,
    L1,
    L2,
    L3,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum Phase2Move {
    U1,
    U2,
    U3,
    D1,
    D2,
    D3,
    F2,
    B2,
    R2,
    L2,
}

// 2187 (11.09 bits)
#[repr(transparent)]
pub struct CornerOrientCoord(u16);

// 2048 (11 bits)
#[repr(transparent)]
pub struct EdgeOrientCoord(u16);

// 40320 (15.29 bits)
#[repr(transparent)]
pub struct CornerPermutationCoord(u16);

// // 39916800 (25.25 bits)
// type EdgePermutationCoord = u32;

// 495 (8.9 bits)
#[repr(transparent)]
pub struct EdgeGroupingCoord(u16);

// 40320 (15.29 bits)
#[repr(transparent)]
pub struct UDEdgePermutationCoord(u16);

// 24 (4.58 bits)
#[repr(transparent)]
pub struct EEdgePermutationCoord(u8);

// fn permutation_coord_12<T: Ord>(perm: &[T; 12]) -> u32 {
// 	let mut sum = 0;
// 	let factorials: [u32; 11] = [1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];
// 	for i in 1..12 {
// 		for j in 0..i {
// 			if perm[j] > perm[i] {
// 				sum += factorials[i - 1]
// 			}
// 		}
// 	}
// 	sum
// }

fn permutation_coord_8<T: Ord>(perm: &[T; 8]) -> u16 {
    let mut sum = 0;
    let factorials: [u16; 7] = [1, 2, 6, 24, 120, 720, 5040];
    for i in 1..8 {
        for j in 0..i {
            if perm[j] > perm[i] {
                sum += factorials[i - 1]
            }
        }
    }
    sum
}

fn permutation_coord_4<T: Ord>(perm: &[T; 4]) -> u8 {
    let mut sum = 0;
    let factorials: [u8; 3] = [1, 2, 6];
    for i in 1..4 {
        for j in 0..i {
            if perm[j] > perm[i] {
                sum += factorials[i - 1]
            }
        }
    }
    sum
}

impl Default for CubieRepr {
    fn default() -> Self {
        Self {
            corner_perm: [
                CornerResident::UFR,
                CornerResident::UFL,
                CornerResident::UBR,
                CornerResident::UBL,
                CornerResident::DFR,
                CornerResident::DFL,
                CornerResident::DBR,
                CornerResident::DBL,
            ],
            corner_orient: [
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
            ],
            edge_perm: [
                EdgeResident::UF,
                EdgeResident::UB,
                EdgeResident::UR,
                EdgeResident::UL,
                EdgeResident::DF,
                EdgeResident::DB,
                EdgeResident::DR,
                EdgeResident::DL,
                EdgeResident::FR,
                EdgeResident::FL,
                EdgeResident::BR,
                EdgeResident::BL,
            ],
            edge_orient: [
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
            ],
        }
    }
}

#[allow(dead_code)]
impl CubieRepr {
    fn phase_1_move(&mut self, m: Phase1Move) {
        let (corner_idx, edge_idx, corner_rot, edge_flip) = match m {
            Phase1Move::U1 => (U1_CORNER_INDEX, U1_EDGE_INDEX, None, None),
            Phase1Move::U2 => (U2_CORNER_INDEX, U2_EDGE_INDEX, None, None),
            Phase1Move::U3 => (U3_CORNER_INDEX, U3_EDGE_INDEX, None, None),
            Phase1Move::D1 => (D1_CORNER_INDEX, D1_EDGE_INDEX, None, None),
            Phase1Move::D2 => (D2_CORNER_INDEX, D2_EDGE_INDEX, None, None),
            Phase1Move::D3 => (D3_CORNER_INDEX, D3_EDGE_INDEX, None, None),
            Phase1Move::F1 => (
                F1_CORNER_INDEX,
                F1_EDGE_INDEX,
                Some(F_CORNER_ROT),
                Some(F_EDGE_FLIP),
            ),
            Phase1Move::F2 => (F2_CORNER_INDEX, F2_EDGE_INDEX, None, None),
            Phase1Move::F3 => (
                F3_CORNER_INDEX,
                F3_EDGE_INDEX,
                Some(F_CORNER_ROT),
                Some(F_EDGE_FLIP),
            ),
            Phase1Move::B1 => (
                B1_CORNER_INDEX,
                B1_EDGE_INDEX,
                Some(B_CORNER_ROT),
                Some(B_EDGE_FLIP),
            ),
            Phase1Move::B2 => (B2_CORNER_INDEX, B2_EDGE_INDEX, None, None),
            Phase1Move::B3 => (
                B3_CORNER_INDEX,
                B3_EDGE_INDEX,
                Some(B_CORNER_ROT),
                Some(B_EDGE_FLIP),
            ),
            Phase1Move::R1 => (R1_CORNER_INDEX, R1_EDGE_INDEX, Some(R_CORNER_ROT), None),
            Phase1Move::R2 => (R2_CORNER_INDEX, R2_EDGE_INDEX, None, None),
            Phase1Move::R3 => (R3_CORNER_INDEX, R3_EDGE_INDEX, Some(R_CORNER_ROT), None),
            Phase1Move::L1 => (L1_CORNER_INDEX, L1_EDGE_INDEX, Some(L_CORNER_ROT), None),
            Phase1Move::L2 => (L2_CORNER_INDEX, L2_EDGE_INDEX, None, None),
            Phase1Move::L3 => (L3_CORNER_INDEX, L3_EDGE_INDEX, Some(L_CORNER_ROT), None),
        };

        let corner_perm: [u8; 8];
        let corner_orient: [u8; 8];
        let mut edge_perm: [u8; 16] = [0; 16];
        let mut edge_orient: [u8; 16] = [0; 16];

        let new_corner_perm: [u8; 8];
        let new_corner_orient: [u8; 8];
        let mut new_edge_perm: [u8; 12] = [0; 12];
        let mut new_edge_orient: [u8; 12] = [0; 12];

        unsafe {
            corner_perm = *std::mem::transmute::<&[_; 8], &[u8; 8]>(&self.corner_perm);
            corner_orient = *std::mem::transmute::<&[_; 8], &[u8; 8]>(&self.corner_orient);
            edge_perm[..12]
                .copy_from_slice(std::mem::transmute::<&[_; 12], &[u8; 12]>(&self.edge_perm));
            edge_orient[..12].copy_from_slice(std::mem::transmute::<&[_; 12], &[u8; 12]>(
                &self.edge_orient,
            ));
        };

        let corner_idx = Simd::from_array(corner_idx);
        let edge_idx = Simd::from_array(edge_idx);
        let corner_rot = corner_rot.map(|x| Simd::from_array(x));
        let edge_flip = edge_flip.map(|x| Simd::from_array(x));
        let splat_2 = Simd::from_array(SPLAT_2);
        let splat_3 = Simd::from_array(SPLAT_3);
        let edge_mask = Mask::from_array(EDGE_MASK);
        let edge_or = Simd::from_array(EDGE_OR);

        new_corner_perm = Simd::gather_or_default(&corner_perm, corner_idx).into();

        let new_edge_perm_padded: [u8; 16] =
            Simd::gather_select(&edge_perm, edge_mask, edge_idx, edge_or).into();
        new_edge_perm.copy_from_slice(&new_edge_perm_padded[..12]);

        let s = Simd::gather_or_default(&corner_orient, corner_idx);
        new_corner_orient = match corner_rot {
            Some(r) => s.add(r).rem(splat_3),
            None => s,
        }
        .into();

        let s = Simd::gather_select(&edge_orient, edge_mask, edge_idx, edge_or);
        let new_edge_orient_padded: [u8; 16] = match edge_flip {
            Some(f) => s.add(f).rem(splat_2),
            None => s,
        }
        .into();

        new_edge_orient.copy_from_slice(&new_edge_orient_padded[..12]);

        unsafe {
            self.corner_perm = std::mem::transmute(new_corner_perm);
            self.corner_orient = std::mem::transmute(new_corner_orient);
            self.edge_perm = std::mem::transmute(new_edge_perm);
            self.edge_orient = std::mem::transmute(new_edge_orient);
        }
    }

    fn phase_2_move(&mut self, m: Phase2Move) {
        let p1_move = match m {
            Phase2Move::U1 => Phase1Move::U1,
            Phase2Move::U2 => Phase1Move::U2,
            Phase2Move::U3 => Phase1Move::U3,
            Phase2Move::D1 => Phase1Move::D1,
            Phase2Move::D2 => Phase1Move::D2,
            Phase2Move::D3 => Phase1Move::D3,
            Phase2Move::F2 => Phase1Move::F2,
            Phase2Move::B2 => Phase1Move::B2,
            Phase2Move::R2 => Phase1Move::R2,
            Phase2Move::L2 => Phase1Move::L2,
        };
        self.phase_1_move(p1_move)
    }
    // phase 1
    pub fn coord_corner_orient(&self) -> CornerOrientCoord {
        let mut sum = 0u16;
        for i in (0..7).rev() {
            sum *= 3;
            sum += self.corner_orient[i] as u16;
        }

        CornerOrientCoord(sum)
    }

    // phase 1
    pub fn coord_edge_orient(&self) -> EdgeOrientCoord {
        let mut sum = 0u16;
        for i in (0..11).rev() {
            sum <<= 1;
            sum += self.edge_orient[i] as u16;
        }

        EdgeOrientCoord(sum)
    }

    // phase 1
    pub fn coord_edge_grouping(&self) -> EdgeGroupingCoord {
        let factorials: [u32; 12] = [
            1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
        ];
        let mut sum = 0;
        let mut k = 3;
        for n in (0..12).rev() {
            if (self.edge_perm[n] as u8) < 8 {
                sum += (factorials[n] / factorials[k] / factorials[n - k]) as u16
            } else if k == 0 {
                break;
            } else {
                k -= 1;
            }
        }

        EdgeGroupingCoord(sum)
    }

    // phase 2
    pub fn coord_corner_perm(&self) -> CornerPermutationCoord {
        CornerPermutationCoord(permutation_coord_8(&self.corner_perm))
    }

    // phase 2
    pub fn coord_ud_edge_perm(&self) -> UDEdgePermutationCoord {
        UDEdgePermutationCoord(permutation_coord_8(self.edge_perm[..8].try_into().unwrap()))
    }

    // phase 2
    pub fn coord_e_edge_perm(&self) -> EEdgePermutationCoord {
        EEdgePermutationCoord(permutation_coord_4(self.edge_perm[8..].try_into().unwrap()))
    }

    fn is_valid(&self) -> bool {
        let mut v = self
            .corner_perm
            .iter()
            .map(|x| *x as u8)
            .collect::<Vec<_>>();
        v.sort();
        if v != (0..8u8).into_iter().collect::<Vec<_>>() {
            return false;
        }

        let mut v = self.edge_perm.iter().map(|x| *x as u8).collect::<Vec<_>>();
        v.sort();
        if v != (0..12u8).into_iter().collect::<Vec<_>>() {
            return false;
        }

        true
    }

    fn is_solved(&self) -> bool {
        let mut v = self
            .corner_perm
            .iter()
            .map(|x| *x as u8)
            .collect::<Vec<_>>();
        if v != (0..8u8).into_iter().collect::<Vec<_>>() {
            return false;
        }

        let mut v = self.edge_perm.iter().map(|x| *x as u8).collect::<Vec<_>>();
        if v != (0..12u8).into_iter().collect::<Vec<_>>() {
            return false;
        }

        true
    }
}

#[test]
fn coord_test() {
    let c = CubieRepr::default();

    // phase 1
    assert_eq!(c.coord_corner_orient().0, 0);
    assert_eq!(c.coord_edge_orient().0, 0);
    assert_eq!(c.coord_edge_grouping().0, 0);

    // phase 2
    assert_eq!(c.coord_corner_perm().0, 0);
    assert_eq!(c.coord_ud_edge_perm().0, 0);
    assert_eq!(c.coord_e_edge_perm().0, 0);
}

#[test]
fn coord_move_test() {
    let mut c = CubieRepr::default();
    c.phase_1_move(Phase1Move::U1);
    c.phase_1_move(Phase1Move::U2);
    c.phase_1_move(Phase1Move::U3);
    c.phase_1_move(Phase1Move::U2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Phase1Move::D1);
    c.phase_1_move(Phase1Move::D2);
    c.phase_1_move(Phase1Move::D3);
    c.phase_1_move(Phase1Move::D2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Phase1Move::F1);
    c.phase_1_move(Phase1Move::F2);
    c.phase_1_move(Phase1Move::F3);
    c.phase_1_move(Phase1Move::F2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Phase1Move::B1);
    c.phase_1_move(Phase1Move::B2);
    c.phase_1_move(Phase1Move::B3);
    c.phase_1_move(Phase1Move::B2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Phase1Move::R1);
    c.phase_1_move(Phase1Move::R2);
    c.phase_1_move(Phase1Move::R3);
    c.phase_1_move(Phase1Move::R2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Phase1Move::L1);
    c.phase_1_move(Phase1Move::L2);
    c.phase_1_move(Phase1Move::L3);
    c.phase_1_move(Phase1Move::L2);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn sexy_move() {
    let mut c = CubieRepr::default();

    for _ in 0..6 {
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::F1);
        c.phase_1_move(Phase1Move::U3);
        c.phase_1_move(Phase1Move::F3);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

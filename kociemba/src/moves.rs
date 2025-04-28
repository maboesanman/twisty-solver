use std::{
    intrinsics::size_of,
    simd::{Mask, Simd},
};

use super::repr_cubie::{
    corner_orient_offset, corner_perm_offset, edge_orient_offset, edge_perm_offset, ReprCubie,
};

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(unused)]
pub enum Move {
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

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Move::U1 => "U",
            Move::U2 => "U2",
            Move::U3 => "U'",
            Move::D1 => "D",
            Move::D2 => "D2",
            Move::D3 => "D'",
            Move::F1 => "F",
            Move::F2 => "F2",
            Move::F3 => "F'",
            Move::B1 => "B",
            Move::B2 => "B2",
            Move::B3 => "B'",
            Move::R1 => "R",
            Move::R2 => "R2",
            Move::R3 => "R'",
            Move::L1 => "L",
            Move::L2 => "L2",
            Move::L3 => "L'",
        };
        f.write_str(string)
    }
}

impl Move {
    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0u8..18u8).map(|x| unsafe { core::mem::transmute(x) })
    }
}

impl From<Phase2Move> for Move {
    fn from(value: Phase2Move) -> Self {
        match value {
            Phase2Move::U1 => Move::U1,
            Phase2Move::U2 => Move::U2,
            Phase2Move::U3 => Move::U3,
            Phase2Move::D1 => Move::D1,
            Phase2Move::D2 => Move::D2,
            Phase2Move::D3 => Move::D3,
            Phase2Move::F2 => Move::F2,
            Phase2Move::B2 => Move::B2,
            Phase2Move::R2 => Move::R2,
            Phase2Move::L2 => Move::L2,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(unused)]
pub enum Phase2Move {
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

impl Phase2Move {
    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0u8..10u8).map(|x| unsafe { core::mem::transmute(x) })
    }
}

pub const fn combined_index(
    corner_index: &[u8; 8],
    edge_index: &[u8; 12],
) -> [usize; size_of::<ReprCubie>()] {
    let mut buf = [0usize; size_of::<ReprCubie>()];

    let corner_perm_offset = corner_perm_offset();
    let corner_orient_offset = corner_orient_offset();
    let edge_perm_offset = edge_perm_offset();
    let edge_orient_offset = edge_orient_offset();

    let mut i = 0;
    while i < 8 {
        buf[i + corner_perm_offset] = (corner_index[i] as usize) + corner_perm_offset;
        i += 1;
    }
    let mut i = 0;
    while i < 8 {
        buf[i + corner_orient_offset] = (corner_index[i] as usize) + corner_orient_offset;
        i += 1;
    }
    let mut i = 0;
    while i < 12 {
        buf[i + edge_perm_offset] = (edge_index[i] as usize) + edge_perm_offset;
        i += 1;
    }
    let mut i = 0;
    while i < 12 {
        buf[i + edge_orient_offset] = (edge_index[i] as usize) + edge_orient_offset;
        i += 1;
    }

    buf
}

const fn combined_mask(index: &[usize; 40]) -> [bool; 40] {
    let mut mask = [true; 40];

    let mut i = 0usize;
    while i < 40 {
        // if index moves i, mask is true
        mask[i] = i != index[i];
        i += 1;
    }

    mask
}

pub const fn combined_orient(corner_rot: &[u8; 8], edge_flip: &[u8; 12]) -> [u8; 20] {
    let mut buf = [0u8; 20];

    if corner_orient_offset() + 8 != edge_orient_offset() {
        panic!();
    }

    let mut i = 0;
    while i < 8 {
        buf[i] = corner_rot[i];
        i += 1;
    }
    while i < 20 {
        buf[i] = edge_flip[i - 8];
        i += 1;
    }

    buf
}

// consts for performing moves on CubieRepr
pub const fn compose(base: &[usize; 40], next: &[usize; 40]) -> [usize; 40] {
    let mut x = [0; 40];
    let mut i = 0;
    while i < 40 {
        x[i] = base[next[i]];
        i += 1;
    }
    x
}

const fn pad<T: Copy>(mask: &[T; 40], default: T) -> [T; 64] {
    let mut buf = [default; 64];

    let mut i = 0;
    while i < 40 {
        buf[i] = mask[i];
        i += 1;
    }
    buf
}

// THIS IS THE MANUAL PERMUTATION DATA FOR THE GENERATIVE ELEMENTS OF THE GROUP

// corner permutations
const U1_CORNER_INDEX: [u8; 8] = [2, 0, 3, 1, 4, 5, 6, 7];
const D1_CORNER_INDEX: [u8; 8] = [0, 1, 2, 3, 5, 7, 4, 6];
const F1_CORNER_INDEX: [u8; 8] = [1, 5, 2, 3, 0, 4, 6, 7];
const B1_CORNER_INDEX: [u8; 8] = [0, 1, 6, 2, 4, 5, 7, 3];
const R1_CORNER_INDEX: [u8; 8] = [4, 1, 0, 3, 6, 5, 2, 7];
const L1_CORNER_INDEX: [u8; 8] = [0, 3, 2, 7, 4, 1, 6, 5];

// corner orientation corrections (added after permuting)
const F_CORNER_ROT: [u8; 8] = [1, 2, 0, 0, 2, 1, 0, 0];
const B_CORNER_ROT: [u8; 8] = [0, 0, 2, 1, 0, 0, 1, 2];
const R_CORNER_ROT: [u8; 8] = [2, 0, 1, 0, 1, 0, 2, 0];
const L_CORNER_ROT: [u8; 8] = [0, 1, 0, 2, 0, 2, 0, 1];

// edge permutations
const U1_EDGE_INDEX: [u8; 12] = [2, 3, 1, 0, 4, 5, 6, 7, 8, 9, 10, 11];
const D1_EDGE_INDEX: [u8; 12] = [0, 1, 2, 3, 7, 6, 4, 5, 8, 9, 10, 11];
const F1_EDGE_INDEX: [u8; 12] = [9, 1, 2, 3, 8, 5, 6, 7, 0, 4, 10, 11];
const B1_EDGE_INDEX: [u8; 12] = [0, 10, 2, 3, 4, 11, 6, 7, 8, 9, 5, 1];
const R1_EDGE_INDEX: [u8; 12] = [0, 1, 8, 3, 4, 5, 10, 7, 6, 9, 2, 11];
const L1_EDGE_INDEX: [u8; 12] = [0, 1, 2, 11, 4, 5, 6, 9, 8, 3, 10, 7];

// edge orientation corrections (added after permuting)
// note this is only 8. only the first 8 edges ever change orientation.
const F_EDGE_FLIP: [u8; 12] = [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0];
const B_EDGE_FLIP: [u8; 12] = [0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1];

// THIS IS THE COMPUTED SIMD ARRAYS FOR THE GATHER INSTRUCTION FOR EACH MOVE

const U1_INDEX: [usize; 40] = combined_index(&U1_CORNER_INDEX, &U1_EDGE_INDEX);
const U2_INDEX: [usize; 40] = compose(&U1_INDEX, &U1_INDEX);
const U3_INDEX: [usize; 40] = compose(&U2_INDEX, &U1_INDEX);
const U_MASK: [bool; 40] = combined_mask(&U1_INDEX);

const D1_INDEX: [usize; 40] = combined_index(&D1_CORNER_INDEX, &D1_EDGE_INDEX);
const D2_INDEX: [usize; 40] = compose(&D1_INDEX, &D1_INDEX);
const D3_INDEX: [usize; 40] = compose(&D2_INDEX, &D1_INDEX);
const D_MASK: [bool; 40] = combined_mask(&D1_INDEX);

const F1_INDEX: [usize; 40] = combined_index(&F1_CORNER_INDEX, &F1_EDGE_INDEX);
const F2_INDEX: [usize; 40] = compose(&F1_INDEX, &F1_INDEX);
const F3_INDEX: [usize; 40] = compose(&F2_INDEX, &F1_INDEX);
const F_MASK: [bool; 40] = combined_mask(&F1_INDEX);

const B1_INDEX: [usize; 40] = combined_index(&B1_CORNER_INDEX, &B1_EDGE_INDEX);
const B2_INDEX: [usize; 40] = compose(&B1_INDEX, &B1_INDEX);
const B3_INDEX: [usize; 40] = compose(&B2_INDEX, &B1_INDEX);
const B_MASK: [bool; 40] = combined_mask(&B1_INDEX);

const R1_INDEX: [usize; 40] = combined_index(&R1_CORNER_INDEX, &R1_EDGE_INDEX);
const R2_INDEX: [usize; 40] = compose(&R1_INDEX, &R1_INDEX);
const R3_INDEX: [usize; 40] = compose(&R2_INDEX, &R1_INDEX);
const R_MASK: [bool; 40] = combined_mask(&R1_INDEX);

const L1_INDEX: [usize; 40] = combined_index(&L1_CORNER_INDEX, &L1_EDGE_INDEX);
const L2_INDEX: [usize; 40] = compose(&L1_INDEX, &L1_INDEX);
const L3_INDEX: [usize; 40] = compose(&L2_INDEX, &L1_INDEX);
const L_MASK: [bool; 40] = combined_mask(&L1_INDEX);

// THESE ARE THE ORIENTATION CORRECTIONS TO ADD FOR THE CORRESPONSING FACE 90/270 TURNS
const F_ORIENT: [u8; 20] = combined_orient(&F_CORNER_ROT, &F_EDGE_FLIP);
const B_ORIENT: [u8; 20] = combined_orient(&B_CORNER_ROT, &B_EDGE_FLIP);
const R_ORIENT: [u8; 8] = R_CORNER_ROT;
const L_ORIENT: [u8; 8] = L_CORNER_ROT;

// THESE ARE ORIENTATION REMAINDER CORRECTIONS. ONLY NEEDED IF ORIENTATION CORRECTION APPLIED
const FULL_REM: [u8; 20] = [3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];

#[allow(dead_code)]
impl ReprCubie {
    pub const fn get_index(&self) -> [usize; size_of::<ReprCubie>()] {
        let corner_index: [u8; 8] = unsafe { core::mem::transmute(self.corner_perm) };
        let edge_index: [u8; 12] = unsafe { core::mem::transmute(self.edge_perm) };

        combined_index(&corner_index, &edge_index)
    }

    pub const fn get_orient(&self) -> &[u8; 20] {
        let o = corner_orient_offset();
        let buf = self.into_ref();
        let x = &buf[o];
        unsafe { core::mem::transmute(x) }
    }

    pub const fn apply_const(
        self,
        index: [usize; size_of::<ReprCubie>()],
        orient: &[u8; 20],
    ) -> Self {
        let buf = self.into_array();
        let mut buf_new = buf;

        let mut i = 0;
        while i < 40 {
            buf_new[i] = buf[index[i]];
            i += 1;
        }

        let mut buf = buf_new;

        const ORIENT_OFFSET: usize = corner_orient_offset();

        let mut i = 0;
        while i < 20 {
            buf[i + ORIENT_OFFSET] = (buf[i + ORIENT_OFFSET] + orient[i]) % FULL_REM[i];
            i += 1;
        }

        unsafe { Self::from_array_unchecked(buf) }
    }

    pub const fn apply_const_no_orient(self, index: [usize; size_of::<ReprCubie>()]) -> Self {
        let buf = self.into_array();
        let mut buf_new = buf;

        let mut i = 0;
        while i < 40 {
            buf_new[i] = buf[index[i]];
            i += 1;
        }

        unsafe { Self::from_array_unchecked(buf_new) }
    }

    fn apply(&mut self, index: [usize; size_of::<ReprCubie>()], orient: &[u8]) {
        let mut buf = core::mem::take(self).into_array();

        let mut padded_buf: [u8; 64] = [0; 64];
        padded_buf[..40].copy_from_slice(&buf);

        let mut padded_idx: [usize; 64] = [0; 64];
        padded_idx[..40].copy_from_slice(&index);

        const PADDED_MASK: [bool; 64] = {
            let mut mask = [false; 64];
            let mut i = 0;
            while i < 40 {
                mask[i] = true;
                i += 1;
            }
            mask
        };

        // this instruction does basically everything.
        //
        // first, the slice into the previous state of the cube
        // is where things are being gathered from.
        // the mask is a per-move mask which is false for everything that doesn't change
        // under that move.
        // the or argument is the previous buffer, because if you don't move you need to use that.
        buf.copy_from_slice(
            &Simd::gather_select(
                &padded_buf,
                Mask::from_array(PADDED_MASK),
                Simd::from_array(padded_idx),
                Simd::from_array(padded_buf),
            )[..40],
        );

        const ORIENT_OFFSET: usize = corner_orient_offset();

        let mut orient_buf = [0u8; 32];
        orient_buf[..20].copy_from_slice(&buf[ORIENT_OFFSET..ORIENT_OFFSET + 20]);
        let orient_buf = Simd::from_array(orient_buf);

        let mut correction = [0u8; 32];
        correction[..20].copy_from_slice(orient);
        let correction = Simd::from_array(correction);

        let mut modulo = [1u8; 32];
        modulo[..20].copy_from_slice(&FULL_REM);
        let modulo = Simd::from_array(modulo);

        let orient_buf = (orient_buf + correction) % modulo;
        buf[ORIENT_OFFSET..ORIENT_OFFSET + 20].copy_from_slice(&orient_buf[..20]);

        *self = unsafe { Self::from_array_unchecked(buf) }
    }

    fn phase_1_move(&mut self, m: Move) {
        *self = core::mem::take(self).const_move(m);
    }

    pub const fn const_move(self, m: Move) -> Self {
        enum Orient {
            Big([u8; 20]),
            Small([u8; 8]),
            None,
        }

        let buf = self.into_array();

        let (idx, mask, orient) = match m {
            Move::U1 => (U1_INDEX, U_MASK, Orient::None),
            Move::U2 => (U2_INDEX, U_MASK, Orient::None),
            Move::U3 => (U3_INDEX, U_MASK, Orient::None),
            Move::D1 => (D1_INDEX, D_MASK, Orient::None),
            Move::D2 => (D2_INDEX, D_MASK, Orient::None),
            Move::D3 => (D3_INDEX, D_MASK, Orient::None),
            Move::F1 => (F1_INDEX, F_MASK, Orient::Big(F_ORIENT)),
            Move::F2 => (F2_INDEX, F_MASK, Orient::None),
            Move::F3 => (F3_INDEX, F_MASK, Orient::Big(F_ORIENT)),
            Move::B1 => (B1_INDEX, B_MASK, Orient::Big(B_ORIENT)),
            Move::B2 => (B2_INDEX, B_MASK, Orient::None),
            Move::B3 => (B3_INDEX, B_MASK, Orient::Big(B_ORIENT)),
            Move::R1 => (R1_INDEX, R_MASK, Orient::Small(R_ORIENT)),
            Move::R2 => (R2_INDEX, R_MASK, Orient::None),
            Move::R3 => (R3_INDEX, R_MASK, Orient::Small(R_ORIENT)),
            Move::L1 => (L1_INDEX, L_MASK, Orient::Small(L_ORIENT)),
            Move::L2 => (L2_INDEX, L_MASK, Orient::None),
            Move::L3 => (L3_INDEX, L_MASK, Orient::Small(L_ORIENT)),
        };
        let mut buf_new = buf;

        let mut i = 0;
        while i < 40 {
            if mask[i] {
                buf_new[i] = buf[idx[i]];
            }
            i += 1;
        }

        let mut buf = buf_new;

        const ORIENT_OFFSET: usize = corner_orient_offset();

        match orient {
            Orient::Big(correction) => {
                let mut i = 0;
                while i < 20 {
                    buf[i + ORIENT_OFFSET] = (buf[i + ORIENT_OFFSET] + correction[i]) % FULL_REM[i];
                    i += 1;
                }
            }
            Orient::Small(correction) => {
                let mut i = 0;
                while i < 8 {
                    buf[i + ORIENT_OFFSET] = (buf[i + ORIENT_OFFSET] + correction[i]) % FULL_REM[i];
                    i += 1;
                }
            }
            Orient::None => {}
        };

        unsafe { Self::from_array_unchecked(buf) }
    }
}

#[test]
fn test_all_moves() {
    let mut c = ReprCubie::default();
    c.phase_1_move(Move::U1);
    c.phase_1_move(Move::U2);
    c.phase_1_move(Move::U3);
    c.phase_1_move(Move::U2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Move::D1);
    c.phase_1_move(Move::D2);
    c.phase_1_move(Move::D3);
    c.phase_1_move(Move::D2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Move::F1);
    c.phase_1_move(Move::F2);
    c.phase_1_move(Move::F3);
    c.phase_1_move(Move::F2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Move::B1);
    c.phase_1_move(Move::B2);
    c.phase_1_move(Move::B3);
    c.phase_1_move(Move::B2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Move::R1);
    c.phase_1_move(Move::R2);
    c.phase_1_move(Move::R3);
    c.phase_1_move(Move::R2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.phase_1_move(Move::L1);
    c.phase_1_move(Move::L2);
    c.phase_1_move(Move::L3);
    c.phase_1_move(Move::L2);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_long_identity() {
    let mut c = ReprCubie::default();
    c.phase_1_move(Move::F1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::L3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::R2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::L1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::B1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::U3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::B3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::R3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::D2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::F2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Move::D1);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn sexy_move() {
    let mut c = ReprCubie::default();

    for _ in 0..6 {
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::F1);
        c.phase_1_move(Move::U3);
        c.phase_1_move(Move::F3);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn hundred_thousand_moves_simd() {
    let mut c = ReprCubie::default();

    for _ in 0..1000 {
        c.phase_1_move(Move::F1);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::B2);
        c.phase_1_move(Move::L3);
        c.phase_1_move(Move::D3);
        c.phase_1_move(Move::R2);
        c.phase_1_move(Move::L1);
        c.phase_1_move(Move::B2);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::U2);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::B1);
        c.phase_1_move(Move::U3);
        c.phase_1_move(Move::B3);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::U2);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::R3);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::D3);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::D2);
        c.phase_1_move(Move::F2);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::F1);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::B2);
        c.phase_1_move(Move::L3);
        c.phase_1_move(Move::D3);
        c.phase_1_move(Move::R2);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::L3);
        c.phase_1_move(Move::B2);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::U2);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::B1);
        c.phase_1_move(Move::U3);
        c.phase_1_move(Move::B3);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::R3);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::D3);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::D2);
        c.phase_1_move(Move::F2);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::F1);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::B2);
        c.phase_1_move(Move::L3);
        c.phase_1_move(Move::D3);
        c.phase_1_move(Move::R2);
        c.phase_1_move(Move::L1);
        c.phase_1_move(Move::B2);
        c.phase_1_move(Move::F2);
        c.phase_1_move(Move::F1);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::U2);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::B1);
        c.phase_1_move(Move::U3);
        c.phase_1_move(Move::B3);
        c.phase_1_move(Move::D1);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::U2);
        c.phase_1_move(Move::F3);
        c.phase_1_move(Move::R1);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::R3);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::U1);
        c.phase_1_move(Move::L2);
        c.phase_1_move(Move::D3);
        c.phase_1_move(Move::L1);
        c.phase_1_move(Move::L1);
        c.phase_1_move(Move::D2);
        c.phase_1_move(Move::F2);
        c.phase_1_move(Move::D1);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn hundred_thousand_moves_const() {
    let mut c = ReprCubie::default();

    for _ in 0..1000 {
        c = c.const_move(Move::F1);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::B2);
        c = c.const_move(Move::L3);
        c = c.const_move(Move::D3);
        c = c.const_move(Move::R2);
        c = c.const_move(Move::L1);
        c = c.const_move(Move::B2);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::U2);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::B1);
        c = c.const_move(Move::U3);
        c = c.const_move(Move::B3);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::U2);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::R3);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::D3);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::D2);
        c = c.const_move(Move::F2);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::F1);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::B2);
        c = c.const_move(Move::L3);
        c = c.const_move(Move::D3);
        c = c.const_move(Move::R2);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::L3);
        c = c.const_move(Move::B2);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::U2);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::B1);
        c = c.const_move(Move::U3);
        c = c.const_move(Move::B3);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::R3);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::D3);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::D2);
        c = c.const_move(Move::F2);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::F1);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::B2);
        c = c.const_move(Move::L3);
        c = c.const_move(Move::D3);
        c = c.const_move(Move::R2);
        c = c.const_move(Move::L1);
        c = c.const_move(Move::B2);
        c = c.const_move(Move::F2);
        c = c.const_move(Move::F1);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::U2);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::B1);
        c = c.const_move(Move::U3);
        c = c.const_move(Move::B3);
        c = c.const_move(Move::D1);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::U2);
        c = c.const_move(Move::F3);
        c = c.const_move(Move::R1);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::R3);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::U1);
        c = c.const_move(Move::L2);
        c = c.const_move(Move::D3);
        c = c.const_move(Move::L1);
        c = c.const_move(Move::L1);
        c = c.const_move(Move::D2);
        c = c.const_move(Move::F2);
        c = c.const_move(Move::D1);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_apply() {
    let mut c = ReprCubie::new();

    c.phase_1_move(Move::R1);
    c.phase_1_move(Move::U1);
    c.phase_1_move(Move::R3);
    c.phase_1_move(Move::U3);

    let i = c.get_index();
    let o = c.get_orient();

    let mut c2 = ReprCubie::new();

    for _ in 0..6 {
        c2.apply(i, o);
    }

    assert!(c2.is_solved());
}

#[test]
fn test_2_move_apply() {
    let mut c = ReprCubie::new();

    c.phase_1_move(Move::R1);
    c.phase_1_move(Move::U1);

    let i = c.get_index();
    let o = c.get_orient();

    let mut c2 = ReprCubie::new();
    c2.apply(i, o);
    let mut count = 1;
    while !c2.is_solved() {
        count += 1;
        c2.apply(i, o);
    }

    assert_eq!(count, 105);
}

#[test]
fn test_long_apply() {
    let mut c = ReprCubie::new();

    c.phase_1_move(Move::R1);
    c.phase_1_move(Move::U2);
    c.phase_1_move(Move::D3);
    c.phase_1_move(Move::B1);
    c.phase_1_move(Move::D3);

    let i = c.get_index();
    let o = c.get_orient();

    let mut c2 = ReprCubie::new();
    c2.apply(i, o);
    let mut count = 1;
    while !c2.is_solved() {
        count += 1;
        c2.apply(i, o);
    }
    assert_eq!(count, 1260);
}

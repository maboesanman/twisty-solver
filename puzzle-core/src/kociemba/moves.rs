use std::{
    intrinsics::size_of,
    simd::{Mask, Simd},
};

use super::cubie_repr::{
    corner_orient_offset, corner_perm_offset, edge_orient_offset, edge_perm_offset, CornerOrient,
    CubieRepr, EdgeOrient,
};

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(unused)]
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
#[allow(unused)]
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

const fn combined_index(
    corner_index: &[u8; 8],
    edge_index: &[u8; 12],
) -> [usize; size_of::<CubieRepr>()] {
    let mut buf = [0usize; size_of::<CubieRepr>()];

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

const fn combined_orient(corner_rot: &[u8; 8], edge_flip: &[u8; 8]) -> [u8; 16] {
    let mut buf = [0u8; 16];

    if corner_orient_offset() + 8 != edge_orient_offset() {
        panic!();
    }

    let mut i = 0;
    while i < 8 {
        buf[i] = corner_rot[i];
        i += 1;
    }
    while i < 16 {
        buf[i] = edge_flip[i - 8];
        i += 1;
    }

    buf
}

// consts for performing moves on CubieRepr
const fn compose<const N: usize>(base: &[usize; N], next: &[usize; N]) -> [usize; N] {
    let mut x = [0; N];
    let mut i = 0;
    while i < N {
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
const U1_EDGE_INDEX: [u8; 12] = [8, 9, 2, 3, 4, 5, 6, 7, 1, 0, 10, 11];
const D1_EDGE_INDEX: [u8; 12] = [0, 1, 11, 10, 4, 5, 6, 7, 8, 9, 2, 3];
const F1_EDGE_INDEX: [u8; 12] = [5, 1, 4, 3, 0, 2, 6, 7, 8, 9, 10, 11];
const B1_EDGE_INDEX: [u8; 12] = [0, 6, 2, 7, 4, 5, 3, 1, 8, 9, 10, 11];
const R1_EDGE_INDEX: [u8; 12] = [0, 1, 2, 3, 10, 5, 8, 7, 4, 9, 6, 11];
const L1_EDGE_INDEX: [u8; 12] = [0, 1, 2, 3, 4, 9, 6, 11, 8, 7, 10, 5];

// edge orientation corrections (added after permuting)
// note this is only 8. only the first 8 edges ever change orientation.
const F_EDGE_FLIP: [u8; 8] = [1, 0, 1, 0, 1, 1, 0, 0];
const B_EDGE_FLIP: [u8; 8] = [0, 1, 0, 1, 0, 0, 1, 1];

const S_YZ_CORNER_INDEX: [u8; 8] = [0, 4, 1, 5, 2, 6, 3, 7];
const S_Z2_CORNER_INDEX: [u8; 8] = [5, 4, 7, 6, 1, 0, 3, 2];
const S_Y_CORNER_INDEX: [u8; 8] = [2, 0, 3, 1, 6, 4, 7, 5];
const S_W_CORNER_INDEX: [u8; 8] = [1, 0, 3, 2, 5, 4, 7, 6];

const S_YZ_EDGE_INDEX: [u8; 12] = [4, 5, 6, 7, 8, 10, 9, 11, 0, 9, 1, 3];
const S_Z2_EDGE_INDEX: [u8; 12] = [2, 3, 0, 1, 5, 4, 7, 6, 11, 10, 9, 8];
const S_Y_EDGE_INDEX: [u8; 12] = [8, 9, 10, 11, 6, 4, 7, 5, 1, 0, 3, 2];
const S_W_EDGE_INDEX: [u8; 12] = [0, 1, 2, 3, 5, 4, 7, 6, 9, 8, 11, 10];

const S_YZ_CORNER_ROT: [u8; 8] = [1, 2, 2, 1, 2, 1, 1, 2];

const S_YZ_EDGE_FLIP: [u8; 12] = [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1];
const S_Y_EDGE_FLIP: [u8; 12] = [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0];

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
const F_ORIENT: [u8; 16] = combined_orient(&F_CORNER_ROT, &F_EDGE_FLIP);
const B_ORIENT: [u8; 16] = combined_orient(&B_CORNER_ROT, &B_EDGE_FLIP);
const R_ORIENT: [u8; 8] = R_CORNER_ROT;
const L_ORIENT: [u8; 8] = L_CORNER_ROT;

// THESE ARE ORIENTATION REMAINDER CORRECTIONS. ONLY NEEDED IF ORIENTATION CORRECTION APPLIED
const FULL_REM: [u8; 20] = [3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
const FB_REM: [u8; 16] = [3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2];
const RL_REM: [u8; 8] = [3; 8];

#[allow(dead_code)]
impl CubieRepr {
    fn get_index(&self) -> [usize; size_of::<CubieRepr>()] {
        let corner_index = self.corner_perm.map(|x| x as u8);
        let edge_index = self.edge_perm.map(|x| x as u8);

        combined_index(&corner_index, &edge_index)
    }

    fn get_orient(&self) -> &[u8; 20] {
        let o = corner_orient_offset();
        let buf = self.into_ref();
        (&buf[o..o + 20]).try_into().unwrap()
    }

    fn apply(&mut self, index: [usize; size_of::<CubieRepr>()], orient: &[u8]) {
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

    fn phase_1_move(&mut self, m: Phase1Move) {
        enum Orient {
            Big([u8; 16]),
            Small([u8; 8]),
            None,
        }

        let mut buf = core::mem::take(self).into_array();

        let (idx, mask, orient) = match m {
            Phase1Move::U1 => (U1_INDEX, U_MASK, Orient::None),
            Phase1Move::U2 => (U2_INDEX, U_MASK, Orient::None),
            Phase1Move::U3 => (U3_INDEX, U_MASK, Orient::None),
            Phase1Move::D1 => (D1_INDEX, D_MASK, Orient::None),
            Phase1Move::D2 => (D2_INDEX, D_MASK, Orient::None),
            Phase1Move::D3 => (D3_INDEX, D_MASK, Orient::None),
            Phase1Move::F1 => (F1_INDEX, F_MASK, Orient::Big(F_ORIENT)),
            Phase1Move::F2 => (F2_INDEX, F_MASK, Orient::None),
            Phase1Move::F3 => (F3_INDEX, F_MASK, Orient::Big(F_ORIENT)),
            Phase1Move::B1 => (B1_INDEX, B_MASK, Orient::Big(B_ORIENT)),
            Phase1Move::B2 => (B2_INDEX, B_MASK, Orient::None),
            Phase1Move::B3 => (B3_INDEX, B_MASK, Orient::Big(B_ORIENT)),
            Phase1Move::R1 => (R1_INDEX, R_MASK, Orient::Small(R_ORIENT)),
            Phase1Move::R2 => (R2_INDEX, R_MASK, Orient::None),
            Phase1Move::R3 => (R3_INDEX, R_MASK, Orient::Small(R_ORIENT)),
            Phase1Move::L1 => (L1_INDEX, L_MASK, Orient::Small(L_ORIENT)),
            Phase1Move::L2 => (L2_INDEX, L_MASK, Orient::None),
            Phase1Move::L3 => (L3_INDEX, L_MASK, Orient::Small(L_ORIENT)),
        };

        let mut padded_buf: [u8; 64] = [0; 64];
        padded_buf[..40].copy_from_slice(&buf);

        let mut padded_idx: [usize; 64] = [0; 64];
        padded_idx[..40].copy_from_slice(&idx);

        let mut padded_mask: [bool; 64] = [false; 64];
        padded_mask[..40].copy_from_slice(&mask);

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
                Mask::from_array(padded_mask),
                Simd::from_array(padded_idx),
                Simd::from_array(padded_buf),
            )[..40],
        );

        const ORIENT_OFFSET: usize = corner_orient_offset();

        match orient {
            Orient::Big(correction) => {
                let s = Simd::<u8, 16>::from_slice(&buf[ORIENT_OFFSET..ORIENT_OFFSET + 16]);
                let correction = Simd::from_array(correction);
                let modulo = Simd::from_array(FB_REM);
                let s = (s + correction) % modulo;
                buf[ORIENT_OFFSET..ORIENT_OFFSET + 16].copy_from_slice(&s[..]);
            }
            Orient::Small(correction) => {
                let s = Simd::<u8, 8>::from_slice(&buf[ORIENT_OFFSET..ORIENT_OFFSET + 8]);
                let correction = Simd::from_array(correction);
                let modulo = Simd::from_array(RL_REM);
                let s = (s + correction) % modulo;
                buf[ORIENT_OFFSET..ORIENT_OFFSET + 8].copy_from_slice(&s[..]);
            }
            Orient::None => {}
        };

        *self = unsafe { Self::from_array_unchecked(buf) }
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
}

#[test]
fn test_all_moves() {
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
fn test_long_identity() {
    let mut c = CubieRepr::default();
    c.phase_1_move(Phase1Move::F1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::L3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::R2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::L1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::B1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::U3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::B3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::R3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::D2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::F2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.phase_1_move(Phase1Move::D1);

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

#[test]
fn hundred_thousand_moves() {
    let mut c = CubieRepr::default();

    for _ in 0..100 {
        c.phase_1_move(Phase1Move::F1);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::B2);
        c.phase_1_move(Phase1Move::L3);
        c.phase_1_move(Phase1Move::D3);
        c.phase_1_move(Phase1Move::R2);
        c.phase_1_move(Phase1Move::L1);
        c.phase_1_move(Phase1Move::B2);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::U2);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::B1);
        c.phase_1_move(Phase1Move::U3);
        c.phase_1_move(Phase1Move::B3);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::U2);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::R3);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::D3);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::D2);
        c.phase_1_move(Phase1Move::F2);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::F1);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::B2);
        c.phase_1_move(Phase1Move::L3);
        c.phase_1_move(Phase1Move::D3);
        c.phase_1_move(Phase1Move::R2);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::L3);
        c.phase_1_move(Phase1Move::B2);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::U2);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::B1);
        c.phase_1_move(Phase1Move::U3);
        c.phase_1_move(Phase1Move::B3);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::R3);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::D3);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::D2);
        c.phase_1_move(Phase1Move::F2);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::F1);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::B2);
        c.phase_1_move(Phase1Move::L3);
        c.phase_1_move(Phase1Move::D3);
        c.phase_1_move(Phase1Move::R2);
        c.phase_1_move(Phase1Move::L1);
        c.phase_1_move(Phase1Move::B2);
        c.phase_1_move(Phase1Move::F2);
        c.phase_1_move(Phase1Move::F1);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::U2);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::B1);
        c.phase_1_move(Phase1Move::U3);
        c.phase_1_move(Phase1Move::B3);
        c.phase_1_move(Phase1Move::D1);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::U2);
        c.phase_1_move(Phase1Move::F3);
        c.phase_1_move(Phase1Move::R1);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::R3);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::U1);
        c.phase_1_move(Phase1Move::L2);
        c.phase_1_move(Phase1Move::D3);
        c.phase_1_move(Phase1Move::L1);
        c.phase_1_move(Phase1Move::L1);
        c.phase_1_move(Phase1Move::D2);
        c.phase_1_move(Phase1Move::F2);
        c.phase_1_move(Phase1Move::D1);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_apply() {
    let mut c = CubieRepr::new();

    c.phase_1_move(Phase1Move::R1);
    c.phase_1_move(Phase1Move::U1);
    c.phase_1_move(Phase1Move::R3);
    c.phase_1_move(Phase1Move::U3);

    let i = c.get_index();
    let o = c.get_orient();

    let mut c2 = CubieRepr::new();

    for _ in 0..6 {
        c2.apply(i, o);
    }

    assert!(c2.is_solved());
}

#[test]
fn test_2_move_apply() {
    let mut c = CubieRepr::new();

    c.phase_1_move(Phase1Move::R1);
    c.phase_1_move(Phase1Move::U1);

    let i = c.get_index();
    let o = c.get_orient();

    let mut c2 = CubieRepr::new();
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
    let mut c = CubieRepr::new();

    c.phase_1_move(Phase1Move::R1);
    c.phase_1_move(Phase1Move::U2);
    c.phase_1_move(Phase1Move::D3);
    c.phase_1_move(Phase1Move::B1);
    c.phase_1_move(Phase1Move::D3);

    let i = c.get_index();
    let o = c.get_orient();

    let mut c2 = CubieRepr::new();
    c2.apply(i, o);
    let mut count = 1;
    while !c2.is_solved() {
        count += 1;
        c2.apply(i, o);
    }
    assert_eq!(count, 1260);
}

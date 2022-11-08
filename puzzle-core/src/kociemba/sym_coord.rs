use std::{collections::BTreeSet, mem::size_of};

use crate::kociemba::cubie_repr::corner_orient_offset;

use super::{
    cubie_repr::CubieRepr,
    moves::{combined_index, compose}, coord::{CornerOrientCoord, EdgeOrientCoord, EdgeGroupingCoord, CornerPermutationCoord, UDEdgePermutationCoord, EEdgePermutationCoord},
};

const S_YZ_CORNER_INDEX: [u8; 8] = [0, 4, 1, 5, 2, 6, 3, 7];
const S_Z2_CORNER_INDEX: [u8; 8] = [5, 4, 7, 6, 1, 0, 3, 2];
const S_Y_CORNER_INDEX: [u8; 8] = [2, 0, 3, 1, 6, 4, 7, 5];
const S_W_CORNER_INDEX: [u8; 8] = [1, 0, 3, 2, 5, 4, 7, 6];

const S_YZ_EDGE_INDEX: [u8; 12] = [4, 5, 6, 7, 8, 10, 9, 11, 0, 9, 1, 3];
const S_Z2_EDGE_INDEX: [u8; 12] = [2, 3, 0, 1, 5, 4, 7, 6, 11, 10, 9, 8];
const S_Y_EDGE_INDEX: [u8; 12] = [8, 9, 10, 11, 6, 4, 7, 5, 1, 0, 3, 2];
const S_W_EDGE_INDEX: [u8; 12] = [0, 1, 2, 3, 5, 4, 7, 6, 9, 8, 11, 10];

const S_YZ_CORNER_ROT: [u8; 8] = [1, 2, 2, 1, 2, 1, 1, 2];

// the corner orientation gets subtracted FROM this on W flip
const S_W_CORNER_INV: [u8; 8] = [3; 8];

const S_YZ_EDGE_FLIP: [u8; 12] = [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1];
const S_Y_EDGE_FLIP: [u8; 12] = [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0];

const S_YZ_INDEX: [usize; 40] = combined_index(&S_YZ_CORNER_INDEX, &S_YZ_EDGE_INDEX);
const S_Z2_INDEX: [usize; 40] = combined_index(&S_Z2_CORNER_INDEX, &S_Z2_EDGE_INDEX);
const S_Y_INDEX: [usize; 40] = combined_index(&S_Y_CORNER_INDEX, &S_Y_EDGE_INDEX);
const S_W_INDEX: [usize; 40] = combined_index(&S_W_CORNER_INDEX, &S_W_EDGE_INDEX);

const S_Y_ORIENT: [u8; 20] = {
    let mut buf = [0; 20];

    let mut i = 0;
    while i < 12 {
        buf[i + 8] = S_Y_EDGE_FLIP[i];
        i += 1
    }

    buf
};

const S_YZ_ORIENT: [u8; 20] = {
    let mut buf = [0; 20];

    let mut i = 0;
    while i < 8 {
        buf[i] = S_YZ_CORNER_ROT[i];
        i += 1
    }
    while i < 20 {
        buf[i] = S_YZ_EDGE_FLIP[i - 8];
        i += 1
    }

    buf
};

const SOLVED: [usize; 40] = [
    0, 1, 2, 3, 4, 5, 6, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3,
    4, 5, 6, 7, 8, 9, 10, 11,
];


const fn get_transforms(enable_yz: bool) -> (
    [CubieRepr; 48],
    [[usize; 40]; 48],
    [[u8; 20]; 48],
) {
    let mut forward_start = [0u8; 40 * 48];
    let mut inverse_index = [0usize; 40 * 48];
    let mut inverse_orient = [0u8; 20 * 48];

    let mut i = 0;
    let mut o = 0;

    let yz_max = if enable_yz { 3 } else { 1 };

    let mut w = 0;
    while w < 2 {
        let mut yz = 0;
        while yz < yz_max {
            let mut y = 0;
            while y < 4 {
                let mut z2 = 0;
                while z2 < 2 {
                    let mut forward = CubieRepr::new();
                    if w == 1 {
                        forward = forward.apply_w_const_no_orient();
                    }
                    if z2 == 1 {
                        forward = forward.apply_const_no_orient(S_Z2_INDEX);
                    }
                    let mut y_temp = 0;
                    while y_temp < y {
                        forward = forward.apply_const(S_Y_INDEX, &S_Y_ORIENT);
                        y_temp += 1;
                    }
                    let mut yz_temp = 0;
                    while yz_temp < yz {
                        forward = forward.apply_const(S_YZ_INDEX, &S_YZ_ORIENT);
                        yz_temp += 1;
                    }

                    let mut inverse = CubieRepr::new();
                    let mut yz_temp = 0;
                    while yz_temp < 3 - yz {
                        inverse = inverse.apply_const(S_YZ_INDEX, &S_YZ_ORIENT);
                        yz_temp += 1;
                    }
                    let mut y_temp = 0;
                    while y_temp < 4 - y {
                        inverse = inverse.apply_const(S_Y_INDEX, &S_Y_ORIENT);
                        y_temp += 1;
                    }
                    if z2 == 1 {
                        inverse = inverse.apply_const_no_orient(S_Z2_INDEX);
                    }
                    if w == 1 {
                        inverse = inverse.apply_w_const_no_orient();
                    }

                    let f_array = forward.into_array();
                    let i_index = inverse.get_index();
                    let i_orient = inverse.get_orient();

                    let mut j = 0;
                    while j < 40 {
                        forward_start[i] = f_array[j];
                        inverse_index[i] = i_index[j];
                        i += 1;
                        j += 1;
                    }
                    j = 0;
                    while j < 20 {
                        inverse_orient[o] = i_orient[j];
                        o += 1;
                        j += 1;
                    }
                    z2 += 1;
                }
                y += 1;
            }
            yz += 1;
        }
        w += 1;
    }

    unsafe { core::mem::transmute((forward_start, inverse_index, inverse_orient)) }
}

const YZ_Y_Z2_W_INDEX: (
    [CubieRepr; 48],
    [[usize; 40]; 48],
    [[u8; 20]; 48],
) = get_transforms(true);

const Y_Z2_W_INDEX: (
    [CubieRepr; 16],
    [[usize; 40]; 16],
    [[u8; 20]; 16],
) = {
    let (a, b, c) = get_transforms(false);

    let mut a2 = [CubieRepr::new(); 16];
    let mut b2 = [[0usize; 40]; 16];
    let mut c2 = [[0u8; 20]; 16];
    

    let mut x = 0;
    while x < 16 {
        a2[x] = a[x];
        b2[x] = b[x];
        c2[x] = c[x];

        x += 1;
    }

    (a2, b2, c2)
};

// const EDGE_GROUPING_SYM_TABLE: [u8; 29] = {
//     let mut out = [255u8; 29];

//     let mut highest_i = 0;
//     let mut i = 0;
//     while i < 495 {
//         let cube = CubieRepr::from_coords(
//             CornerOrientCoord(0),
//             EdgeOrientCoord(0),
//             EdgeGroupingCoord(i),
//             CornerPermutationCoord(0),
//             UDEdgePermutationCoord(0),
//             EEdgePermutationCoord(0),
//         );

//         let sym_coord = cube.sym_rep_edge_group();

//         let new_i = match out.binary_search(&sym_coord) {
//             Err(i) => i,
//             Ok(_) => continue,
//         };


//     }
//     assert_eq!(out.len(), 29);

//     // out.into_iter().enumerate() {
//     //     sym_array[i] = o;
//     // }
//     // println!("{:?}", sym_array);
//     out
// };

impl CubieRepr {
    const fn apply_w_const(self) -> Self {
        let buf = self.into_array();
        let mut buf_new = buf;

        let mut i = 0;
        while i < 40 {
            buf_new[i] = buf[S_W_INDEX[i]];
            i += 1;
        }

        let mut buf = buf_new;

        const ORIENT_OFFSET: usize = corner_orient_offset();

        let mut i = 0;
        while i < 8 {
            buf[i + ORIENT_OFFSET] = (3 - buf[i + ORIENT_OFFSET]) % 3;
            i += 1;
        }

        unsafe { Self::from_array_unchecked(buf) }
    }

    const fn apply_w_const_no_orient(self) -> Self {
        let buf = self.into_array();
        let mut buf_new = buf;

        let mut i = 0;
        while i < 40 {
            buf_new[i] = buf[S_W_INDEX[i]];
            i += 1;
        }

        unsafe { Self::from_array_unchecked(buf_new) }
    }

    const fn apply_w_const_only_orient(self) -> Self {
        let mut buf = self.into_array();

        const ORIENT_OFFSET: usize = corner_orient_offset();

        let mut i = 0;
        while i < 8 {
            buf[i + ORIENT_OFFSET] = (3 - buf[i + ORIENT_OFFSET]) % 3;
            i += 1;
        }

        unsafe { Self::from_array_unchecked(buf) }
    }

    const fn apply_all_transforms(&self) -> [CubieRepr; 48] {
        let index = self.get_index();
        let orient = self.get_orient();

        let mut cubes: [CubieRepr; 48] = YZ_Y_Z2_W_INDEX.0;
        let t_index: [[usize; 40]; 48] = YZ_Y_Z2_W_INDEX.1;
        let t_orient: [[u8; 20]; 48] = YZ_Y_Z2_W_INDEX.2;

        let mut i = 0;
        while i < 24 {
            cubes[i] = cubes[i].apply_const(index, orient);
            cubes[i] = cubes[i].apply_const(t_index[i], &t_orient[i]);
            i += 1;
        }
        while i < 48 {
            cubes[i] = cubes[i].apply_const(index, orient);
            cubes[i] = cubes[i].apply_const(t_index[i], &t_orient[i]);
            cubes[i] = cubes[i].apply_w_const_only_orient();
            i += 1;
        }

        cubes
    }

    const fn apply_non_yz_transforms(&self) -> [CubieRepr; 16] {
        let index = self.get_index();
        let orient = self.get_orient();

        let mut cubes: [CubieRepr; 16] = Y_Z2_W_INDEX.0;
        let t_index: [[usize; 40]; 16] = Y_Z2_W_INDEX.1;
        let t_orient: [[u8; 20]; 16] = Y_Z2_W_INDEX.2;

        let mut i = 0;
        while i < 8 {
            cubes[i] = cubes[i].apply_const(index, orient);
            cubes[i] = cubes[i].apply_const(t_index[i], &t_orient[i]);
            i += 1;
        }
        while i < 16 {
            cubes[i] = cubes[i].apply_const(index, orient);
            cubes[i] = cubes[i].apply_const(t_index[i], &t_orient[i]);
            cubes[i] = cubes[i].apply_w_const_only_orient();
            i += 1;
        }

        cubes
    }

    // all symmetries generated by Z2, Y, W (16)
    fn sym_rep_corner_orient() -> u16 {
        todo!()
    }

    // all symmetries generated by Z2, Y2, W (8)
    fn sym_rep_edge_orient() -> u16 {
        todo!()
    }

    // all symmetries generated by Z2, Y, W (16)
    fn sym_rep_edge_group(&self) -> u8 {
        let applied = self.apply_all_transforms();
        let mut min = 255u8;
        let mut i = 0;
        while i < 48 {
            let c = applied[i].coord_edge_grouping().0;
            if c < min as u16 {
                min = c as u8;
            }
            i += 1;
        }

        min
    }

    // all symmetries generated by Z2, Y, W (16)
    fn sym_rep_corner_perm() -> u16 {
        todo!()
    }

    // all symmetries generated by Z2, Y, W (16)
    fn sym_rep_ud_edge_perm() -> u16 {
        todo!()
    }

    // all symmetries generated by Z2, Y, W (16)
    fn sym_rep_e_edge_perm() -> u8 {
        todo!()
    }
}

#[test]
fn flips() {}

#[test]
fn build_sym_coord() {
    let mut out = BTreeSet::new();

    for i in 0..495 {
        let cube = CubieRepr::from_coords(
            CornerOrientCoord(0),
            EdgeOrientCoord(0),
            EdgeGroupingCoord(i),
            CornerPermutationCoord(0),
            UDEdgePermutationCoord(0),
            EEdgePermutationCoord(0),
        );

        out.insert(cube.sym_rep_edge_group());
    }
    assert_eq!(out.len(), 29);

    let mut sym_array = [0; 29];
    for (i, o) in out.into_iter().enumerate() {
        sym_array[i] = o;
    }
    println!("{:?}", sym_array);
}

#[test]
fn cube_rotations() {
    let c = CubieRepr::new();

    let b = c.const_phase_1_move(super::moves::Phase1Move::B1);
    let f = c.const_phase_1_move(super::moves::Phase1Move::F1);
    let d = c.const_phase_1_move(super::moves::Phase1Move::D1);
    let u = c.const_phase_1_move(super::moves::Phase1Move::U1);
    let r = c.const_phase_1_move(super::moves::Phase1Move::R1);
    let l = c.const_phase_1_move(super::moves::Phase1Move::L1);

    let transformed = b.apply_all_transforms();

    assert!(transformed.contains(&b));
    assert!(transformed.contains(&f));
    assert!(transformed.contains(&d));
    assert!(transformed.contains(&u));
    assert!(transformed.contains(&r));
    assert!(transformed.contains(&l));
}
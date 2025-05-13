use std::fmt::Debug;

use rand::distr::{Distribution, StandardUniform};

use crate::{
    coords::{RawCornerOrientCoord, RawEdgeGroupCoord, RawEdgeOrientCoord}, moves::{Move, Phase2Move}, permutation_coord::{
        is_odd, is_perm, permutation_coord_12_inverse, permutation_coord_8_inverse
    }, permutation_math::permutation::Permutation, symmetries::SubGroupTransform
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReprCube {
    pub corner_perm: Permutation<8>,
    pub edge_perm: Permutation<12>,
    pub corner_orient: CornerOrient,
    pub edge_orient: EdgeOrient,
}




impl Default for ReprCube {
    fn default() -> Self {
        SOLVED_CUBE
    }
}

#[macro_export]
macro_rules! cube {
    // 1) ENTRY POINT: invoked as `cube![ F U2 Dp ]`
    [ $($mv:ident)+ ] => {
        $crate::repr_cubie::SOLVED_CUBE
        $(
            .then(cube!(@mv $mv))
        )+
    };

    // 2) “up to 2” and “up prime” on each face:
    (@mv U)  => { $crate::repr_cubie::U1 };
    (@mv U2) => { $crate::repr_cubie::U2 };
    (@mv Up) => { crate::repr_cubie::U3 };

    (@mv D)  => { crate::repr_cubie::D1 };
    (@mv D2) => { crate::repr_cubie::D2 };
    (@mv Dp) => { crate::repr_cubie::D3 };

    (@mv F)  => { crate::repr_cubie::F1 };
    (@mv F2) => { crate::repr_cubie::F2 };
    (@mv Fp) => { crate::repr_cubie::F3 };

    (@mv B)  => { crate::repr_cubie::B1 };
    (@mv B2) => { crate::repr_cubie::B2 };
    (@mv Bp) => { crate::repr_cubie::B3 };

    (@mv L)  => { crate::repr_cubie::L1 };
    (@mv L2) => { crate::repr_cubie::L2 };
    (@mv Lp) => { crate::repr_cubie::L3 };

    (@mv R)  => { crate::repr_cubie::R1 };
    (@mv R2) => { crate::repr_cubie::R2 };
    (@mv Rp) => { crate::repr_cubie::R3 };
}

pub const SOLVED_CUBE: ReprCube = ReprCube {
    corner_perm: Permutation::IDENTITY,
    edge_perm: Permutation::IDENTITY,
    corner_orient: CornerOrient([0; 8]),
    edge_orient: EdgeOrient([0; 12]),
};

pub const U_CORNER_PERM: Permutation<8> = Permutation::const_from_array([2, 0, 3, 1, 4, 5, 6, 7]);
pub const U_EDGE_PERM: Permutation<12> = Permutation::const_from_array([2, 3, 1, 0, 4, 5, 6, 7, 8, 9, 10, 11]);

pub const U1: ReprCube = ReprCube {
    corner_perm: U_CORNER_PERM,
    edge_perm: U_EDGE_PERM,
    corner_orient: CornerOrient([0; 8]),
    edge_orient: EdgeOrient([0; 12]),
};

pub const U2: ReprCube = U1.then(U1);
pub const U3: ReprCube = U2.then(U1);

pub const D_CORNER_PERM: Permutation<8> = Permutation::const_from_array([0, 1, 2, 3, 5, 7, 4, 6]);
pub const D_EDGE_PERM: Permutation<12> = Permutation::const_from_array([0, 1, 2, 3, 7, 6, 4, 5, 8, 9, 10, 11]);

pub const D1: ReprCube = ReprCube {
    corner_perm: D_CORNER_PERM,
    edge_perm: D_EDGE_PERM,
    corner_orient: CornerOrient([0; 8]),
    edge_orient: EdgeOrient([0; 12]),
};

pub const D2: ReprCube = D1.then(D1);
pub const D3: ReprCube = D2.then(D1);

pub const F_CORNER_PERM: Permutation<8> = Permutation::const_from_array([1, 5, 2, 3, 0, 4, 6, 7]);
pub const F_EDGE_PERM: Permutation<12> = Permutation::const_from_array([9, 1, 2, 3, 8, 5, 6, 7, 0, 4, 10, 11]);
pub const F_CORNER_ORIENT_CORRECT: CornerOrient = CornerOrient([1, 2, 0, 0, 2, 1, 0, 0]);
pub const F_EDGE_ORIENT_CORRECT: EdgeOrient = EdgeOrient([1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0]);

pub const F1: ReprCube = ReprCube {
    corner_perm: F_CORNER_PERM,
    edge_perm: F_EDGE_PERM,
    corner_orient: F_CORNER_ORIENT_CORRECT,
    edge_orient: F_EDGE_ORIENT_CORRECT,
};

pub const F2: ReprCube = F1.then(F1);
pub const F3: ReprCube = F2.then(F1);

pub const B_CORNER_PERM: Permutation<8> = Permutation::const_from_array([0, 1, 6, 2, 4, 5, 7, 3]);
pub const B_EDGE_PERM: Permutation<12> = Permutation::const_from_array([0, 10, 2, 3, 4, 11, 6, 7, 8, 9, 5, 1]);
pub const B_CORNER_ORIENT_CORRECT: CornerOrient = CornerOrient([0, 0, 2, 1, 0, 0, 1, 2]);
pub const B_EDGE_ORIENT_CORRECT: EdgeOrient = EdgeOrient([0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1]);

pub const B1: ReprCube = ReprCube {
    corner_perm: B_CORNER_PERM,
    edge_perm: B_EDGE_PERM,
    corner_orient: B_CORNER_ORIENT_CORRECT,
    edge_orient: B_EDGE_ORIENT_CORRECT,
};

pub const B2: ReprCube = B1.then(B1);
pub const B3: ReprCube = B2.then(B1);

pub const R_CORNER_PERM: Permutation<8> = Permutation::const_from_array([4, 1, 0, 3, 6, 5, 2, 7]);
pub const R_EDGE_PERM: Permutation<12> = Permutation::const_from_array([0, 1, 8, 3, 4, 5, 10, 7, 6, 9, 2, 11]);
pub const R_CORNER_ORIENT_CORRECT: CornerOrient = CornerOrient([2, 0, 1, 0, 1, 0, 2, 0]);

pub const R1: ReprCube = ReprCube {
    corner_perm: R_CORNER_PERM,
    edge_perm: R_EDGE_PERM,
    corner_orient: R_CORNER_ORIENT_CORRECT,
    edge_orient: EdgeOrient::SOLVED,
};

pub const R2: ReprCube = R1.then(R1);
pub const R3: ReprCube = R2.then(R1);

pub const L_CORNER_PERM: Permutation<8> = Permutation::const_from_array([0, 3, 2, 7, 4, 1, 6, 5]);
pub const L_EDGE_PERM: Permutation<12> = Permutation::const_from_array([0, 1, 2, 11, 4, 5, 6, 9, 8, 3, 10, 7]);
pub const L_CORNER_ORIENT_CORRECT: CornerOrient = CornerOrient([0, 1, 0, 2, 0, 2, 0, 1]);

pub const L1: ReprCube = ReprCube {
    corner_perm: L_CORNER_PERM,
    edge_perm: L_EDGE_PERM,
    corner_orient: L_CORNER_ORIENT_CORRECT,
    edge_orient: EdgeOrient::SOLVED,
};

pub const L2: ReprCube = L1.then(L1);
pub const L3: ReprCube = L2.then(L1);

// const S_URF3_1: ReprCube = cube![R Lp F Bp U Dp R Lp];
// const S_URF3_2: ReprCube = S_URF3_1.then(S_URF3_1);

// const S_F2: ReprCube = cube![R2 L2 F Bp U2 D2 F Bp];
// const S_U4_1: ReprCube = ReprCube {
//     corner_perm: Permutation::const_from_array([2, 0, 3, 1, 6, 4, 7, 5]),
//     edge_perm: Permutation::const_from_array([2, 3, 1, 0, 6, 7, 5, 4, 10, 8, 11, 9]),
//     corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
//     edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
// };

// const S_U4_2: ReprCube = S_U4_1.then(S_U4_1);
// const S_U4_3: ReprCube = S_U4_2.then(S_U4_1);

// const S_LR2_PARTIAL: ReprCube = cube![U D R2 F2 Up Dp R2 F2 U R2 L2 F2 R2 L2 F2 U L2 F2];

impl From<Move> for ReprCube {
    fn from(value: Move) -> Self {
        match value {
            Move::U1 => U1,
            Move::U2 => U2,
            Move::U3 => U3,
            Move::D1 => D1,
            Move::D2 => D2,
            Move::D3 => D3,
            Move::F1 => F1,
            Move::F2 => F2,
            Move::F3 => F3,
            Move::B1 => B1,
            Move::B2 => B2,
            Move::B3 => B3,
            Move::R1 => R1,
            Move::R2 => R2,
            Move::R3 => R3,
            Move::L1 => L1,
            Move::L2 => L2,
            Move::L3 => L3,
        }
    }
}

impl TryFrom<ReprCube> for Move {
    type Error = ReprCube;

    fn try_from(value: ReprCube) -> Result<Self, Self::Error> {
        Ok(match value {
            U1 => Move::U1,
            U2 => Move::U2,
            U3 => Move::U3,
            D1 => Move::D1,
            D2 => Move::D2,
            D3 => Move::D3,
            F1 => Move::F1,
            F2 => Move::F2,
            F3 => Move::F3,
            B1 => Move::B1,
            B2 => Move::B2,
            B3 => Move::B3,
            R1 => Move::R1,
            R2 => Move::R2,
            R3 => Move::R3,
            L1 => Move::L1,
            L2 => Move::L2,
            L3 => Move::L3,
            _ => return Err(value),
        })
    }
}

impl TryFrom<ReprCube> for Phase2Move {
    type Error = ReprCube;

    fn try_from(value: ReprCube) -> Result<Self, Self::Error> {
        Ok(match value {
            U1 => Phase2Move::U1,
            U2 => Phase2Move::U2,
            U3 => Phase2Move::U3,
            D1 => Phase2Move::D1,
            D2 => Phase2Move::D2,
            D3 => Phase2Move::D3,
            F2 => Phase2Move::F2,
            B2 => Phase2Move::B2,
            R2 => Phase2Move::R2,
            L2 => Phase2Move::L2,
            _ => return Err(value),
        })
    }
}

impl From<Phase2Move> for ReprCube {
    fn from(value: Phase2Move) -> Self {
        match value {
            Phase2Move::U1 => U1,
            Phase2Move::U2 => U2,
            Phase2Move::U3 => U3,
            Phase2Move::D1 => D1,
            Phase2Move::D2 => D2,
            Phase2Move::D3 => D3,
            Phase2Move::F2 => F2,
            Phase2Move::B2 => B2,
            Phase2Move::R2 => R2,
            Phase2Move::L2 => L2,
        }
    }
}

impl ReprCube {
    pub const fn const_eq(self, other: Self) -> bool {
        // let mut i = 0;
        // while i < 8 {
        //     if self.corner_orient[i] != other.corner_orient[i] {
        //         return false;
        //     }
        //     if self.corner_perm[i] != other.corner_perm[i] {
        //         return false;
        //     }
        //     i += 1;
        // }
        // while i < 12 {
        //     if self.edge_orient[i] != other.edge_orient[i] {
        //         return false;
        //     }
        //     if self.edge_perm[i] != other.edge_perm[i] {
        //         return false;
        //     }
        //     i += 1;
        // }
        // true

        todo!()
    }

    /// determine if the cube can be reached from
    pub const fn is_valid(self) -> bool {
        let mut sum = 0;
        let mut i = 0;
        while i < 12 {
            if self.edge_orient[i] > 1 {
                return false;
            }
            sum += self.edge_orient[i];
            i += 1;
        }
        if sum % 2 != 0 {
            return false;
        }

        let mut sum = 0;
        let mut i = 0;
        while i < 8 {
            if self.corner_orient[i] > 2 {
                return false;
            }
            sum += self.corner_orient[i];
            i += 1;
        }
        if sum % 3 != 0 {
            return false;
        }

        if self.edge_perm.is_odd() != self.corner_perm.is_odd() {
            return false;
        }

        true
    }

    pub fn is_solved(self) -> bool {
        self == SOLVED_CUBE
    }

    /// concatenate two cubes, as transformations from the solved cube.
    pub const fn then(self, other: Self) -> Self {
        Self {
            corner_perm: self.corner_perm.then(other.corner_perm);
            self.
        }
    }

    /// conjugate by one of the subgroup transforms, generated by
    ///
    /// S_LR2^A then S_U4^B then S_F2^C for A in 0..2, B in 0..4, C in 0..2
    pub const fn conjugate_by_subgroup_transform(self, transform: SubGroupTransform) -> Self {
        let s_lr2 = transform.0 & 0b0001;
        let s_u4 = (transform.0 & 0b0110) >> 1;
        let s_f2 = (transform.0 & 0b1000) >> 3;

        let mut working = SOLVED_CUBE;

        if s_lr2 == 1 {
            working = working.then(S_LR2_PARTIAL);
        }

        working = match s_u4 {
            0 => working,
            1 => working.then(S_U4_3),
            2 => working.then(S_U4_2),
            3 => working.then(S_U4_1),
            _ => unreachable!(),
        };

        if s_f2 == 1 {
            working = working.then(S_F2);
        }

        working = working.then(self);

        if s_f2 == 1 {
            working = working.then(S_F2);
        }

        working = match s_u4 {
            0 => working,
            1 => working.then(S_U4_1),
            2 => working.then(S_U4_2),
            3 => working.then(S_U4_3),
            _ => unreachable!(),
        };

        if s_lr2 == 1 {
            working = working.then(S_LR2_PARTIAL);

            let mut i = 0;
            while i < 8 {
                working.corner_orient[i] = (3 - working.corner_orient[i]) % 3;
                i += 1;
            }
        }

        working
    }

    /// Return the inverse cube transform `T⁻¹` such that
    /// `T.then(T⁻¹) == SOLVED_CUBE` and `T⁻¹.then(T) == SOLVED_CUBE`.
    pub const fn inverse(self) -> Self {
        // Start with an “empty” cube (we’ll fill in every index).
        let mut inv = ReprCube {
            corner_perm: [0; 8],
            corner_orient: [0; 8],
            edge_perm: [0; 12],
            edge_orient: [0; 12],
        };

        // Invert the corner‐permutation and twist:
        // if self.corner_perm[j] == src, then inv.corner_perm[src] = j,
        // and we must undo the twist at j by (3 - twist) % 3.
        let mut j = 0;
        while j < 8 {
            let src = self.corner_perm[j] as usize;
            inv.corner_perm[src] = j as u8;
            inv.corner_orient[src] = (3 - (self.corner_orient[j] % 3)) % 3;
            j += 1;
        }

        // Invert the edge‐permutation and flip‐bit:
        // if self.edge_perm[k] == src, then inv.edge_perm[src] = k,
        // and we undo the flip by (2 - flip) % 2 (≡ flip mod 2).
        let mut k = 0;
        while k < 12 {
            let src = self.edge_perm[k] as usize;
            inv.edge_perm[src] = k as u8;
            inv.edge_orient[src] = (2 - (self.edge_orient[k] % 2)) % 2;
            k += 1;
        }

        inv
    }

    pub fn all_nontrivial_subgroup_conjugations(
        self,
    ) -> impl Iterator<Item = (SubGroupTransform, Self)> {
        (1..16)
            .map(SubGroupTransform)
            .map(move |t| (t, self.conjugate_by_subgroup_transform(t)))
    }

    pub fn all_phase_1_adjacent(self) -> impl Iterator<Item = (Move, Self)> {
        Move::all_iter().map(move |m| (m, self.then(m.into())))
    }

    pub fn all_phase_2_adjacent(self) -> impl Iterator<Item = (Phase2Move, Self)> {
        Phase2Move::all_iter().map(move |m| (m, self.then(m.into())))
    }

    /// returns exactly 18 + 15 = 33 items.
    pub fn phase_1_move_table_entry_cubes(self) -> impl Iterator<Item = Self> {
        Move::all_iter().map(move |m| self.then(m.into())).chain(
            (1..16)
                .map(SubGroupTransform)
                .map(move |t| self.conjugate_by_subgroup_transform(t)),
        )
    }

    /// returns exactly 10 + 15 = 25 items.
    pub fn phase_2_move_table_entry_cubes(self) -> impl Iterator<Item = Self> {
        Phase2Move::all_iter()
            .map(move |m| self.then(m.into()))
            .chain(
                (1..16)
                    .map(SubGroupTransform)
                    .map(move |t| self.conjugate_by_subgroup_transform(t)),
            )
    }

    pub fn into_phase_1_raw_coords(
        self,
    ) -> (RawCornerOrientCoord, RawEdgeOrientCoord, RawEdgeGroupCoord) {
        (
            RawCornerOrientCoord::from_cubie(self),
            RawEdgeOrientCoord::from_cubie(self),
            RawEdgeGroupCoord::from_cubie(self),
        )
    }

    pub fn pretty_print(self) {
        //-> [[&'static str; 9]; 6] {
        const COLOR_CHARS: [&str; 6] = [
            "\x1b[47m  \x1b[0m", // W (U)
            "\x1b[43m  \x1b[0m", // Y (D)
            "\x1b[41m  \x1b[0m", // R (F)
            "\x1b[44m  \x1b[0m", // O (B)
            "\x1b[45m  \x1b[0m", // B (R)
            "\x1b[42m  \x1b[0m", // G (L)
        ];

        // const COLOR_CHARS: [&str; 6] = [
        //     "W ", // W
        //     "G ", // G
        //     "R ", // R
        //     "B ", // B
        //     "O ", // O
        //     "Y ", // Y
        // ];

        const CORNER_FACELETS: [[(usize, usize); 3]; 8] = [
            [(0, 8), (4, 0), (2, 2)], // UFR
            [(0, 6), (2, 0), (5, 2)], // UFL
            [(0, 2), (3, 0), (4, 2)], // UBR
            [(0, 0), (5, 0), (3, 2)], // UBL
            [(1, 2), (2, 8), (4, 6)], // DFR
            [(1, 0), (5, 8), (2, 6)], // DFL
            [(1, 8), (4, 8), (3, 6)], // DBR
            [(1, 6), (3, 8), (5, 6)], // DBL
        ];

        const EDGE_FACELETS: [[(usize, usize); 2]; 12] = [
            [(0, 7), (2, 1)], // UF
            [(0, 1), (3, 1)], // UB
            [(0, 5), (4, 1)], // UR
            [(0, 3), (5, 1)], // UL
            [(1, 1), (2, 7)], // DF
            [(1, 7), (3, 7)], // DB
            [(1, 5), (4, 7)], // DR
            [(1, 3), (5, 7)], // DL
            [(2, 5), (4, 3)], // FR
            [(2, 3), (5, 5)], // FL
            [(3, 3), (4, 5)], // BR
            [(3, 5), (5, 3)], // BL
        ];

        // start with “blank” faces (or you could fill with e.g. '·')
        let mut faces = [["· "; 9]; 6];

        // Place corners
        for (slot, &piece) in self.corner_perm.iter().enumerate() {
            let ori = self.corner_orient[slot] as usize;

            for j in 0..3 {
                let slot = CORNER_FACELETS[slot][j];
                let color_i = CORNER_FACELETS[piece as usize][(j + 3 - ori) % 3].0;

                faces[slot.0][slot.1] = COLOR_CHARS[color_i];
            }
        }

        for (slot, &piece) in self.edge_perm.iter().enumerate() {
            let ori = self.edge_orient[slot] as usize;

            for j in 0..2 {
                let slot = EDGE_FACELETS[slot][j];
                let color_i = EDGE_FACELETS[piece as usize][(j + ori) % 2].0;

                faces[slot.0][slot.1] = COLOR_CHARS[color_i];
            }
        }

        // // Place edges
        // for (slot, &piece) in self.edge_perm.iter().enumerate() {
        //     let ori = (self.edge_orient[slot] % 2) as usize;
        //     let facelets = EDGE_FACELETS[piece as usize];

        //     for j in 0..2 {
        //         let src_face = facelets[j].0;
        //         let (dst_face, dst_pos) = facelets[(j + ori) % 2];
        //         faces[dst_face][dst_pos] = COLOR_CHARS[src_face];
        //     }
        // }

        // Place centers
        for (i, c) in COLOR_CHARS.into_iter().enumerate() {
            faces[i][4] = c;
        }

        // faces

        const EMPTY_FACE: [&str; 9] = ["  "; 9];

        let spaced_faces = [
            EMPTY_FACE, faces[0], EMPTY_FACE, EMPTY_FACE, faces[5], faces[2], faces[4], faces[3],
            EMPTY_FACE, faces[1], EMPTY_FACE, EMPTY_FACE,
        ];

        for inter_face_row in 0..3 {
            for intra_face_row in 0..3 {
                for inter_face_col in 0..4 {
                    for intra_face_col in 0..3 {
                        print!(
                            "{}",
                            spaced_faces[inter_face_row * 4 + inter_face_col]
                                [intra_face_row * 3 + intra_face_col]
                        );
                    }
                }
                println!()
            }
        }
    }
}

impl Distribution<ReprCube> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ReprCube {
        let mut cube = SOLVED_CUBE;

        // retry until permutations are legal. this is lazy but should be quick enough still.
        loop {
            cube.edge_perm = permutation_coord_12_inverse(rng.random_range(0..479_001_600));
            cube.corner_perm = permutation_coord_8_inverse(rng.random_range(0..40320));
            if is_odd(&cube.edge_perm) == is_odd(&cube.corner_perm) {
                break;
            }
        }

        let mut edge_orient_coord: u16 = rng.random_range(0..2048);
        let mut sum = 0;
        for i in 0..11 {
            let val = edge_orient_coord % 2;
            sum += val;
            edge_orient_coord >>= 1;
            cube.edge_orient[i] = val as u8;
        }
        cube.edge_orient[11] = ((12 - sum) % 2) as u8;

        let mut corner_orient_coord: u16 = rng.random_range(0..2187);
        let mut sum = 0;
        for i in 0..7 {
            let val = corner_orient_coord % 3;
            sum += val;
            corner_orient_coord /= 3;
            cube.corner_orient[i] = val as u8;
        }
        cube.corner_orient[7] = ((18 - sum) % 3) as u8;

        cube
    }
}

impl Distribution<Move> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Move {
        unsafe { core::mem::transmute(rng.random_range(0..18u8)) }
    }
}

#[test]
fn test_lr2() {
    assert_eq!(U1, U3.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(U2, U2.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(U3, U1.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(D1, D3.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(D2, D2.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(D3, D1.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(F1, F3.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(F2, F2.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(F3, F1.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(B1, B3.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(B2, B2.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(B3, B1.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(R1, L3.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(R2, L2.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(R3, L1.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(L1, R3.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(L2, R2.conjugate_by_subgroup_transform(SubGroupTransform(1)));
    assert_eq!(L3, R1.conjugate_by_subgroup_transform(SubGroupTransform(1)));
}

#[test]
fn test_f2() {
    assert_eq!(U1, D1.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(U2, D2.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(U3, D3.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(D1, U1.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(D2, U2.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(D3, U3.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(F1, F1.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(F2, F2.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(F3, F3.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(B1, B1.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(B2, B2.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(B3, B3.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(R1, L1.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(R2, L2.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(R3, L3.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(L1, R1.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(L2, R2.conjugate_by_subgroup_transform(SubGroupTransform(8)));
    assert_eq!(L3, R3.conjugate_by_subgroup_transform(SubGroupTransform(8)));
}

#[test]
fn test_u4() {
    assert_eq!(U1, U1.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(U2, U2.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(U3, U3.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(D1, D1.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(D2, D2.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(D3, D3.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(F1, R1.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(F2, R2.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(F3, R3.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(B1, L1.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(B2, L2.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(B3, L3.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(R1, B1.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(R2, B2.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(R3, B3.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(L1, F1.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(L2, F2.conjugate_by_subgroup_transform(SubGroupTransform(2)));
    assert_eq!(L3, F3.conjugate_by_subgroup_transform(SubGroupTransform(2)));
}

#[test]
fn test_random_moves() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for t in SubGroupTransform::all_iter() {
        for _ in 0..50 {
            let mut cubea = SOLVED_CUBE;
            let mut cubeb = SOLVED_CUBE;

            for _ in 0..20 {
                let mv: Move = rng.sample(StandardUniform);
                let mv = mv.into();

                cubea = cubea.then(mv);
                cubeb = cubeb.then(mv.conjugate_by_subgroup_transform(t));
            }

            cubea = cubea.conjugate_by_subgroup_transform(t);

            assert_eq!(cubea, cubeb);
        }
    }
}

#[test]
fn test_inversion_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);

    for _ in 0..2000 {
        let cube: ReprCube = rng.sample(StandardUniform);
        assert!(cube.is_valid());
        assert_eq!(SOLVED_CUBE, cube.then(cube.inverse()))
    }
}

#[test]
fn superflip() {
    let superflip = cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2];

    assert_eq!(superflip.corner_perm, SOLVED_CUBE.corner_perm);
    assert_eq!(superflip.edge_perm, SOLVED_CUBE.edge_perm);
    assert_eq!(superflip.corner_orient, SOLVED_CUBE.corner_orient);

    assert_eq!(superflip.edge_orient, [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
}

#[test]
fn move_entries() {
    for e in R1.phase_1_move_table_entry_cubes() {
        match e {
            SOLVED_CUBE => println!("Solved"),
            U1 => println!("U1"),
            U2 => println!("U2"),
            U3 => println!("U3"),
            D1 => println!("D1"),
            D2 => println!("D2"),
            D3 => println!("D3"),
            F1 => println!("F1"),
            F2 => println!("F2"),
            F3 => println!("F3"),
            B1 => println!("B1"),
            B2 => println!("B2"),
            B3 => println!("B3"),
            R1 => println!("R1"),
            R2 => println!("R2"),
            R3 => println!("R3"),
            L1 => println!("L1"),
            L2 => println!("L2"),
            L3 => println!("L3"),
            _ => println!("{e:?}"),
        }
    }
}

#[test]
fn test_all_moves() {
    let mut c = ReprCube::default();
    c = c.then(U1);
    c = c.then(U2);
    c = c.then(U3);
    c = c.then(U2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c = c.then(D1);
    c = c.then(D2);
    c = c.then(D3);
    c = c.then(D2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c = c.then(F1);
    c = c.then(F2);
    c = c.then(F3);
    c = c.then(F2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c = c.then(B1);
    c = c.then(B2);
    c = c.then(B3);
    c = c.then(B2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c = c.then(R1);
    c = c.then(R2);
    c = c.then(R3);
    c = c.then(R2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c = c.then(L1);
    c = c.then(L2);
    c = c.then(L3);
    c = c.then(L2);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_long_identity() {
    let mut c = ReprCube::default();
    c = c.then(F1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(L3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(R2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(L1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(B1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(U3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(B3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(R3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(D2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(F2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c = c.then(D1);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn sexy_move() {
    let mut c = ReprCube::default();

    for _ in 0..6 {
        c = c.then(cube![U F Up Fp]);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn hundred_thousand_moves_simd() {
    let mut c = ReprCube::default();

    for _ in 0..1000 {
        c = c.then(F1);
        c = c.then(R1);
        c = c.then(F3);
        c = c.then(U1);
        c = c.then(B2);
        c = c.then(L3);
        c = c.then(D3);
        c = c.then(R2);
        c = c.then(L1);
        c = c.then(B2);
        c = c.then(F3);
        c = c.then(D1);
        c = c.then(U2);
        c = c.then(R1);
        c = c.then(B1);
        c = c.then(U3);
        c = c.then(B3);
        c = c.then(D1);
        c = c.then(F3);
        c = c.then(U2);
        c = c.then(F3);
        c = c.then(R1);
        c = c.then(U1);
        c = c.then(R3);
        c = c.then(L2);
        c = c.then(U1);
        c = c.then(L2);
        c = c.then(D3);
        c = c.then(L2);
        c = c.then(D2);
        c = c.then(F2);
        c = c.then(D1);
        c = c.then(F1);
        c = c.then(R1);
        c = c.then(F3);
        c = c.then(U1);
        c = c.then(B2);
        c = c.then(L3);
        c = c.then(D3);
        c = c.then(R2);
        c = c.then(L2);
        c = c.then(L3);
        c = c.then(B2);
        c = c.then(F3);
        c = c.then(D1);
        c = c.then(U2);
        c = c.then(R1);
        c = c.then(B1);
        c = c.then(U3);
        c = c.then(B3);
        c = c.then(D1);
        c = c.then(F3);
        c = c.then(U1);
        c = c.then(U1);
        c = c.then(F3);
        c = c.then(R1);
        c = c.then(U1);
        c = c.then(R3);
        c = c.then(L2);
        c = c.then(U1);
        c = c.then(L2);
        c = c.then(D3);
        c = c.then(L2);
        c = c.then(D2);
        c = c.then(F2);
        c = c.then(D1);
        c = c.then(F1);
        c = c.then(R1);
        c = c.then(F3);
        c = c.then(U1);
        c = c.then(B2);
        c = c.then(L3);
        c = c.then(D3);
        c = c.then(R2);
        c = c.then(L1);
        c = c.then(B2);
        c = c.then(F2);
        c = c.then(F1);
        c = c.then(D1);
        c = c.then(U2);
        c = c.then(R1);
        c = c.then(B1);
        c = c.then(U3);
        c = c.then(B3);
        c = c.then(D1);
        c = c.then(F3);
        c = c.then(U2);
        c = c.then(F3);
        c = c.then(R1);
        c = c.then(U1);
        c = c.then(R3);
        c = c.then(L2);
        c = c.then(U1);
        c = c.then(L2);
        c = c.then(D3);
        c = c.then(L1);
        c = c.then(L1);
        c = c.then(D2);
        c = c.then(F2);
        c = c.then(D1);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_apply() {
    let c = cube![R U Rp Up];

    let mut c2 = ReprCube::default();

    for _ in 0..6 {
        c2 = c2.then(c);
    }

    assert!(c2.is_solved());
}

#[test]
fn test_2_move_apply() {
    let c = cube![R U];

    let mut c2 = ReprCube::default();
    c2 = c2.then(c);
    let mut count = 1;
    while !c2.is_solved() {
        count += 1;
        c2 = c2.then(c);
    }

    assert_eq!(count, 105);
}

#[test]
fn test_long_apply() {
    let c = cube![R U2 Dp B Dp];

    let mut c2 = ReprCube::default();
    c2 = c2.then(c);
    let mut count = 1;
    while !c2.is_solved() {
        count += 1;
        c2 = c2.then(c);
    }
    assert_eq!(count, 1260);
}

#[test]
fn different_cubes() {
    let cube = cube![U R];

    let mut set = std::collections::HashSet::new();
    for t1 in SubGroupTransform::all_iter() {
        for t2 in SubGroupTransform::all_iter() {
            set.insert(
                cube.conjugate_by_subgroup_transform(t1)
                    .conjugate_by_subgroup_transform(t2),
            );
        }
    }

    println!("set_len: {:?}", set.len());
}

#[test]
fn print_cube() {
    SOLVED_CUBE.pretty_print();
    cube![U].pretty_print();
    cube![D].pretty_print();
    cube![F].pretty_print();
    cube![B].pretty_print();
    cube![R].pretty_print();
    cube![L].pretty_print();

    cube![Rp Dp R D Rp Dp R D Up Dp Rp D R Dp Rp D R U].pretty_print();
    cube![R U Rp U R U2 Rp].pretty_print();

    // manually inspect cube afterwards to see if it matches. (it does)
    cube![U R F D L B U2 R2 F2 D2 L2 B2 Up Rp Fp Dp Lp Bp].pretty_print();
}

#[test]
fn print_all_syms() {
    let cube = cube![U R];
    for t in SubGroupTransform::all_iter() {
        cube.conjugate_by_subgroup_transform(t).pretty_print();
        println!();
    }
}

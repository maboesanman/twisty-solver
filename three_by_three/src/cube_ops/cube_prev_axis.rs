use core::simd;

use crate::{
    CubeMove,
    cube_ops::{cube_move::DominoMove, cube_sym::DominoSymmetry}, simd_utils::lut_u8x16x3,
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CubePreviousAxis {
    U,
    D,
    UD,
    F,
    B,
    FB,
    R,
    L,
    RL,
    None,
    NoneAlt,
}

impl CubePreviousAxis {
    #[inline(always)]
    pub const fn update_with_new_move(self, mv: CubeMove, remaining_moves: u8) -> Self {
        if remaining_moves == 1 {
            match mv {
                CubeMove::F2 | CubeMove::B2 => return Self::FB,
                CubeMove::R2 | CubeMove::L2 => return Self::RL,
                _ => {}
            }
        }
        match (mv as u8 / 3, self) {
            (0, Self::D) | (1, Self::U) | (0, Self::UD) | (1, Self::UD) => Self::UD,
            (2, Self::B) | (3, Self::F) | (2, Self::FB) | (3, Self::FB) => Self::FB,
            (4, Self::L) | (5, Self::R) | (4, Self::RL) | (5, Self::RL) => Self::RL,
            (0, _) => Self::U,
            (1, _) => Self::D,
            (2, _) => Self::F,
            (3, _) => Self::B,
            (4, _) => Self::R,
            (5, _) => Self::L,
            _ => unreachable!(),
        }
    }

    pub const fn update_with_new_domino_move(self, mv: DominoMove) -> Self {
        let x = match mv {
            DominoMove::U1 => 0,
            DominoMove::U2 => 0,
            DominoMove::U3 => 0,
            DominoMove::D1 => 1,
            DominoMove::D2 => 1,
            DominoMove::D3 => 1,
            DominoMove::F2 => 2,
            DominoMove::B2 => 3,
            DominoMove::R2 => 4,
            DominoMove::L2 => 5,
        };
        match (x, self) {
            (0, Self::D) | (1, Self::U) | (0, Self::UD) | (1, Self::UD) => Self::UD,
            (2, Self::B) | (3, Self::F) | (2, Self::FB) | (3, Self::FB) => Self::FB,
            (4, Self::L) | (5, Self::R) | (4, Self::RL) | (5, Self::RL) => Self::RL,
            (0, _) => Self::U,
            (1, _) => Self::D,
            (2, _) => Self::F,
            (3, _) => Self::B,
            (4, _) => Self::R,
            (5, _) => Self::L,
            _ => unreachable!(),
        }
    }

    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        const LOOKUP: [CubePreviousAxis; 9 * 16] = const {
            let mut lookup = [CubePreviousAxis::U; 9 * 16];

            let mut i = 0usize;

            while i < 9 * 16 {
                let prev_axis: CubePreviousAxis = unsafe { std::mem::transmute((i >> 4) as u8) };
                let sym: DominoSymmetry = unsafe { std::mem::transmute((i & 15) as u8) };

                let (mv, double) = match prev_axis {
                    CubePreviousAxis::U => (CubeMove::U1, false),
                    CubePreviousAxis::D => (CubeMove::D1, false),
                    CubePreviousAxis::UD => (CubeMove::U1, true),
                    CubePreviousAxis::F => (CubeMove::F1, false),
                    CubePreviousAxis::B => (CubeMove::B1, false),
                    CubePreviousAxis::FB => (CubeMove::F1, true),
                    CubePreviousAxis::R => (CubeMove::R1, false),
                    CubePreviousAxis::L => (CubeMove::L1, false),
                    CubePreviousAxis::RL => (CubeMove::R1, true),
                    CubePreviousAxis::None | CubePreviousAxis::NoneAlt => unreachable!(),
                };

                let mv = mv.domino_conjugate(sym);

                let mut val = CubePreviousAxis::None.update_with_new_move(mv, 10);

                if double {
                    val = match val as u8 / 3 {
                        0 => CubePreviousAxis::UD,
                        1 => CubePreviousAxis::FB,
                        2 => CubePreviousAxis::RL,
                        _ => unreachable!(),
                    };
                }

                lookup[i] = val;

                i += 1;
            }

            lookup
        };

        LOOKUP[((self as u8 as usize) << 4) + (sym.0 as usize)]
    }
}

#[test]
fn draw_cube_axis_conj_table() {
    println!();
    print!("    ");
    for a in 0..6 {
        print!(" {a:03b}");
    }
    println!();

    for s in 0..16 {
        print!("{s:04b}");

        let s = DominoSymmetry(s);
        for a in 0..6 {
            let ax = match a {
                0 => CubePreviousAxis::F,
                1 => CubePreviousAxis::R,
                2 => CubePreviousAxis::B,
                3 => CubePreviousAxis::L,
                4 => CubePreviousAxis::U,
                5 => CubePreviousAxis::D,
                _ => unreachable!()
            };
            let r = ax.domino_conjugate(s);
            let x = match r {
                CubePreviousAxis::U => 4,
                CubePreviousAxis::D => 5,
                CubePreviousAxis::F => 0,
                CubePreviousAxis::B => 2,
                CubePreviousAxis::R => 1,
                CubePreviousAxis::L => 3,
                _ => unreachable!()
            };

            print!(" {x:03b}");
        }
        println!();
    }

}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CubeFace(pub u8);

impl CubeFace {
    const F: Self = CubeFace(0b000);
    const R: Self = CubeFace(0b001);
    const B: Self = CubeFace(0b010);
    const L: Self = CubeFace(0b011);
    const U: Self = CubeFace(0b100);
    const D: Self = CubeFace(0b101);

    #[inline(always)]
    fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        todo!()
    }

    #[inline(always)]
    fn domino_conjugate_16(faces: [u8; 16], syms: [u8; 16]) -> [u8; 16] {
        let faces = simd::u8x16::from_array(faces);
        let syms = simd::u8x16::from_array(syms);
        let indices = (faces << 3) | (syms >> 3);
        let shifts = (syms & simd::u8x16::splat(1)) << 2;
        let vals = lut_u8x16x3(&FACE_CONJ, &indices.to_array());
        let out = (simd::u8x16::from_array(vals) >> shifts) & simd::u8x16::splat(0b1111);
        out.to_array()
    }
}

const FACE_CONJ: [u8; 48] = {
    let mut out = [0u8; 48];

    let mut s = 0u8;
    while s < 16 {
        let sy = DominoSymmetry(s);
        let mut f = 0u8;
        while f < 6 {
            let ax = match f {
                0 => CubePreviousAxis::F,
                1 => CubePreviousAxis::R,
                2 => CubePreviousAxis::B,
                3 => CubePreviousAxis::L,
                4 => CubePreviousAxis::U,
                5 => CubePreviousAxis::D,
                _ => unreachable!()
            };
            let r = ax.domino_conjugate(sy);
            let x: u8 = match r {
                CubePreviousAxis::U => 4,
                CubePreviousAxis::D => 5,
                CubePreviousAxis::F => 0,
                CubePreviousAxis::B => 2,
                CubePreviousAxis::R => 1,
                CubePreviousAxis::L => 3,
                _ => unreachable!()
            };

            let i = (x << 3) | (s >> 3);
            let v = x << ((s & 1) << 2);
            out[i as usize] |= v;

            f += 1;
        }
        s += 1;
    }

    out
};

// encodes the face in 0b0111 and "blocks opposite face also" as
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CubePreviousFace(pub u8);

impl CubePreviousFace {
    #[must_use]
    #[inline(always)]
    pub fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let f = CubeFace(self.0 & 0b111);
        let new_f = f.domino_conjugate(sym);
        Self((self.0 & 0b1000) | new_f.0)
    }

    #[inline(always)]
     pub fn domino_conjugate_16(axes: [u8; 16], syms: [u8; 16]) -> [u8; 16] {
        let axes = simd::u8x16::from_array(axes);
        let faces = axes & simd::u8x16::splat(0b0111);
        let blocks = axes & simd::u8x16::splat(0b1000);
        let face_out = simd::u8x16::from_array(CubeFace::domino_conjugate_16(faces.to_array(), syms));
        let out = blocks | face_out;
        out.to_array()
    }

    #[inline(always)]
     pub fn get_moves(self) -> &'static [CubeMove; 16] {
        &MOVES_TABLE[(self.0 & 0b111) as usize]
    }

    #[inline(always)]
     pub fn get_new_prev_axes(self) -> &'static [Self; 16] {
        const TABLE: [[CubePreviousFace; 16]; 6] = {
            use CubeMove::*;

            let mut new_table = [[CubePreviousFace(0); 16]; 6];
            let mut f = 0;
            while f < 6 {
                let mut mv = 0;
                while mv < 16 {
                    #[rustfmt::skip]
                    let new_f = match MOVES_TABLE[f][mv] {
                        U1 | U2 | U3 => CubeFace::U,
                        D1 | D2 | D3 => CubeFace::D,
                        F1 | F2 | F3 => CubeFace::F,
                        B1 | B2 | B3 => CubeFace::B,
                        R1 | R2 | R3 => CubeFace::R,
                        L1 | L2 | L3 => CubeFace::L,
                    }.0;

                    let blocking = ((mv > 12) as u8) << 3;
                    new_table[f][mv] = CubePreviousFace(new_f | blocking);

                    mv += 1;
                }
                f += 1;
            }

            new_table
        };
        &TABLE[(self.0 & 0b111) as usize]
    }

    #[inline(always)]
     pub fn get_num_moves(self) -> u8 {
        let bit = self.0 & 8;
        12u8 + (bit >> 2) + (bit >> 3)
    }
}

#[rustfmt::skip]
const MOVES_TABLE: [[CubeMove; 16]; 6] = {
    use CubeMove::*;
    [
        /* F */ [ F1, U1, U2, U3, D1, D2, D3, R1, R2, R3, L1, L2, L3, B1, B2, B3],
        /* R */ [ R1, U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, L1, L2, L3],
        /* B */ [ B1, U1, U2, U3, D1, D2, D3, R1, R2, R3, L1, L2, L3, F1, F2, F3],
        /* L */ [ L1, U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3],
        /* U */ [ U1, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3, D1, D2, D3],
        /* D */ [ D1, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3, U1, U2, U3],
    ]
};

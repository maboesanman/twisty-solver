use crate::{
    cube_ops::cube_prev_axis::CubePreviousAxis, permutation_math::permutation::Permutation,
};

use super::{
    partial_reprs::{
        corner_orient::CornerOrient, corner_perm::CornerPerm, edge_orient::EdgeOrient,
        edge_perm::EdgePerm,
    },
    repr_cube::ReprCube,
};

pub const U_CORNER_PERM: CornerPerm =
    CornerPerm(Permutation::const_from_array([2, 0, 3, 1, 4, 5, 6, 7]));
pub const U_EDGE_PERM: EdgePerm = EdgePerm(Permutation::const_from_array([
    2, 3, 1, 0, 4, 5, 6, 7, 8, 9, 10, 11,
]));

pub const D_CORNER_PERM: CornerPerm =
    CornerPerm(Permutation::const_from_array([0, 1, 2, 3, 5, 7, 4, 6]));
pub const D_EDGE_PERM: EdgePerm = EdgePerm(Permutation::const_from_array([
    0, 1, 2, 3, 7, 6, 4, 5, 8, 9, 10, 11,
]));

pub const F_CORNER_PERM: CornerPerm =
    CornerPerm(Permutation::const_from_array([1, 5, 2, 3, 0, 4, 6, 7]));
pub const F_EDGE_PERM: EdgePerm = EdgePerm(Permutation::const_from_array([
    9, 1, 2, 3, 8, 5, 6, 7, 0, 4, 10, 11,
]));
pub const F_CORNER_ORIENT_CORRECT: CornerOrient =
    CornerOrient::const_from_array([1, 2, 0, 0, 2, 1, 0, 0]);
pub const F_EDGE_ORIENT_CORRECT: EdgeOrient =
    EdgeOrient::const_from_array([1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0]);

pub const B_CORNER_PERM: CornerPerm =
    CornerPerm(Permutation::const_from_array([0, 1, 6, 2, 4, 5, 7, 3]));
pub const B_EDGE_PERM: EdgePerm = EdgePerm(Permutation::const_from_array([
    0, 10, 2, 3, 4, 11, 6, 7, 8, 9, 5, 1,
]));
pub const B_CORNER_ORIENT_CORRECT: CornerOrient =
    CornerOrient::const_from_array([0, 0, 2, 1, 0, 0, 1, 2]);
pub const B_EDGE_ORIENT_CORRECT: EdgeOrient =
    EdgeOrient::const_from_array([0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1]);

pub const R_CORNER_PERM: CornerPerm =
    CornerPerm(Permutation::const_from_array([4, 1, 0, 3, 6, 5, 2, 7]));
pub const R_EDGE_PERM: EdgePerm = EdgePerm(Permutation::const_from_array([
    0, 1, 8, 3, 4, 5, 10, 7, 6, 9, 2, 11,
]));
pub const R_CORNER_ORIENT_CORRECT: CornerOrient =
    CornerOrient::const_from_array([2, 0, 1, 0, 1, 0, 2, 0]);

pub const L_CORNER_PERM: CornerPerm =
    CornerPerm(Permutation::const_from_array([0, 3, 2, 7, 4, 1, 6, 5]));
pub const L_EDGE_PERM: EdgePerm = EdgePerm(Permutation::const_from_array([
    0, 1, 2, 11, 4, 5, 6, 9, 8, 3, 10, 7,
]));
pub const L_CORNER_ORIENT_CORRECT: CornerOrient =
    CornerOrient::const_from_array([0, 1, 0, 2, 0, 2, 0, 1]);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[allow(unused)]
pub enum CubeMove {
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

impl std::fmt::Display for CubeMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            CubeMove::U1 => "U",
            CubeMove::U2 => "U2",
            CubeMove::U3 => "U'",
            CubeMove::D1 => "D",
            CubeMove::D2 => "D2",
            CubeMove::D3 => "D'",
            CubeMove::F1 => "F",
            CubeMove::F2 => "F2",
            CubeMove::F3 => "F'",
            CubeMove::B1 => "B",
            CubeMove::B2 => "B2",
            CubeMove::B3 => "B'",
            CubeMove::R1 => "R",
            CubeMove::R2 => "R2",
            CubeMove::R3 => "R'",
            CubeMove::L1 => "L",
            CubeMove::L2 => "L2",
            CubeMove::L3 => "L'",
        };
        f.write_str(string)
    }
}

impl CubeMove {
    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0u8..18u8).map(|x| unsafe { core::mem::transmute(x) })
    }

    pub fn new_axis_iter(prev_axis: CubePreviousAxis) -> impl IntoIterator<Item = Self> {
        use CubeMove::*;

        let slice: &[CubeMove] = match prev_axis {
            CubePreviousAxis::U => &[D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3],
            CubePreviousAxis::D | CubePreviousAxis::UD => &[F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3],
            CubePreviousAxis::F => &[U1, U2, U3, D1, D2, D3, B1, B2, B3, R1, R2, R3, L1, L2, L3],
            CubePreviousAxis::B | CubePreviousAxis::FB => &[U1, U2, U3, D1, D2, D3, R1, R2, R3, L1, L2, L3],
            CubePreviousAxis::R => &[U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, L1, L2, L3],
            CubePreviousAxis::L | CubePreviousAxis::RL => &[U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3],
            CubePreviousAxis::None => &[U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3],
        };

        slice.into_iter().copied()
    }

    pub fn new_axis_iter_end_phase_1(prev_axis: CubePreviousAxis) -> impl IntoIterator<Item = Self> {
        use CubeMove::*;

        let slice: &[CubeMove] = match prev_axis {
            CubePreviousAxis::U | CubePreviousAxis::D | CubePreviousAxis::UD => &[F1, F3, B1, B3, R1, R3, L1, L3],
            CubePreviousAxis::F => &[B1, B3, R1, R3, L1, L3],
            CubePreviousAxis::B | CubePreviousAxis::FB => &[R1, R3, L1, L3],
            CubePreviousAxis::R => &[F1, F3, B1, B3, L1, L3],
            CubePreviousAxis::L | CubePreviousAxis::RL => &[F1, F3, B1, B3],

            CubePreviousAxis::None => &[U1, U2, U3, D1, D2, D3, F1, F2, F3, B1, B2, B3, R1, R2, R3, L1, L2, L3],
        };

        slice.into_iter().copied()
    }

    pub const fn into_u8(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }

    pub const fn into_index(self) -> usize {
        self.into_u8() as usize
    }

    pub const fn into_edge_orient(self) -> EdgeOrient {
        const TABLE: [EdgeOrient; 18] = const {
            let mut val = [EdgeOrient::SOLVED; 18];
            let mut i = 0;
            while i < 18 {
                let mv: CubeMove = unsafe { core::mem::transmute(i as u8) };
                val[i] = match mv {
                    CubeMove::F1 => F_EDGE_ORIENT_CORRECT,
                    CubeMove::F3 => F_EDGE_ORIENT_CORRECT,
                    CubeMove::B1 => B_EDGE_ORIENT_CORRECT,
                    CubeMove::B3 => B_EDGE_ORIENT_CORRECT,
                    _ => EdgeOrient::SOLVED,
                };
                i += 1;
            }

            val
        };
        TABLE[self.into_index()]
    }

    pub const fn into_corner_orient(self) -> CornerOrient {
        const TABLE: [CornerOrient; 18] = const {
            let mut val = [CornerOrient::SOLVED; 18];
            let mut i = 0;
            while i < 18 {
                let mv: CubeMove = unsafe { core::mem::transmute(i as u8) };
                val[i] = match mv {
                    CubeMove::F1 | CubeMove::F3 => F_CORNER_ORIENT_CORRECT,
                    CubeMove::B1 | CubeMove::B3 => B_CORNER_ORIENT_CORRECT,
                    CubeMove::R1 | CubeMove::R3 => R_CORNER_ORIENT_CORRECT,
                    CubeMove::L1 | CubeMove::L3 => L_CORNER_ORIENT_CORRECT,
                    _ => CornerOrient::SOLVED,
                };
                i += 1;
            }

            val
        };
        TABLE[self.into_index()]
    }

    pub const fn into_edge_perm(self) -> EdgePerm {
        const TABLE: [EdgePerm; 18] = const {
            let mut val = [EdgePerm::SOLVED; 18];
            let mut i = 0;
            while i < 18 {
                let mv: CubeMove = unsafe { core::mem::transmute(i as u8) };
                val[i] = match mv {
                    CubeMove::U1 => U_EDGE_PERM,
                    CubeMove::U2 => U_EDGE_PERM.then(U_EDGE_PERM),
                    CubeMove::U3 => U_EDGE_PERM.then(U_EDGE_PERM).then(U_EDGE_PERM),
                    CubeMove::D1 => D_EDGE_PERM,
                    CubeMove::D2 => D_EDGE_PERM.then(D_EDGE_PERM),
                    CubeMove::D3 => D_EDGE_PERM.then(D_EDGE_PERM).then(D_EDGE_PERM),
                    CubeMove::F1 => F_EDGE_PERM,
                    CubeMove::F2 => F_EDGE_PERM.then(F_EDGE_PERM),
                    CubeMove::F3 => F_EDGE_PERM.then(F_EDGE_PERM).then(F_EDGE_PERM),
                    CubeMove::B1 => B_EDGE_PERM,
                    CubeMove::B2 => B_EDGE_PERM.then(B_EDGE_PERM),
                    CubeMove::B3 => B_EDGE_PERM.then(B_EDGE_PERM).then(B_EDGE_PERM),
                    CubeMove::R1 => R_EDGE_PERM,
                    CubeMove::R2 => R_EDGE_PERM.then(R_EDGE_PERM),
                    CubeMove::R3 => R_EDGE_PERM.then(R_EDGE_PERM).then(R_EDGE_PERM),
                    CubeMove::L1 => L_EDGE_PERM,
                    CubeMove::L2 => L_EDGE_PERM.then(L_EDGE_PERM),
                    CubeMove::L3 => L_EDGE_PERM.then(L_EDGE_PERM).then(L_EDGE_PERM),
                };
                i += 1;
            }

            val
        };
        TABLE[self.into_index()]
    }

    pub const fn into_corner_perm(self) -> CornerPerm {
        const TABLE: [CornerPerm; 18] = const {
            let mut val = [CornerPerm::SOLVED; 18];
            let mut i = 0;
            while i < 18 {
                let mv: CubeMove = unsafe { core::mem::transmute(i as u8) };
                val[i] = match mv {
                    CubeMove::U1 => U_CORNER_PERM,
                    CubeMove::U2 => U_CORNER_PERM.then(U_CORNER_PERM),
                    CubeMove::U3 => U_CORNER_PERM.then(U_CORNER_PERM).then(U_CORNER_PERM),
                    CubeMove::D1 => D_CORNER_PERM,
                    CubeMove::D2 => D_CORNER_PERM.then(D_CORNER_PERM),
                    CubeMove::D3 => D_CORNER_PERM.then(D_CORNER_PERM).then(D_CORNER_PERM),
                    CubeMove::F1 => F_CORNER_PERM,
                    CubeMove::F2 => F_CORNER_PERM.then(F_CORNER_PERM),
                    CubeMove::F3 => F_CORNER_PERM.then(F_CORNER_PERM).then(F_CORNER_PERM),
                    CubeMove::B1 => B_CORNER_PERM,
                    CubeMove::B2 => B_CORNER_PERM.then(B_CORNER_PERM),
                    CubeMove::B3 => B_CORNER_PERM.then(B_CORNER_PERM).then(B_CORNER_PERM),
                    CubeMove::R1 => R_CORNER_PERM,
                    CubeMove::R2 => R_CORNER_PERM.then(R_CORNER_PERM),
                    CubeMove::R3 => R_CORNER_PERM.then(R_CORNER_PERM).then(R_CORNER_PERM),
                    CubeMove::L1 => L_CORNER_PERM,
                    CubeMove::L2 => L_CORNER_PERM.then(L_CORNER_PERM),
                    CubeMove::L3 => L_CORNER_PERM.then(L_CORNER_PERM).then(L_CORNER_PERM),
                };
                i += 1;
            }

            val
        };
        TABLE[self.into_index()]
    }
}

impl From<DominoMove> for CubeMove {
    fn from(value: DominoMove) -> Self {
        match value {
            DominoMove::U1 => CubeMove::U1,
            DominoMove::U2 => CubeMove::U2,
            DominoMove::U3 => CubeMove::U3,
            DominoMove::D1 => CubeMove::D1,
            DominoMove::D2 => CubeMove::D2,
            DominoMove::D3 => CubeMove::D3,
            DominoMove::F2 => CubeMove::F2,
            DominoMove::B2 => CubeMove::B2,
            DominoMove::R2 => CubeMove::R2,
            DominoMove::L2 => CubeMove::L2,
        }
    }
}

impl TryFrom<CubeMove> for DominoMove {
    type Error = CubeMove;

    fn try_from(value: CubeMove) -> Result<Self, Self::Error> {
        match value {
            CubeMove::U1 => Ok(DominoMove::U1),
            CubeMove::U2 => Ok(DominoMove::U2),
            CubeMove::U3 => Ok(DominoMove::U3),
            CubeMove::D1 => Ok(DominoMove::D1),
            CubeMove::D2 => Ok(DominoMove::D2),
            CubeMove::D3 => Ok(DominoMove::D3),
            CubeMove::F2 => Ok(DominoMove::F2),
            CubeMove::B2 => Ok(DominoMove::B2),
            CubeMove::R2 => Ok(DominoMove::R2),
            CubeMove::L2 => Ok(DominoMove::L2),
            _ => Err(value),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(unused)]
pub enum DominoMove {
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

impl DominoMove {
    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0u8..10u8).map(|x| unsafe { core::mem::transmute(x) })
    }

    pub const fn into_u8(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }

    pub const fn into_index(self) -> usize {
        self.into_u8() as usize
    }

    pub const fn into_corner_perm(self) -> CornerPerm {
        const TABLE: [CornerPerm; 10] = const {
            let mut val = [CornerPerm::SOLVED; 10];
            let mut i = 0;
            while i < 10 {
                let mv: DominoMove = unsafe { core::mem::transmute(i as u8) };
                val[i] = match mv {
                    DominoMove::U1 => U_CORNER_PERM,
                    DominoMove::U2 => U_CORNER_PERM.then(U_CORNER_PERM),
                    DominoMove::U3 => U_CORNER_PERM.then(U_CORNER_PERM).then(U_CORNER_PERM),
                    DominoMove::D1 => D_CORNER_PERM,
                    DominoMove::D2 => D_CORNER_PERM.then(D_CORNER_PERM),
                    DominoMove::D3 => D_CORNER_PERM.then(D_CORNER_PERM).then(D_CORNER_PERM),
                    DominoMove::F2 => F_CORNER_PERM.then(F_CORNER_PERM),
                    DominoMove::B2 => B_CORNER_PERM.then(B_CORNER_PERM),
                    DominoMove::R2 => R_CORNER_PERM.then(R_CORNER_PERM),
                    DominoMove::L2 => L_CORNER_PERM.then(L_CORNER_PERM),
                };
                i += 1;
            }

            val
        };
        TABLE[self.into_index()]
    }
}

impl CornerPerm {
    pub const fn apply_cube_move(self, mv: CubeMove) -> Self {
        self.then(mv.into_corner_perm())
    }

    pub const fn apply_domino_move(self, mv: DominoMove) -> Self {
        self.then(mv.into_corner_perm())
    }
}

impl EdgePerm {
    pub const fn apply_cube_move(self, mv: CubeMove) -> Self {
        self.then(mv.into_edge_perm())
    }
}

impl CornerOrient {
    pub const fn apply_cube_move(self, mv: CubeMove) -> Self {
        self.permute(mv.into_corner_perm())
            .correct(mv.into_corner_orient())
    }
}

impl EdgeOrient {
    pub const fn apply_cube_move(self, mv: CubeMove) -> Self {
        self.permute(mv.into_edge_perm())
            .correct(mv.into_edge_orient())
    }
}

impl ReprCube {
    pub const fn apply_cube_move(self, mv: CubeMove) -> Self {
        Self {
            corner_perm: self.corner_perm.apply_cube_move(mv),
            corner_orient: self.corner_orient.apply_cube_move(mv),
            edge_perm: self.edge_perm.apply_cube_move(mv),
            edge_orient: self.edge_orient.apply_cube_move(mv),
        }
    }
}

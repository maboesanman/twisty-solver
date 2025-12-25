use crate::{CubeMove, cube_ops::{cube_move::DominoMove, cube_sym::DominoSymmetry}};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[allow(unused)]
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
}

impl CubePreviousAxis {
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

    pub fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
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
                    CubePreviousAxis::None => unreachable!(),
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

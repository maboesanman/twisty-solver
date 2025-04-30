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

    pub fn into_u8(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }

    pub fn into_index(self) -> usize {
        self.into_u8() as usize
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

    pub fn into_u8(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }

    pub fn into_index(self) -> usize {
        self.into_u8() as usize
    }
}

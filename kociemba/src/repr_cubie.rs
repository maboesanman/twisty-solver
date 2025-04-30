use std::fmt::Debug;

use rand::distr::{Distribution, StandardUniform};

use crate::{
    moves::{Move, Phase2Move},
    permutation_coord::{
        is_odd, is_perm, permutation_coord_8_inverse, permutation_coord_12_inverse,
    },
    symmetries::SubGroupTransform,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ReprCube {
    pub corner_perm: [u8; 8],
    pub edge_perm: [u8; 12],
    pub corner_orient: [u8; 8],
    pub edge_orient: [u8; 12],
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
        SOLVED_CUBE
        $(
            .then(cube!(@mv $mv))
        )+
    };

    // 2) “up to 2” and “up prime” on each face:
    (@mv U)  => { U1 };
    (@mv U2) => { U2 };
    (@mv Up) => { U3 };

    (@mv D)  => { D1 };
    (@mv D2) => { D2 };
    (@mv Dp) => { D3 };

    (@mv F)  => { F1 };
    (@mv F2) => { F2 };
    (@mv Fp) => { F3 };

    (@mv B)  => { B1 };
    (@mv B2) => { B2 };
    (@mv Bp) => { B3 };

    (@mv L)  => { L1 };
    (@mv L2) => { L2 };
    (@mv Lp) => { L3 };

    (@mv R)  => { R1 };
    (@mv R2) => { R2 };
    (@mv Rp) => { R3 };
}

pub const SOLVED_CUBE: ReprCube = ReprCube {
    corner_perm: [0, 1, 2, 3, 4, 5, 6, 7],
    edge_perm: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

const U1: ReprCube = ReprCube {
    corner_perm: [2, 0, 3, 1, 4, 5, 6, 7],
    edge_perm: [2, 3, 1, 0, 4, 5, 6, 7, 8, 9, 10, 11],
    corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

const U2: ReprCube = U1.then(U1);
const U3: ReprCube = U2.then(U1);

const D1: ReprCube = ReprCube {
    corner_perm: [0, 1, 2, 3, 5, 7, 4, 6],
    edge_perm: [0, 1, 2, 3, 7, 6, 4, 5, 8, 9, 10, 11],
    corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

const D2: ReprCube = D1.then(D1);
const D3: ReprCube = D2.then(D1);

const F1: ReprCube = ReprCube {
    corner_perm: [1, 5, 2, 3, 0, 4, 6, 7],
    edge_perm: [9, 1, 2, 3, 8, 5, 6, 7, 0, 4, 10, 11],
    corner_orient: [1, 2, 0, 0, 2, 1, 0, 0],
    edge_orient: [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0],
};

const F2: ReprCube = F1.then(F1);
const F3: ReprCube = F2.then(F1);

const B1: ReprCube = ReprCube {
    corner_perm: [0, 1, 6, 2, 4, 5, 7, 3],
    edge_perm: [0, 10, 2, 3, 4, 11, 6, 7, 8, 9, 5, 1],
    corner_orient: [0, 0, 2, 1, 0, 0, 1, 2],
    edge_orient: [0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1],
};

const B2: ReprCube = B1.then(B1);
const B3: ReprCube = B2.then(B1);

const R1: ReprCube = ReprCube {
    corner_perm: [4, 1, 0, 3, 6, 5, 2, 7],
    edge_perm: [0, 1, 8, 3, 4, 5, 10, 7, 6, 9, 2, 11],
    corner_orient: [2, 0, 1, 0, 1, 0, 2, 0],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

const R2: ReprCube = R1.then(R1);
const R3: ReprCube = R2.then(R1);

const L1: ReprCube = ReprCube {
    corner_perm: [0, 3, 2, 7, 4, 1, 6, 5],
    edge_perm: [0, 1, 2, 11, 4, 5, 6, 9, 8, 3, 10, 7],
    corner_orient: [0, 1, 0, 2, 0, 2, 0, 1],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

const L2: ReprCube = L1.then(L1);
const L3: ReprCube = L2.then(L1);

const S_URF3_1: ReprCube = cube![R Lp F Bp U Dp R Lp];
const S_URF3_2: ReprCube = S_URF3_1.then(S_URF3_1);
const S_F2: ReprCube = cube![R2 L2 F Bp U2 D2 F Bp];
const S_U4_1: ReprCube = ReprCube {
    corner_perm: [2, 0, 3, 1, 6, 4, 7, 5],
    edge_perm: [2, 3, 1, 0, 6, 7, 5, 4, 10, 8, 11, 9],
    corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
};

const S_U4_2: ReprCube = S_U4_1.then(S_U4_1);
const S_U4_3: ReprCube = S_U4_2.then(S_U4_1);

const S_LR2_PARTIAL: ReprCube = ReprCube {
    corner_perm: [1, 0, 3, 2, 5, 4, 7, 6],
    edge_perm: [0, 1, 3, 2, 4, 5, 7, 6, 9, 8, 11, 10],
    corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

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
    const fn apply_corner_permutation(self, permutation: [u8; 8]) -> Self {
        let mut new = Self {
            corner_perm: [0; 8],
            corner_orient: [0; 8],
            edge_orient: self.edge_orient,
            edge_perm: self.edge_perm,
        };
        let mut i = 0;
        while i < 8 {
            new.corner_perm[i] = self.corner_perm[permutation[i] as usize];
            new.corner_orient[i] = self.corner_orient[permutation[i] as usize];
            i += 1;
        }

        new
    }

    const fn apply_edge_permutation(self, permutation: [u8; 12]) -> Self {
        let mut new = Self {
            edge_perm: [0; 12],
            edge_orient: [0; 12],
            corner_orient: self.corner_orient,
            corner_perm: self.corner_perm,
        };
        let mut i = 0;
        while i < 12 {
            new.edge_perm[i] = self.edge_perm[permutation[i] as usize];
            new.edge_orient[i] = self.edge_orient[permutation[i] as usize];
            i += 1;
        }

        new
    }

    const fn apply_corner_orientation(mut self, orientation: [u8; 8]) -> Self {
        let mut i = 0;
        while i < 8 {
            self.corner_orient[i] = (self.corner_orient[i] + orientation[i]) % 3;
            i += 1;
        }

        self
    }

    const fn apply_edge_orientation(mut self, orientation: [u8; 12]) -> Self {
        let mut i = 0;
        while i < 12 {
            self.edge_orient[i] = (self.edge_orient[i] + orientation[i]) % 2;
            i += 1;
        }

        self
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

        if !is_perm(&self.edge_perm) {
            return false;
        }

        if !is_perm(&self.corner_perm) {
            return false;
        }

        if is_odd(&self.edge_perm) != is_odd(&self.corner_perm) {
            return false;
        }

        true
    }

    pub fn is_solved(self) -> bool {
        self == SOLVED_CUBE
    }

    /// concatenate two cubes, as transformations from the solved cube.
    pub const fn then(self, other: Self) -> Self {
        self.apply_corner_permutation(other.corner_perm)
            .apply_corner_orientation(other.corner_orient)
            .apply_edge_permutation(other.edge_perm)
            .apply_edge_orientation(other.edge_orient)
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
    let c = ReprCube::default();
    c.then(U1);
    c.then(U2);
    c.then(U3);
    c.then(U2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.then(D1);
    c.then(D2);
    c.then(D3);
    c.then(D2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.then(F1);
    c.then(F2);
    c.then(F3);
    c.then(F2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.then(B1);
    c.then(B2);
    c.then(B3);
    c.then(B2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.then(R1);
    c.then(R2);
    c.then(R3);
    c.then(R2);

    assert!(c.is_valid());
    assert!(c.is_solved());

    c.then(L1);
    c.then(L2);
    c.then(L3);
    c.then(L2);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_long_identity() {
    let c = ReprCube::default();
    c.then(F1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(L3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(R2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(L1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(B2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(B1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(U3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(B3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(D1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(U2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(F3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(R1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(R3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(U1);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(D3);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(L2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(D2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(F2);

    assert!(c.is_valid());
    assert!(!c.is_solved());
    c.then(D1);

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn sexy_move() {
    let c = ReprCube::default();

    for _ in 0..6 {
        c.then(U1);
        c.then(F1);
        c.then(U3);
        c.then(F3);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn hundred_thousand_moves_simd() {
    let c = ReprCube::default();

    for _ in 0..1000 {
        c.then(F1);
        c.then(R1);
        c.then(F3);
        c.then(U1);
        c.then(B2);
        c.then(L3);
        c.then(D3);
        c.then(R2);
        c.then(L1);
        c.then(B2);
        c.then(F3);
        c.then(D1);
        c.then(U2);
        c.then(R1);
        c.then(B1);
        c.then(U3);
        c.then(B3);
        c.then(D1);
        c.then(F3);
        c.then(U2);
        c.then(F3);
        c.then(R1);
        c.then(U1);
        c.then(R3);
        c.then(L2);
        c.then(U1);
        c.then(L2);
        c.then(D3);
        c.then(L2);
        c.then(D2);
        c.then(F2);
        c.then(D1);
        c.then(F1);
        c.then(R1);
        c.then(F3);
        c.then(U1);
        c.then(B2);
        c.then(L3);
        c.then(D3);
        c.then(R2);
        c.then(L2);
        c.then(L3);
        c.then(B2);
        c.then(F3);
        c.then(D1);
        c.then(U2);
        c.then(R1);
        c.then(B1);
        c.then(U3);
        c.then(B3);
        c.then(D1);
        c.then(F3);
        c.then(U1);
        c.then(U1);
        c.then(F3);
        c.then(R1);
        c.then(U1);
        c.then(R3);
        c.then(L2);
        c.then(U1);
        c.then(L2);
        c.then(D3);
        c.then(L2);
        c.then(D2);
        c.then(F2);
        c.then(D1);
        c.then(F1);
        c.then(R1);
        c.then(F3);
        c.then(U1);
        c.then(B2);
        c.then(L3);
        c.then(D3);
        c.then(R2);
        c.then(L1);
        c.then(B2);
        c.then(F2);
        c.then(F1);
        c.then(D1);
        c.then(U2);
        c.then(R1);
        c.then(B1);
        c.then(U3);
        c.then(B3);
        c.then(D1);
        c.then(F3);
        c.then(U2);
        c.then(F3);
        c.then(R1);
        c.then(U1);
        c.then(R3);
        c.then(L2);
        c.then(U1);
        c.then(L2);
        c.then(D3);
        c.then(L1);
        c.then(L1);
        c.then(D2);
        c.then(F2);
        c.then(D1);
    }

    assert!(c.is_valid());
    assert!(c.is_solved());
}

#[test]
fn test_apply() {
    let c = ReprCube::default();

    c.then(R1);
    c.then(U1);
    c.then(R3);
    c.then(U3);

    let mut c2 = ReprCube::default();

    for _ in 0..6 {
        c2 = c2.then(c);
    }

    assert!(c2.is_solved());
}

#[test]
fn test_2_move_apply() {
    let c = ReprCube::default();

    c.then(R1);
    c.then(U1);

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
    let c = ReprCube::default();

    c.then(R1);
    c.then(U2);
    c.then(D3);
    c.then(B1);
    c.then(D3);

    let mut c2 = ReprCube::default();
    c2 = c2.then(c);
    let mut count = 1;
    while !c2.is_solved() {
        count += 1;
        c2 = c2.then(c);
    }
    assert_eq!(count, 1260);
}

use crate::cube;

use super::{
    cube_move::{CubeMove, DominoMove},
    partial_reprs::{
        corner_orient::CornerOrient, corner_perm::CornerPerm, e_edge_perm::EEdgePerm,
        edge_group::EdgeGroup, edge_group_orient::EdgeGroupOrient, edge_orient::EdgeOrient,
        edge_perm::EdgePerm, ud_edge_perm::UDEdgePerm,
    },
    repr_cube::ReprCube,
};

pub const S_URF3_1_CORNER_PERM: CornerPerm = cube![R Lp F Bp U Dp R Lp].corner_perm;
pub const S_URF3_1_EDGE_PERM: EdgePerm = cube![R Lp F Bp U Dp R Lp].edge_perm;
pub const S_URF3_1_CORNER_ORIENT_CORRECT: CornerOrient = cube![R Lp F Bp U Dp R Lp].corner_orient;
pub const S_URF3_1_EDGE_ORIENT_CORRECT: EdgeOrient = cube![R Lp F Bp U Dp R Lp].edge_orient;

pub const S_URF3_1_CUBE: ReprCube = cube![R Lp F Bp U Dp R Lp];

pub const S_F2_CORNER_PERM: CornerPerm = cube![R2 L2 F Bp U2 D2 F Bp].corner_perm;
pub const S_F2_EDGE_PERM: EdgePerm = cube![R2 L2 F Bp U2 D2 F Bp].edge_perm;

pub const S_LR_CORNER_PERM: CornerPerm =
    cube![U Dp F2 U F2 L2 U2 B2 R2 F2 L2 U2 F2 L2 Dp R2].corner_perm;
pub const S_LR_EDGE_PERM: EdgePerm = cube![U2 F2 D2 U2 F2 U2 F2 B2].edge_perm;

pub const S_U4_1_CORNER_PERM: CornerPerm = cube![B2 F2 D2 L2 R2 D B2 F2 U2 L2 R2 Up].corner_perm;
pub const S_U4_1_EDGE_PERM: EdgePerm =
    cube![D B2 U B2 F2 R2 U2 L2 Dp R2 F2 Dp L D2 B2 Dp F L2 D F Up Rp].edge_perm;
pub const S_U4_1_EDGE_ORIENT_CORRECT: EdgeOrient =
    EdgeOrient::const_from_array([0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1]);

// transform is 0b_00_dd_c_bb_a
// S_LR^a ( S_U4^b ( S_F2^c ( S_URF3^d ) ) )
// (perform these right to left)
// conjugation is then

// M
// S_LR^-a -> S_U4^-b -> S_F2^-c -> S_URF3^-d -> M -> S_URF3^d -> S_F2^c -> S_U4^b -> S_LR^a

const CORNER_PERM_LOOKUP: [CornerPerm; 48] = {
    let mut table = [CornerPerm::SOLVED; 48];
    let mut i = 0usize;
    while i < 48 {
        let a = i & 0b000001;
        let b = (i & 0b000110) >> 1;
        let c = (i & 0b001000) >> 3;
        let d = (i & 0b110000) >> 4;

        let mut j = 0;
        while j < d {
            table[i] = table[i].then(S_URF3_1_CORNER_PERM);
            j += 1;
        }

        if c == 1 {
            table[i] = table[i].then(S_F2_CORNER_PERM);
        }

        let mut j = 0;
        while j < b {
            table[i] = table[i].then(S_U4_1_CORNER_PERM);
            j += 1;
        }

        if a == 1 {
            table[i] = table[i].then(S_LR_CORNER_PERM);
        }

        i += 1;
    }

    table
};

const EDGE_PERM_LOOKUP: [EdgePerm; 48] = {
    let mut table = [EdgePerm::SOLVED; 48];
    let mut i = 0usize;
    while i < 48 {
        let a = i & 0b000001;
        let b = (i & 0b000110) >> 1;
        let c = (i & 0b001000) >> 3;
        let d = (i & 0b110000) >> 4;

        let mut j = 0;
        while j < d {
            table[i] = table[i].then(S_URF3_1_EDGE_PERM);
            j += 1;
        }

        if c == 1 {
            table[i] = table[i].then(S_F2_EDGE_PERM);
        }

        let mut j = 0;
        while j < b {
            table[i] = table[i].then(S_U4_1_EDGE_PERM);
            j += 1;
        }

        if a == 1 {
            table[i] = table[i].then(S_LR_EDGE_PERM);
        }

        i += 1;
    }

    table
};

const EDGE_ORIENT_CORRECT: [EdgeOrient; 48] = {
    let mut table = [EdgeOrient::SOLVED; 48];
    let mut i = 0usize;
    while i < 48 {
        let a = i & 0b000001;
        let b = (i & 0b000110) >> 1;
        let c = (i & 0b001000) >> 3;
        let d = (i & 0b110000) >> 4;

        let mut j = 0;
        while j < d {
            table[i] = table[i].permute(S_URF3_1_EDGE_PERM);
            table[i] = table[i].correct(S_URF3_1_EDGE_ORIENT_CORRECT);
            j += 1;
        }

        if c == 1 {
            table[i] = table[i].permute(S_F2_EDGE_PERM);
        }

        let mut j = 0;
        while j < b {
            table[i] = table[i].permute(S_U4_1_EDGE_PERM);
            table[i] = table[i].correct(S_U4_1_EDGE_ORIENT_CORRECT);
            j += 1;
        }

        if a == 1 {
            table[i] = table[i].permute(S_LR_EDGE_PERM);
        }

        i += 1;
    }

    table
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CubeSymmetry(pub u8);

/// only the 4 low bits of the transform described above. we omit the S_URF3 element and its consequent symmetries
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct DominoSymmetry(pub u8);

impl DominoSymmetry {
    pub const fn to_cube_symmetry(self) -> CubeSymmetry {
        CubeSymmetry(self.0)
    }

    pub fn into_index(self) -> usize {
        self.0 as usize
    }
}

impl From<DominoSymmetry> for CubeSymmetry {
    fn from(value: DominoSymmetry) -> Self {
        CubeSymmetry(value.0)
    }
}

impl TryFrom<CubeSymmetry> for DominoSymmetry {
    type Error = CubeSymmetry;

    fn try_from(value: CubeSymmetry) -> Result<Self, Self::Error> {
        if value.0 < 16 {
            Ok(DominoSymmetry(value.0))
        } else {
            Err(value)
        }
    }
}

impl CubeSymmetry {
    pub const fn then(self, _other: Self) -> Self {
        unimplemented!()
    }

    pub const fn inverse(self) -> Self {
        unimplemented!()
    }

    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0..48).map(Self)
    }

    pub fn nontrivial_iter() -> impl Iterator<Item = Self> {
        (1..48).map(Self)
    }
}

impl DominoSymmetry {
    pub const fn then(self, other: Self) -> Self {
        const TRANSFORM_COMPOSE_LOOKUP: [DominoSymmetry; 256] = {
            let reference_cube = cube![R U D2];

            let mut output = [DominoSymmetry(0); 256];

            let mut t1 = 0u8;
            while t1 < 16 {
                let conj_1 = reference_cube
                    .corner_perm
                    .domino_conjugate(DominoSymmetry(t1));
                let mut t2 = 0u8;
                while t2 < 16 {
                    let conj_2 = conj_1.domino_conjugate(DominoSymmetry(t2));
                    let i = ((t1 << 4) + t2) as usize;
                    let mut t3 = 0;
                    'deep: while t3 < 16 {
                        let conj_3 = reference_cube
                            .corner_perm
                            .domino_conjugate(DominoSymmetry(t3));
                        if conj_2.const_eq(conj_3) {
                            output[i] = DominoSymmetry(t3);
                            break 'deep;
                        }
                        t3 += 1;
                    }
                    t2 += 1;
                }
                t1 += 1;
            }

            output
        };

        TRANSFORM_COMPOSE_LOOKUP[((self.0 << 4) + other.0) as usize]
    }

    pub const fn inverse(self) -> Self {
        const TRANSFORM_INVERT_LOOKUP: [DominoSymmetry; 16] = {
            let reference_cube = cube![R U D2];

            let mut output = [DominoSymmetry(0); 16];

            let mut t1 = 0u8;
            while t1 < 16 {
                let conj_1 = reference_cube
                    .corner_perm
                    .domino_conjugate(DominoSymmetry(t1));
                let mut t2 = 0u8;
                while t2 < 16 {
                    let conj_2 = conj_1.domino_conjugate(DominoSymmetry(t2));
                    if conj_2.const_eq(reference_cube.corner_perm) {
                        output[t1 as usize] = DominoSymmetry(t2);
                    }
                    t2 += 1;
                }
                t1 += 1;
            }

            output
        };

        TRANSFORM_INVERT_LOOKUP[self.0 as usize]
    }

    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0..16).map(Self)
    }

    pub fn nontrivial_iter() -> impl Iterator<Item = Self> {
        (1..16).map(Self)
    }
}

impl CornerPerm {
    pub const fn conjugate(self, sym: CubeSymmetry) -> Self {
        let perm = CORNER_PERM_LOOKUP[sym.0 as usize];
        let inv_perm = perm.inverse();
        inv_perm.then(self).then(perm)
    }

    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        self.conjugate(sym.to_cube_symmetry())
    }
}

impl EdgePerm {
    pub const fn conjugate(self, sym: CubeSymmetry) -> Self {
        let perm = EDGE_PERM_LOOKUP[sym.0 as usize];
        let inv_perm = perm.inverse();
        inv_perm.then(self).then(perm)
    }

    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        self.conjugate(sym.to_cube_symmetry())
    }
}

impl CornerOrient {
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = CORNER_PERM_LOOKUP[sym.0 as usize];
        let permuted = self.permute(perm);
        if sym.0 % 2 == 1 {
            permuted.mirror()
        } else {
            permuted
        }
    }
}

impl EdgeGroupOrient {
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = EDGE_PERM_LOOKUP[sym.0 as usize];
        let new_group = self.0.permute(perm);
        let mut orient = self.1.permute(perm);

        if ((sym.0 >> 1) & 1) == 1 {
            let mut i = 0;
            while i < 12 {
                if new_group.0.0[i] {
                    orient.0[i] ^= 1;
                }
                i += 1;
            }
            orient = orient.correct(S_U4_1_EDGE_ORIENT_CORRECT);
        }

        EdgeGroupOrient(new_group, orient)
    }
}

impl EdgeGroup {
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        self.permute(EDGE_PERM_LOOKUP[sym.0 as usize])
    }
}

impl ReprCube {
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        Self {
            corner_perm: self.corner_perm.domino_conjugate(sym),
            corner_orient: self.corner_orient.domino_conjugate(sym),
            edge_perm: self.edge_perm.domino_conjugate(sym),
            edge_orient: EdgeGroupOrient(self.edge_perm.into_grouping(), self.edge_orient)
                .domino_conjugate(sym)
                .1,
        }
    }
}

impl UDEdgePerm {
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = unsafe { EDGE_PERM_LOOKUP[sym.0 as usize].into_ud_perm_unchecked() };
        let inv_perm = perm.inverse();
        inv_perm.then(self).then(perm)
    }
}

impl EEdgePerm {
    pub const fn domino_conjugate(self, sym: DominoSymmetry) -> Self {
        let perm = unsafe { EDGE_PERM_LOOKUP[sym.0 as usize].into_e_perm_unchecked() };
        let inv_perm = perm.inverse();
        inv_perm.then(self).then(perm)
    }
}

impl CubeMove {
    pub const fn conjugate(self, _sym: CubeSymmetry) -> Self {
        unimplemented!()
    }

    pub const fn domino_conjugate(self, _sym: DominoSymmetry) -> Self {
        todo!()
    }
}

impl DominoMove {
    pub const fn domino_conjugate(self, _sym: DominoSymmetry) -> Self {
        todo!()
    }
}

#[test]
fn print_all_domino_conjugations() {
    println!("{EDGE_ORIENT_CORRECT:?}");
    let cube = cube![R U];

    for sym in DominoSymmetry::all_iter() {
        cube.domino_conjugate(sym).pretty_print();
    }
}

#[test]
fn check_no_duplicates_for_corner_perm_repr() {
    let cube = cube![R U D2];

    let mut set = std::collections::HashSet::new();

    for sym in CubeSymmetry::all_iter() {
        assert!(set.insert(cube.corner_perm.conjugate(sym)))
    }
}

#[test]
fn random_cubes_with_conjugation() {
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for _ in 0..100 {
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
        for a in DominoSymmetry::all_iter() {
            for b in DominoSymmetry::all_iter() {
                assert_eq!(
                    cube.domino_conjugate(a).domino_conjugate(b),
                    cube.domino_conjugate(a.then(b))
                )
            }
        }
    }
}

#[test]
fn test_long_apply() {
    let c = cube![R U2 Dp B Dp];

    let mut c2 = ReprCube::SOLVED;
    c2 = c2.then(c);
    let mut count = 1;
    while c2 != ReprCube::SOLVED {
        count += 1;
        c2 = c2.then(c);
    }
    assert_eq!(count, 1260);
}

#[test]
fn basic_sequence() {
    let cube = cube![R U D2 U L F B L];

    for sym in DominoSymmetry::all_iter() {
        cube.domino_conjugate(sym).pretty_print();
        println!()
    }
}
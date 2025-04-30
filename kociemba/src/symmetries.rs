// 0b_xx00_0_00_0

use crate::{
    moves::{combined_index, combined_orient},
    repr_cubie::{CornerOrient, ReprCubie},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Transform(pub u8);

impl From<SubGroupTransform> for Transform {
    fn from(value: SubGroupTransform) -> Self {
        Self(value.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SubGroupTransform(pub u8);

impl SubGroupTransform {
    pub fn nontrivial_iter() -> impl Iterator<Item = Self> {
        (1..16).map(SubGroupTransform)
    }

    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0..16).map(SubGroupTransform)
    }
}

// THIS IS THE MANUAL PERMUTATION DATA FOR THE GENERATIVE ELEMENTS OF THE GROUP

const IDENTITY_CORNER_ROT: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

// corner permutations
const S_URF3_CORNER_INDEX: [u8; 8] = [0, 4, 1, 5, 2, 6, 3, 7];
const S_F2_CORNER_INDEX: [u8; 8] = [5, 4, 7, 6, 1, 0, 3, 2];
const S_U4_CORNER_INDEX: [u8; 8] = [2, 0, 3, 1, 6, 4, 7, 5];
const S_LR2_CORNER_INDEX: [u8; 8] = [1, 0, 3, 2, 5, 4, 7, 6];

// corner orientation corrections (added after permuting)
const S_URF3_CORNER_ROT: [u8; 8] = [1, 2, 2, 1, 2, 1, 1, 2];

// edge permutations
const S_URF3_EDGE_INDEX: [u8; 12] = [8, 9, 0, 4, 10, 11, 1, 5, 2, 6, 3, 7];
const S_F2_EDGE_INDEX: [u8; 12] = [4, 5, 7, 6, 0, 1, 3, 2, 9, 8, 11, 10];
const S_U4_EDGE_INDEX: [u8; 12] = [2, 3, 1, 0, 6, 7, 5, 4, 10, 8, 11, 9];
const S_LR2_EDGE_INDEX: [u8; 12] = [0, 1, 3, 2, 4, 5, 7, 6, 9, 8, 11, 10];

// edge orientation corrections (added after permuting)
const S_URF3_EDGE_FLIP: [u8; 12] = [0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1];
const S_U4_EDGE_FLIP: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1];

const S_URF3_INDEX: [usize; 40] = combined_index(&S_URF3_CORNER_INDEX, &S_URF3_EDGE_INDEX);
const S_F2_INDEX: [usize; 40] = combined_index(&S_F2_CORNER_INDEX, &S_F2_EDGE_INDEX);
const S_U4_INDEX: [usize; 40] = combined_index(&S_U4_CORNER_INDEX, &S_U4_EDGE_INDEX);
const S_LR2_INDEX: [usize; 40] = combined_index(&S_LR2_CORNER_INDEX, &S_LR2_EDGE_INDEX);

const S_URF3_ORIENT: [u8; 20] = combined_orient(&S_URF3_CORNER_ROT, &S_URF3_EDGE_FLIP);
const S_U4_ORIENT: [u8; 20] = combined_orient(&IDENTITY_CORNER_ROT, &S_U4_EDGE_FLIP);

impl ReprCubie {
    const fn mirror_corner_orientations(mut self) -> Self {
        let mut i = 0;
        while i < 8 {
            match self.corner_orient[i] {
                CornerOrient::Solved => {}
                CornerOrient::Clockwise => self.corner_orient[i] = CornerOrient::CounterClockwise,
                CornerOrient::CounterClockwise => self.corner_orient[i] = CornerOrient::Clockwise,
            }
            i += 1;
        }

        self
    }

    pub const fn conjugate_by_subgroup_transform(self, transform: SubGroupTransform) -> Self {
        let mut base = ReprCubie::new();

        let s_lr2 = transform.0 & 0b0001;
        let s_u4 = (transform.0 & 0b0110) >> 1;
        let s_f2 = (transform.0 & 0b1000) >> 3;

        if s_lr2 == 1 {
            base = base.apply_const_no_orient(S_LR2_INDEX);
        }

        let mut i = 0;
        while i < s_u4 {
            base = base.apply_const(S_U4_INDEX, &S_U4_ORIENT);
            i += 1;
        }

        if s_f2 == 1 {
            base = base.apply_const_no_orient(S_F2_INDEX);
        }

        let self_index = self.get_index();
        let self_orient = self.get_orient();

        base = base.apply_const(self_index, self_orient);

        if s_f2 == 1 {
            base = base.apply_const_no_orient(S_F2_INDEX);
        }

        while i < 4 && s_u4 != 0 {
            base = base.apply_const(S_U4_INDEX, &S_U4_ORIENT);
            i += 1;
        }

        if s_lr2 == 1 {
            base = base.apply_const_no_orient(S_LR2_INDEX);
            base = base.mirror_corner_orientations();
        }

        base
    }

    pub const fn conjugate_by_transform(self, transform: Transform) -> Self {
        let mut base = ReprCubie::new();

        let s_lr2 = transform.0 & 0b0000_0001;
        let s_u4 = (transform.0 & 0b0000_0110) >> 1;
        let s_f2 = (transform.0 & 0b0000_1000) >> 3;
        let s_urf3 = transform.0 >> 4;

        if s_lr2 == 1 {
            base = base.apply_const_no_orient(S_LR2_INDEX);
        }

        let mut i = 0;
        while i < s_u4 {
            base = base.apply_const(S_U4_INDEX, &S_U4_ORIENT);
            i += 1;
        }

        if s_f2 == 1 {
            base = base.apply_const_no_orient(S_F2_INDEX);
        }

        let mut j = 0;
        while j < s_urf3 {
            base = base.apply_const(S_URF3_INDEX, &S_URF3_ORIENT);
            j += 1;
        }

        let self_index = self.get_index();
        let self_orient = self.get_orient();

        base = base.apply_const(self_index, self_orient);

        while j < 3 && s_urf3 != 0 {
            base = base.apply_const(S_URF3_INDEX, &S_URF3_ORIENT);
            j += 1;
        }

        if s_f2 == 1 {
            base = base.apply_const_no_orient(S_F2_INDEX);
        }

        while i < 4 && s_u4 != 0 {
            base = base.apply_const(S_U4_INDEX, &S_U4_ORIENT);
            i += 1;
        }

        if s_lr2 == 1 {
            base = base.apply_const_no_orient(S_LR2_INDEX);
            base = base.mirror_corner_orientations();
        }

        base
    }

    // returns all cubes in the equivalence class, not deduped and including self.
    pub const fn get_subgroup_equivalence_class(self) -> [ReprCubie; 16] {
        let mut working = [self; 16];

        let mut i = 1;
        while i < 16 {
            working[i] = working[i].conjugate_by_subgroup_transform(SubGroupTransform(i as u8));
            i += 1;
        }

        working
    }

    // returns all cubes in the equivalence class, not deduped and including self.
    pub const fn get_full_group_equivalence_class(self) -> [ReprCubie; 48] {
        let mut working = [self; 48];

        let mut i = 1;
        while i < 48 {
            working[i] = working[i].conjugate_by_transform(Transform(i as u8));
            i += 1;
        }

        working
    }
}

#[test]
fn check_symmetries() {
    use crate::moves::Move;
    use std::collections::HashMap;

    let mut move_lookup = HashMap::new();
    for m in Move::all_iter() {
        let val = format!("{m}");
        let key = ReprCubie::default().const_move(m);
        move_lookup.insert(key, val);
    }

    let c = ReprCubie::default().const_move(Move::U1);

    for i in 0..48 {
        let key = c.conjugate_by_transform(Transform(i));

        println!("{:?}", move_lookup.get(&key));
    }

    println!();

    for i in 0..16 {
        let key = c.conjugate_by_subgroup_transform(SubGroupTransform(i));

        println!("{:?}", move_lookup.get(&key));
    }

    println!();

    let c = ReprCubie::default().const_move(Move::F1);

    for i in 0..48 {
        let key = c.conjugate_by_transform(Transform(i));

        println!("{:?}", move_lookup.get(&key));
    }

    println!();

    for i in 0..16 {
        let key = c.conjugate_by_subgroup_transform(SubGroupTransform(i));

        println!("{:?}", move_lookup.get(&key));
    }

    // c.conjugate_by_transform(transform)
}

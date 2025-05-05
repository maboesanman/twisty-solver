// 0b_xx00_0_00_0

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct Transform(pub u8);

impl From<SubGroupTransform> for Transform {
    fn from(value: SubGroupTransform) -> Self {
        Self(value.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct SubGroupTransform(pub u8);

impl SubGroupTransform {
    pub fn nontrivial_iter() -> impl Iterator<Item = Self> {
        (1..16).map(SubGroupTransform)
    }

    pub fn all_iter() -> impl Iterator<Item = Self> {
        (0..16).map(SubGroupTransform)
    }

    pub fn then(self, other: Self) -> Self {
        let i = (self.0 << 4) + other.0;
        Self(TRANSFORM_COMPOSE_LOOKUP[i as usize])
    }

    pub fn inverse(self) -> Self {
        Self(TRANSFORM_INVERT_LOOKUP[self.0 as usize])
    }
}

const TRANSFORM_COMPOSE_LOOKUP: [u8; 256] = {
    let reference_cube = cube![U R];

    let mut output = [0; 256];

    let mut t1 = 0u8;
    while t1 < 16 {
        let conj_1 = reference_cube.conjugate_by_subgroup_transform(SubGroupTransform(t1));
        let mut t2 = 0u8;
        while t2 < 16 {
            let conj_2 = conj_1.conjugate_by_subgroup_transform(SubGroupTransform(t2));
            let i = ((t1 << 4) + t2) as usize;
            let mut t3 = 0;
            while t3 < 16 {
                let conj_3= reference_cube.conjugate_by_subgroup_transform(SubGroupTransform(t3));
                if conj_2.const_eq(conj_3) {
                    output[i] = t3;
                }
                t3 += 1;
            }
            t2 += 1;
        }
        t1 += 1;
    }

    output
};

const TRANSFORM_INVERT_LOOKUP: [u8; 16] = {
    let reference_cube = cube![U R];

    let mut output = [0; 16];

    let mut t1 = 0u8;
    while t1 < 16 {
        let conj_1 = reference_cube.conjugate_by_subgroup_transform(SubGroupTransform(t1));
        let mut t2 = 0u8;
        while t2 < 16 {
            let conj_2 = conj_1.conjugate_by_subgroup_transform(SubGroupTransform(t2));
            if conj_2.const_eq(reference_cube) {
                output[t1 as usize] = t2;
            }
            t2 += 1;
        }
        t1 += 1;
    }

    output
};
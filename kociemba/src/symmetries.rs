// 0b_xx00_0_00_0

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

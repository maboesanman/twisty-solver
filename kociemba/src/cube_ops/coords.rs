/// The raw coordinate for edge grouping.
/// tracks the specific way the E Edges (FR, FL, BR, BL) are distributed around the cube.
/// fits in 9 bits. (495 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgeGroupRawCoord(pub u16);

/// The raw coordinate for edge orientation.
/// fits in 11 bits. (2048 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgeOrientRawCoord(pub u16);

/// The raw coordinate for edge orientation.
/// fits in 12 bits. (2187 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CornerOrientRawCoord(pub u16);

/// The raw coordinate for corner permutation.
/// tracks the permutation of the corners.
/// fits in 16 bits. (40320 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CornerPermRawCoord(pub u16);

/// The raw coordinate for ud-edge permutation, assuming the EdgeGroupRawCoord is 0.
/// tracks the permutation of the edges in the U and D layers amongst themselves.
/// fits in 16 bits. (40320 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct UDEdgePermRawCoord(pub u16);

/// The raw coordinate for e-edge permutation, assuming the EdgeGroupRawCoord is 0.
/// tracks the permutation of the edges in the E layer amongst themselves.
/// fits in 5 bits. (24 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EEdgePermRawCoord(pub u8);

/// The sym coordinate for edge grouping and orientation, reduced by domino symmetries
/// fits in 16 bits. (64430 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgeGroupOrientSymCoord(pub u16);

/// The sym coordinate for corner permutation, reduced by domino symmetries
/// fits in 12 bits. (2768 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CornerPermSymCoord(pub u16);

/// The raw coordinate for edge grouping.
/// tracks the specific way the E Edges (FR, FL, BR, BL) are distributed around the cube.
/// fits in 9 bits. (495 values)
///
/// parity of representative permutation is always even
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
///
/// parity of coord matches parity of permutation
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CornerPermRawCoord(pub u16);

/// The raw coordinate for ud-edge permutation, assuming the EdgeGroupRawCoord is 0.
/// tracks the permutation of the edges in the U and D layers amongst themselves.
/// 
/// This is different from normal permutation coordinates, in order to avoid an additional lookup.
/// It is a combination of the (d_group - 425) value, the d_perm value, and the u_perm value.
/// Because of this it does not have the parity preservation property.
/// 
/// fits in 16 bits. (40320 values)
///
/// parity of coord matches parity of permutation
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct UDEdgePermRawCoord(pub u16);

/// The raw coordinate for e-edge permutation, assuming the EdgeGroupRawCoord is 0.
/// tracks the permutation of the edges in the E layer amongst themselves.
/// fits in 5 bits. (24 values)
///
/// parity of coord matches parity of permutation
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EEdgePermRawCoord(pub u8);

// fits in 20 bits
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EdgeGroupOrientRawCoord(pub u32);

/// The sym coordinate for edge grouping and orientation, reduced by domino symmetries
/// fits in 16 bits. (64430 values)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct EdgeGroupOrientSymCoord(pub u16);

/// The sym coordinate for corner permutation, reduced by domino symmetries
/// fits in 12 bits. (2768 values)
///
/// parity of coord matches parity of permutation
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct CornerPermSymCoord(pub u16);

impl EdgeGroupOrientRawCoord {
    pub fn split(self) -> (EdgeGroupRawCoord, EdgeOrientRawCoord) {
        (
            EdgeGroupRawCoord((self.0 >> 11) as u16),
            EdgeOrientRawCoord((self.0 & 0b11111111111) as u16),
        )
    }

    pub fn join(group: EdgeGroupRawCoord, orient: EdgeOrientRawCoord) -> Self {
        Self(((group.0 as u32) << 11) | (orient.0 as u32))
    }
}

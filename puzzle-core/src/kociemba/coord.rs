use super::{
    cubie_repr::CubieRepr,
    permutation_coord::{permutation_coord_4, permutation_coord_8},
};

// 2187 (11.09 bits)
#[repr(transparent)]
pub struct CornerOrientCoord(u16);

// 2048 (11 bits)
#[repr(transparent)]
pub struct EdgeOrientCoord(u16);

// 40320 (15.29 bits)
#[repr(transparent)]
pub struct CornerPermutationCoord(u16);

// 495 (8.9 bits)
#[repr(transparent)]
pub struct EdgeGroupingCoord(u16);

// 40320 (15.29 bits)
#[repr(transparent)]
pub struct UDEdgePermutationCoord(u16);

// 24 (4.58 bits)
#[repr(transparent)]
pub struct EEdgePermutationCoord(u8);

#[allow(dead_code)]
impl CubieRepr {
    // phase 1
    pub fn coord_corner_orient(&self) -> CornerOrientCoord {
        let mut sum = 0u16;
        for i in (0..7).rev() {
            sum *= 3;
            sum += self.corner_orient[i] as u16;
        }

        CornerOrientCoord(sum)
    }

    // phase 1
    pub fn coord_edge_orient(&self) -> EdgeOrientCoord {
        let mut sum = 0u16;
        for i in (0..11).rev() {
            sum <<= 1;
            sum += self.edge_orient[i] as u16;
        }

        EdgeOrientCoord(sum)
    }

    // phase 1
    pub fn coord_edge_grouping(&self) -> EdgeGroupingCoord {
        let factorials: [u32; 12] = [
            1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
        ];
        let mut sum = 0;
        let mut k = 3;
        for n in (0..12).rev() {
            if (self.edge_perm[n] as u8) < 8 {
                sum += (factorials[n] / factorials[k] / factorials[n - k]) as u16
            } else if k == 0 {
                break;
            } else {
                k -= 1;
            }
        }

        EdgeGroupingCoord(sum)
    }

    // phase 2
    pub fn coord_corner_perm(&self) -> CornerPermutationCoord {
        CornerPermutationCoord(permutation_coord_8(&self.corner_perm))
    }

    // phase 2
    pub fn coord_ud_edge_perm(&self) -> UDEdgePermutationCoord {
        UDEdgePermutationCoord(permutation_coord_8(self.edge_perm[..8].try_into().unwrap()))
    }

    // phase 2
    pub fn coord_e_edge_perm(&self) -> EEdgePermutationCoord {
        EEdgePermutationCoord(permutation_coord_4(self.edge_perm[8..].try_into().unwrap()))
    }
}

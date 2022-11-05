

pub struct CubieRepr {
    // see corner resident enum for ordering
    corner_perm: [CornerResident; 8],
    corner_orient: [CornerOrient; 8],
    // see edge resident enum for ordering
    edge_perm: [EdgeResident; 12],
    edge_orient: [EdgeOrient; 12],
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum CornerResident {
    UFR = 0,
    UFL = 1,
    UBR = 2,
    UBL = 3,
    DFR = 4,
    DFL = 5,
    DBR = 6,
    DBL = 7,
}

#[repr(u8)]
enum CornerOrient {
    Solved = 0,
    Clockwise,
    CounterClockwise,
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum EdgeResident {
    UF = 0,
    UB = 1,
    UR = 2,
    UL = 3,
    DF = 4,
    DB = 5,
    DR = 6,
    DL = 7,
    FR = 8,
    FL = 9,
    BR = 10,
    BL = 11,
}

#[repr(u8)]
enum EdgeOrient {
    Solved = 0,
    Unsolved = 1,
}

#[repr(u8)]
enum Phase1Move {
    U,
    U2,
    U3,
    D,
    D2,
    D3,
    F,
    F2,
    F3,
    B,
    B2,
    B3,
    R,
    R2,
    R3,
    L,
    L2,
    L3,
}

enum Phase2Move {
    U,
    U2,
    U3,
    D,
    D2,
    D3,
    F2,
    B2,
    R2,
    L2,
}

// 2187 (11.09 bits)
#[repr(transparent)]
pub struct CornerOrientCoord(u16);

// 2048 (11 bits)
#[repr(transparent)]
pub struct EdgeOrientCoord(u16);

// 40320 (15.29 bits)
#[repr(transparent)]
pub struct CornerPermutationCoord(u16);

// // 39916800 (25.25 bits)
// type EdgePermutationCoord = u32;

// 495 (8.9 bits)
#[repr(transparent)]
pub struct EdgeGroupingCoord(u16);

// 40320 (15.29 bits)
#[repr(transparent)]
pub struct UDEdgePermutationCoord(u16);

// 24 (4.58 bits)
#[repr(transparent)]
pub struct EEdgePermutationCoord(u8);

// fn permutation_coord_12<T: Ord>(perm: &[T; 12]) -> u32 {
// 	let mut sum = 0;
// 	let factorials: [u32; 11] = [1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];
// 	for i in 1..12 {
// 		for j in 0..i {
// 			if perm[j] > perm[i] {
// 				sum += factorials[i - 1]
// 			}
// 		}
// 	}
// 	sum
// }

fn permutation_coord_8<T: Ord>(perm: &[T; 8]) -> u16 {
	let mut sum = 0;
	let factorials: [u16; 7] = [1, 2, 6, 24, 120, 720, 5040];
	for i in 1..8 {
		for j in 0..i {
			if perm[j] > perm[i] {
				sum += factorials[i - 1]
			}
		}
	}
	sum
}

fn permutation_coord_4<T: Ord>(perm: &[T; 4]) -> u8 {
	let mut sum = 0;
	let factorials: [u8; 3] = [1, 2, 6];
	for i in 1..4 {
		for j in 0..i {
			if perm[j] > perm[i] {
				sum += factorials[i - 1]
			}
		}
	}
	sum
}

impl Default for CubieRepr {
    fn default() -> Self {
        Self {
            corner_perm: [
                CornerResident::UFR,
                CornerResident::UFL,
                CornerResident::UBR,
                CornerResident::UBL,
                CornerResident::DFR,
                CornerResident::DFL,
                CornerResident::DBR,
                CornerResident::DBL,
            ],
            corner_orient: [
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
            ],
            edge_perm: [
                EdgeResident::UF,
                EdgeResident::UB,
                EdgeResident::UR,
                EdgeResident::UL,
                EdgeResident::DF,
                EdgeResident::DB,
                EdgeResident::DR,
                EdgeResident::DL,
                EdgeResident::FR,
                EdgeResident::FL,
                EdgeResident::BR,
                EdgeResident::BL,
            ],
            edge_orient: [
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
            ],
        }
    }
}

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
        let factorials: [u32; 12] = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];
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

#[test]
fn coord_test() {
    let c = CubieRepr::default();

    // phase 1
    assert_eq!(c.coord_corner_orient().0, 0);
    assert_eq!(c.coord_edge_orient().0, 0);
    assert_eq!(c.coord_edge_grouping().0, 0);

    // phase 2
    assert_eq!(c.coord_corner_perm().0, 0);
    assert_eq!(c.coord_ud_edge_perm().0, 0);
    assert_eq!(c.coord_e_edge_perm().0, 0);
}

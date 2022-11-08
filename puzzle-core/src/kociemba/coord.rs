use super::{
    cubie_repr::{CornerResident, CubieRepr},
    permutation_coord::{
        permutation_coord_4, permutation_coord_4_inverse, permutation_coord_8,
        permutation_coord_8_inverse,
    },
};

// 2187 (11.09 bits)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CornerOrientCoord(pub u16);

// 2048 (11 bits)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EdgeOrientCoord(pub u16);

// 495 (8.9 bits)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EdgeGroupingCoord(pub u16);

// 40320 (15.29 bits)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CornerPermutationCoord(pub u16);

// 40320 (15.29 bits)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UDEdgePermutationCoord(pub u16);

// 24 (4.58 bits)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EEdgePermutationCoord(pub u8);

const COMBINATIONS: [[u16; 4]; 12] = {
    const FACTORIALS: [u32; 12] = [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
    ];

    let mut buf = [[0u16; 4]; 12];

    let mut i = 0;
    while i < 12 {
        let mut j = 0;
        while j < 4 && j <= i {
            buf[i][j] = (FACTORIALS[i] / FACTORIALS[j] / FACTORIALS[i - j]) as u16;
            j += 1;
        }
        i += 1;
    }

    buf
};

const fn edge_grouping(items: &[bool; 12]) -> u16 {
    let mut sum = 0;
    let mut k = 3;
    let mut n = 11;

    loop {
        if !items[n] {
            sum += COMBINATIONS[n][k] as u16
        } else if k == 0 {
            break;
        } else {
            k -= 1;
        }

        if n == 0 {
            break;
        }
        n -= 1;
    }

    sum
}

const fn edge_grouping_inverse(mut coord: u16) -> [bool; 12] {
    let mut buf = [false; 12];
    let mut k = 11;
    let mut i = 3;
    loop {
        let c = COMBINATIONS[k][i] as u16;
        if coord >= c {
            coord -= c;
            k -= 1;
        } else {
            buf[k] = true;
            if i == 0 {
                break;
            }
            i -= 1;
            k -= 1;
        }
    }

    buf
}

#[test]
fn edge_grouping_test() {
    // the domain is small enough so we just check the whole thing.
    for i in 0..495 {
        let s = edge_grouping_inverse(i);
        assert_eq!(edge_grouping(&s), i);
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
        EdgeGroupingCoord(edge_grouping(&self.edge_perm.map(|i| i as u8 >= 8)))
    }

    // phase 2
    pub fn coord_corner_perm(&self) -> CornerPermutationCoord {
        CornerPermutationCoord(permutation_coord_8(unsafe { core::mem::transmute(&self.corner_perm) } ))
    }

    // phase 2
    pub fn coord_ud_edge_perm(&self) -> UDEdgePermutationCoord {
        let mut ud_edges = [0u8; 8];
        let mut j = 0;
        for i in 0..12 {
            if (self.edge_perm[i] as u8) < 8 {
                ud_edges[j] = self.edge_perm[i] as u8;
                j += 1;
            }
        }
        UDEdgePermutationCoord(permutation_coord_8(&ud_edges))
    }

    // phase 2
    pub fn coord_e_edge_perm(&self) -> EEdgePermutationCoord {
        let mut e_edges = [0u8; 4];
        let mut j = 0;
        for i in 0..12 {
            if (self.edge_perm[i] as u8) >= 8 {
                e_edges[j] = self.edge_perm[i] as u8;
                j += 1;
            }
        }
        EEdgePermutationCoord(permutation_coord_4(&e_edges))
    }

    pub fn from_coords(
        mut corner_orient: CornerOrientCoord,
        mut edge_orient: EdgeOrientCoord,
        edge_group: EdgeGroupingCoord,
        corner_perm: CornerPermutationCoord,
        ud_edge_perm: UDEdgePermutationCoord,
        e_edge_perm: EEdgePermutationCoord,
    ) -> Self {
        let edge_group = edge_grouping_inverse(edge_group.0);
        let corner_perm = permutation_coord_8_inverse(corner_perm.0);
        let ud_edge_perm = permutation_coord_8_inverse(ud_edge_perm.0);
        let e_edge_perm = permutation_coord_4_inverse(e_edge_perm.0);

        let mut edge_perm = [0u8; 12];

        let mut i = 0;

        let mut n = 0;
        while n < edge_group.len() {
            if edge_group[n] {
                edge_perm[n] = e_edge_perm[n - i] + 8;
            } else {
                edge_perm[n] = ud_edge_perm[i];
                i += 1;
            }

            n += 1;
        }

        let mut corner_orient_buf = [0u8; 8];

        let mut i = 0;
        while i < 7 {
            let r = corner_orient.0 % 3;
            corner_orient_buf[i] = r as u8;
            corner_orient_buf[7] += 3 - corner_orient_buf[i];
            corner_orient.0 /= 3;

            i += 1;
        }
        corner_orient_buf[7] %= 3;

        let mut edge_orient_buf = [0u8; 12];
        let mut i = 0;
        while i < 11 {
            let r = edge_orient.0 & 1;
            edge_orient_buf[i] = r as u8;
            edge_orient_buf[11] += 2 - edge_orient_buf[i];
            edge_orient.0 >>= 1;

            i += 1;
        }
        edge_orient_buf[11] %= 2;

        Self {
            corner_perm: unsafe { core::mem::transmute(corner_perm) },
            corner_orient: unsafe { core::mem::transmute(corner_orient_buf) },
            edge_orient: unsafe { core::mem::transmute(edge_orient_buf) },
            edge_perm: unsafe { core::mem::transmute(edge_perm) },
        }
    }
}

#[test]
fn test_coords() {
    for i in 0..1000 {
        let co = CornerOrientCoord(i * 44 % 2187);
        let eo = EdgeOrientCoord(i * 17 % 2048);
        let eg = EdgeGroupingCoord((i * 4) % 495);
        let cp = CornerPermutationCoord((((i as u32) * 23) % 40320) as u16);
        let udep = UDEdgePermutationCoord((((i as u32) * 101) % 40320) as u16);
        let eep = EEdgePermutationCoord(i as u8 % 24);

        let cube = CubieRepr::from_coords(co, eo, eg, cp, udep, eep);

        assert_eq!(cube.coord_corner_orient(), co);
        assert_eq!(cube.coord_edge_orient(), eo);
        assert_eq!(cube.coord_edge_grouping(), eg);
        assert_eq!(cube.coord_corner_perm(), cp);
        assert_eq!(cube.coord_ud_edge_perm(), udep);
        assert_eq!(cube.coord_e_edge_perm(), eep);
    }
}

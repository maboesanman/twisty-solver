use super::{
    cubie_repr::{CornerResident, CubieRepr},
    permutation_coord::{
        permutation_coord_4, permutation_coord_4_inverse, permutation_coord_8,
        permutation_coord_8_inverse,
    },
};

// corner orientation
// 2187 (11.09 bits) u16
// sym 66 (6.04 bits) u8
// max sym candidate 1202 (10.23 bits) u16
// sym table = 66 * 2 = 132 bytes

// edge orientation
// 2048 (11 bits) u16
// sym 75 (6.22 bits) u8
// max sym candidate 959 (9.91 bits) u16
// sym table = 75 * 2 = 150 bytes

// edge grouping
// 495 (8.9 bits) u16
// sym 34 (5.08 bits) u8
// max sym candidate 255 (8 bits) u8
// sym table = 34 * 1 = 34 bytes

// corner permutation
// 40320 (15.29 bits) u16
// sym 2768 (11.43 bits) u16
// max sym candidate 40319 (15.29 bits) u16
// sym table = 2768 * 2 = 5536 bytes

// ud edge permutation
// 40320 (15.29 bits) u16
// sym 2768 (11.43 bits) u16
// max sym candidate 35278 (15.10 bits) u16
// sym table = 2768 * 2 = 5536 bytes

// e edge permutation
// 24 (4.58 bits) u8
// sym 12 (3.58 bits) u8
// max sym candidate 23 (4.58 bits) (u8)
// sym table = 12 * 1 = 12 bytes

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
    pub const fn coord_corner_orient(&self) -> u16 {
        let mut sum = 0u16;
        let mut i = 7;
        while i > 0 {
            i -= 1;
            sum *= 3;
            sum += self.corner_orient[i] as u16;
        }
        sum
    }

    // phase 1 (kinda)
    pub const fn coord_edge_orient(&self) -> u16 {
        let mut sum = 0u16;
        let mut i = 11;
        while i > 0 {
            i -= 1;
            sum <<= 1;
            sum += self.edge_orient[i] as u16;
        }
        sum
    }

    // phase 1 (kinda)
    pub const fn coord_edge_grouping(&self) -> u16 {
        let mut items = [false; 12];
        let mut i = 0;
        while i < 12 {
            items[i] = self.edge_perm[i] as u8 >= 8;
            i += 1;
        }

        edge_grouping(&items)
    }

    // phase 1
    pub const fn coord_edge_grouping_and_orient(&self) -> u32 {
        let orient = self.coord_edge_orient() as u32;
        let grouping = self.coord_edge_grouping() as u32;

        (grouping << 11) + orient
    }

    // phase 2
    pub const fn coord_corner_perm(&self) -> u16 {
        permutation_coord_8(unsafe {
            core::mem::transmute(&self.corner_perm)
        })
    }

    // phase 2
    pub const fn coord_ud_edge_perm(&self) -> u16 {
        let mut ud_edges = [0u8; 8];
        let mut i = 0;
        let mut j = 0;
        while i < 12 {
            if (self.edge_perm[i] as u8) < 8 {
                ud_edges[j] = self.edge_perm[i] as u8;
                j += 1;
            }
            i += 1;
        }
        permutation_coord_8(&ud_edges)
    }

    // phase 2
    pub const fn coord_e_edge_perm(&self) -> u8 {
        let mut e_edges = [0u8; 4];
        let mut i = 0;
        let mut j = 0;
        while i < 12 {
            if (self.edge_perm[i] as u8) >= 8 {
                e_edges[j] = self.edge_perm[i] as u8;
                j += 1;
            }
            i += 1;
        }
        permutation_coord_4(&e_edges)
    }

    pub const fn from_coords(
        mut corner_orient: u16,
        mut edge_orient: u16,
        edge_group: u16,
        corner_perm: u16,
        ud_edge_perm: u16,
        e_edge_perm: u8,
    ) -> Self {
        let edge_group = edge_grouping_inverse(edge_group);
        let corner_perm = permutation_coord_8_inverse(corner_perm);
        let ud_edge_perm = permutation_coord_8_inverse(ud_edge_perm);
        let e_edge_perm = permutation_coord_4_inverse(e_edge_perm);

        let mut edge_perm = [0u8; 12];

        let mut ud = 0;
        let mut e = 0;
        let mut n = 0;
        while n < 12 {
            if edge_group[n] {
                edge_perm[n] = e_edge_perm[e] + 8;
                e += 1;
            } else {
                edge_perm[n] = ud_edge_perm[ud];
                ud += 1;
            }
            n += 1;
        }

        let mut corner_orient_buf = [0u8; 8];

        let mut i = 0;
        while i < 7 {
            let r = corner_orient % 3;
            corner_orient_buf[i] = r as u8;
            corner_orient_buf[7] += 3 - corner_orient_buf[i];
            corner_orient /= 3;

            i += 1;
        }
        corner_orient_buf[7] %= 3;

        let mut edge_orient_buf = [0u8; 12];
        let mut i = 0;
        while i < 11 {
            let r = edge_orient & 1;
            edge_orient_buf[i] = r as u8;
            edge_orient_buf[11] += 2 - edge_orient_buf[i];
            edge_orient >>= 1;

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
    let mut co = 0;
    let mut eo = 0;
    let mut eg = 0;
    let mut cp = 0;
    let mut udep = 0;
    let mut eep = 0;

    for _ in 0..1000 {
        co = (co + 44) % 2187;
        eo = (eo + 17) % 2048;
        eg = (eg + 4) % 495;
        cp = (cp + 23) % 40320;
        udep = (udep + 101) % 40320;
        eep = (eep + 5) % 24;

        let cube = CubieRepr::from_coords(
            co,
            eo,
            eg,
            cp,
            udep,
            eep
        );

        assert_eq!(cube.coord_corner_orient(), co);
        assert_eq!(cube.coord_edge_orient(), eo);
        assert_eq!(cube.coord_edge_grouping(), eg);
        assert_eq!(cube.coord_corner_perm(), cp);
        assert_eq!(cube.coord_ud_edge_perm(), udep);
        assert_eq!(cube.coord_e_edge_perm(), eep);
    }
}

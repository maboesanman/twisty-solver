use crate::permutation_coord::{edge_grouping_inverse, edge_grouping};

use super::{
    repr_cubie::{CornerResident, ReprCubie},
    permutation_coord::{
        permutation_coord_4, permutation_coord_4_inverse, permutation_coord_8,
        permutation_coord_8_inverse,
    }, coords::{CornerOrientCoord, EdgeOrientCoord, EdgeGroupCoord, CornerPermCoord, UDEdgePermCoord, EEdgePermCoord},
};

pub struct ReprCoord {
    pub corner_orient: CornerOrientCoord,
    pub edge_orient: EdgeOrientCoord,
    pub edge_group: EdgeGroupCoord,
    pub corner_perm: CornerPermCoord,
    pub ud_edge_perm: UDEdgePermCoord,
    pub e_edge_perm: EEdgePermCoord,
}

impl const From<ReprCoord> for ReprCubie {
    fn from(value: ReprCoord) -> Self {
        let edge_group = edge_grouping_inverse(value.edge_group.into());
        let corner_perm = permutation_coord_8_inverse(value.corner_perm.into());
        let ud_edge_perm = permutation_coord_8_inverse(value.ud_edge_perm.into());
        let e_edge_perm = permutation_coord_4_inverse(value.e_edge_perm.into());

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
        let mut corner_orient: u16 = value.corner_orient.into();
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
        let mut edge_orient: u16 = value.edge_orient.into();
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

impl const From<ReprCubie> for ReprCoord {
    fn from(value: ReprCubie) -> Self {
        Self {
            corner_orient: value.coord_corner_orient().into(),
            edge_orient: value.coord_edge_orient().into(),
            edge_group: value.coord_edge_grouping().into(),
            corner_perm: value.coord_corner_perm().into(),
            ud_edge_perm: value.coord_ud_edge_perm().into(),
            e_edge_perm: value.coord_e_edge_perm().into(),
        }
    }
}

#[allow(dead_code)]
impl ReprCubie {
    // phase 1
    const fn coord_corner_orient(&self) -> u16 {
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
    const fn coord_edge_orient(&self) -> u16 {
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
    const fn coord_edge_grouping(&self) -> u16 {
        let mut items = [false; 12];
        let mut i = 0;
        while i < 12 {
            items[i] = self.edge_perm[i] as u8 >= 8;
            i += 1;
        }

        edge_grouping(&items)
    }

    // phase 1
    const fn coord_edge_grouping_and_orient(&self) -> u32 {
        let orient = self.coord_edge_orient() as u32;
        let grouping = self.coord_edge_grouping() as u32;

        (grouping << 11) + orient
    }

    // phase 2
    const fn coord_corner_perm(&self) -> u16 {
        permutation_coord_8(unsafe {
            core::mem::transmute(&self.corner_perm)
        })
    }

    // phase 2
    const fn coord_ud_edge_perm(&self) -> u16 {
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
    const fn coord_e_edge_perm(&self) -> u8 {
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
}

#[test]
fn test_coords() {
    let mut corner_orient = 0;
    let mut edge_orient = 0;
    let mut edge_group = 0;
    let mut corner_perm = 0;
    let mut ud_edge_perm = 0;
    let mut e_edge_perm = 0;

    for _ in 0..1000 {
        corner_orient = (corner_orient + 44) % 2187;
        edge_orient = (edge_orient + 17) % 2048;
        edge_group = (edge_group + 4) % 495;
        corner_perm = (corner_perm + 23) % 40320;
        ud_edge_perm = (ud_edge_perm + 101) % 40320;
        e_edge_perm = (e_edge_perm + 5) % 24;

        let cube: ReprCubie = ReprCoord {
            corner_orient: corner_orient.into(),
            edge_orient: edge_orient.into(),
            edge_group: edge_group.into(),
            corner_perm: corner_perm.into(),
            ud_edge_perm: ud_edge_perm.into(),
            e_edge_perm: e_edge_perm.into(),
        }.into();

        assert_eq!(cube.coord_corner_orient(), corner_orient);
        assert_eq!(cube.coord_edge_orient(), edge_orient);
        assert_eq!(cube.coord_edge_grouping(), edge_group);
        assert_eq!(cube.coord_corner_perm(), corner_perm);
        assert_eq!(cube.coord_ud_edge_perm(), ud_edge_perm);
        assert_eq!(cube.coord_e_edge_perm(), e_edge_perm);
    }
}

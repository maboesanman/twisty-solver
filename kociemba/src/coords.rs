use crate::{
    permutation_coord::{self},
    repr_cubie::{ReprCube, SOLVED_CUBE},
};

macro_rules! define_coord {
    ($name:ident, $inner:ty, $range:expr, $bits:expr) => {
        #[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Default)]
        #[repr(transparent)]
        pub struct $name($inner);

        impl $name {
            pub fn inner(self) -> $inner {
                self.0
            }
        }

        impl Into<$inner> for $name {
            fn into(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                // debug_assert!(value < $range);
                Self(value)
            }
        }
    };
}

// Phase 1 Raw Coordinates

// 2187 (11.09 bits)
define_coord!(CornerOrientCoord, u16, 2187, 12);

impl CornerOrientCoord {
    pub const fn from_cubie(cube: ReprCube) -> Self {
        let mut total = 0u16;
        let mut i = 0;
        while i < 7 {
            total = total * 3 + cube.corner_orient[i] as u16;
            i += 1;
        }

        Self(total)
    }

    pub const fn into_cubie(mut self) -> ReprCube {
        let mut cube = SOLVED_CUBE;

        let mut sum = 0;
        let mut i = 7;
        while i > 0 {
            i -= 1;
            let value = (self.0 % 3) as u8;
            cube.corner_orient[i] = unsafe { core::mem::transmute(value) };
            sum += value as u16;
            self.0 /= 3;
        }

        cube.corner_orient[7] = unsafe { core::mem::transmute(((3 - (sum % 3)) % 3) as u8) };

        cube
    }
}

// 2048 (11 bits) u16
define_coord!(EdgeOrientCoord, u16, 2048, 11);

impl EdgeOrientCoord {
    pub const fn from_cubie(cube: ReprCube) -> Self {
        let mut total = 0u16;
        let mut i = 0;
        while i < 11 {
            total = (total << 1) + cube.edge_orient[i] as u16;
            i += 1;
        }

        Self(total)
    }

    pub const fn into_cubie(mut self) -> ReprCube {
        let mut sum = 0;
        let mut i = 11;

        let mut cube = SOLVED_CUBE;

        while i > 0 {
            i -= 1;
            let value = (self.0 % 2) as u8;
            cube.edge_orient[i] = unsafe { core::mem::transmute(value) };
            sum += value as u16;
            self.0 /= 2;
        }

        cube.edge_orient[11] = unsafe { core::mem::transmute(((2 - (sum % 2)) % 2) as u8) };

        cube
    }
}

// 495 (8.9 bits) u16
// also referred to as the "Phase 1 UDSlice Coordinate"
define_coord!(EdgeGroupCoord, u16, 495, 9);

impl EdgeGroupCoord {
    pub const fn from_cubie(value: ReprCube) -> Self {
        let mut items = [false; 12];
        let mut i = 0;
        while i < 12 {
            items[i] = value.edge_perm[i] > 7;
            i += 1;
        }
        let value = permutation_coord::edge_grouping(&items);
        debug_assert!(value < 495);
        Self(value)
    }

    pub const fn into_cubie(self) -> ReprCube {
        let mut cube = SOLVED_CUBE;

        let edge_group = permutation_coord::edge_grouping_inverse(self.0);

        let mut odd = false;
        let mut i = 0;
        while i < 11 {
            let mut j = i + 1;
            while j < 12 {
                if edge_group[i] && !edge_group[j] {
                    odd = !odd;
                }
                j += 1;
            }
            i += 1;
        }

        let ud: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let e: [u8; 4] = if odd { [8, 9, 11, 10] } else { [8, 9, 10, 11] };

        let mut ud_i = 0;
        let mut e_i = 0;
        while ud_i + e_i < 12 {
            if edge_group[ud_i + e_i] {
                cube.edge_perm[ud_i + e_i] = unsafe { core::mem::transmute(e[e_i]) };
                e_i += 1;
            } else {
                cube.edge_perm[ud_i + e_i] = unsafe { core::mem::transmute(ud[ud_i]) };
                ud_i += 1;
            }
        }

        cube
    }
}

pub const fn phase_1_cubies(
    corners: CornerOrientCoord,
    edges: EdgeOrientCoord,
    edge_group: EdgeGroupCoord,
) -> ReprCube {
    edge_group
        .into_cubie()
        .then(corners.into_cubie())
        .then(edges.into_cubie())
}

// Phase 2 Raw Coordinates

// 40320 (15.29 bits) u16
define_coord!(CornerPermCoord, u16, 40320, 16);

impl CornerPermCoord {
    pub const fn from_cubie(value: ReprCube) -> Self {
        Self(permutation_coord::permutation_coord_8(&value.corner_perm))
    }

    pub const fn into_cubie(self) -> ReprCube {
        ReprCube {
            corner_perm: permutation_coord::permutation_coord_8_inverse(self.0),
            ..SOLVED_CUBE
        }
    }
}

// 40320 (15.29 bits) u16
// also referred to as "Phase 2 Edge Permutation Coordinate"
define_coord!(UDEdgePermCoord, u16, 40320, 16);

impl UDEdgePermCoord {
    pub const fn from_cubie(value: ReprCube) -> Self {
        let first8 = [
            value.edge_perm[0],
            value.edge_perm[1],
            value.edge_perm[2],
            value.edge_perm[3],
            value.edge_perm[4],
            value.edge_perm[5],
            value.edge_perm[6],
            value.edge_perm[7],
        ];
        Self(permutation_coord::permutation_coord_8(&first8))
    }

    pub const fn into_cubie(self) -> ReprCube {
        let perm = permutation_coord::permutation_coord_8_inverse(self.0);
        ReprCube {
            edge_perm: [
                perm[0], perm[1], perm[2], perm[3], perm[4], perm[5], perm[6], perm[7], 8, 9, 10,
                11,
            ],
            ..SOLVED_CUBE
        }
    }
}

// 24 (4.58 bits) u8
// also referred to as "Phase 2 UDSlice Coordinate"
define_coord!(EEdgePermCoord, u8, 24, 5);

impl EEdgePermCoord {
    pub const fn from_cubie(value: ReprCube) -> Self {
        let last4 = [
            value.edge_perm[8],
            value.edge_perm[9],
            value.edge_perm[10],
            value.edge_perm[11],
        ];
        Self(permutation_coord::permutation_coord_4(&last4))
    }

    pub const fn into_cubie(self) -> ReprCube {
        let perm = permutation_coord::permutation_coord_4_inverse(self.0);
        ReprCube {
            edge_perm: [
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                perm[0] + 8,
                perm[1] + 8,
                perm[2] + 8,
                perm[3] + 8,
            ],
            ..SOLVED_CUBE
        }
    }
}

pub const fn phase_2_cubies(
    corners: CornerPermCoord,
    ud_edges: UDEdgePermCoord,
    e_edges: EEdgePermCoord,
) -> ReprCube {
    corners
        .into_cubie()
        .then(ud_edges.into_cubie())
        .then(e_edges.into_cubie())
}

// sym coordinates

// 64430 (15.97 bits) u16
define_coord!(Phase1EdgeSymCoord, u16, 64430, 16);

// 2768 (11.43 bits) u16
define_coord!(Phase2CornerSymCoord, u16, 2768, 12);

#[test]
fn test_corner_orient() {
    for i in 0..2187 {
        let cube = phase_1_cubies(i.into(), 0.into(), 0.into());
        assert_eq!(CornerOrientCoord::from_cubie(cube), i.into());
        assert_eq!(EdgeOrientCoord::from_cubie(cube), 0.into());
        assert_eq!(EdgeGroupCoord::from_cubie(cube), 0.into());
    }
}

#[test]
fn test_corner_orient_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for i in 0..2187 {
        let a = i.into();
        let b = rng.random_range(0..2048).into();
        let c = rng.random_range(0..495).into();

        let cube = phase_1_cubies(a, b, c);
        assert_eq!(CornerOrientCoord::from_cubie(cube), a);
        assert_eq!(EdgeOrientCoord::from_cubie(cube), b);
        assert_eq!(EdgeGroupCoord::from_cubie(cube), c);
    }
}

#[test]
fn test_edge_orient() {
    for i in 0..2048 {
        let cube = phase_1_cubies(0.into(), i.into(), 0.into());
        assert_eq!(CornerOrientCoord::from_cubie(cube), 0.into());
        assert_eq!(EdgeOrientCoord::from_cubie(cube), i.into());
        assert_eq!(EdgeGroupCoord::from_cubie(cube), 0.into());
    }
}

#[test]
fn test_edge_orient_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for i in 0..2048 {
        let a = rng.random_range(0..2187).into();
        let b = i.into();
        let c = rng.random_range(0..495).into();

        let cube = phase_1_cubies(a, b, c);
        assert_eq!(CornerOrientCoord::from_cubie(cube), a);
        assert_eq!(EdgeOrientCoord::from_cubie(cube), b);
        assert_eq!(EdgeGroupCoord::from_cubie(cube), c);
    }
}

#[test]
fn test_edge_group() {
    for i in 0..495 {
        let cube = phase_1_cubies(0.into(), 0.into(), i.into());
        assert_eq!(CornerOrientCoord::from_cubie(cube), 0.into());
        assert_eq!(EdgeOrientCoord::from_cubie(cube), 0.into());
        assert_eq!(EdgeGroupCoord::from_cubie(cube), i.into());
    }
}

#[test]
fn test_edge_group_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for i in 0..495 {
        let a = rng.random_range(0..2187).into();
        let b = rng.random_range(0..2048).into();
        let c = i.into();

        let cube = phase_1_cubies(a, b, c);
        assert_eq!(CornerOrientCoord::from_cubie(cube), a);
        assert_eq!(EdgeOrientCoord::from_cubie(cube), b);
        assert_eq!(EdgeGroupCoord::from_cubie(cube), c);
    }
}

#[test]
fn test_corner_perm() {
    for i in 0..40320 {
        let cube = phase_2_cubies(i.into(), 0.into(), 0.into());
        assert_eq!(CornerPermCoord::from_cubie(cube), i.into());
        assert_eq!(UDEdgePermCoord::from_cubie(cube), 0.into());
        assert_eq!(EEdgePermCoord::from_cubie(cube), 0.into());
    }
}

#[test]
fn test_corner_perm_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for i in 0..40320 {
        let a = i.into();
        let b = rng.random_range(0..40320).into();
        let c = rng.random_range(0..24).into();

        let cube = phase_2_cubies(a, b, c);
        assert_eq!(CornerPermCoord::from_cubie(cube), a);
        assert_eq!(UDEdgePermCoord::from_cubie(cube), b);
        assert_eq!(EEdgePermCoord::from_cubie(cube), c);
    }
}

#[test]
fn test_ud_edge_perm() {
    for i in 0..40320 {
        let cube = phase_2_cubies(0.into(), i.into(), 0.into());
        assert_eq!(CornerPermCoord::from_cubie(cube), 0.into());
        assert_eq!(UDEdgePermCoord::from_cubie(cube), i.into());
        assert_eq!(EEdgePermCoord::from_cubie(cube), 0.into());
    }
}

#[test]
fn test_ud_edge_perm_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for i in 0..40320 {
        let a = rng.random_range(0..40320).into();
        let b = i.into();
        let c = rng.random_range(0..24).into();

        let cube = phase_2_cubies(a, b, c);
        // assert!(cube.is_valid());
        assert_eq!(CornerPermCoord::from_cubie(cube), a);
        assert_eq!(UDEdgePermCoord::from_cubie(cube), b);
        assert_eq!(EEdgePermCoord::from_cubie(cube), c);
    }
}

#[test]
fn test_e_edge_perm() {
    for i in 0..24 {
        let cube = phase_2_cubies(0.into(), 0.into(), i.into());
        assert_eq!(CornerPermCoord::from_cubie(cube), 0.into());
        assert_eq!(UDEdgePermCoord::from_cubie(cube), 0.into());
        assert_eq!(EEdgePermCoord::from_cubie(cube), i.into());
    }
}

#[test]
fn test_e_edge_perm_random() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for i in 0..24 {
        let a = rng.random_range(0..40320).into();
        let b = rng.random_range(0..40320).into();
        let c = i.into();

        let cube = phase_2_cubies(a, b, c);
        assert_eq!(CornerPermCoord::from_cubie(cube), a);
        assert_eq!(UDEdgePermCoord::from_cubie(cube), b);
        assert_eq!(EEdgePermCoord::from_cubie(cube), c);
    }
}

#[test]
fn test_phase_1_cube() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for _ in 0..10000 {
        let a = rng.random_range(0..2187).into();
        let b = rng.random_range(0..2048).into();
        let c = rng.random_range(0..495).into();
        let cube = phase_1_cubies(a, b, c);
        assert_eq!(a, CornerOrientCoord::from_cubie(cube));
        assert_eq!(b, EdgeOrientCoord::from_cubie(cube));
        assert_eq!(c, EdgeGroupCoord::from_cubie(cube));
    }
}

#[test]
fn test_phase_2_cube() {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(17);
    for _ in 0..20000 {
        let a = rng.random_range(0..40320).into();
        let b = rng.random_range(0..40320).into();
        let c = rng.random_range(0..24).into();
        let cube = phase_2_cubies(a, b, c);
        if !cube.is_valid() {
            continue;
        }
        assert_eq!(0, CornerOrientCoord::from_cubie(cube).0);
        assert_eq!(0, EdgeOrientCoord::from_cubie(cube).0);
        assert_eq!(0, EdgeGroupCoord::from_cubie(cube).0);
        assert_eq!(a, CornerPermCoord::from_cubie(cube));
        assert_eq!(b, UDEdgePermCoord::from_cubie(cube));
        assert_eq!(c, EEdgePermCoord::from_cubie(cube));
    }
}

// #[test]
// fn test_corner_sym() {
//     let mut items = std::collections::HashSet::new();
//     for i in 0..40320 {
//         let cubie = phase_2_cubies(i.into(), 0.into(), 0.into());
//         let representative = cubie.get_subgroup_equivalence_class().into_iter().map(|c| CornerPermCoord::from(c).0).min().unwrap();
//         items.insert(representative);
//     }

//     println!("count: {:?}", items.len());
// }

// #[test]
// fn flip_ud_slice_sym() {
//     let mut items = std::collections::HashSet::new();
//     for i in 0..2048 {
//         for j in 0..495 {
//             let cubie = phase_1_cubies(0.into(), i.into(), j.into());
//             let representative = cubie.get_subgroup_equivalence_class().into_iter().map(|c| {
//                 (EdgeOrientCoord::from(c).0, EdgeGroupCoord::from(c).0)
//             }).min().unwrap();
//             items.insert(representative);
//         }
//     }

//     println!("count: {:?}", items.len());
// }

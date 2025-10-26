use rand::distr::{Distribution, StandardUniform};

use crate::{
    kociemba::coords::coords::{CornerOrientRawCoord, EdgeOrientRawCoord},
    permutation_math::permutation::Permutation,
};

use super::{
    cube_move::CubeMove,
    partial_reprs::{
        corner_orient::CornerOrient, corner_perm::CornerPerm, edge_orient::EdgeOrient,
        edge_perm::EdgePerm,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct ReprCube {
    pub corner_perm: CornerPerm,
    pub corner_orient: CornerOrient,
    pub edge_perm: EdgePerm,
    pub edge_orient: EdgeOrient,
}

impl ReprCube {
    pub const SOLVED: Self = Self {
        corner_perm: CornerPerm::SOLVED,
        corner_orient: CornerOrient::SOLVED,
        edge_perm: EdgePerm::SOLVED,
        edge_orient: EdgeOrient::SOLVED,
    };

    pub const fn apply_move(self, mv: CubeMove) -> Self {
        let mv_corner_perm = mv.into_corner_perm();
        let mv_corner_orient = mv.into_corner_orient();
        let mv_edge_perm = mv.into_edge_perm();
        let mv_edge_orient = mv.into_edge_orient();

        Self {
            corner_perm: self.corner_perm.then(mv_corner_perm),
            corner_orient: self
                .corner_orient
                .permute(mv_corner_perm)
                .correct(mv_corner_orient),
            edge_perm: self.edge_perm.then(mv_edge_perm),
            edge_orient: self
                .edge_orient
                .permute(mv_edge_perm)
                .correct(mv_edge_orient),
        }
    }

    pub const fn then(self, other: Self) -> Self {
        Self {
            corner_perm: self.corner_perm.then(other.corner_perm),
            corner_orient: self
                .corner_orient
                .permute(other.corner_perm)
                .correct(other.corner_orient),
            edge_perm: self.edge_perm.then(other.edge_perm),
            edge_orient: self
                .edge_orient
                .permute(other.edge_perm)
                .correct(other.edge_orient),
        }
    }

    pub const fn const_eq(self, other: Self) -> bool {
        self.corner_perm.const_eq(other.corner_perm)
            && self.corner_orient.const_eq(other.corner_orient)
            && self.edge_perm.const_eq(other.edge_perm)
            && self.edge_orient.const_eq(other.edge_orient)
    }

    pub fn pretty_print(self) {
        //-> [[&'static str; 9]; 6] {
        const COLOR_CHARS: [&str; 6] = [
            "\x1b[47m  \x1b[0m", // W (U)
            "\x1b[43m  \x1b[0m", // Y (D)
            "\x1b[41m  \x1b[0m", // R (F)
            "\x1b[44m  \x1b[0m", // O (B)
            "\x1b[45m  \x1b[0m", // B (R)
            "\x1b[42m  \x1b[0m", // G (L)
        ];

        // const COLOR_CHARS: [&str; 6] = [
        //     "W ", // W
        //     "G ", // G
        //     "R ", // R
        //     "B ", // B
        //     "O ", // O
        //     "Y ", // Y
        // ];

        const CORNER_FACELETS: [[(usize, usize); 3]; 8] = [
            [(0, 8), (4, 0), (2, 2)], // UFR
            [(0, 6), (2, 0), (5, 2)], // UFL
            [(0, 2), (3, 0), (4, 2)], // UBR
            [(0, 0), (5, 0), (3, 2)], // UBL
            [(1, 2), (2, 8), (4, 6)], // DFR
            [(1, 0), (5, 8), (2, 6)], // DFL
            [(1, 8), (4, 8), (3, 6)], // DBR
            [(1, 6), (3, 8), (5, 6)], // DBL
        ];

        const EDGE_FACELETS: [[(usize, usize); 2]; 12] = [
            [(0, 7), (2, 1)], // UF
            [(0, 1), (3, 1)], // UB
            [(0, 5), (4, 1)], // UR
            [(0, 3), (5, 1)], // UL
            [(1, 1), (2, 7)], // DF
            [(1, 7), (3, 7)], // DB
            [(1, 5), (4, 7)], // DR
            [(1, 3), (5, 7)], // DL
            [(2, 5), (4, 3)], // FR
            [(2, 3), (5, 5)], // FL
            [(3, 3), (4, 5)], // BR
            [(3, 5), (5, 3)], // BL
        ];

        // start with “blank” faces (or you could fill with e.g. '·')
        let mut faces = [["· "; 9]; 6];

        // Place corners
        for (slot, &piece) in self.corner_perm.0.0.iter().enumerate() {
            let ori = self.corner_orient.0[slot] as usize;

            for j in 0..3 {
                let slot = CORNER_FACELETS[slot][j];
                let color_i = CORNER_FACELETS[piece as usize][(j + 3 - ori) % 3].0;

                faces[slot.0][slot.1] = COLOR_CHARS[color_i];
            }
        }

        for (slot, &piece) in self.edge_perm.0.0.iter().enumerate() {
            let ori = self.edge_orient.0[slot] as usize;

            for j in 0..2 {
                let slot = EDGE_FACELETS[slot][j];
                let color_i = EDGE_FACELETS[piece as usize][(j + ori) % 2].0;

                faces[slot.0][slot.1] = COLOR_CHARS[color_i];
            }
        }

        // // Place edges
        // for (slot, &piece) in self.edge_perm.iter().enumerate() {
        //     let ori = (self.edge_orient[slot] % 2) as usize;
        //     let facelets = EDGE_FACELETS[piece as usize];

        //     for j in 0..2 {
        //         let src_face = facelets[j].0;
        //         let (dst_face, dst_pos) = facelets[(j + ori) % 2];
        //         faces[dst_face][dst_pos] = COLOR_CHARS[src_face];
        //     }
        // }

        // Place centers
        for (i, c) in COLOR_CHARS.into_iter().enumerate() {
            faces[i][4] = c;
        }

        // faces

        const EMPTY_FACE: [&str; 9] = ["  "; 9];

        let spaced_faces = [
            EMPTY_FACE, faces[0], EMPTY_FACE, EMPTY_FACE, faces[5], faces[2], faces[4], faces[3],
            EMPTY_FACE, faces[1], EMPTY_FACE, EMPTY_FACE,
        ];

        for inter_face_row in 0..3 {
            for intra_face_row in 0..3 {
                for inter_face_col in 0..4 {
                    for intra_face_col in 0..3 {
                        print!(
                            "{}",
                            spaced_faces[inter_face_row * 4 + inter_face_col]
                                [intra_face_row * 3 + intra_face_col]
                        );
                    }
                }
                println!()
            }
        }
    }
}

#[macro_export]
macro_rules! cube {
    // 1) ENTRY POINT: invoked as `cube![ F U2 Dp ]`
    [ $($mv:ident)+ ] => {
        $crate::cube_ops::repr_cube::ReprCube::SOLVED
        $(
            .then(cube!(@mv $mv))
        )+
    };

    // 2) “up to 2” and “up prime” on each face:
    (@mv U)  => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::U1) };
    (@mv U2) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::U2) };
    (@mv Up) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::U3) };

    (@mv D)  => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::D1) };
    (@mv D2) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::D2) };
    (@mv Dp) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::D3) };

    (@mv F)  => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::F1) };
    (@mv F2) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::F2) };
    (@mv Fp) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::F3) };

    (@mv B)  => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::B1) };
    (@mv B2) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::B2) };
    (@mv Bp) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::B3) };

    (@mv L)  => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::L1) };
    (@mv L2) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::L2) };
    (@mv Lp) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::L3) };

    (@mv R)  => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::R1) };
    (@mv R2) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::R2) };
    (@mv Rp) => { $crate::cube_ops::repr_cube::ReprCube::SOLVED.apply_move($crate::cube_ops::cube_move::CubeMove::R3) };
}

impl Distribution<ReprCube> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ReprCube {
        let mut cube = ReprCube::SOLVED;
        let edge_perm_high_bits = rng.random_range(0..(479_001_600 >> 1)) << 1;
        let corner_perm_high_bits = rng.random_range(0..(40320 >> 1)) << 1;
        let parity = rng.random_range(0..2);
        cube.edge_perm.0 = Permutation::<12>::const_from_coord(edge_perm_high_bits | parity);
        cube.corner_perm.0 =
            Permutation::<8>::const_from_coord((corner_perm_high_bits | parity) as u16);
        cube.edge_orient = EdgeOrient::from_coord(EdgeOrientRawCoord(rng.random_range(0..2048)));
        cube.corner_orient =
            CornerOrient::from_coord(CornerOrientRawCoord(rng.random_range(0..2048)));

        cube
    }
}

#[test]
fn do_some_moves() {
    ReprCube::SOLVED.pretty_print();

    for mv in CubeMove::all_iter() {
        println!("{mv}");
        ReprCube::SOLVED
            .apply_move(CubeMove::F1)
            .apply_move(mv)
            .pretty_print();
    }
}

#[test]
fn do_some_other_moves() {
    cube![R U Rp Up].pretty_print();
}

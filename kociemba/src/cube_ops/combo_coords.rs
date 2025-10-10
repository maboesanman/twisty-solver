use crate::cube_ops::{coords::{CornerPermSymCoord, EdgeGroupOrientRawCoord, EdgeGroupOrientSymCoord, EdgeGroupRawCoord, EdgeOrientRawCoord}, cube_move::CubeMove, cube_sym::DominoSymmetry};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EdgeGroupOrientComboCoord {
    pub sym_coord: EdgeGroupOrientSymCoord,
    pub domino_conjugation: DominoSymmetry,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CornerPermComboCoord {
    pub sym_coord: CornerPermSymCoord,
    pub domino_conjugation: DominoSymmetry,
}

impl EdgeGroupOrientRawCoord {
    pub fn split(self) -> (EdgeGroupRawCoord, EdgeOrientRawCoord) {
        (
            EdgeGroupRawCoord((self.0 >> 11) as u16),
            EdgeOrientRawCoord((self.0 & 0b11111111111) as u16),
        )
    }

    pub fn join(group: EdgeGroupRawCoord, orient: EdgeOrientRawCoord) -> Self {
        Self(((group.0 as u32) << 11) & (orient.0 as u32))
    }
}

impl EdgeGroupOrientComboCoord {
    pub fn from_raw(tables: &crate::tables::Tables, raw_coord: EdgeGroupOrientRawCoord) -> Self {
        tables.lookup_sym_edge_group_orient.get_combo_from_raw(raw_coord)
    }

    pub fn into_raw(self, tables: &crate::tables::Tables) -> EdgeGroupOrientRawCoord {
        tables.lookup_sym_edge_group_orient.get_raw_from_combo(self)
    }

    pub fn apply_cube_move(self, tables: &crate::tables::Tables, cube_move: CubeMove) -> Self {
        let preimage_move = cube_move.domino_conjugate(self.domino_conjugation.inverse());
        let mut result = tables.move_sym_edge_group_orient.apply_cube_move(self.sym_coord, preimage_move);

        result.domino_conjugation = self.domino_conjugation.then(result.domino_conjugation);
        // result.domino_conjugation = result.domino_conjugation.then(self.domino_conjugation);

        result
    }

    pub fn domino_conjugate(self, domino_symmetry: DominoSymmetry) -> Self { 
        Self {
            sym_coord: self.sym_coord,
            domino_conjugation: self.domino_conjugation.then(domino_symmetry),
            // domino_conjugation: domino_symmetry.then(self.domino_conjugation),
        }
    }
}

// impl CornerPermComboCoord {
//     pub fn from_raw(tables: &crate::tables::Tables, raw_coord: CornerPermRawCoord) -> Self {
//         tables.lookup_sym_corner_perm.get_combo_from_raw(raw_coord)
//     }

//     pub fn into_raw(self, tables: &crate::tables::Tables) -> CornerPermRawCoord {
//         tables.lookup_sym_corner_perm.get_raw_from_combo(self)
//     }

//     pub fn apply_cube_move(self, tables: &crate::tables::Tables, cube_move: CubeMove) -> Self {
//         let preimage_move = cube_move.domino_conjugate(self.domino_conjugation.inverse());
//         let mut result = tables.move_sym_corner_perm.apply_cube_move(self.sym_coord, preimage_move);

//         result.domino_conjugation = self.domino_conjugation.then(result.domino_conjugation);
//         // result.domino_conjugation = result.domino_conjugation.then(self.domino_conjugation);

//         result
//     }

//     pub fn domino_conjugate(self, domino_symmetry: DominoSymmetry) -> Self {
//         Self {
//             sym_coord: self.sym_coord,
//             domino_conjugation: self.domino_conjugation.then(domino_symmetry),
//             // domino_conjugation: domino_symmetry.then(self.domino_conjugation),
//         }
//     }
// }

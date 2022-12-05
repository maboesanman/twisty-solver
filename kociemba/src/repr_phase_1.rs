use super::{moves::Phase1Move, repr_cubie::ReprCubie, repr_coord::ReprCoord, coords::{CornerOrientCoord, CornerPermCoord, UDEdgePermCoord, EEdgePermCoord, EdgeOrientGroupSymCoord}};

pub struct ReprPhase1 {
    pub sym_edge_orient_group: EdgeOrientGroupSymCoord,
    pub corner_orient: CornerOrientCoord,
    pub corner_perm: CornerPermCoord,
    pub ud_edge_perm: UDEdgePermCoord,
    pub e_edge_perm: EEdgePermCoord,
}

impl From<ReprCoord> for ReprPhase1 {
    fn from(value: ReprCoord) -> Self {
        todo!()

        // this first does the lookup for edge_orient_group to find which coord and transform
        // then applies the transform to all other coords from value.
    }
}

impl ReprPhase1 {
    pub fn is_complete(&self) -> bool {
        let c: u16 = self.corner_orient.into();
        let e: u16 = self.sym_edge_orient_group.into();

        c == 0 && e == 0
    }

    pub fn get_distance_lower_bound(&self) -> u8 {
        let offset = self.get_pruning_table_offset();
        todo!()

        // 1: look up the lower bound in the pruning table with the offset.
    }

    pub fn perform_all_moves(&self) -> [ReprPhase1; 18] {
        todo!()

        // 77 seeks or less
        // 67.5 seeks on average

        // 1: lookup all moves/transforms for sym coord (1 seek),
        // 2: loopup all moves for each raw coord (4 seeks),
        // 3: apply the transforms from 1 to the new raw coords from 2 (4 * 18 seeks or less)
    }

    fn get_pruning_table_offset(&self) -> usize {
        let c: u16 = self.corner_orient.into();
        let e: u16 = self.sym_edge_orient_group.into();
        let c = c as usize;
        let e = e as usize;
        c + e * 2187
    }
}

pub const PHASE_1_MOVES: [Phase1Move; 18] = {
    [
        Phase1Move::U1,
        Phase1Move::U2,
        Phase1Move::U3,
        Phase1Move::D1,
        Phase1Move::D2,
        Phase1Move::D3,
        Phase1Move::F1,
        Phase1Move::F2,
        Phase1Move::F3,
        Phase1Move::B1,
        Phase1Move::B2,
        Phase1Move::B3,
        Phase1Move::R1,
        Phase1Move::R2,
        Phase1Move::R3,
        Phase1Move::L1,
        Phase1Move::L2,
        Phase1Move::L3,
    ]
};

// const PHASE_1_SYMMETRIES: [[]]

// const MOVE_TABLE_CORNER_ORIENT: [[u16; 18]; 2187] = {
//     let mut table = [[0; 18]; 2187];

//     let mut i = 0;
//     while i < 2187 {
//         let cube = ReprCubie::from_coords(i, 0, 0, 0, 0, 0);
//         let mut j = 0;
//         while j < 18 {
//             table[i as usize][j] = cube.const_phase_1_move(PHASE_1_MOVES[j]).coord_corner_orient();
//             j += 1;
//         }
//         i += 1;
//     }

//     table
// };

// const MOVE_TABLE_EDGE_ORIENT: [[u16; 18]; 2048] = {
//     let mut table = [[0; 18]; 2048];

//     let mut i = 0;
//     while i < 2048 {
//         let cube = ReprCubie::from_coords(0, i, 0, 0, 0, 0);
//         let mut j = 0;
//         while j < 18 {
//             table[i as usize][j] = cube.const_phase_1_move(PHASE_1_MOVES[j]).coord_edge_orient();
//             j += 1;
//         }
//         i += 1;
//     }

//     table
// };

// const MOVE_TABLE_EDGE_GROUP: [[u16; 18]; 24] = {
//     let mut table = [[0; 18]; 24];

//     let mut i = 0;
//     while i < 24 {
//         let cube = ReprCubie::from_coords(0, 0, i, 0, 0, 0);
//         let mut j = 0;
//         while j < 18 {
//             table[i as usize][j] = cube.const_phase_1_move(PHASE_1_MOVES[j]).coord_edge_grouping();
//             j += 1;
//         }
//         i += 1;
//     }

//     table
// };

// // this is for the pruning_table_offset calculation
// const SYM_TABLE_CORNER_ORIENT: [[u16; 15]; 2187] = {
//     [[0; 15]; 2187]
// };

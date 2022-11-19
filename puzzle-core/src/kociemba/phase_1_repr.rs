use super::{moves::Phase1Move, cubie_repr::CubieRepr};

pub struct Phase1Repr {
    corner_orient: u16,
    edge_orient: u16,
    edge_group: u16,
}

const PHASE_1_MOVES: [Phase1Move; 18] = {
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

const MOVE_TABLE_CORNER_ORIENT: [[u16; 18]; 2187] = {
    let mut table = [[0; 18]; 2187];

    let mut i = 0;
    while i < 2187 {
        let cube = CubieRepr::from_coords(i, 0, 0, 0, 0, 0);
        let mut j = 0;
        while j < 18 {
            table[i as usize][j] = cube.const_phase_1_move(PHASE_1_MOVES[j]).coord_corner_orient();
            j += 1;
        }
        i += 1;
    }

    table
};

const MOVE_TABLE_EDGE_ORIENT: [[u16; 18]; 2048] = {
    let mut table = [[0; 18]; 2048];

    let mut i = 0;
    while i < 2048 {
        let cube = CubieRepr::from_coords(0, i, 0, 0, 0, 0);
        let mut j = 0;
        while j < 18 {
            table[i as usize][j] = cube.const_phase_1_move(PHASE_1_MOVES[j]).coord_edge_orient();
            j += 1;
        }
        i += 1;
    }

    table
};

const MOVE_TABLE_EDGE_GROUP: [[u16; 18]; 24] = {
    let mut table = [[0; 18]; 24];

    let mut i = 0;
    while i < 24 {
        let cube = CubieRepr::from_coords(0, 0, i, 0, 0, 0);
        let mut j = 0;
        while j < 18 {
            table[i as usize][j] = cube.const_phase_1_move(PHASE_1_MOVES[j]).coord_edge_grouping();
            j += 1;
        }
        i += 1;
    }

    table
};

// this is for the pruning_table_offset calculation
const SYM_TABLE_CORNER_ORIENT: [[u16; 15]; 2187] = {
    [[0; 15]; 2187]
};

impl Phase1Repr {
    pub fn pruning_table_offset(&self) -> usize {
        let orient = self.edge_orient as usize;
        let grouping = self.edge_group as usize;

        let edge_coord = (grouping << 11) + orient;

        // get the sym coord, and the corresponding transform
        let sym_edge_coord: usize = todo!();
        // apply the transform to the corner orient coord
        let sym_corner_coord: usize = todo!();
        
        sym_edge_coord * 2187 + sym_corner_coord
    }

    pub fn adjacent(&self) -> [Phase1Repr; 18] {
        todo!()
    }

    
}

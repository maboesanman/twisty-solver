
use kociemba::coords::{CornerOrientMoveTable, CornerOrientMoveEntry, CornerOrientCoord, EdgeOrientCoord, EdgeGroupCoord, CornerPermCoord, UDEdgePermCoord, EEdgePermCoord};
use kociemba::repr_coord::ReprCoord;
use kociemba::repr_cubie::ReprCubie;
use kociemba::repr_phase_1::PHASE_1_MOVES;

fn main() {
    let mut corner_orient_move_table: CornerOrientMoveTable = [CornerOrientMoveEntry {
        transforms: [CornerOrientCoord::from(0); 15],
        moves: [CornerOrientCoord::from(0); 18],
    }; 2187];

    for (i, entry) in corner_orient_move_table.iter_mut().enumerate() {
        let cube: ReprCubie = ReprCoord {
            corner_orient: CornerOrientCoord::from(i as u16),
            edge_orient: EdgeOrientCoord::from(0),
            edge_group: EdgeGroupCoord::from(0),
            corner_perm: CornerPermCoord::from(0),
            ud_edge_perm: UDEdgePermCoord::from(0),
            e_edge_perm: EEdgePermCoord::from(0),
        }.into();

        for (j, m) in PHASE_1_MOVES.iter().enumerate() {
            let c: ReprCoord = cube.const_phase_1_move(*m).into();
            entry.moves[j] = c.corner_orient;
        }

        
    }
}
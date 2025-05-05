use crate::{coords::{CornerOrientCoord, EEdgePermCoord, Phase1EdgeSymCoord, Phase2CornerSymCoord, UDEdgePermCoord}, symmetries::SubGroupTransform};



pub struct ReprPhase1 {
    edge_group_orient_sym_coord: Phase1EdgeSymCoord,
    edge_group_orient_sym_correction: SubGroupTransform,
    corner_orient_raw_coord: CornerOrientCoord,
    
    edge_perm_slots: [u8; 6],
    corner_perm_sym_coord: Phase2CornerSymCoord,
    corner_perm_sym_correction: SubGroupTransform,
}

pub struct ReprPhase2 {
    corner_perm_sym_coord: Phase2CornerSymCoord,
    corner_perm_sym_correction: SubGroupTransform,
    ud_edge_perm_raw_coord: UDEdgePermCoord,
    e_edge_perm_raw_coord: EEdgePermCoord,
}

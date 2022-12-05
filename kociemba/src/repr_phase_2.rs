use super::{moves::Phase1Move, repr_cubie::ReprCubie, repr_coord::ReprCoord, repr_phase_1::ReprPhase1, coords::{CornerPermSymCoord, UDEdgePermCoord, EEdgePermCoord}};

pub struct ReprPhase2 {
    pub sym_corner_perm: CornerPermSymCoord,
    pub ud_edge_perm: UDEdgePermCoord,
    pub e_edge_perm: EEdgePermCoord,
}

impl From<ReprPhase1> for ReprPhase2 {
    fn from(value: ReprPhase1) -> Self {
        debug_assert_eq!(value.sym_edge_orient_group, 0u16.into());
        debug_assert_eq!(value.corner_orient, 0u16.into());

        todo!()
        // this first does the lookup for corner_permto find which coord and transform
        // then applies the transform to all other coords from value.
    }
}

impl ReprPhase2 {
    pub fn is_complete(&self) -> bool {
        self.sym_corner_perm == 0u16.into() &&
        self.ud_edge_perm == 0u16.into() &&
        self.e_edge_perm == 0u8.into()
    }

    pub fn get_distance_lower_bound(&self) -> u8 {
        let offset = self.get_pruning_table_offset();
        todo!()

        // 1: look up the lower bound in the pruning table with the offset.
    }

    pub fn perform_all_moves(&self) -> [ReprPhase1; 18] {
        todo!()

        // 1: perform the lookup on the sym coord for each move (1 seek),
        // 2: perform the loopup on the raw coords for each move (2 seeks),
        // 3: apply the transforms from 1 to the new raw coords from 2 (0 seeks)
    }

    fn get_pruning_table_offset(&self) -> usize {
        let c: u16 = self.sym_corner_perm.into();
        let e: u16 = self.ud_edge_perm.into();
        let c = c as usize;
        let e = e as usize;
        c + e * 2768
    }
}

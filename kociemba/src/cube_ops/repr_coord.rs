use crate::cube_ops::{coords::{CornerOrientRawCoord, CornerPermSymCoord, EdgeGroupOrientSymCoord, EdgeOrientRawCoord}, cube_sym::DominoSymmetry, partial_reprs::{edge_orient::EdgeOrient, edge_perm::EdgePerm}, repr_cube::ReprCube};



pub struct ReprCoord {

    // corner perm
    corner_perm_sym: CornerPermSymCoord,
    corner_perm_correct: DominoSymmetry,

    // edge perm (non coord)
    edge_perm: EdgePerm,

    // corner orient
    corner_orient: CornerOrientRawCoord,

    // edge orient
    edge_orient: EdgeOrientRawCoord,
}

impl From<ReprCube> for ReprCoord {
    fn from(value: ReprCube) -> Self {
        todo!()
    }
}

impl From<ReprCoord> for ReprCube {
    fn from(value: ReprCoord) -> Self {
        todo!()
    }
}

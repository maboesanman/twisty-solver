#![feature(test)]
#![feature(slice_swap_unchecked)]
#![feature(likely_unlikely)]
#![allow(long_running_const_eval)]

mod cube_entry;
mod cube_ops;
mod kociemba;
mod permutation_math;

pub use cube_ops::repr_cube::ReprCube;

pub use cube_ops::partial_reprs::corner_orient::CornerOrient;
pub use cube_ops::partial_reprs::corner_perm::CornerPerm;
pub use cube_ops::partial_reprs::edge_orient::EdgeOrient;
pub use cube_ops::partial_reprs::edge_perm::EdgePerm;

pub use permutation_math::permutation::Permutation;

pub use cube_ops::cube_move::CubeMove;

pub use kociemba::tables::Tables;

pub use kociemba::search::get_incremental_solutions_stream;

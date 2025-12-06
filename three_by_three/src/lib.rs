#![feature(portable_simd)]
#![feature(slice_as_array)]
#![feature(slice_swap_unchecked)]
#![feature(hash_set_entry)]
#![feature(const_precise_live_drops)]
#![allow(long_running_const_eval)]

mod cube_entry;
mod cube_ops;
mod kociemba;
mod permutation_math;
mod tables;

pub use cube_ops::repr_cube::ReprCube;

pub use cube_ops::partial_reprs::corner_orient::CornerOrient;
pub use cube_ops::partial_reprs::corner_perm::CornerPerm;
pub use cube_ops::partial_reprs::edge_orient::EdgeOrient;
pub use cube_ops::partial_reprs::edge_perm::EdgePerm;

pub use permutation_math::permutation::Permutation;

pub use cube_ops::cube_move::CubeMove;

pub use tables::Tables;

pub use kociemba::search::get_incremental_solutions_stream;

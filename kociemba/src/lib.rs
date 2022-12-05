#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(const_eval_limit)]
#![feature(const_convert)]
#![const_eval_limit = "0"]

pub mod repr_coord;
pub mod repr_cubie;
pub mod moves;
pub mod permutation_coord;
// mod sym_coord;
pub mod repr_phase_1;
pub mod repr_phase_2;
pub mod coords;
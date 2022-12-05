#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![feature(const_trait_impl)]
#![feature(const_eval_limit)]
#![feature(const_convert)]
#![const_eval_limit = "0"]

mod repr_coord;
mod repr_cubie;
mod moves;
mod permutation_coord;
// mod sym_coord;
mod repr_phase_1;
mod repr_phase_2;
mod coords;
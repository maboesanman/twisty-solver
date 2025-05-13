#![feature(portable_simd)]
#![feature(slice_as_array)]
#![feature(slice_as_chunks)]
#![feature(slice_swap_unchecked)]
#![feature(hash_set_entry)]
#![feature(const_precise_live_drops)]
#![allow(long_running_const_eval)]

// pub mod coords;
// pub mod moves;
// pub mod permutation_coord;

// #[macro_use]
// pub mod repr_cubie;
// pub mod symmetries;
pub mod tables;

// // pub mod search;
// pub mod repr_phase_1;

pub mod permutation_math;
pub mod cube_ops;
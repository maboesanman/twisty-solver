#![feature(portable_simd)]
#![feature(slice_as_array)]
#![feature(slice_as_chunks)]
pub mod coords;
pub mod moves;
pub mod permutation_coord;

#[macro_use]
pub mod repr_cubie;
pub mod symmetries;
pub mod tables;

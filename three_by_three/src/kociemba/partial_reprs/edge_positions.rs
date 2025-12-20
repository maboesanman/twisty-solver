use num_integer::div_rem;

use crate::{EdgePerm, Permutation, kociemba::{coords::coords::EEdgePermRawCoord, partial_reprs::{e_edge_perm::EEdgePerm, ud_edge_perm::UDEdgePerm}}, permutation_math::grouping::EdgeCombination};



// 495 * 24 = 11880 -> 14 bit
#[derive(Clone, Copy)]
pub struct EEdgePositions(EdgePositions);

impl EEdgePositions {
    pub fn into_phase_2(self) -> EEdgePermRawCoord {
        debug_assert!(self.0.0 < 24);

        EEdgePermRawCoord(self.0.0 as u8)
    }
}

#[derive(Clone, Copy)]
pub struct UEdgePositions(EdgePositions);

#[derive(Clone, Copy)]
pub struct DEdgePositions(EdgePositions);

#[derive(Clone, Copy)]
struct EdgePositions(u16);

impl EdgePositions {
    pub fn split(self) -> (EdgeCombination, Permutation<4>) {
        let (combo, perm) = div_rem(self.0, 24);
        (EdgeCombination::from_coord(combo), Permutation::<4>::const_from_coord(perm as u8))
    }

    pub fn join(combo: EdgeCombination, perm: Permutation<4>) -> Self {
        Self(combo.into_coord() * 24 + (perm.const_into_coord() as u16))
    }
}

pub fn combine_edge_positions(u: UEdgePositions, d: DEdgePositions, e: EEdgePositions) -> EdgePerm {
    let (u_combo, u_perm) = u.0.split();
    let u_combo = u_combo.0;
    let u_perm = u_perm.0;

    let (d_combo, d_perm) = d.0.split();
    let d_combo = d_combo.0;
    let d_perm = d_perm.0;

    let (e_combo, e_perm) = e.0.split();
    let e_combo = e_combo.0;
    let e_perm = e_perm.0;

    let mut array = [0u8; 12];
    let mut u_i = 0;
    let mut d_i = 0;
    let mut e_i = 0;
    for (i, slot) in array.iter_mut().enumerate() {
        if u_combo[i] {
            *slot = u_perm[u_i];
            u_i += 1;
            continue;
        }
        if d_combo[i] {
            *slot = d_perm[d_i] + 4;
            d_i += 1;
            continue;
        }
        if e_combo[i] {
            *slot = e_perm[e_i] + 8;
            e_i += 1;
            continue;
        }
    }

    EdgePerm(Permutation(array))
}

pub fn split_edge_positions(
    perm: EdgePerm,
) -> (UEdgePositions, DEdgePositions, EEdgePositions) {
    let array = perm.0 .0;

    let mut u_combo = [false; 12];
    let mut d_combo = [false; 12];
    let mut e_combo = [false; 12];

    let mut u_perm = [0u8; 4];
    let mut d_perm = [0u8; 4];
    let mut e_perm = [0u8; 4];

    let mut u_i = 0;
    let mut d_i = 0;
    let mut e_i = 0;

    for (i, &slot) in array.iter().enumerate() {
        match slot {
            0..=3 => {
                u_combo[i] = true;
                u_perm[u_i] = slot;
                u_i += 1;
            }
            4..=7 => {
                d_combo[i] = true;
                d_perm[d_i] = slot - 4;
                d_i += 1;
            }
            8..=11 => {
                e_combo[i] = true;
                e_perm[e_i] = slot - 8;
                e_i += 1;
            }
            _ => unreachable!(),
        }
    }

    (
        UEdgePositions(EdgePositions::join(EdgeCombination(u_combo), Permutation(u_perm))),
        DEdgePositions(EdgePositions::join(EdgeCombination(d_combo), Permutation(d_perm))),
        EEdgePositions(EdgePositions::join(EdgeCombination(e_combo), Permutation(e_perm))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_integer::div_rem;

    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    use rayon::prelude::*;

    #[test]
    fn edge_positions_split_join_roundtrip() {
        const MAX: u16 = 495 * 24;

        for coord in 0..MAX {
            let egp = EdgePositions(coord);

            let (combo, perm) = egp.split();
            let rejoined = EdgePositions::join(combo, perm);

            assert_eq!(
                egp.0, rejoined.0,
                "split/join failed at coord {}",
                coord
            );
        }
    }

    
    #[test]
    fn edge_perm_combine_split_seeded_parallel() {
        // Deterministic RNG
        let mut rng = StdRng::seed_from_u64(69);

        // Pre-generate all samples deterministically
        let coords: Vec<u32> = (0..10_0000)
            .map(|_| rng.random_range(0..479_001_600) as u32)
            .collect();

        coords.par_iter().for_each(|&coord| {
            let perm = EdgePerm(Permutation::<12>::const_from_coord(coord));

            let (u, d, e) = split_edge_positions(perm);
            let recombined = combine_edge_positions(u, d, e);

            assert_eq!(
                perm, recombined,
                "combine/split failed at coord {}",
                coord
            );
        });
    }
}
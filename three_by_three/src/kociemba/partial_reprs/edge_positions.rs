use num_integer::div_rem;

use crate::{EdgePerm, Permutation, kociemba::{coords::coords::{EEdgePermRawCoord, EdgeGroupRawCoord}, partial_reprs::{e_edge_perm::EEdgePerm, ud_edge_perm::UDEdgePerm}}, permutation_math::grouping::EdgeCombination};



// 495 * 24 = 11880 -> 14 bit
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EEdgePositions(pub EdgePositions);

impl EEdgePositions {
    pub const SOLVED: Self = Self(EdgePositions(0));

    pub fn into_phase_2(self) -> EEdgePermRawCoord {
        debug_assert!(self.0.0 < 24);

        EEdgePermRawCoord(self.0.0 as u8)
    }

    pub const  fn into_index(self) -> usize {
        self.0.0 as usize
    }

    pub const  fn into_inner(self) -> u16 {
        self.0.0
    }

    pub const  fn from_inner(inner: u16) -> Self {
        Self(EdgePositions(inner))
    }

    pub fn rep_edge_perm(self) -> EdgePerm {
        let (pos_a, pos_b) = self.0.valid_sibling_pair();
        combine_edge_positions(UEdgePositions(pos_a), DEdgePositions(pos_b), self)
    }
    
    pub const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }

    pub fn into_edge_group_raw(self) -> EdgeGroupRawCoord {
        EdgeGroupRawCoord(self.0.0 / 24)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct UEdgePositions(pub EdgePositions);

impl UEdgePositions {
    pub const SOLVED: Self = Self(EdgePositions(0));

    pub  const fn into_index(self) -> usize {
        self.0.0 as usize
    }

    pub const fn into_inner(self) -> u16 {
        self.0.0
    }

    pub const  fn from_inner(inner: u16) -> Self {
        Self(EdgePositions(inner))
    }

    pub fn rep_edge_perm(self) -> EdgePerm {
        let (pos_a, pos_b) = self.0.valid_sibling_pair();
        combine_edge_positions(self, DEdgePositions(pos_a), EEdgePositions(pos_b))
    }

    // d_group_residue is already 0..70
    pub const fn get_phase_2_u(d_group_residue: u16, perm: u16) -> UEdgePositions {
        const GROUP_LOOKUP: [u16; 70] = {
            let mut table = [0u16; 70];

            let mut i = 0;
            while i < 70 {
                let mut combo = EdgeCombination::from_coord(i + 425).const_into_array();
                let mut j = 0;
                while j < 8 {
                    combo[j] = !combo[j];
                    j += 1;
                }
                table[i as usize] = EdgeCombination::const_from_array(combo).into_coord();
                i += 1;
            }

            table
        };
        let group = GROUP_LOOKUP[d_group_residue as usize] as u16;

        Self(EdgePositions(group * 24 + perm))
    }
    
    pub const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DEdgePositions(pub EdgePositions);

impl DEdgePositions {
    pub const SOLVED: Self = Self(EdgePositions(0));

    pub const fn into_index(self) -> usize {
        self.0.0 as usize
    }

    pub const fn into_inner(self) -> u16 {
        self.0.0
    }

    pub const fn from_inner(inner: u16) -> Self {
        Self(EdgePositions(inner))
    }

    pub fn rep_edge_perm(self) -> EdgePerm {
        let (pos_a, pos_b) = self.0.valid_sibling_pair();
        combine_edge_positions(UEdgePositions(pos_a), self, EEdgePositions(pos_b))
    }
    
    pub const fn const_eq(self, other: Self) -> bool {
        self.0.const_eq(other.0)
    }
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EdgePositions(pub u16);

impl EdgePositions {
    pub const fn split(self) -> (EdgeCombination, Permutation<4>) {
        let combo = self.0 / 24;
        let perm = self.0 % 24;
        (EdgeCombination::from_coord(combo), Permutation::<4>::const_from_coord(perm as u8))
    }

    pub const fn join(combo: EdgeCombination, perm: Permutation<4>) -> Self {
        Self(combo.into_coord() * 24 + (perm.const_into_coord() as u16))
    }

    pub fn valid_sibling_pair(self) -> (Self, Self) {
        let combo = EdgeCombination::from_coord(self.0 / 24).0;

        let mut sib_combo_a = [false; 12];
        let mut sib_combo_b = [false; 12];
        let mut j = 0;

        for i in 0..12 {
            if !combo[i] {
                if j < 4 {
                    sib_combo_a[i] = true;
                } else {
                    sib_combo_b[i] = true;
                }
                j += 1;
            }
        }

        (
            Self::join(EdgeCombination(sib_combo_a), Permutation::IDENTITY),
            Self::join(EdgeCombination(sib_combo_b), Permutation::IDENTITY)
        )
    }

    pub const fn valid_sibling_from_pair(self, sib: Self) -> Self {
        let combo_a = EdgeCombination::from_coord(self.0 / 24).0;
        let combo_b = EdgeCombination::from_coord(sib.0 / 24).0;

        let mut new_sib_combo = [false; 12];
        let mut i = 0;
        while i < 12 {
            if !(combo_a[i] || combo_b[i]) {
                new_sib_combo[i] = true;
            }
            i += 1;
        }

        Self::join(EdgeCombination(new_sib_combo), Permutation::IDENTITY)
    }
    
    pub const fn const_eq(self, other: Self) -> bool {
        self.0 == other.0
    }
}

pub const fn combine_edge_positions(u: UEdgePositions, d: DEdgePositions, e: EEdgePositions) -> EdgePerm {
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

    let mut i = 0;
    while i < 12 {
        if u_combo[i] {
            array[i] = u_perm[u_i];
            u_i += 1;
            i += 1;
            continue;
        }
        if d_combo[i] {
            array[i] = d_perm[d_i] + 4;
            d_i += 1;
            i += 1;
            continue;
        }
        if e_combo[i] {
            array[i] = e_perm[e_i] + 8;
            e_i += 1;
            i += 1;
            continue;
        }
        panic!()
    }

    EdgePerm(Permutation(array))
}

pub const fn split_edge_positions(
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

    let mut i = 0;
    while i < 12 {
        match array[i] {
            0..=3 => {
                u_combo[i] = true;
                u_perm[u_i] = array[i];
                u_i += 1;
            }
            4..=7 => {
                d_combo[i] = true;
                d_perm[d_i] = array[i] - 4;
                d_i += 1;
            }
            8..=11 => {
                e_combo[i] = true;
                e_perm[e_i] = array[i] - 8;
                e_i += 1;
            }
            _ => unreachable!(),
        }
        i += 1;
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
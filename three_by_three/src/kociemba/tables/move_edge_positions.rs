use std::{ops::Index, path::Path};

use anyhow::Result;
use bitvec::{field::BitField, order::Msb0, slice::BitSlice, view::BitView};
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    EdgePerm,
    cube_ops::{
        cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_orient::CornerOrient,
    },
    kociemba::{
        coords::coords::CornerOrientRawCoord,
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, UEdgePositions, combine_edge_positions,
            split_edge_positions,
        },
    },
};

use super::table_loader::{as_u16_slice, as_u16_slice_mut, load_table};

const TABLE_SIZE_BYTES: usize = 495 * 24 * 32;
const FILE_CHECKSUM: u32 = 3288712858;

pub struct MoveEdgePositions(Mmap);

#[repr(transparent)]
struct PackedEdgePositionRow([u8; 32]);

impl PackedEdgePositionRow {
    #[inline]
    fn get(&self, index: CubeMove) -> u16 {
        let bits = self.0.view_bits::<Msb0>();
        let start = index.into_index() * 14;
        let end = start + 14;
        bits[start..end].load::<u16>()
    }
}

impl MoveEdgePositions {
    fn chunks(table: &Mmap) -> &[PackedEdgePositionRow] {
        unsafe {
            let slice: &[[u8; 32]] = table.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const PackedEdgePositionRow, slice.len())
        }
    }

    pub fn apply_all_cube_moves(
        &self,
        u_coord: UEdgePositions,
        d_coord: DEdgePositions,
        e_coord: EEdgePositions,
        move_iter: impl IntoIterator<Item = CubeMove>,
    ) -> impl IntoIterator<Item = (UEdgePositions, DEdgePositions, EEdgePositions)> {
        let chunks = &Self::chunks(&self.0);

        let u_chunk = &chunks[u_coord.into_index()];
        let d_chunk = &chunks[d_coord.into_index()];
        let e_chunk = &chunks[e_coord.into_index()];

        move_iter.into_iter().map(|mv| {
            (
                UEdgePositions::from_inner(u_chunk.get(mv)),
                DEdgePositions::from_inner(d_chunk.get(mv)),
                EEdgePositions::from_inner(e_chunk.get(mv)),
            )
        })
    }

    fn generate(buffer: &mut [u8]) {
        buffer.par_chunks_mut(32).enumerate().for_each(|(i, row)| {
            let u_coord = UEdgePositions::from_inner(i as u16);
            let edge_perm = u_coord.rep_edge_perm();
            let bits = row.view_bits_mut::<Msb0>();

            for mv in CubeMove::all_iter() {
                let (new_u_coord, _, _) = split_edge_positions(edge_perm.apply_cube_move(mv));
                let start = mv.into_index() * 14;
                let end = start + 14;
                bits[start..end].store::<u16>(new_u_coord.into_inner());
            }
        });
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, Self::generate).map(Self)
    }
}

#[cfg(test)]
mod test {
    use crate::Permutation;
    use crate::kociemba::tables::Tables;

    use super::*;

    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn edge_perm_combine_split_seeded_parallel() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table = tables.move_edge_position;
        // Deterministic RNG
        let mut rng = StdRng::seed_from_u64(69);

        // Pre-generate all samples deterministically
        let coords: Vec<u32> = (0..10_0000)
            .map(|_| rng.random_range(0..479_001_600) as u32)
            .collect();

        coords.par_iter().for_each(|&coord| {
            let perm = EdgePerm(Permutation::<12>::const_from_coord(coord));

            let (u_coord, d_coord, e_coord) = split_edge_positions(perm);
            let moved_by_table = table
                .apply_all_cube_moves(u_coord, d_coord, e_coord, CubeMove::all_iter())
                .into_iter()
                .map(|(u, d, e)| combine_edge_positions(u, d, e));
            let moved_by_perm = CubeMove::all_iter().map(|mv| perm.apply_cube_move(mv));

            assert!(moved_by_table.eq(moved_by_perm));
        });

        Ok(())
    }
}

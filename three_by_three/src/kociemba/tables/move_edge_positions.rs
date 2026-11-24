use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry},
    kociemba::{
        partial_reprs::edge_positions::{
            DEdgePositions, EEdgePositions, EdgePositions, UEdgePositions, split_edge_positions,
        },
        tables::table_loader::as_u16_slice_mut,
    },
};

use super::table_loader::load_table;

pub(crate) const TABLE_SIZE_BYTES: usize = 495 * 24 * 64;
const FILE_CHECKSUM: u32 = 2871907041;

pub struct MoveEdgePositionsTable([u8]);

#[repr(align(64))]
pub struct MoveEdgePositionsRow(pub [u16; 32]);

impl MoveEdgePositionsRow {
    pub fn get_mv(&self, index: CubeMove) -> u16 {
        self.0[index as u8 as usize]
    }

    pub fn get_conj(&self, index: DominoSymmetry) -> u16 {
        extract(&self.0, index.0 as u8 as usize)
    }
}

impl MoveEdgePositionsTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr() as *const u16
    }

    fn chunks(&self) -> &[MoveEdgePositionsRow] {
        unsafe {
            let slice: &[[u8; 32]] = self.0.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const MoveEdgePositionsRow, slice.len())
        }
    }

    pub fn row(&self, edge_pos: EdgePositions) -> &MoveEdgePositionsRow {
        unsafe { self.chunks().get_unchecked(edge_pos.0 as usize) }
    }

    pub fn apply_all_cube_moves(
        &self,
        u_coord: UEdgePositions,
        d_coord: DEdgePositions,
        e_coord: EEdgePositions,
        move_iter: impl IntoIterator<Item = CubeMove>,
    ) -> impl IntoIterator<Item = (UEdgePositions, DEdgePositions, EEdgePositions)> {
        let chunks = self.chunks();

        let u_chunk = &chunks[u_coord.into_index()];
        let d_chunk = &chunks[d_coord.into_index()];
        let e_chunk = &chunks[e_coord.into_index()];

        move_iter.into_iter().map(|mv| {
            (
                UEdgePositions::from_inner(u_chunk.get_mv(mv)),
                DEdgePositions::from_inner(d_chunk.get_mv(mv)),
                EEdgePositions::from_inner(e_chunk.get_mv(mv)),
            )
        })
    }

    fn generate(buffer: &mut [u8]) {
        as_u16_slice_mut(buffer)
            .par_chunks_mut(32)
            .enumerate()
            .for_each(|(i, row)| {
                let u_coord = UEdgePositions::from_inner(i as u16);
                let edge_perm = u_coord.rep_edge_perm();

                for mv in CubeMove::all_iter() {
                    let (new_u_coord, _, _) = split_edge_positions(edge_perm.apply_cube_move(mv));
                    row[mv.into_index()] = new_u_coord.into_inner();
                }

                let syms: Vec<u16> = DominoSymmetry::all_iter()
                    .map(|sym| {
                        let (new_u_coord, _, _) =
                            split_edge_positions(edge_perm.domino_conjugate(sym));
                        new_u_coord.0.0
                    })
                    .collect();

                pack(row.as_mut_array().unwrap(), *syms.as_array().unwrap());
            });
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, Self::generate)
    }

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

#[inline(always)]
fn extract(buf: &[u16; 32], i: usize) -> u16 {
    let j = (18 + i - 1 - (i >> 3)) as usize;
    let k = (18 + (i + 1) - 1 - ((i + 1) >> 3)) as usize;
    let n = (i & 0b0111) << 1;
    let m = 16 - n;
    (((buf[j] as u32) >> m) as u16 | (buf[k] << n)) & 0x3FFF
}

fn pack(out: &mut [u16; 32], val: [u16; 16]) {
    let [val_a, val_b] = unsafe { core::mem::transmute::<[u16; 16], [[u16; 8]; 2]>(val) };
    let out_a: &mut [u16; 7] = out[18..25].as_mut_array().unwrap();
    pack_half(out_a, val_a);
    let out_b: &mut [u16; 7] = out[25..32].as_mut_array().unwrap();
    pack_half(out_b, val_b);
}

fn pack_half(out: &mut [u16; 7], arr: [u16; 8]) {
    let mut val = 0u128;
    for i in (0..8).rev() {
        val = val << 14;
        val |= (arr[i] & 0x3FFF) as u128;
    }
    for i in 0..7 {
        out[i] = val as u16;
        val = val >> 16;
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use crate::kociemba::partial_reprs::edge_positions::combine_edge_positions;
    use crate::kociemba::tables::Tables;
    use crate::{EdgePerm, Permutation};

    use super::*;

    use itertools::Itertools as _;
    use rand::distr::StandardUniform;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn edge_perm_move() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table: &MoveEdgePositionsTable = tables.as_ref();
        // Deterministic RNG
        let mut rng = StdRng::seed_from_u64(69);

        // Pre-generate all samples deterministically
        let coords: Vec<u32> = (0..10_0000)
            .map(|_| rng.random_range(0..479_001_600) as u32)
            .collect();

        coords.par_iter().for_each(|&coord| {
            let perm = EdgePerm(Permutation::<12>::const_lehmer_decode(coord));

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

    #[test]
    fn edge_perm_sym() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let table: &MoveEdgePositionsTable = tables.as_ref();
        // Deterministic RNG
        let mut rng = StdRng::seed_from_u64(69);

        // Pre-generate all samples deterministically
        let coords: Vec<u32> = (0..10_0000)
            .map(|_| rng.random_range(0..479_001_600) as u32)
            .collect();

        coords.iter().for_each(|&coord| {
            let perm = EdgePerm(Permutation::<12>::const_lehmer_decode(coord));

            let (u_coord, d_coord, e_coord) = split_edge_positions(perm);
            let u_row = table.row(u_coord.0);
            let d_row = table.row(d_coord.0);
            let e_row = table.row(e_coord.0);

            let moved_by_table = DominoSymmetry::all_iter()
                .map(|s| {
                    let u = UEdgePositions(EdgePositions(d_row.get_conj(s)));
                    let d = DEdgePositions(EdgePositions(u_row.get_conj(s)));
                    let e = EEdgePositions(EdgePositions(e_row.get_conj(s)));
                    combine_edge_positions(u, d, e)
                })
                .collect_vec();
            let moved_by_perm = DominoSymmetry::all_iter()
                .map(|s| perm.domino_conjugate(s))
                .collect_vec();

            assert_eq!(moved_by_table, moved_by_perm);
        });

        Ok(())
    }

    #[test]
    fn pack_extract_round_trip() {
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..1000 {
            let vals: [u16; 16] = std::array::from_fn(|_| rng.random_range(0..0x3FFF));
            let mut out = [0u16; 32];
            pack(&mut out, vals);

            for i in 0..16 {
                assert_eq!(extract(&out, i), vals[i], "mismatch at index {i}");
            }
        }
    }
}

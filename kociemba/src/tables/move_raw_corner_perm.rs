use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::cube_ops::{
    coords::CornerPermRawCoord, cube_move::CubeMove, cube_sym::DominoSymmetry,
    partial_reprs::corner_perm::CornerPerm,
};

use super::table_loader::{as_u16_slice, as_u16_slice_mut, load_table};

const TABLE_SIZE_BYTES: usize = 40320 * 33 * 2;
const FILE_CHECKSUM: u32 = 1588942778;

pub struct MoveRawCornerPermTable(Mmap);

impl MoveRawCornerPermTable {
    fn chunks(&self) -> &[[CornerPermRawCoord; 33]] {
        let buffer = as_u16_slice(&self.0);
        unsafe {
            let slice: &[[u16; 33]] = buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const [CornerPermRawCoord; 33],
                slice.len(),
            )
        }
    }

    fn chunk(&self, coord: CornerPermRawCoord) -> &[CornerPermRawCoord; 33] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(&self, coord: CornerPermRawCoord, mv: CubeMove) -> CornerPermRawCoord {
        self.chunk(coord)[mv.into_index()]
    }

    pub fn domino_conjugate(
        &self,
        coord: CornerPermRawCoord,
        transform: DominoSymmetry,
    ) -> CornerPermRawCoord {
        if transform.0 == 0 {
            return coord;
        }
        self.chunk(coord)[transform.into_index() + 17]
    }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u16_slice_mut(buffer);
        buffer.par_chunks_mut(33).enumerate().for_each(|(i, row)| {
            let perm = CornerPerm::from_coord(CornerPermRawCoord(i as u16));
            for (j, coord) in CubeMove::all_iter()
                .map(move |mv| perm.apply_cube_move(mv))
                .chain(DominoSymmetry::nontrivial_iter().map(move |sym| perm.domino_conjugate(sym)))
                .map(|perm| perm.into_coord())
                .enumerate()
            {
                row[j] = coord.0
            }
        });
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, Self::generate).map(Self)
    }
}

#[test]
fn test() -> Result<()> {
    let table = MoveRawCornerPermTable::load("corner_perm_move_table.dat")?;
    for i in 0..40320u16 {
        let coord = CornerPermRawCoord(i);
        let perm = CornerPerm::from_coord(coord);

        for mv in CubeMove::all_iter() {
            assert_eq!(
                table.apply_cube_move(coord, mv),
                perm.apply_cube_move(mv).into_coord()
            );
        }

        for sym in DominoSymmetry::all_iter() {
            assert_eq!(
                table.domino_conjugate(coord, sym),
                perm.domino_conjugate(sym).into_coord()
            );
        }
    }

    Ok(())
}

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::cube_ops::{
    coords::CornerOrientRawCoord, cube_move::CubeMove, cube_sym::DominoSymmetry,
    partial_reprs::corner_orient::CornerOrient,
};

use super::table_loader::{as_u16_slice, as_u16_slice_mut, load_table};

const TABLE_SIZE_BYTES: usize = (2187 * 33) * 2;
const FILE_CHECKSUM: u32 = 1089186443;

pub struct MoveRawCornerOrientTable(Mmap);

impl MoveRawCornerOrientTable {
    fn chunks(&self) -> &[[CornerOrientRawCoord; 33]] {
        let buffer = as_u16_slice(&self.0);
        unsafe { 
            let slice: &[[u16; 33]] = buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const [CornerOrientRawCoord; 33],
                slice.len(),
            )
        }
    }

    fn chunk(&self, coord: CornerOrientRawCoord) -> &[CornerOrientRawCoord; 33] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(&self, coord: CornerOrientRawCoord, mv: CubeMove) -> CornerOrientRawCoord {
        self.chunk(coord)[mv.into_index()]
    }

    pub fn domino_conjugate(
        &self,
        coord: CornerOrientRawCoord,
        transform: DominoSymmetry,
    ) -> CornerOrientRawCoord {
        if transform.0 == 0 {
            return coord;
        }
        self.chunk(coord)[transform.into_index() + 17]
    }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u16_slice_mut(buffer);
        buffer.par_chunks_mut(33).enumerate().for_each(|(i, row)| {
            let orient = CornerOrient::from_coord(CornerOrientRawCoord(i as u16));
            for (j, coord) in CubeMove::all_iter()
                .map(move |mv| orient.apply_cube_move(mv))
                .chain(
                    DominoSymmetry::nontrivial_iter().map(move |sym| orient.domino_conjugate(sym)),
                )
                .map(|orient| orient.into_coord())
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
    let table = MoveRawCornerOrientTable::load("corner_orient_move_table.dat")?;
    for i in 0..2187u16 {
        let coord = CornerOrientRawCoord(i);
        let orient = CornerOrient::from_coord(coord);

        for mv in CubeMove::all_iter() {
            assert_eq!(table.apply_cube_move(coord, mv), orient.apply_cube_move(mv).into_coord());
        }

        for sym in DominoSymmetry::all_iter() {
            assert_eq!(table.domino_conjugate(coord, sym), orient.domino_conjugate(sym).into_coord());
        }
    }

    Ok(())
}

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use num_integer::Integer;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_orient::CornerOrient,
    },
    kociemba::coords::coords::CornerOrientRawCoord,
};

use super::table_loader::load_table;

const TABLE_SIZE_BYTES: usize = 2187 * const { core::mem::size_of::<Row>() };
const FILE_CHECKSUM: u32 = 3314415234;

pub struct MoveRawCornerOrientTable(Mmap);

#[repr(C)]
#[repr(align(64))]
struct Row {
    moves: [u16; 18],
    conjugations: [u64; 3],
}

impl MoveRawCornerOrientTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr() as *const u16
    }

    fn chunks(&self) -> &[Row] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<Row>()]] = self.0.as_chunks_unchecked();
            core::slice::from_raw_parts(slice.as_ptr() as *const Row, slice.len())
        }
    }

    fn chunks_mut(buffer: &mut [u8]) -> &mut [Row] {
        unsafe {
            let slice: &mut [[u8; core::mem::size_of::<Row>()]] = buffer.as_chunks_unchecked_mut();
            core::slice::from_raw_parts_mut(slice.as_ptr() as *mut Row, slice.len())
        }
    }

    fn chunk(&self, coord: CornerOrientRawCoord) -> &Row {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: CornerOrientRawCoord,
        mv: CubeMove,
    ) -> CornerOrientRawCoord {
        CornerOrientRawCoord(self.chunk(coord).moves[mv.into_index()])
    }

    #[inline]
    pub fn domino_conjugate(
        &self,
        coord: CornerOrientRawCoord,
        transform: DominoSymmetry,
    ) -> CornerOrientRawCoord {
        if transform.0 == 0 {
            return coord;
        }
        let row = self.chunk(coord).conjugations;
        let (a, b) = (transform.0 - 1).div_rem(&5);
        CornerOrientRawCoord(((row[a as usize] >> (b * 12)) & 0b0000_1111_1111_1111) as u16)
    }

    fn generate(buffer: &mut [u8]) {
        Self::chunks_mut(buffer)
            .into_par_iter()
            .enumerate()
            .for_each(|(i, row)| {
                let orient = CornerOrient::from_coord(CornerOrientRawCoord(i as u16));
                for (j, mv) in CubeMove::all_iter().enumerate() {
                    row.moves[j] = orient.apply_cube_move(mv).into_coord().0;
                }

                for sym in DominoSymmetry::nontrivial_iter() {
                    let value = orient.domino_conjugate(sym).into_coord().0;
                    let (a, b) = (sym.0 - 1).div_rem(&5);
                    let shift = b * 12;
                    let mask = 0x0FFFu64 << shift;

                    row.conjugations[a as usize] =
                        (row.conjugations[a as usize] & !mask) | ((value as u64) << shift);
                }
            })
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, Self::generate).map(Self)
    }
}

#[cfg(test)]
mod test {
    use crate::kociemba::tables::Tables;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let tables = Tables::new("tables")?;
        let table: &MoveRawCornerOrientTable = tables.as_ref();
        for i in 0..2187u16 {
            let coord = CornerOrientRawCoord(i);
            let orient = CornerOrient::from_coord(coord);

            for mv in CubeMove::all_iter() {
                assert_eq!(
                    table.apply_cube_move(coord, mv),
                    orient.apply_cube_move(mv).into_coord()
                );
            }

            for sym in DominoSymmetry::all_iter() {
                assert_eq!(
                    table.domino_conjugate(coord, sym),
                    orient.domino_conjugate(sym).into_coord()
                );
            }
        }

        Ok(())
    }
}

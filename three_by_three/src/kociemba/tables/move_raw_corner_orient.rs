use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        cube_move::CubeMove, cube_sym::DominoSymmetry, partial_reprs::corner_orient::CornerOrient,
    },
    kociemba::coords::CornerOrientRawCoord,
};

use super::table_loader::load_table;

pub(crate) const TABLE_SIZE_BYTES: usize =
    2187 * const { core::mem::size_of::<MoveRawCornerOrientRow>() };
const FILE_CHECKSUM: u32 = 1953065213;

pub struct MoveRawCornerOrientTable {
    buffer: [u8],
}

#[repr(C)]
#[repr(align(64))]
pub struct MoveRawCornerOrientRow {
    pub moves: [u16; 18],
    pub conj_lo: [u8; 16],
    pub conj_hi: [u8; 8],
}

impl MoveRawCornerOrientRow {
    pub fn as_u16_array(&self) -> &[u16; 32] {
        unsafe { core::mem::transmute(self) }
    }
}

impl MoveRawCornerOrientTable {
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self.buffer.as_ptr() as *const u16
    }

    fn rows(&self) -> &[MoveRawCornerOrientRow] {
        unsafe {
            let slice: &[[u8; core::mem::size_of::<MoveRawCornerOrientRow>()]] =
                self.buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const MoveRawCornerOrientRow,
                slice.len(),
            )
        }
    }

    fn rows_mut(buffer: &mut [u8]) -> &mut [MoveRawCornerOrientRow] {
        unsafe {
            let slice: &mut [[u8; core::mem::size_of::<MoveRawCornerOrientRow>()]] =
                buffer.as_chunks_unchecked_mut();
            core::slice::from_raw_parts_mut(
                slice.as_ptr() as *mut MoveRawCornerOrientRow,
                slice.len(),
            )
        }
    }

    pub fn row(&self, coord: CornerOrientRawCoord) -> &MoveRawCornerOrientRow {
        unsafe { self.rows().get_unchecked(coord.0 as usize) }
    }

    #[inline]
    pub fn apply_cube_move(
        &self,
        coord: CornerOrientRawCoord,
        mv: CubeMove,
    ) -> CornerOrientRawCoord {
        CornerOrientRawCoord(self.row(coord).moves[mv.into_index()])
    }

    #[inline]
    pub fn domino_conjugate(
        &self,
        coord: CornerOrientRawCoord,
        transform: DominoSymmetry,
    ) -> CornerOrientRawCoord {
        let row = self.row(coord);
        let i = transform.0 as usize;
        let lo = row.conj_lo[i] as u16;
        let hi = (row.conj_hi[i >> 1] >> ((i & 1) << 2)) as u16 & 0xF;
        CornerOrientRawCoord(lo | (hi << 8))
    }

    pub(crate) fn as_buffer(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

impl MoveRawCornerOrientTable {
    fn generate(buffer: &mut [u8]) {
        Self::rows_mut(buffer)
            .into_par_iter()
            .enumerate()
            .for_each(|(i, row)| {
                let orient = CornerOrient::from_coord(CornerOrientRawCoord(i as u16));
                for (j, mv) in CubeMove::all_iter().enumerate() {
                    row.moves[j] = orient.apply_cube_move(mv).into_coord().0;
                }

                for sym in DominoSymmetry::all_iter() {
                    let value = orient.domino_conjugate(sym).into_coord().0;
                    let i = sym.0 as usize;
                    row.conj_lo[i] = value as u8;
                    let shift = (i & 1) << 2;
                    row.conj_hi[i >> 1] =
                        (row.conj_hi[i >> 1] & !(0xFu8 << shift)) | (((value >> 8) as u8) << shift);
                }
            })
    }

    pub fn load<Pa: AsRef<Path>>(path: Pa) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, Self::generate)
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

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;


use crate::cube_ops::{coords::EEdgePermRawCoord, cube_move::DominoMove, cube_sym::DominoSymmetry, partial_reprs::e_edge_perm::EEdgePerm};

use super::table_loader::load_table;

const TABLE_SIZE_BYTES: usize = 24 * (10 + 15);
const FILE_CHECKSUM: u32 = 665180893;

pub struct MoveRawEEdgePermTable(Mmap);

impl MoveRawEEdgePermTable {
    fn chunks(&self) -> &[[EEdgePermRawCoord; 25]] {
        let buffer = &self.0[..];
        unsafe { 
            let slice: &[[u8; 25]] = buffer.as_chunks_unchecked();
            core::slice::from_raw_parts(
                slice.as_ptr() as *const [EEdgePermRawCoord; 25],
                slice.len(),
            )
        }
    }

    fn chunk(&self, coord: EEdgePermRawCoord) -> &[EEdgePermRawCoord; 25] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_domino_move(&self, coord: EEdgePermRawCoord, mv: DominoMove) -> EEdgePermRawCoord {
        self.chunk(coord)[mv.into_index()]
    }

    pub fn domino_conjugate(
        &self,
        coord: EEdgePermRawCoord,
        transform: DominoSymmetry,
    ) -> EEdgePermRawCoord {
        if transform.0 == 0 {
            return coord;
        }
        self.chunk(coord)[transform.into_index() + 9]
    }

    fn generate(buffer: &mut [u8]) {
        buffer.par_chunks_mut(25).enumerate().for_each(|(i, row)| {
            let perm = EEdgePerm::from_coord(EEdgePermRawCoord(i as u8));
            for (j, coord) in DominoMove::all_iter()
                .map(move |mv| perm.apply_domino_move(mv))
                .chain(
                    DominoSymmetry::nontrivial_iter().map(move |sym| perm.domino_conjugate(sym)),
                )
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
    let table = MoveRawEEdgePermTable::load("e_edge_perm_move_table.dat")?;
    for i in 0..24 {
        let coord = EEdgePermRawCoord(i);
        let perm = EEdgePerm::from_coord(coord);

        for mv in DominoMove::all_iter() {
            assert_eq!(table.apply_domino_move(coord, mv), perm.apply_domino_move(mv).into_coord());
        }

        for sym in DominoSymmetry::all_iter() {
            assert_eq!(table.domino_conjugate(coord, sym), perm.domino_conjugate(sym).into_coord());
        }
    }

    Ok(())
}


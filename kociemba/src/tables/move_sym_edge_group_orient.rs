use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::{
    cube_ops::{
        coords::EdgeGroupOrientSymCoord, cube_move::CubeMove, cube_sym::DominoSymmetry,
        partial_reprs::edge_group_orient::EdgeGroupOrient,
    },
    tables::{table_loader::{as_u16_slice, as_u16_slice_mut}, Tables},
};

use super::{
    lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable, table_loader::load_table,
};

const TABLE_SIZE_BYTES: usize = (64430 * 18) * 2 * 2;
const FILE_CHECKSUM: u32 = 3661454509;

pub struct MoveSymEdgeGroupOrientTable(Mmap);

impl MoveSymEdgeGroupOrientTable {
    fn chunks(&self) -> &[[u16; 36]] {
        let buffer = as_u16_slice(&self.0);
        unsafe { buffer.as_chunks_unchecked() }
    }

    fn chunk(&self, coord: EdgeGroupOrientSymCoord) -> &[u16; 36] {
        &self.chunks()[coord.0 as usize]
    }

    pub fn apply_cube_move(
        &self,
        coord: EdgeGroupOrientSymCoord,
        mv: CubeMove,
    ) -> (EdgeGroupOrientSymCoord, DominoSymmetry) {
        (
            EdgeGroupOrientSymCoord(self.chunk(coord)[mv.into_index() * 2]),
            DominoSymmetry(self.chunk(coord)[mv.into_index() * 2 + 1] as u8),
        )
    }

    fn generate(buffer: &mut [u8], sym_lookup_table: &LookupSymEdgeGroupOrientTable) {
        assert_eq!(buffer.len(), TABLE_SIZE_BYTES);
        let buffer = as_u16_slice_mut(buffer);

        buffer
            .par_chunks_mut(36)
            .enumerate()
            .for_each(|(i, store)| {
                let sym_coord = EdgeGroupOrientSymCoord(i as u16);
                let rep = sym_lookup_table.get_raw_from_sym(sym_coord);
                let group_orient = EdgeGroupOrient::from_coord(rep);

                CubeMove::all_iter()
                    .zip(store.as_chunks_mut::<2>().0)
                    .for_each(|(mv, slot)| {
                        let new_rep = group_orient.apply_cube_move(mv).into_coord();
                        let (sym_coord, sym_correct) = sym_lookup_table.get_sym_from_raw(new_rep);
                        *slot = [sym_coord.0, sym_correct.0 as u16];
                    });
            })
    }

    pub fn load<P: AsRef<Path>>(
        path: P,
        sym_lookup_table: &LookupSymEdgeGroupOrientTable,
    ) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, sym_lookup_table)
        })
        .map(Self)
    }
}


#[cfg(test)]
mod test {
    use crate::cube_ops::partial_reprs::edge_group_orient::EdgeGroupOrientRawCoord;

    use super::*;
    #[test]
    fn test() -> anyhow::Result<()> {
    
        let tables = Tables::new("tables")?;

        (0..64430).into_par_iter().for_each(|i| {
            let sym_coord = EdgeGroupOrientSymCoord(i);

            let rep_coord = tables.lookup_sym_edge_group_orient.get_raw_from_sym(sym_coord);
            let rep_edge_group_orient = EdgeGroupOrient::from_coord(rep_coord);

            for mv in CubeMove::all_iter() {
                let (moved_sym, sym) = tables.move_sym_edge_group_orient.apply_cube_move(sym_coord, mv);
                let moved = rep_edge_group_orient.apply_cube_move(mv).domino_conjugate(sym);
                assert_eq!(moved.into_coord(), tables.lookup_sym_edge_group_orient.get_raw_from_sym(moved_sym));
            }
        });

        Ok(())
    }
}

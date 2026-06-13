use bitvec::field::BitField;
use bitvec::view::BitView;

use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;

use crate::kociemba::coords::{CornerPermSymCoord, UDEdgePermRawCoord};
use crate::kociemba::tables::prune_phase_2::PrunePhase2Table;

use super::table_loader::load_table;

const TABLE_ENTRY_COUNT: usize = 2768;
const WORKING_TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT;
const TABLE_SIZE_BYTES: usize = TABLE_ENTRY_COUNT / 2;
const FILE_CHECKSUM: u32 = 163716575;

pub struct PrunePhase2CornerSymTable([u8]);

impl PrunePhase2CornerSymTable {
    pub fn get_value(&self, corner_perm_sym_coord: CornerPermSymCoord) -> u8 {
        let i = corner_perm_sym_coord.0 as usize;

        let byte = self.0[i >> 1];
        let shift = (i & 1) << 2;
        (byte >> shift) & 0b1111
    }

    fn generate(buffer: &mut [u8], prune_phase_2: &PrunePhase2Table) {
        let bits = buffer.view_bits_mut::<bitvec::order::Lsb0>();

        let mut set = |i: usize, val: u8| {
            assert!(val < 16);
            let start = i * 4;
            bits[start..start + 4].store_le::<u8>(val);
        };

        for i in 0..2768 {
            let x = (0..40320)
                .map(|j| prune_phase_2.get_value(CornerPermSymCoord(i), UDEdgePermRawCoord(j)))
                .min()
                .unwrap();

            set(i as usize, x);
        }
    }

    pub fn load<P: AsRef<Path>>(path: P, prune_phase_2: &PrunePhase2Table) -> Result<Mmap> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf, prune_phase_2)
        })
    }

    pub(crate) unsafe fn from_buffer(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const Self) }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::Tables;

    use super::*;

    #[test]
    fn generate() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let table: &PrunePhase2CornerSymTable = tables.as_ref();

        let mut histogram = BTreeMap::<_, u16>::new();

        (0..2768).for_each(|i| {
            let v = table.get_value(CornerPermSymCoord(i));
            *histogram.entry(v).or_default() += 1;
        });

        for (k, v) in histogram {
            println!("{k:02} {v}");
        }

        // println!("histogram: {histogram:?}");

        Ok(())
    }
}

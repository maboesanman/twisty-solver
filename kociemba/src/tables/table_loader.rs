use rayon::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
    fs::OpenOptions,
    path::Path,
};

use anyhow::{Context, Result};
use fs2::FileExt; // ← add `fs2 = "0.4"` to Cargo.toml
use memmap2::{Mmap, MmapMut, MmapOptions};

pub fn load_table<P, G>(path: P, size_bytes: usize, checksum: u32, mut generator: G) -> Result<Mmap>
where
    P: AsRef<Path>,
    G: for<'a> FnMut(&'a mut [u8]),
{
    loop {
        // ──────────────── 1. fast path: try to open for reading ────────────────
        if let Ok(file) = OpenOptions::new().read(true).open(&path) {
            // block until any writer releases its lock
            fs2::FileExt::lock_shared(&file)
                .with_context(|| format!("locking (shared) {}", path.as_ref().display()))?;

            // SAFETY: the file is at least `size_bytes` if it was generated correctly
            let mmap = unsafe { MmapOptions::new().len(size_bytes).map(&file)? };
            let hash_actual = crc32fast::hash(&mmap);

            // we’re done – unlock and return the clean table
            fs2::FileExt::unlock(&file)?;
            if hash_actual == checksum {
                return Ok(mmap);
            }

            // checksum is wrong – probably somebody crashed half-way.  Drop
            // shared lock and fall through to the writer branch
        }

        // ──────────────── 2. regenerate under an exclusive lock ────────────────
        //
        //   • open read-write (create/truncate)
        //   • take an EXCLUSIVE lock – this blocks while another writer owns it
        //   • **re-check** the checksum in case another writer finished while
        //     we were waiting
        //   • if still wrong / file freshly created, regenerate
        //
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("opening {} for regeneration", path.as_ref().display()))?;

        file.lock_exclusive()
            .with_context(|| format!("locking (exclusive) {}", path.as_ref().display()))?;

        // If the file already has the right size + checksum, another process
        // finished the job while we waited.  Verify before clobbering.
        let mut need_generate = true;
        if file.metadata()?.len() == size_bytes as u64 {
            let mmap = unsafe { MmapOptions::new().len(size_bytes).map(&file)? };
            if crc32fast::hash(&mmap) == checksum {
                need_generate = false; // nothing to do – reuse it
            }
        }

        if need_generate {
            println!("generating table {}", path.as_ref().display());

            file.set_len(size_bytes as u64)
                .with_context(|| "set_len() during regeneration")?;
            let mut mmap_mut: MmapMut =
                unsafe { MmapOptions::new().len(size_bytes).map_mut(&file)? };

            generator(&mut mmap_mut);

            let hash_actual = crc32fast::hash(&mmap_mut);
            if hash_actual != checksum {
                fs2::FileExt::unlock(&file)?;
                return Err(anyhow::anyhow!(
                    "generation of {} failed – checksum {} ≠ expected {}",
                    path.as_ref().display(),
                    hash_actual,
                    checksum
                ));
            }

            mmap_mut.flush()?; // make sure all pages hit the file
            println!(
                "generated {} with checksum {}",
                path.as_ref().display(),
                hash_actual
            );
        }

        // Release the exclusive lock *before* we go back to the read path.
        fs2::FileExt::unlock(&file)?;
        // The loop reiterates, re-opening as read-only and returning the map.
    }
}

pub fn as_u16_slice(bytes: &[u8]) -> &[u16] {
    // 1) length must be even
    assert!(bytes.len() % 2 == 0, "length not a multiple of 2");
    // 2) pointer must be aligned
    let ptr = bytes.as_ptr();
    assert!(
        ptr as usize % std::mem::align_of::<u16>() == 0,
        "pointer not aligned for u16"
    );
    // 3) reinterpret
    let len_u16 = bytes.len() / 2;
    unsafe { std::slice::from_raw_parts(ptr as *mut u16, len_u16) }
}

// pub fn as_u16_2_slice(bytes: &[u8]) -> &[[u16; 2]] {
//     // 1) length must be a multiple of 4 bytes
//     assert!(
//         bytes.len() % std::mem::size_of::<[u16; 2]>() == 0,
//         "length not a multiple of 4"
//     );
//     // 2) pointer must be aligned for [u16;2] (same as u16)
//     let ptr = bytes.as_ptr();
//     assert!(
//         (ptr as usize) % std::mem::align_of::<[u16; 2]>() == 0,
//         "pointer not aligned for u16"
//     );
//     // 3) reinterpret as &[ [u16;2] ]
//     let count = bytes.len() / std::mem::size_of::<[u16; 2]>();
//     unsafe {
//         std::slice::from_raw_parts(
//             ptr as *const [u16; 2], // cast straight to array‐of‐2
//             count,
//         )
//     }
// }

pub fn as_u16_slice_mut(bytes: &mut [u8]) -> &mut [u16] {
    // 1) length must be even
    debug_assert!(bytes.len() % 2 == 0, "length not a multiple of 2");
    // 2) pointer must be aligned
    let ptr = bytes.as_ptr();
    debug_assert!(
        ptr as usize % std::mem::align_of::<u16>() == 0,
        "pointer not aligned for u16"
    );
    // 3) reinterpret
    let len_u16 = bytes.len() / 2;
    unsafe { std::slice::from_raw_parts_mut(ptr as *mut u16, len_u16) }
}

pub fn as_u32_slice(bytes: &[u8]) -> &[u32] {
    // 1) length must be even
    debug_assert!(bytes.len() % 4 == 0, "length not a multiple of 2");
    // 2) pointer must be aligned
    let ptr = bytes.as_ptr();
    debug_assert!(
        ptr as usize % std::mem::align_of::<u32>() == 0,
        "pointer not aligned for u32"
    );
    // 3) reinterpret
    let len_u32 = bytes.len() / 4;
    unsafe { std::slice::from_raw_parts(ptr as *mut u32, len_u32) }
}

pub fn as_u32_slice_mut(bytes: &mut [u8]) -> &mut [u32] {
    // 1) length must be even
    debug_assert!(bytes.len() % 4 == 0, "length not a multiple of 2");
    // 2) pointer must be aligned
    let ptr = bytes.as_ptr();
    debug_assert!(
        ptr as usize % std::mem::align_of::<u32>() == 0,
        "pointer not aligned for u32"
    );
    // 3) reinterpret
    let len_u32 = bytes.len() / 4;
    unsafe { std::slice::from_raw_parts_mut(ptr as *mut u32, len_u32) }
}

// pub fn generate_cube_move_and_domino_sym_table<const SIZE: usize, T, F>(
//     buffer: &mut [u8],
//     to_fn: T,
//     from_fn: F,
// ) where
//     T: Send + Sync + Fn(usize) -> ReprCube,
//     F: Send + Sync + Fn(ReprCube) -> u16,
// {
//     assert_eq!(buffer.len(), SIZE);
//     let buffer = as_u16_slice_mut(buffer);

//     buffer.par_chunks_mut(33).enumerate().for_each(|(i, row)| {
//         for (j, coord) in to_fn(i)
//             .phase_1_move_table_entry_cubes()
//             .map(&from_fn)
//             .enumerate()
//         {
//             row[j] = coord
//         }
//     });
// }

// pub fn generate_domino_move_and_domino_sym_table<const SIZE: usize, T, F>(buffer: &mut [u8], to_fn: T, from_fn: F)
// where
//     T: Send + Sync + Fn(usize) -> ReprCube,
//     F: Send + Sync + Fn(ReprCube) -> u16,
// {
//     assert_eq!(buffer.len(), SIZE);
//     let buffer = as_u16_slice_mut(buffer);

//     buffer.par_chunks_mut(25).enumerate().for_each(|(i, row)| {
//         for (j, coord) in to_fn(i)
//             .phase_2_move_table_entry_cubes()
//             .map(&from_fn)
//             .enumerate()
//         {
//             row[j] = coord
//         }
//     });
// }

// /// # Safety
// /// * `data` must not be accessed again through the old `[u8]` view
// /// * all further writes / reads **must** use atomic methods
// pub unsafe fn as_atomic_u8_slice(data: &mut [u8]) -> &[std::sync::atomic::AtomicU8] {
//     debug_assert_eq!(core::mem::size_of::<std::sync::atomic::AtomicU8>(), 1); // true on every platform
//     let len = data.len();
//     let ptr = data.as_mut_ptr() as *const std::sync::atomic::AtomicU8;
//     unsafe { core::slice::from_raw_parts(ptr, len) }
// }

pub fn collect_unique_sorted_parallel<T, I>(par_iter: I) -> impl Iterator<Item = T>
where
    I: ParallelIterator<Item = T>,
    T: Send + Sync + Eq + std::hash::Hash + Ord + Copy,
{
    let sets: Vec<_> = par_iter
        .fold(
            || BTreeSet::new(),
            |mut set, value| {
                set.insert(value);
                set
            },
        )
        .map(|set| set.into_iter())
        .collect();

    UniqueSorted::new(sets)
}

struct UniqueSorted<T> {
    sets: Vec<std::collections::btree_set::IntoIter<T>>,
    heap: BinaryHeap<(Reverse<T>, usize)>,
}

impl<T: Send + Sync + Eq + std::hash::Hash + Ord> UniqueSorted<T> {
    pub fn new(mut sets: Vec<std::collections::btree_set::IntoIter<T>>) -> Self {
        let heap = sets
            .iter_mut()
            .map(|set| set.next())
            .enumerate()
            .filter_map(|(i, t)| t.map(|t| (Reverse(t), i)))
            .collect();

        Self { sets, heap }
    }

    fn pop(&mut self) -> Option<T> {
        let (Reverse(candidate), i) = self.heap.pop()?;
        self.heap.push((Reverse(self.sets.get_mut(i)?.next()?), i));
        Some(candidate)
    }
}

impl<T: Send + Sync + Eq + std::hash::Hash + Ord + Copy + Clone> Iterator for UniqueSorted<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let candidate = self.pop()?;

        if self.heap.is_empty() {
            return Some(candidate);
        }

        while self.heap.peek().map(|x| x.0.0) == Some(candidate) {
            self.pop();
        }

        Some(candidate)
    }
}

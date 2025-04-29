use rayon::prelude::*;
use std::{fs::OpenOptions, path::Path};

use anyhow::Result;
use memmap2::{Mmap, MmapOptions};

use crate::{moves::Move, repr_cubie::ReprCubie, symmetries::SubGroupTransform};

pub fn load_table<P, G>(path: P, size_bytes: usize, checksum: u32, generator: G) -> Result<Mmap>
where
    P: AsRef<Path>,
    G: for<'a> FnOnce(&'a mut [u8]),
{
    if let Ok(file) = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .truncate(false)
        .open(&path)
    {
        let mmap = unsafe { MmapOptions::new().len(size_bytes).map(&file).unwrap() };
        let hash_actual = crc32fast::hash(&mmap);
        if hash_actual == checksum {
            return Ok(mmap);
        } else {
            println!("table {} has checksum {}, which does not match expected checksum {}. regenerating...", path.as_ref().to_string_lossy(), hash_actual, checksum);
        }
    }

    println!("generating table {}", path.as_ref().to_string_lossy());

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len(size_bytes as u64).unwrap();

    let mut mmap_mut = unsafe { MmapOptions::new().len(size_bytes).map_mut(&file)? };

    generator(&mut mmap_mut);

    let hash_actual = crc32fast::hash(&mmap_mut);
    if hash_actual != checksum {
        return Err(anyhow::format_err!(
            "generation of {} failed. generated checksum {} does not match expected checksum {}",
            path.as_ref().to_string_lossy(),
            hash_actual,
            checksum
        ));
    }

    println!(
        "generated {} with checksum {}",
        path.as_ref().to_string_lossy(),
        hash_actual
    );

    Ok(mmap_mut.make_read_only().unwrap())
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

pub fn as_u16_2_slice(bytes: &[u8]) -> &[[u16; 2]] {
    // 1) length must be a multiple of 4 bytes
    assert!(
        bytes.len() % std::mem::size_of::<[u16; 2]>() == 0,
        "length not a multiple of 4"
    );
    // 2) pointer must be aligned for [u16;2] (same as u16)
    let ptr = bytes.as_ptr();
    assert!(
        (ptr as usize) % std::mem::align_of::<[u16; 2]>() == 0,
        "pointer not aligned for u16"
    );
    // 3) reinterpret as &[ [u16;2] ]
    let count = bytes.len() / std::mem::size_of::<[u16; 2]>();
    unsafe {
        std::slice::from_raw_parts(
            ptr as *const [u16; 2], // cast straight to array‐of‐2
            count,
        )
    }
}

pub fn as_u16_slice_mut(bytes: &mut [u8]) -> &mut [u16] {
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
    unsafe { std::slice::from_raw_parts_mut(ptr as *mut u16, len_u16) }
}

pub fn generate_full_move_table<const SIZE: usize, T, F>(buffer: &mut [u8], to_fn: T, from_fn: F)
where
    T: Send + Sync + Fn(usize) -> ReprCubie,
    F: Send + Sync + Fn(ReprCubie) -> u16,
{
    assert_eq!(buffer.len(), SIZE);
    let buffer = as_u16_slice_mut(buffer);

    buffer.par_chunks_mut(34).enumerate().for_each(|(i, row)| {
        let cube = to_fn(i);
        let mut j = 0usize;
        while j < 18 {
            let m: Move = unsafe { core::mem::transmute(j as u8) };
            row[j] = from_fn(cube.const_move(m));
            j += 1;
        }
        while j < 34 {
            let t = SubGroupTransform((j - 18) as u8);
            row[j] = from_fn(cube.conjugate_by_subgroup_transform(t));
            j += 1;
        }
    });
}

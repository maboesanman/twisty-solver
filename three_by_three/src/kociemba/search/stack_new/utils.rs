use std::{
    borrow::{Borrow, BorrowMut}, mem::MaybeUninit, ops::{Index, IndexMut},
};

use fearless_simd::SimdMask;

#[repr(C)]
pub struct ConstGenericWorkaround<const N: usize, T> {
    prefix: [MaybeUninit<T>; 3],
    body: [[MaybeUninit<T>; 15]; N],
}

impl<const N: usize, T> core::fmt::Debug for ConstGenericWorkaround<N, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Borrow<[T]>>::borrow(self).fmt(f)
    }
}

impl<const N: usize, T> Borrow<[T]> for ConstGenericWorkaround<N, T> {
    fn borrow(&self) -> &[T] {
        let ptr = self.prefix.as_ptr() as *const T;
        let len = N * 15 + 3;  // total number of elements
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl<const N: usize, T> BorrowMut<[T]> for ConstGenericWorkaround<N, T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        let ptr = self.prefix.as_mut_ptr() as *mut T;
        let len = N * 15 + 3;  // total number of elements
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<const N: usize, T: Copy> ConstGenericWorkaround<N, T> {
    pub fn new() -> Self {
        Self {
            prefix: [MaybeUninit::uninit(); 3],
            body: [[MaybeUninit::uninit(); 15]; N],
        }
    }

    pub fn get_array_mut(&mut self, index: usize) -> &mut [T; 16] {
        debug_assert!(index + 15 < 3 + 15 * N);

        let ptr: *mut Self = self;
        let ptr: *mut T = ptr.cast();
        let ptr = unsafe { ptr.add(index) };
        let ptr: *mut [T; 16] = ptr.cast();
        unsafe { ptr.as_mut_unchecked() }
    }
}

impl<const N: usize, T: Copy> Index<usize> for ConstGenericWorkaround<N, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 3 + 15 * N);

        let ptr: *const Self = self;
        let ptr: *const T = ptr.cast();

        unsafe { ptr.add(index).as_ref_unchecked() }
    }
}

impl<const N: usize, T: Copy> Clone for ConstGenericWorkaround<N, T> {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            body: self.body.clone(),
        }
    }
}

impl<const N: usize, T: Copy> IndexMut<usize> for ConstGenericWorkaround<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < 3 + 15 * N);

        let ptr: *mut Self = self;
        let ptr: *mut T = ptr.cast();

        unsafe { ptr.add(index).as_mut_unchecked() }
    }
}

#[inline(always)]
pub fn lut_u8x32<S: fearless_simd::Simd>(
    simd: S,
    table: fearless_simd::u8x32<S>,
    idx: fearless_simd::u8x16<S>,
) -> fearless_simd::u8x16<S> {
    // scalar fallback
    let mut result = simd.splat_u8x16(0);
    for i in 0..16 {
        result[i] = table[idx[i] as usize];
    }
    result
}

#[inline(always)]
pub fn lut_u8x64<S: fearless_simd::Simd>(
    simd: S,
    table: fearless_simd::u8x64<S>,
    idx: fearless_simd::u8x16<S>,
) -> fearless_simd::u8x16<S> {
    // scalar fallback
    let mut result = simd.splat_u8x16(0);
    for i in 0..16 {
        result[i] = table[idx[i] as usize];
    }
    result
}

#[inline(always)]
pub fn gather<S: fearless_simd::Simd>(
    simd: S,
    table: &[u8],
    word_indices: fearless_simd::u32x16<S>,
    byte_indices: fearless_simd::u8x16<S>,
) -> fearless_simd::u8x16<S> {
    // scalar fallback
    let mut result = simd.splat_u8x16(0);
    for i in 0..16 {
        let word_index = word_indices[i];
        let byte_index = byte_indices[i];

        let full_byte_index = (word_index << 2) | (byte_index as u32);
        let byte = unsafe { *table.get_unchecked(full_byte_index as usize) };

        result[i] = byte;
    }

    result
}

#[inline(always)]
pub fn gather_masked<S: fearless_simd::Simd>(
    simd: S,
    table: &[u8],
    word_indices: fearless_simd::u32x16<S>,
    byte_indices: fearless_simd::u8x16<S>,
    mask: fearless_simd::mask8x16<S>,
) -> fearless_simd::u8x16<S> {
    // scalar fallback
    let mut result = simd.splat_u8x16(0);
    for i in 0..16 {
        if !mask.test(i) {
            continue;
        }
        let word_index = word_indices[i];
        let byte_index = byte_indices[i];

        let full_byte_index = (word_index << 2) | (byte_index as u32);
        let byte = unsafe { *table.get_unchecked(full_byte_index as usize) };

        result[i] = byte;
    }

    result
}

#[inline(always)]
pub fn mask_to_swizzle<S: fearless_simd::Simd>(
    simd: S,
    mask: fearless_simd::mask8x16<S>,
) -> (fearless_simd::u8x16<S>, u8) {
    // scalar fallback
    let mut x = 0;
    let mut result = simd.splat_u8x16(0);
    for i in 0..16 {
        if mask.test(i as usize) {
            result[x as usize] = i;
            x += 1;
        }
    }

    (result, x)
}

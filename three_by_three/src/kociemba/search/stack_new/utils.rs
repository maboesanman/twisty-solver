use fearless_simd::SimdMask;

#[inline(always)]
/// treat the two dimensional array as a single dimensional array.
pub fn get_entry<'a, const N: usize, T>(pos: usize, arr: &'a [[T; 15]; N]) -> &'a T {
    debug_assert!(pos < 15 * N);

    let ptr: *const [T; 15] = arr.as_ptr();
    let ptr: *const T = ptr.cast();

    unsafe { ptr.add(pos).as_ref_unchecked() }
}

#[inline(always)]
/// treat the two dimensional array as a single dimensional array.
pub fn get_entry_mut<'a, const N: usize, T>(pos: usize, arr: &'a mut [[T; 15]; N]) -> &'a mut T {
    debug_assert!(pos < 15 * N);

    let ptr: *mut [T; 15] = arr.as_mut_ptr();
    let ptr: *mut T = ptr.cast();

    unsafe { ptr.add(pos).as_mut_unchecked() }
}

#[inline(always)]
pub fn get_entry_slice_mut<'a, const N: usize, T>(
    pos: usize,
    arr: &'a mut [[T; 15]; N],
) -> &'a mut [T; 16] {
    debug_assert!(pos + 15 < 15 * N);

    let ptr: *mut [T; 15] = arr.as_mut_ptr();
    let ptr: *mut T = ptr.cast();
    let ptr = unsafe { ptr.add(pos) };
    let ptr: *mut [T; 16] = ptr.cast();
    unsafe { ptr.as_mut_unchecked() }
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
pub fn mask_to_swizzle<S: fearless_simd::Simd>(
    simd: S,
    mask: fearless_simd::mask8x16<S>
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

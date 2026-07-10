/// 16 parallel byte lookups from a small table.
/// Contract: table.len() >= 32 (pad it), all idx[i] < table.len().
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u8x16(table: &[u8; 32], idx: &[u8; 16]) -> [u8; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u8x16(table: &[u8; 32], idx: &[u8; 16]) -> [u8; 16] {
    use std::arch::aarch64::*;
    unsafe {
        // Load 32 bytes as two 128-bit lanes
        let t = vld1q_u8_x2(table.as_ptr());
        // Load indices
        let i = vld1q_u8(idx.as_ptr());
        // Parallel table lookup
        let res = vqtbl2q_u8(t, i);
        // Store result
        let mut out = [0u8; 16];
        vst1q_u8(out.as_mut_ptr(), res);
        out
    }
}

/// 16 parallel u16 lookups from one 64-byte table.
/// Contract: all idx[i] < 32 relative to `table`'s base.
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u16x16(table: &[u16; 32], idx: &[u8; 16]) -> [u16; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u16x16(table: &[u16; 32], idx: &[u8; 16]) -> [u16; 16] {
    use std::arch::aarch64::*;
    unsafe {
        let t = vld1q_u8_x4(table.as_ptr() as *const u8);

        let i = vld1q_u8(idx.as_ptr());
        let i_lo = vaddq_u8(i, i); // idx * 2
        let i_hi = vaddq_u8(i_lo, vdupq_n_u8(1)); // idx * 2 + 1

        let lo = vqtbl4q_u8(t, i_lo);
        let hi = vqtbl4q_u8(t, i_hi);

        // vzipq_u8 gives two halves: .0 covers idx[0..8], .1 covers idx[8..16]
        let zipped = vzipq_u8(lo, hi);
        let res_lo = vreinterpretq_u16_u8(zipped.0); // u16 results for idx[0..8]
        let res_hi = vreinterpretq_u16_u8(zipped.1); // u16 results for idx[8..16]

        let mut out = [0u16; 16];
        vst1q_u16(out.as_mut_ptr(), res_lo);
        vst1q_u16(out.as_mut_ptr().add(8), res_hi);
        out
    }
}
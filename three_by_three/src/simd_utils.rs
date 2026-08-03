use std::simd::{Mask, Select as _, Simd, cmp::SimdOrd as _, num::{SimdInt as _, SimdUint}, ptr::SimdConstPtr};

/// 16 parallel byte lookups from a small table.
/// Contract: table.len() >= 32 (pad it), all idx[i] < table.len().
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u8x16x2(table: &[u8; 32], idx: &[u8; 16]) -> [u8; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u8x16x2(table: &[u8; 32], idx: &[u8; 16]) -> [u8; 16] {
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

/// 16 parallel byte lookups from a small table.
/// Contract: table.len() >= 32 (pad it), all idx[i] < table.len().
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u8x16x3(table: &[u8; 48], idx: &[u8; 16]) -> [u8; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u8x16x3(table: &[u8; 48], idx: &[u8; 16]) -> [u8; 16] {
    use core::arch::aarch64::*;
    // SAFETY: the `neon` feature is guaranteed by the cfg gate above, so all
    // intrinsics below are available. Loads/stores are 16-byte reads/writes
    // from/to arrays that are large enough (48 and 16 bytes respectively).
    unsafe {
        // ld1 {v0.16b, v1.16b, v2.16b}, [table]  -- 48-byte table in 3 regs
        let t = vld1q_u8_x3(table.as_ptr());
        // ld1 {v3.16b}, [idx]
        let i = vld1q_u8(idx.as_ptr());
        // tbl v0.16b, {v0.16b, v1.16b, v2.16b}, v3.16b
        // idx >= 48 -> 0, but the contract guarantees idx[i] < 48.
        let r = vqtbl3q_u8(t, i);

        let mut out = [0u8; 16];
        vst1q_u8(out.as_mut_ptr(), r);
        out
    }
}

/// 16 parallel u16 lookups from one 64-byte table.
/// Contract: all idx[i] < 32 relative to `table`'s base.
#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn lut_u16x16x3(table: &[u16; 24], idx: &[u8; 16]) -> [u16; 16] {
    idx.map(|i| table[i as usize])
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn lut_u16x16x3(table: &[u16; 24], idx: &[u8; 16]) -> [u16; 16] {
    use std::arch::aarch64::*;
    unsafe {
        let t = vld1q_u8_x3(table.as_ptr() as *const u8);

        let i = vld1q_u8(idx.as_ptr());
        let i_lo = vaddq_u8(i, i); // idx * 2
        let i_hi = vaddq_u8(i_lo, vdupq_n_u8(1)); // idx * 2 + 1

        let lo = vqtbl3q_u8(t, i_lo);
        let hi = vqtbl3q_u8(t, i_hi);

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

#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
#[inline(always)]
pub fn combine(
    a: [u16; 16],
    b: [u16; 16],
    c: [u8; 16],
    d: [u8; 16],
    e: [u8; 16],
    f: [u8; 16],
) -> [[u8; 8]; 16] {
    let mut out = [[0u8; 8]; 16];
    for i in 0..16 {
        out[i][0] = a[i] as u8;
        out[i][1] = (a[i] >> 8) as u8;
        out[i][2] = b[i] as u8;
        out[i][3] = (b[i] >> 8) as u8;
        out[i][4] = c[i];
        out[i][5] = d[i];
        out[i][6] = e[i];
        out[i][7] = f[i];
    }
    out
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline(always)]
pub fn combine(
    a: [u16; 16],
    b: [u16; 16],
    c: [u8; 16],
    d: [u8; 16],
    e: [u8; 16],
    f: [u8; 16],
) -> [[u8; 8]; 16] {
    use core::arch::aarch64::*;
    let mut out = [[0; 8]; 16];

    unsafe {
        let a0 = vld1q_u16(a.as_ptr());
        let a1 = vld1q_u16(a.as_ptr().add(8));
        let b0 = vld1q_u16(b.as_ptr());
        let b1 = vld1q_u16(b.as_ptr().add(8));

        let cv = vld1q_u8(c.as_ptr());
        let dv = vld1q_u8(d.as_ptr());

        let ev = vld1q_u8(e.as_ptr());
        let fv = vld1q_u8(f.as_ptr());

        // Interleave d/e bytes: [d0 e0 d1 e1 ...]. Viewed as u16 lanes on a
        // little-endian target, lane i == d[i] | (e[i] << 8), i.e. byte 0 = d
        // (offset 6 in the struct), byte 1 = e (offset 7).

        let cd0 = vreinterpretq_u16_u8(vzip1q_u8(cv, dv));
        let cd1 = vreinterpretq_u16_u8(vzip2q_u8(cv, dv));

        let ef0 = vreinterpretq_u16_u8(vzip1q_u8(ev, fv));
        let ef1 = vreinterpretq_u16_u8(vzip2q_u8(ev, fv));

        // st4 writes lanes interleaved: a0 b0 c0 de0 a1 b1 c1 de1 ...
        // which is exactly the AoS layout of SplitPhase1NodeA.
        let base = out.as_mut_ptr() as *mut u16;
        vst4q_u16(base, uint16x8x4_t(a0, b0, cd0, ef0));
        vst4q_u16(base.add(32), uint16x8x4_t(a1, b1, cd1, ef1));
    }

    out
}


#[inline(always)]
pub unsafe fn gather16(table: &[u8], offsets: Simd<u32, 16>) -> Simd<u8, 16> {
    let base = Simd::splat(table.as_ptr());
    let base_addr = base.addr();
    let offsets = offsets.cast::<usize>();

    unsafe { Simd::gather_ptr(Simd::with_addr(base, base_addr + offsets)) }
}

#[inline(always)]
pub fn mask_to_backpointers(mask: Mask<i8, 16>) -> Simd<u8, 16> {
    const IDX: Simd<i8, 16> =
    Simd::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

    // Lane i holds i if mask[i] is set, else -1.
    let v = mask.select(IDX, Simd::splat(-1));

    // We want the last set index *strictly before* each lane, so shift by one
    // (shifting -1 in at the bottom), then prefix-max in log2(16) = 4 steps.
    let v = v.shift_elements_right::<1>(-1);
    let v = v.simd_max(v.shift_elements_right::<1>(-1));
    let v = v.simd_max(v.shift_elements_right::<2>(-1));
    let v = v.simd_max(v.shift_elements_right::<4>(-1));
    let v = v.simd_max(v.shift_elements_right::<8>(-1));

    // i - j; with j = -1 this gives i + 1, matching the scalar version.
    (IDX - v).cast::<u8>()
}

pub fn mask_to_backpointers_expected(mask: Mask<i8, 16>) -> Simd<u8, 16> {
    let mut since_last = 1;
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = since_last;
        if mask.test(i) {
            since_last = 0;
        }
        since_last += 1;
    }

    Simd::from_array(out)
}

#[test]
fn mask_to_backpointers_test() {
    for i in 0..u16::MAX {
        let mask = Mask::from_bitmask(i as u64);
        assert_eq!(mask_to_backpointers_expected(mask), mask_to_backpointers(mask));
    }
}
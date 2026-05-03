use std::{num::NonZeroU8, ops::RangeBounds};

use crate::Tables;




// 5 bits of distance, 3 bits of moves since measured.
// the range of possible values defined is less than or equal to N at all times.

// this always assumes the cube beind described is not domino reduced (distance > 0) since that's observable
// without any table lookups.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DistanceBookkeeping<const N: u8>(NonZeroU8);

impl<const N: u8> DistanceBookkeeping<N> {
    #[inline(always)]
    pub fn new(distance: u8) -> Option<Self> {
        debug_assert!(distance == (distance & 0b00011111));
        NonZeroU8::new(distance).map(Self)
    }

    #[inline(always)]
    pub unsafe fn new_unchecked(inner: u8) -> Self {
        Self(unsafe { NonZeroU8::new_unchecked(inner) })
    }

    #[inline(never)]
    fn sharpen(self, mod_n: u8) -> u8 {
        let center = self.0.get() & 0b00011111;
        let radius = self.0.get() >> 5;

        let current_max_possible = center + radius;
        let current_min_possible = center.saturating_sub(radius + 1) + 1; // min(center - radius, 1)

        let center_new = (current_min_possible..=current_max_possible).find(|i| i % N == mod_n);

        match center_new {
            Some(center) => center,
            None => {
                panic!("{mod_n} {center} {radius}")
            },
        }
    }

    #[inline(never)]
    pub fn widen(self, get_pre_widen_mod_n_dist: impl FnOnce() -> u8) -> Self {
        let mut center = self.0.get() & 0b00011111;
        let mut radius = self.0.get() >> 5;

        let next_min_possible = center.saturating_sub(radius + 2) + 1;
        let next_max_possible = center + radius + 1;

        let next_range = next_max_possible + 1 - next_min_possible;

        if next_range > N || radius == 7 {
            let sharp = self.sharpen(get_pre_widen_mod_n_dist());

            center = sharp;
            radius = 1;
        } else {
            radius += 1;
        }

        Self(unsafe { NonZeroU8::new_unchecked(center | (radius << 5)) })
    }

    #[inline(never)]
    // SAFETY: the actual cube can't be domino reduced. check for that first since the check is cheap.
    pub unsafe fn test(&mut self, max: u8, get_mod_n_dist: impl FnOnce() -> u8) -> bool {
        let center = self.0.get() & 0b00011111;
        let radius = self.0.get() >> 5;

        let current_max_possible = center + radius;
        let current_min_possible = center.saturating_sub(radius + 1) + 1; // min(center - radius, 1)

        if current_max_possible <= max {
            return true;
        }

        // not really possible?
        if current_min_possible > max {
            return false;
        }

        let sharp = self.sharpen(get_mod_n_dist());
        debug_assert_ne!(sharp, 0);

        *self = Self(unsafe { NonZeroU8::new_unchecked(sharp)});
 
        sharp <= max
    }
}

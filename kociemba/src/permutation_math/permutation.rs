use rand::distr::Distribution;
use rand::distr::StandardUniform;
use rand::seq::SliceRandom;

/// A permutation, represented by which element of the identity permutation (0, 1, 2, 3, .., N-1)
/// resides in the slot at each index, after the permutation is applied.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Permutation<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for Permutation<N> {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<const N: usize> From<Permutation<N>> for [u8; N] {
    fn from(val: Permutation<N>) -> Self {
        val.const_into_array()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for Permutation<N> {
    type Error = [u8; N];

    fn try_from(array: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_const_from_array(array)
    }
}

impl<const N: usize> Permutation<N> {
    const _ASSERT_N_FITS: () = {
        // if the condition is false, this `panic!` aborts const‑eval with an error
        assert!(N <= u8::MAX as usize, "N must fit in a u8");
    };

    pub const IDENTITY: Self = {
        let mut out = [0u8; N];
        let mut i = 0;
        while i < N {
            out[i] = i as u8;
            i += 1;
        }
        Self(out)
    };

    const fn is_valid(array: [u8; N]) -> bool {
        let mut seen = [false; N];
        let mut i = 0;
        while i < N {
            let entry = array[i] as usize;
            if entry >= N {
                return false;
            }
            if seen[entry] {
                return false;
            }
            seen[entry] = true;
            i += 1;
        }

        true
    }

    pub const fn const_into_array(self) -> [u8; N] {
        self.0
    }

    pub const fn try_const_from_array(array: [u8; N]) -> Result<Self, [u8; N]> {
        if Self::is_valid(array) {
            Ok(Self(array))
        } else {
            Err(array)
        }
    }

    pub const fn const_from_array(array: [u8; N]) -> Self {
        match Self::try_const_from_array(array) {
            Ok(val) => val,
            Err(_) => panic!("invalid permutation array"),
        }
    }

    pub const unsafe fn const_from_array_unchecked(array: [u8; N]) -> Self {
        debug_assert!(Self::is_valid(array));
        Self(array)
    }

    /// Apply this permutation to the array `other` in-place by decomposing into cycles
    /// and swapping elements along each cycle (no extra buffer needed).
    pub const fn apply_to<T>(self, other: &mut [T; N]) {
        // get the inverse permutation
        let inv_perm = self.invert();
        let inv = inv_perm.0;
        // track visited indices
        let mut visited = [false; N];
        let mut start = 0;
        // for each cycle
        while start < N {
            if !visited[start] {
                visited[start] = true;
                let mut next = inv[start] as usize;
                while next != start {
                    visited[next] = true;
                    // swap the values at positions `start` and `next`
                    unsafe { other.swap_unchecked(start, next) };
                    next = inv[next] as usize;
                }
            }
            start += 1;
        }
    }

    /// Return the permutation that first does `self`, then `other`.
    pub const fn then(mut self, other: Self) -> Self {
        other.apply_to(&mut self.0);
        self
    }

    /// Invert this permutation: find the permutation that undoes `self`.
    pub const fn invert(self) -> Self {
        let mut inv = [0u8; N];
        let mut i = 0;
        while i < N {
            inv[self.0[i] as usize] = i as u8;
            i += 1;
        }
        Permutation(inv)
    }

    pub const fn is_odd(self) -> bool {
        let mut seen = [false; N];
        let mut parity = 0;

        let mut i = 0;
        while i < N {
            if !seen[i] {
                let mut len = 0;
                let mut j = i;
                while !seen[j] {
                    seen[j] = true;
                    j = self.0[j] as usize;
                    len += 1;
                }
                if len > 0 {
                    parity ^= (len - 1) & 1;
                }
            }
            i += 1;
        }

        parity == 1
    }

    pub const fn const_eq(self, other: Self) -> bool {
        let mut i = 1;
        while i < N {
            if self.0[i] != other.0[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl<const N: usize> Distribution<Permutation<N>> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Permutation<N> {
        let mut out = Permutation::IDENTITY;
        out.0.as_mut_slice().shuffle(rng);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::Permutation;

    // helper to check that identity permutes nothing
    #[test]
    fn identity_leaves_array_unchanged() {
        let mut a = [10, 20, 30, 40];
        let p = Permutation::<4>::IDENTITY;
        p.apply_to(&mut a);
        assert_eq!(a, [10, 20, 30, 40]);

        let mut s = ['a', 'b', 'c'];
        Permutation::<3>::IDENTITY.apply_to(&mut s);
        assert_eq!(s, ['a', 'b', 'c']);
    }

    // N=0 and N=1 corner cases
    #[test]
    fn small_ns() {
        let mut empty: [u8; 0] = [];
        Permutation::<0>::IDENTITY.apply_to(&mut empty);
        assert_eq!(empty, []);

        let mut single = [99u8; 1];
        Permutation::<1>::IDENTITY.apply_to(&mut single);
        assert_eq!(single, [99]);
    }

    // test invert: for a 3‑cycle, invert should reverse it
    #[test]
    fn invert_3_cycle() {
        // cycle (0→2→1→0)
        let p = Permutation::<3>([2, 0, 1]);
        let inv = p.invert();
        // p then inv = identity
        let composed = p.then(inv);
        assert_eq!(composed, Permutation::IDENTITY);

        // inv.then(p) also identity
        assert_eq!(inv.then(p), Permutation::IDENTITY);
    }

    // test apply_to actually reorders in‑place by comparing with a freshly built array
    #[test]
    fn apply_to_matches_out_of_place() {
        let perm = Permutation::<5>([4, 2, 0, 1, 3]);
        let mut in_place = [0, 10, 20, 30, 40];
        perm.apply_to(&mut in_place);

        // out‑of‑place result
        let mut out = [0; 5];
        for (i, o) in out.iter_mut().enumerate() {
            *o = [0, 10, 20, 30, 40][perm.0[i] as usize];
        }

        assert_eq!(in_place, out);
    }

    // test that double invert returns original
    #[test]
    fn double_invert_is_identity() {
        let perms = [
            Permutation::<4>([1, 3, 0, 2]),
            Permutation::<4>([3, 2, 1, 0]),
            Permutation::<4>([0, 1, 2, 3]),
        ];
        for &p in &perms {
            assert_eq!(p.invert().invert(), p);
        }
    }
}

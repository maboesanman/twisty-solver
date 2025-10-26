use crate::{
    cube_ops::partial_reprs::edge_perm::EdgePerm,
    permutation_math::{grouping::EdgeCombination, permutation::Permutation},
};

use super::{e_edge_perm::EEdgePerm, edge_group::EdgeGroup, ud_edge_perm::UDEdgePerm};

impl EdgePerm {
    pub const fn split(self) -> (EdgeGroup, UDEdgePerm, EEdgePerm) {
        let g = self.get_grouping();

        let divided = self.0.then(g.0.into_representative_even_perm().invert()).0;

        let mut ud = [0; 8];
        let mut e = [0; 4];

        let mut i = 0;
        while i < 8 {
            ud[i] = divided[i];
            i += 1;
        }
        while i < 12 {
            e[i - 8] = divided[i] - 8;
            i += 1;
        }

        (
            g,
            UDEdgePerm(unsafe { Permutation::const_from_array_unchecked(ud) }),
            EEdgePerm(unsafe { Permutation::const_from_array_unchecked(e) }),
        )
    }

    pub const fn join(group: EdgeGroup, ud: UDEdgePerm, e: EEdgePerm) -> Self {
        let mut perm = [0u8; 12];

        let mut i = 0;
        while i < 8 {
            perm[i] = ud.0.0[i];
            i += 1;
        }
        while i < 12 {
            perm[i] = e.0.0[i - 8] + 8;
            i += 1;
        }

        Self(Permutation(perm).then(group.0.into_representative_even_perm()))
    }

    pub const fn get_grouping(self) -> EdgeGroup {
        let mut g = [false; 12]; // false = UD slot, true = E slot

        let mut i = 0;
        while i < 12 {
            let val = self.0.0[i];
            let is_e = val > 7;
            g[i] = is_e;
            i += 1;
        }

        EdgeGroup(unsafe { EdgeCombination::const_from_array_unchecked(g) })
    }
}

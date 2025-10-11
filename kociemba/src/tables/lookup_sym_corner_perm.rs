use std::path::Path;

use anyhow::Result;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::cube_ops::{
    coords::{CornerPermRawCoord, CornerPermSymCoord},
    corner_perm_combo_coord::CornerPermComboCoord,
    cube_sym::DominoSymmetry,
    partial_reprs::corner_perm::CornerPerm,
};

use super::table_loader::{
    as_u16_slice, as_u16_slice_mut, collect_unique_sorted_parallel, load_table,
};

const TABLE_SIZE_BYTES: usize = 2768 * 2;
const FILE_CHECKSUM: u32 = 188933558;

static STABILIZING_CONJUGATIONS: phf::Map<u16, u16> = phf::phf_map! {
    2709 | 1485 | 2645 | 1611 | 9 | 333 | 2695 => 9225,
    2543 | 1142 | 2014 | 2616 | 2083 | 2637 | 2683 | 1355 | 394 | 405 | 1109 | 1847 | 2665 | 153 | 2184 | 548 | 505 | 2617 | 1978 | 1441 | 2465 | 1877 | 861 | 1636 | 712 | 1737 | 2433 | 2544 | 2138 | 926 | 1300 | 1997 | 2573 | 1669 | 685 | 2643 | 2699 | 2744 => 257,
    2750 | 1700 => 52275,
    2574 | 2320 => 8721,
    2720 | 2678 | 1602 | 324 | 1478 | 2692 => 26265,
    2576 | 2322 => 17425,
    2702 | 6 => 255,
    329 | 1480 | 2745 | 1604 | 2691 | 2719 | 1477 | 1603 | 3 | 1479 => 153,
    2208 | 2566 | 2575 | 1855 | 2363 | 2759 | 2364 | 2476 | 2361 | 5 | 2765 | 2479 | 2199 | 2201 | 2721 | 1699 => 33,
    2068 | 2762 => 43605,
    2568 | 2760 | 1902 | 2360 => 8481,
    2700 | 1702 | 2752 | 4 => 51,
    234 | 2650 | 164 | 2710 | 1752 | 66 | 2648 | 1794 => 12291,
    2754 | 2206 | 2474 | 1896 => 4641,
    7 | 2767 | 2723 | 2069 => 85,
    1885 | 1970 | 29 | 1186 | 652 | 839 | 386 | 1839 | 2689 | 1949 | 2073 | 2493 | 579 | 2675 | 2377 | 1267 | 2366 | 2233 | 798 | 1709 | 2525 | 1111 | 2711 | 1417 | 1124 | 2277 | 910 | 1541 | 2216 | 2140 | 463 | 357 | 1526 | 2092 | 2736 | 2006 | 81 | 2415 => 4097,
    1698 | 2748 => 13107,
    601 => 1665,
    1188 | 976 | 1064 | 2738 | 1333 | 2715 | 931 | 2032 | 2220 | 2243 | 431 | 2178 | 2607 | 1035 | 99 | 483 | 388 | 696 | 1907 | 1938 | 2455 | 2703 | 1835 | 2391 | 1891 | 1715 | 1308 | 2583 | 2368 | 2128 | 2075 | 2325 | 528 | 1231 => 8193,
    1503 | 53 | 2693 | 367 | 2707 | 1627 | 2641 => 16905,
    691 => 24705,
    64 | 1948 | 1719 | 2638 | 2198 | 832 | 1516 | 2448 | 2035 | 2084 | 2600 | 1900 | 1697 | 2708 | 2611 | 2401 | 2758 | 990 | 1033 | 2671 | 280 | 765 | 2730 | 109 | 2257 | 1070 | 748 | 2354 | 2114 | 2661 | 2077 | 1863 | 2461 | 2589 | 1654 | 929 | 1853 | 2042 | 1583 | 1822 | 2337 | 380 | 439 => 16385,
    2696 | 0 | 2766 | 2634 => 65535,
    2704 | 2642 | 304 | 1836 | 60 | 2636 | 320 | 1846 => 771,
    2409 | 2681 | 2519 | 1407 => 34833,
    1491 | 365 | 1488 | 341 | 368 | 1506 | 342 | 1623 | 1500 | 48 | 1624 | 49 | 20 | 1613 | 52 | 1614 | 21 | 10 | 2682 | 1490 | 1501 | 2680 => 9,
    2079 | 1023 | 984 | 2034 | 508 | 2503 | 606 | 2627 | 2705 | 2160 | 1833 | 2297 | 1762 | 1292 | 1329 | 2559 | 1940 | 411 | 1893 | 1239 | 2646 | 2345 | 479 | 1208 | 2734 | 384 | 2192 | 2593 | 1909 | 1060 | 2713 | 125 | 182 | 1725 | 2270 | 947 | 2482 => 1025,
    131 | 812 | 2098 | 2599 | 2081 | 415 | 767 | 2307 | 2546 | 1074 | 992 | 2669 | 1019 | 2038 | 2620 | 1727 | 2146 | 1579 | 2561 | 1638 | 1695 | 2351 | 2631 | 943 | 2033 | 378 | 1851 | 2513 | 2728 | 1534 | 2657 | 1857 | 1946 | 660 => 513,
    598 | 690 => 129,
    2518 | 2410 | 1408 | 2670 => 4369,
    330 | 2726 => 39321,
    2070 | 2764 => 21845,
    2729 | 2737 | 1819 | 1825 | 2647 | 73 | 181 | 322 | 245 | 170 | 65 | 74 | 1749 | 1747 | 1796 | 2763 | 237 | 1785 | 298 | 293 | 309 | 228 | 2717 | 1842 | 314 | 183 | 292 | 1823 | 1829 | 308 | 243 | 299 | 1787 | 1798 | 306 | 1758 | 2716 | 1815 | 1703 | 175 | 2735 | 2718 | 169 | 1760 | 72 | 315 | 2655 | 321 | 242 | 1838 | 305 | 1828 | 1832 | 71 | 240 | 1751 | 1 | 176 | 307 | 178 => 3,
    2411 | 1701 | 2318 | 2579 | 2520 | 325 | 2319 | 2724 | 2412 | 2761 | 2317 | 1416 | 1412 | 1411 | 1414 | 2572 | 328 | 2413 | 1599 | 1403 | 2414 | 2315 | 1413 | 2741 | 1474 | 1600 | 2521 | 1475 => 17,
};

pub struct LookupSymCornerPermTable(Mmap);

impl LookupSymCornerPermTable {
    pub fn get_rep_from_sym(&self, sym_coord: CornerPermSymCoord) -> CornerPermRawCoord {
        let buffer = as_u16_slice(&self.0);
        let (even, odd) = buffer.split_at(2768 / 2);

        let buffer = if sym_coord.0.is_multiple_of(2) {
            even
        } else {
            odd
        };

        CornerPermRawCoord(buffer[sym_coord.0 as usize / 2])
    }

    pub fn get_raw_from_combo(&self, combo_coord: CornerPermComboCoord) -> CornerPermRawCoord {
        CornerPerm::from_coord(self.get_rep_from_sym(combo_coord.sym_coord))
            .domino_conjugate(combo_coord.domino_conjugation.inverse())
            .into_coord()
    }

    pub fn get_combo_from_raw(&self, raw_coord: CornerPermRawCoord) -> CornerPermComboCoord {
        let buffer = as_u16_slice(&self.0);
        let (even, odd) = buffer.split_at(2768 >> 1);
        let corner_perm = CornerPerm::from_coord(raw_coord);
        let (rep_coord, sym) = DominoSymmetry::all_iter()
            .map(|sym| (corner_perm.domino_conjugate(sym).into_coord(), sym))
            .min_by_key(|x| x.0)
            .unwrap();

        // index within its parity half
        let pos_in_half = if raw_coord.0.is_multiple_of(2) {
            even.binary_search(&rep_coord.0).unwrap()
        } else {
            odd.binary_search(&rep_coord.0).unwrap()
        };

        // pack: (pos << 1) | parity
        let packed = ((pos_in_half as u16) << 1) | (raw_coord.0 & 1);

        CornerPermComboCoord {
            sym_coord: CornerPermSymCoord(packed),
            domino_conjugation: sym,
        }
    }

    /// includes the identity
    pub fn get_all_stabilizing_conjugations(
        sym_coord: CornerPermSymCoord,
    ) -> impl IntoIterator<Item = DominoSymmetry> {
        let mask = STABILIZING_CONJUGATIONS
            .get(&sym_coord.0)
            .copied()
            .unwrap_or(1);

        DominoSymmetry::all_iter().filter(move |sym| (mask >> sym.0) & 1 == 1)
    }

    fn generate(buffer: &mut [u8]) {
        let buffer = as_u16_slice_mut(buffer);
        let (even, odd) = buffer.split_at_mut(2768 >> 1);

        let even_reps = (0..(40320 >> 1)).into_par_iter().map(|i| {
            let raw_coord = CornerPermRawCoord(i << 1);
            let corner_perm = CornerPerm::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| corner_perm.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        let odd_reps = (0..(40320 >> 1)).into_par_iter().map(|i| {
            let raw_coord = CornerPermRawCoord((i << 1) + 1);
            let corner_perm = CornerPerm::from_coord(raw_coord);
            DominoSymmetry::all_iter()
                .map(|sym| corner_perm.domino_conjugate(sym).into_coord())
                .min()
                .unwrap()
        });

        for (i, rep) in collect_unique_sorted_parallel(even_reps).enumerate() {
            even[i] = rep.0
        }

        for (i, rep) in collect_unique_sorted_parallel(odd_reps).enumerate() {
            odd[i] = rep.0
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        load_table(path, TABLE_SIZE_BYTES, FILE_CHECKSUM, |buf| {
            Self::generate(buf)
        })
        .map(Self)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use itertools::Itertools;

    use crate::tables::Tables;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_corner_perm;

        (0..40320).into_iter().for_each(|i| {
            let raw_coord = CornerPermRawCoord(i);
            let corner_perm = CornerPerm::from_coord(raw_coord);

            let CornerPermComboCoord {
                sym_coord,
                domino_conjugation: sym,
            } = table.get_combo_from_raw(raw_coord);
            let updated_raw = corner_perm.domino_conjugate(sym).into_coord();
            let rep_coord = table.get_rep_from_sym(sym_coord);

            assert_eq!(rep_coord, updated_raw)
        });

        Ok(())
    }

    #[test]
    fn test_parity_preserved() -> Result<()> {
        let tables = Tables::new("tables")?;

        let table = &tables.lookup_sym_corner_perm;

        (0..2768).into_iter().for_each(|i| {
            let sym_coord = CornerPermSymCoord(i);
            let raw_coord = table.get_rep_from_sym(sym_coord);
            let corner_perm = CornerPerm::from_coord(raw_coord);

            assert_eq!(corner_perm.0.is_odd(), i & 0b1 == 1);
        });

        Ok(())
    }

    #[test]
    fn test_stabilizing_conjugations() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        (0..2768).into_par_iter().for_each(|i| {
            let sym = CornerPermSymCoord(i);
            let rep = tables.lookup_sym_corner_perm.get_rep_from_sym(sym);
            let group_orient = CornerPerm::from_coord(rep);
            for s in LookupSymCornerPermTable::get_all_stabilizing_conjugations(sym) {
                assert_eq!(group_orient, group_orient.domino_conjugate(s))
            }
        });

        Ok(())
    }

    #[test]
    fn check_for_stabilizing_conj() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        // 444 of the 2768 sym coords have nontrivial stabilizing symmetries
        // there are 34 possible cardinalities

        let nonzero_count: HashMap<_, _> = (0..2768)
            .into_par_iter()
            .map(|i| {
                let sym = CornerPermSymCoord(i);
                let rep = tables.lookup_sym_corner_perm.get_rep_from_sym(sym);
                let perm = CornerPerm::from_coord(rep);

                (
                    sym.0,
                    DominoSymmetry::all_iter().fold(0u16, |acc, sym| {
                        acc | ((perm == perm.domino_conjugate(sym)) as u16) << sym.0
                    }),
                )
            })
            .filter(|x| x.1 != 1)
            .collect();

        let mut reversed: HashMap<u16, Vec<u16>> = HashMap::new();

        for (k, v) in nonzero_count {
            reversed.entry(v).or_default().push(k);
        }

        let mut out_string =
            "static STABILIZING_CONJUGATIONS: phf::Map<u16, u16> = phf::phf_map! {\n".to_string();
        for (k, v) in reversed {
            out_string.push_str(&format!(
                "    {} => {},\n",
                v.into_iter().map(|x| format!("{x}")).join(" | "),
                k
            ));
        }
        out_string.push_str("};");

        println!("{out_string}");

        Ok(())
    }
}

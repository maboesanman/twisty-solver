use crate::{
    cube_ops::{
        coords::{
            CornerOrientRawCoord, CornerPermSymCoord, EEdgePermRawCoord, EdgeGroupOrientRawCoord,
            EdgeGroupOrientSymCoord, UDEdgePermRawCoord,
        },
        corner_perm_combo_coord::CornerPermComboCoord,
        cube_move::{CubeMove, DominoMove},
        cube_sym::DominoSymmetry,
        edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
        partial_reprs::{
            corner_orient::CornerOrient, corner_perm::CornerPerm, e_edge_perm::EEdgePerm,
            edge_group::EdgeGroup, edge_orient::EdgeOrient, edge_perm::EdgePerm,
            ud_edge_perm::UDEdgePerm,
        },
        repr_cube::ReprCube,
    },
    tables::Tables,
};

// there are 4 items:
// [AAAA_AAAA_AAAA_AAAA] A: edge_group_orient_sym_coord
// [BBBB_CCCC_CCCC_CCCC] B: corner_perm_combo_coord.domino_symmetry; C: corner_orient_coord
// [DDDD_EEEE_EEEE_EEEE] D: 4 high bits of e_edge_perm; E: corner_perm_combo_coord.sym_coord
// [FFFF_FFFF_FFFF_FFFF] F: ud_edge_perm_coord
//
// notes:
// B and C == 0 => phase 1 solved. at which point we can apply A inverse to D and truncate. to DE

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct SymReducedRepr(pub [u16; 4]);

pub enum Unpacked {
    Phase1(Phase1Unpacked),
    Phase2(Phase2Unpacked),
}

impl SymReducedRepr {
    fn unpack_phase_1(self) -> Phase1Unpacked {
        let [a, bc, de, f] = self.0;
        let b = (bc >> 12) as u8;
        let c = bc & 0x0FFF;
        let d = (((de >> 11) & 0b11110) | ((de ^ f) & 0b1)) as u8;
        let e = de & 0x0FFF;

        Phase1Unpacked {
            edge_group_orient_combo_coord: EdgeGroupOrientComboCoord {
                sym_coord: EdgeGroupOrientSymCoord(a),
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            corner_orient_raw_coord: CornerOrientRawCoord(c),
            corner_perm_combo_coord: CornerPermComboCoord {
                sym_coord: CornerPermSymCoord(e),
                domino_conjugation: DominoSymmetry(b),
            },
            e_edge_perm_raw_coord: EEdgePermRawCoord(d),
            ud_edge_perm_raw_coord: UDEdgePermRawCoord(f),
        }
    }

    fn unpack_phase_2(self) -> Phase2Unpacked {
        let [_, _, de, f] = self.0;
        let d = (((de >> 11) & 0b11110) | ((de ^ f) & 0b1)) as u8;
        let e = de & 0x0FFF;

        Phase2Unpacked {
            corner_perm_combo_coord: CornerPermComboCoord {
                sym_coord: CornerPermSymCoord(e),
                domino_conjugation: DominoSymmetry::IDENTITY,
            },
            e_edge_perm_raw_coord: EEdgePermRawCoord(d),
            ud_edge_perm_raw_coord: UDEdgePermRawCoord(f),
        }
    }

    pub fn is_solved(self) -> bool {
        self.0 == [0; 4]
    }

    pub fn is_phase_1_solved(self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }

    pub fn unpack(self) -> Unpacked {
        match self.0 {
            [0, 0, _, _] => Unpacked::Phase2(self.unpack_phase_2()),
            [_, _, _, _] => Unpacked::Phase1(self.unpack_phase_1()),
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        match self.unpack() {
            Unpacked::Phase1(phase1_unpacked) => phase1_unpacked.into_cube(tables),
            Unpacked::Phase2(phase2_unpacked) => phase2_unpacked.into_cube(tables),
        }
    }

    pub fn prune_distance_phase_1(self, tables: &Tables) -> u8 {
        let [a, bc, ..] = self.0;
        let c = bc & 0x0FFF;

        let edge_group_orient_sym_coord = EdgeGroupOrientSymCoord(a);
        let corner_orient_raw_coord = CornerOrientRawCoord(c);
        tables
            .get_prune_phase_1()
            .get_value(edge_group_orient_sym_coord, corner_orient_raw_coord)
    }

    pub fn prune_distance_phase_2(self, tables: &Tables) -> u8 {
        let [_, _, de, f] = self.0;
        let e = de & 0x0FFF;

        let corner_perm_sym_coord = CornerPermSymCoord(e);
        let ud_edge_perm_raw_coord = UDEdgePermRawCoord(f);

        tables
            .get_prune_phase_2()
            .get_value(corner_perm_sym_coord, ud_edge_perm_raw_coord)
    }

    // 18 neighbors in phase 1,
    pub fn neighbors(self, tables: &Tables) -> impl IntoIterator<Item = Self> {
        enum EitherIter<A, B> {
            A(A),
            B(B),
        }

        impl<A, B> Iterator for EitherIter<A, B>
        where
            A: Iterator,
            B: Iterator<Item = A::Item>,
        {
            type Item = A::Item;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    EitherIter::A(a) => a.next(),
                    EitherIter::B(b) => b.next(),
                }
            }
        }

        match self.unpack() {
            Unpacked::Phase1(phase1_unpacked) => {
                EitherIter::A(CubeMove::all_iter().map(move |cube_move| {
                    phase1_unpacked
                        .apply_cube_move(tables, cube_move)
                        .pack(tables)
                }))
            }
            Unpacked::Phase2(phase2_unpacked) => {
                EitherIter::B(DominoMove::all_iter().map(move |domino_move| {
                    phase2_unpacked
                        .apply_domino_move(tables, domino_move)
                        .pack(tables)
                }))
            }
        }
    }

    pub fn from_cube(cube: ReprCube, tables: &Tables) -> Self {
        Phase1Unpacked::from_cube(cube, tables).pack(tables)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Phase1Unpacked {
    pub edge_group_orient_combo_coord: EdgeGroupOrientComboCoord,
    pub corner_orient_raw_coord: CornerOrientRawCoord,
    pub corner_perm_combo_coord: CornerPermComboCoord,
    pub e_edge_perm_raw_coord: EEdgePermRawCoord,
    pub ud_edge_perm_raw_coord: UDEdgePermRawCoord,
}

impl Phase1Unpacked {
    pub fn apply_cube_move(self, tables: &Tables, cube_move: CubeMove) -> Self {
        let edge_group_orient_combo_coord = self
            .edge_group_orient_combo_coord
            .apply_cube_move(tables, cube_move);

        let (group, _) = tables
            .lookup_sym_edge_group_orient
            .get_raw_from_combo(self.edge_group_orient_combo_coord)
            .split();

        let corner_orient_raw_coord = tables
            .move_raw_corner_orient
            .apply_cube_move(self.corner_orient_raw_coord, cube_move);

        let corner_perm_combo_coord = self
            .corner_perm_combo_coord
            .apply_cube_move(tables, cube_move);

        let (_, ud_edge_perm_raw_coord, e_edge_perm_raw_coord) =
            tables.grouped_edge_moves.update_edge_perms_cube_move(
                group,
                cube_move,
                self.ud_edge_perm_raw_coord,
                self.e_edge_perm_raw_coord,
            );

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn domino_conjugate(self, tables: &Tables, sym: DominoSymmetry) -> Self {
        if sym == DominoSymmetry::IDENTITY {
            return self;
        }

        let edge_group_orient_combo_coord =
            self.edge_group_orient_combo_coord.domino_conjugate(sym);

        let (group, _) = tables
            .lookup_sym_edge_group_orient
            .get_raw_from_combo(self.edge_group_orient_combo_coord)
            .split();

        let corner_orient_raw_coord = tables
            .move_raw_corner_orient
            .domino_conjugate(self.corner_orient_raw_coord, sym);

        let corner_perm_combo_coord = self.corner_perm_combo_coord.domino_conjugate(sym);

        let (_, ud_edge_perm_raw_coord, e_edge_perm_raw_coord) = tables
            .grouped_edge_moves
            .update_edge_perms_domino_conjugate(
                group,
                sym,
                self.ud_edge_perm_raw_coord,
                self.e_edge_perm_raw_coord,
            );

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn normalize(self, tables: &Tables) -> Self {
        let val = self.domino_conjugate(
            tables,
            self.edge_group_orient_combo_coord.domino_conjugation,
        );

        if val.corner_orient_raw_coord.0 == 0 && val.edge_group_orient_combo_coord.sym_coord.0 == 0
        {
            let mut val =
                val.domino_conjugate(tables, self.corner_perm_combo_coord.domino_conjugation);

            val.corner_orient_raw_coord = CornerOrientRawCoord(0);
            val.edge_group_orient_combo_coord = EdgeGroupOrientComboCoord {
                domino_conjugation: DominoSymmetry::IDENTITY,
                sym_coord: EdgeGroupOrientSymCoord(0),
            };

            val
        } else {
            val
        }
    }

    pub fn pack(self, tables: &Tables) -> SymReducedRepr {
        let norm = self.normalize(tables);

        let a = norm.edge_group_orient_combo_coord.sym_coord.0;
        let b = norm.corner_perm_combo_coord.domino_conjugation.0;
        let c = norm.corner_orient_raw_coord.0;
        let d = norm.e_edge_perm_raw_coord.0 as u16;
        let e = norm.corner_perm_combo_coord.sym_coord.0;
        let f = norm.ud_edge_perm_raw_coord.0;

        let bc = ((b as u16) << 12) | c;
        let de = ((d >> 1) << 12) | e;

        SymReducedRepr([a, bc, de, f])
    }

    pub fn from_cube(cube: ReprCube, tables: &Tables) -> Self {
        let ReprCube {
            corner_perm,
            corner_orient,
            edge_perm,
            edge_orient,
        } = cube;

        let (edge_group, ud_edge_perm, e_edge_perm) = edge_perm.split();

        let edge_group_raw_coord = edge_group.into_coord();
        let edge_orient_raw_coord = edge_orient.into_coord();
        let edge_group_orient_raw_coord =
            EdgeGroupOrientRawCoord::join(edge_group_raw_coord, edge_orient_raw_coord);

        let edge_group_orient_combo_coord =
            EdgeGroupOrientComboCoord::from_raw(tables, edge_group_orient_raw_coord);
        let corner_perm_combo_coord =
            CornerPermComboCoord::from_raw(tables, corner_perm.into_coord());
        let corner_orient_raw_coord = corner_orient.into_coord();
        let ud_edge_perm_raw_coord = ud_edge_perm.into_coord();
        let e_edge_perm_raw_coord = e_edge_perm.into_coord();

        Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let Self {
            edge_group_orient_combo_coord,
            corner_orient_raw_coord,
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        } = self;

        let edge_group_orient_raw_coord = edge_group_orient_combo_coord.into_raw(tables);
        let corner_perm_raw_coord = corner_perm_combo_coord.into_raw(tables);
        let corner_perm = CornerPerm::from_coord(corner_perm_raw_coord);
        let (edge_group_raw_coord, edge_orient_raw_coord) = edge_group_orient_raw_coord.split();
        let edge_group = EdgeGroup::from_coord(edge_group_raw_coord);
        let edge_orient = EdgeOrient::from_coord(edge_orient_raw_coord);
        let ud_edge_perm = UDEdgePerm::from_coord(ud_edge_perm_raw_coord);
        let e_edge_perm = EEdgePerm::from_coord(e_edge_perm_raw_coord);
        let edge_perm = EdgePerm::join(edge_group, ud_edge_perm, e_edge_perm);
        let corner_orient = CornerOrient::from_coord(corner_orient_raw_coord);

        ReprCube {
            corner_perm,
            corner_orient,
            edge_perm,
            edge_orient,
        }
    }

    pub fn prune_dist(self, tables: &Tables) -> u8 {
        let norm = self.normalize(tables);
        tables.get_prune_phase_1().get_value(
            norm.edge_group_orient_combo_coord.sym_coord,
            norm.corner_orient_raw_coord,
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Phase2Unpacked {
    pub corner_perm_combo_coord: CornerPermComboCoord,
    pub e_edge_perm_raw_coord: EEdgePermRawCoord,
    pub ud_edge_perm_raw_coord: UDEdgePermRawCoord,
}

impl Phase2Unpacked {
    pub fn apply_domino_move(self, tables: &Tables, domino_move: DominoMove) -> Self {
        let corner_perm_combo_coord = self
            .corner_perm_combo_coord
            .apply_cube_move(tables, domino_move.into());

        let (ud_edge_perm_raw_coord, e_edge_perm_raw_coord) = tables
            .grouped_edge_moves
            .update_edge_perm_phase_2_domino_move(
                domino_move,
                self.ud_edge_perm_raw_coord,
                self.e_edge_perm_raw_coord,
            );

        Self {
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn domino_conjugate(self, tables: &Tables, sym: DominoSymmetry) -> Self {
        if sym == DominoSymmetry::IDENTITY {
            return self;
        }

        let corner_perm_combo_coord = self.corner_perm_combo_coord.domino_conjugate(sym);

        let (ud_edge_perm_raw_coord, e_edge_perm_raw_coord) = tables
            .grouped_edge_moves
            .update_edge_perm_phase_2_domino_symmetry(
                sym,
                self.ud_edge_perm_raw_coord,
                self.e_edge_perm_raw_coord,
            );

        Self {
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        }
    }

    pub fn normalize(self, tables: &Tables) -> Self {
        self.domino_conjugate(tables, self.corner_perm_combo_coord.domino_conjugation)
    }

    pub fn pack(self, tables: &Tables) -> SymReducedRepr {
        let norm = self.normalize(tables);

        let d = norm.e_edge_perm_raw_coord.0 as u16;
        let e = norm.corner_perm_combo_coord.sym_coord.0;
        let f = norm.ud_edge_perm_raw_coord.0;

        let de = ((d >> 1) << 12) | e;

        SymReducedRepr([0, 0, de, f])
    }

    pub fn into_cube(self, tables: &Tables) -> ReprCube {
        let Self {
            corner_perm_combo_coord,
            e_edge_perm_raw_coord,
            ud_edge_perm_raw_coord,
        } = self;

        let edge_group_orient_combo_coord = EdgeGroupOrientComboCoord {
            sym_coord: EdgeGroupOrientSymCoord(0),
            domino_conjugation: DominoSymmetry::IDENTITY,
        };
        let corner_orient_raw_coord = CornerOrientRawCoord(0);

        let edge_group_orient_raw_coord = edge_group_orient_combo_coord.into_raw(tables);
        let corner_perm_raw_coord = corner_perm_combo_coord.into_raw(tables);
        let corner_perm = CornerPerm::from_coord(corner_perm_raw_coord);
        let (edge_group_raw_coord, edge_orient_raw_coord) = edge_group_orient_raw_coord.split();
        let edge_group = EdgeGroup::from_coord(edge_group_raw_coord);
        let edge_orient = EdgeOrient::from_coord(edge_orient_raw_coord);
        let ud_edge_perm = UDEdgePerm::from_coord(ud_edge_perm_raw_coord);
        let e_edge_perm = EEdgePerm::from_coord(e_edge_perm_raw_coord);
        let edge_perm = EdgePerm::join(edge_group, ud_edge_perm, e_edge_perm);
        let corner_orient = CornerOrient::from_coord(corner_orient_raw_coord);

        ReprCube {
            corner_perm,
            corner_orient,
            edge_perm,
            edge_orient,
        }
    }

    pub fn prune_dist(self, tables: &Tables) -> u8 {
        let norm = self.normalize(tables);
        tables.get_prune_phase_2().get_value(
            norm.corner_perm_combo_coord.sym_coord,
            norm.ud_edge_perm_raw_coord,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn phase1_move_equivalence() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        for _ in 0..100 {
            let cube: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
            let phase1 = Phase1Unpacked::from_cube(cube, &tables);

            for mv in CubeMove::all_iter() {
                let cube_moved = cube.apply_cube_move(mv);
                let phase1_moved = phase1.apply_cube_move(&tables, mv).into_cube(&tables);
                assert_eq!(cube_moved, phase1_moved,);
            }
        }

        Ok(())
    }

    #[test]
    fn phase1_symmetry_equivalence() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(6789);

        for _ in 0..100 {
            let cube: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
            let phase1 = Phase1Unpacked::from_cube(cube, &tables);

            for sym in DominoSymmetry::nontrivial_iter() {
                let cube_conj = cube.domino_conjugate(sym);
                let phase1_conj = phase1.domino_conjugate(&tables, sym).into_cube(&tables);
                assert_eq!(cube_conj, phase1_conj,);
            }
        }

        Ok(())
    }

    #[test]
    fn pack_unpack_roundtrip_equivalence() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(99);

        for _ in 0..100 {
            let cube: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);
            let phase1 = Phase1Unpacked::from_cube(cube, &tables).normalize(&tables);

            let a = phase1.into_cube(&tables);
            let b = Phase1Unpacked::from_cube(a, &tables).pack(&tables);
            let c = match b.unpack() {
                Unpacked::Phase1(phase1_unpacked) => phase1_unpacked,
                Unpacked::Phase2(_phase2_unpacked) => panic!(),
            };
            let d = c.into_cube(&tables);

            assert_eq!(
                phase1.edge_group_orient_combo_coord.domino_conjugation,
                DominoSymmetry::IDENTITY
            );

            assert_eq!(a, d);
        }

        Ok(())
    }

    // fn get_rand_phase_2_cube(rng: &mut impl Rng, tables: &Tables) -> ReprCube {
    //     let ud = UDEdgePerm(rand::distr::Distribution::sample(
    //         &rand::distr::StandardUniform,
    //         rng,
    //     ));
    //     let c = CornerPerm(rand::distr::Distribution::sample(
    //         &rand::distr::StandardUniform,
    //         rng,
    //     ));
    //     let ud_parity = ud.0.is_odd() as u8;
    //     let c_parity = c.0.is_odd() as u8;

    //     let e = loop {
    //         let e = EEdgePerm(rand::distr::Distribution::sample(
    //             &rand::distr::StandardUniform,
    //             rng,
    //         ));
    //         let e_parity = e.0.is_odd() as u8;

    //         if (ud_parity + e_parity + c_parity) % 2 == 0 {
    //             break e;
    //         }
    //     };

    //     ReprCube {
    //         corner_perm: c,
    //         corner_orient: CornerOrient::SOLVED,
    //         edge_perm: EdgePerm::join(EdgeGroup::SOLVED, ud, e),
    //         edge_orient: EdgeOrient::SOLVED,
    //     }
    // }

    // #[test]
    // fn phase2_move_equivalence() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let mut rng = ChaCha8Rng::seed_from_u64(23456);

    //     for _ in 0..100 {
    //         let cube = get_rand_phase_2_cube(&mut rng, &tables);
    //         // We only care about Phase 2 components, so canonicalize first
    //         let phase2 = match Phase1Unpacked::from_cube(cube, &tables).pack(&tables).unpack() {
    //             Unpacked::Phase1(phase1_unpacked) => panic!(),
    //             Unpacked::Phase2(phase2_unpacked) => phase2_unpacked,
    //         };

    //         for mv in DominoMove::all_iter() {
    //             let cube_moved = cube.apply_cube_move(mv.into());
    //             let phase2_moved = phase2.apply_domino_move(&tables, mv).into_cube(&tables);
    //             assert_eq!(cube_moved, phase2_moved,);
    //         }
    //     }

    //     Ok(())
    // }

    // #[test]
    // fn phase2_symmetry_equivalence() -> anyhow::Result<()> {
    //     let tables = Tables::new("tables")?;
    //     let mut rng = ChaCha8Rng::seed_from_u64(7890);

    //     for _ in 0..100 {
    //         let cube = get_rand_phase_2_cube(&mut rng, &tables);
    //         // We only care about Phase 2 components, so canonicalize first
    //         let phase2 = match Phase1Unpacked::from_cube(cube, &tables).pack(&tables).unpack() {
    //             Unpacked::Phase1(phase1_unpacked) => panic!(),
    //             Unpacked::Phase2(phase2_unpacked) => phase2_unpacked,
    //         };

    //         for sym in DominoSymmetry::nontrivial_iter() {
    //             let cube_conj = cube.domino_conjugate(sym);
    //             let phase2_conj = phase2.domino_conjugate(&tables, sym).into_cube(&tables);
    //             assert_eq!(
    //                 cube_conj, phase2_conj,
    //                 "Mismatch for symmetry {:?}",
    //                 sym
    //             );
    //         }
    //     }

    //     Ok(())
    // }
}

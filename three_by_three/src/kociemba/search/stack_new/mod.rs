mod max_threshold;
mod partial_stack;
mod stack;
mod utils;

#[cfg(test)]
mod test {
    use std::collections::{BTreeSet, HashSet};

    use itertools::Itertools;
    use rand::{Rng, SeedableRng as _};
    use rand_chacha::ChaCha8Rng;
    use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator as _};

    use crate::{
        CornerOrient, CornerPerm, EdgeOrient, EdgePerm, ReprCube, Tables,
        cube_ops::cube_sym::{CubeSymmetry, DominoSymmetry},
        kociemba::{
            coords::{
                CornerOrientRawCoord, CornerPermRawCoord, EEdgePermRawCoord,
                EdgeGroupOrientRawCoord, EdgeGroupOrientSymCoord, EdgeGroupRawCoord,
                EdgeOrientRawCoord, UDEdgePermRawCoord,
                edge_group_orient_combo_coord::EdgeGroupOrientComboCoord,
            },
            partial_reprs::{
                e_edge_perm::EEdgePerm,
                edge_group::EdgeGroup,
                edge_group_orient::EdgeGroupOrient,
                edge_positions::{
                    EEdgePositions, EdgePositions, combine_edge_positions, split_edge_positions,
                },
                ud_edge_perm::UDEdgePerm,
            },
            tables::{
                lookup_sym_corner_orient::{
                    CornerOrientComboCoord, CornerOrientSymCoord, LookupSymCornerOrientTable,
                },
                lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable,
            },
        },
    };

    #[test]
    fn count_alternate_view_possibilities_ego() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let ego_tbl: &LookupSymEdgeGroupOrientTable = tables.as_ref();

        let mut rng = ChaCha8Rng::seed_from_u64(2);

        let out = (0..64430u16)
            .into_iter()
            .map(|sym| {
                (
                    sym,
                    DominoSymmetry::all_iter()
                        .map(move |domino_conjugation| {
                            ego_tbl.get_raw_from_combo(EdgeGroupOrientComboCoord {
                                sym_coord: EdgeGroupOrientSymCoord(sym),
                                domino_conjugation,
                            })
                        })
                        .unique(),
                )
            })
            .map(move |(sym, raws)| {
                let mut sym_reduced = BTreeSet::new();

                for raw in raws {
                    let (eg, eo) = raw.split();
                    let edge_group = EdgeGroup::from_coord(eg);

                    let mut possibilities = HashSet::new();
                    while possibilities.len() < 70 {
                        let parity = rng.random_range(0..1);
                        let e = ((rng.random_range(0..12) as u16) << 1) + parity;
                        let e = EEdgePositions(EdgePositions(e));
                        let ud = ((rng.random_range(0..20160) as u16) << 1) + parity;
                        let UDEdgePerm(u, d) = UDEdgePerm::from_coord(UDEdgePermRawCoord(ud));
                        let pre_grouped = combine_edge_positions(u, d, e);
                        let edge_perm = pre_grouped
                            .then(EdgePerm(edge_group.0.into_representative_even_perm()));

                        let mut cube: ReprCube = rand::distr::Distribution::sample(
                            &rand::distr::StandardUniform,
                            &mut rng,
                        );

                        cube.edge_orient = EdgeOrient::from_coord(eo);
                        cube.edge_perm = edge_perm;

                        for cube_rot in (1..2).map(|x| cube.conjugate(CubeSymmetry(x << 4))) {
                            let (_, _, e_edge_positions) = split_edge_positions(edge_perm);

                            let g = e_edge_positions.into_edge_group_raw();
                            let o = cube_rot.edge_orient.into_coord();

                            let s = ego_tbl
                                .get_combo_from_raw(EdgeGroupOrientRawCoord::join(g, o))
                                .sym_coord;

                            possibilities.insert((g, o));
                            sym_reduced.insert(s.0);
                        }
                    }
                }

                (sym, sym_reduced)
            });

        for (base, others) in out.into_iter().take(1000) {
            let len = others.len();
            let mut iter = others.into_iter();
            print!("{base} ({len}) -> [{}", iter.next().unwrap());
            for x in iter {
                print!(", {x}");
            }
            println!("]");
        }

        Ok(())
    }

    #[test]
    fn count_alternate_view_possibilities_co() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let co_tbl: &LookupSymCornerOrientTable = tables.as_ref();

        let mut rng = ChaCha8Rng::seed_from_u64(2);

        let out = (0..168u8)
            .into_iter()
            .map(|sym| {
                (
                    sym,
                    DominoSymmetry::all_iter()
                        .map(move |domino_conjugation| {
                            co_tbl.get_raw_from_combo(CornerOrientComboCoord {
                                sym_coord: CornerOrientSymCoord(sym),
                                domino_conjugation,
                            })
                        })
                        .unique(),
                )
            })
            .map(move |(sym, raws)| {
                let mut sym_reduced = BTreeSet::new();

                for raw in raws {
                    let possibilities = (0..40320)
                        .into_iter()
                        .flat_map(|perm| {
                            let mut cube = ReprCube::SOLVED;
                            cube.corner_orient = CornerOrient::from_coord(raw);
                            cube.corner_perm = CornerPerm::from_coord(CornerPermRawCoord(perm));
                            (1..2)
                                .map(move |x| cube.conjugate(CubeSymmetry(x << 4)))
                                .map(|cube_rot| cube_rot.corner_orient.into_coord())
                        })
                        .unique();

                    for possibility in possibilities {
                        let s = co_tbl.get_combo_from_raw(possibility).sym_coord;
                        sym_reduced.insert(s.0);
                    }
                }

                (sym, sym_reduced)
            });

        for (base, others) in out {
            let len = others.len();
            // let mut iter = others.into_iter();
            // print!("{base} ({len}) -> [{}", iter.next().unwrap());
            // for x in iter {
            //     print!(", {x}");
            // }
            // println!("]");
            println!("{base} ({len})");
        }

        Ok(())
    }

    #[test]
    fn count_raw_orient() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let co_tbl: &LookupSymCornerOrientTable = tables.as_ref();

        for co in 0..2187 {
            let possibilities = (0..40320)
                .into_iter()
                .flat_map(|perm| {
                    let mut cube = ReprCube::SOLVED;
                    cube.corner_perm = CornerPerm::from_coord(CornerPermRawCoord(perm));
                    (1..2)
                        .map(move |x| cube.conjugate(CubeSymmetry(x << 4)))
                        .map(|cube_rot| cube_rot.corner_orient)
                })
                .unique()
                .count();

            println!("{}", possibilities);
        }

        Ok(())
    }
}

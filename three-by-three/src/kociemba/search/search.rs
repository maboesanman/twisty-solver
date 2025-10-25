use std::convert::identity;

use smallvec::{SmallVec, smallvec};

use crate::{
    cube_ops::{cube_move::CubeMove, cube_sym::DominoSymmetry, repr_cube::ReprCube},
    kociemba::{coords::repr_coord::SymReducedRepr, search::capped_idastar::idastar_limited},
    tables::Tables,
};

pub fn domino_reduce(cube: ReprCube, tables: &Tables) -> Vec<CubeMove> {
    let start_cube = SymReducedRepr::from_cube(cube, tables);

    let (sequence, _) = pathfinding::directed::idastar::idastar(
        &start_cube,
        |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
        |&cube| cube.prune_distance_phase_1(tables),
        |&cube| cube.is_phase_1_solved(),
    )
    .unwrap();

    let mut moves = vec![];
    let mut last = cube;

    for solve_c in sequence[1..].iter().map(|c| c.into_cube(tables)) {
        let (_, l, mv) = CubeMove::all_iter()
            .flat_map(|mv| {
                let next = last.apply_cube_move(mv);
                DominoSymmetry::all_iter().map(move |s| (next.domino_conjugate(s), next, mv))
            })
            .find(|(c, _, _)| *c == solve_c)
            .unwrap();
        last = l;
        moves.push(mv);
    }

    moves
}

pub fn solve_phased(cube: ReprCube, tables: &Tables) -> Vec<CubeMove> {
    let start_cube = SymReducedRepr::from_cube(cube, tables);

    let (mut sequence, _) = pathfinding::directed::idastar::idastar(
        &start_cube,
        |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
        |&cube| cube.prune_distance_phase_1(tables),
        |&cube| cube.is_phase_1_solved(),
    )
    .unwrap();

    let start = sequence.pop().unwrap();

    let (mut phase_2_sequence, _) = pathfinding::directed::idastar::idastar(
        &start,
        |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
        |&cube| cube.prune_distance_phase_2(tables),
        |&cube| cube.is_solved(),
    )
    .unwrap();

    sequence.append(&mut phase_2_sequence);

    let mut moves = vec![];
    let mut last = cube;

    for solve_c in sequence[1..].iter().map(|c| c.into_cube(tables)) {
        let (_, l, mv) = match CubeMove::all_iter()
            .flat_map(|mv| {
                let next = last.apply_cube_move(mv);
                DominoSymmetry::all_iter().map(move |s| (next.domino_conjugate(s), next, mv))
            })
            .find(|(c, _, _)| *c == solve_c)
        {
            Some(a) => a,
            None => panic!(),
        };

        last = l;
        moves.push(mv);
    }

    moves
}

// pub fn solve_combined(cube: ReprCube, tables: &Tables) -> Vec<CubeMove> {
//     let cube_solutions = super::phase_1_iter::Stack::new(cube, tables)
//         .scan(
//         None,
//         |current_best: &mut Option<u8>,
//          (sequence_start, sequence_end)| {
//             let phase_2_start = sequence_end.last().or_else(f);
//             let (phase_2_sequence, phase_2_len) = match current_best {
//                 Some(current_best) => {
//                     let phase_2_prune = phase_2_start.prune_distance_phase_2(tables);
//                     let phase_2_allowed = *current_best - phase_1_len;
//                     if phase_2_prune > phase_2_allowed {
//                         return Some(None);
//                     }
//                     match idastar_limited(
//                         *phase_2_start,
//                         |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
//                         |&cube| cube.prune_distance_phase_2(tables),
//                         |&cube| cube.is_solved(),
//                         phase_2_allowed,
//                     ) {
//                         Some(path) => path,
//                         None => return Some(None),
//                     }
//                 }
//                 None => pathfinding::directed::idastar::idastar(
//                     phase_2_start,
//                     |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
//                     |&cube| cube.prune_distance_phase_2(tables),
//                     |&cube| cube.is_solved(),
//                 )
//                 .unwrap(),
//             };

//             *current_best = Some(phase_1_len + phase_2_len);

//             Some(Some((phase_1_sequence, phase_2_sequence)))
//         },
//     ).filter_map(identity)
//     .map(|(phase_1, phase_2)| {
//         let mut moves = vec![];
//         let mut last = cube;

//         for solve_c in phase_1[1..].iter().chain(phase_2[1..].iter()).map(|c| c.into_cube(tables)) {
//             let (_, l, mv) = match CubeMove::all_iter()
//                 .flat_map(|mv| {
//                     let next = last.apply_cube_move(mv);
//                     DominoSymmetry::all_iter().map(move |s| (next.domino_conjugate(s), next, mv))
//                 })
//                 .find(|(c, _, _)| *c == solve_c)
//             {
//                 Some(a) => a,
//                 None => panic!(),
//             };

//             last = l;
//             moves.push(mv);
//         }

//         moves
//     });

//     for solution in cube_solutions {
//         println!("solution: {:?} - {:?}", solution.len(), solution);
//     }
    

//     todo!()
// }

fn successors(node: [u16; 4], tables: &Tables) -> SmallVec<[([u16; 4], u8); 18]> {
    match node {
        [u16::MAX, phase_2_prune, c, d] => {
            let start = SymReducedRepr([0, 0, c, d]);
            let phase_2_prune = phase_2_prune as u8;
            // set first coords to 0, then perform an ida* phase 2 search,
            // note the length of the search, and subtract the prune weight from it.
            // return a single edge to solved, with length actual - prune
            let (_, len) = pathfinding::directed::idastar::idastar(
                &start,
                |&cube| cube.neighbors(tables).into_iter().map(move |c| (c, 1)),
                |&cube| cube.prune_distance_phase_2(tables),
                |&cube| cube.is_solved(),
            )
            .unwrap();

            smallvec![([0; 4], len - phase_2_prune)]
        }
        [0, 0, c, d] => {
            // we have domino reduced.
            // set bits to high, then return a single edge with prune weight to the self with high bits
            let cube = SymReducedRepr(node);
            let phase_2_prune = cube.prune_distance_phase_2(tables);

            smallvec![([u16::MAX, phase_2_prune as u16, c, d], phase_2_prune)]
        }
        node => {
            // return the phase 1 neighbots
            let cube = SymReducedRepr(node);
            cube.neighbors(tables)
                .into_iter()
                .map(|SymReducedRepr(node)| (node, 1))
                .collect()
        }
    }
}

fn heuristic(node: [u16; 4], tables: &Tables) -> u8 {
    match node {
        [u16::MAX, ..] | [0, 0, ..] => 0,
        node => SymReducedRepr(node).prune_distance_phase_1(tables),
    }
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use crate::cube;

    use super::*;

    #[test]
    fn domino_reduce_test() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let moves = domino_reduce(cube![R U Rp Up], &tables);

        for m in moves {
            print!("{m} ");
        }

        Ok(())
    }

    #[test]
    fn domino_reduce_test_superflip() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let moves = domino_reduce(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
        );

        for m in moves {
            print!("{m} ");
        }

        Ok(())
    }

    #[test]
    fn domino_reduce_test_other() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let moves = domino_reduce(
            cube![Fp B2 D2 L D R2 L F2 D Bp U Dp B L2 B2 L Fp Rp Fp Up F2 Up Dp Bp R2],
            &tables,
        );

        for m in moves {
            print!("{m} ");
        }

        Ok(())
    }

    #[test]
    fn solve_phased_test() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let moves = solve_phased(cube![Up R U Rp], &tables);

        for m in moves {
            print!("{m} ");
        }

        Ok(())
    }

    #[test]
    fn solve_phased_test_superflip() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let moves = solve_phased(
            cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
            &tables,
        );

        for m in moves {
            print!("{m} ");
        }

        Ok(())
    }

    #[test]
    fn solve_phased_test_other() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;

        let moves = solve_phased(
            cube![Fp B2 D2 L D R2 L F2 D Bp U Dp B L2 B2 L Fp Rp Fp Up F2 Up Dp Bp R2],
            &tables,
        );

        for m in moves {
            print!("{m} ");
        }

        Ok(())
    }

    #[test]
    fn verify_phased_solutions_random() -> anyhow::Result<()> {
        let tables = Tables::new("tables")?;
        let mut rng = ChaCha8Rng::seed_from_u64(99);

        for _ in 0..100 {
            let mut cube: ReprCube =
                rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

            let moves = solve_phased(cube, &tables);

            for mv in moves {
                cube = cube.apply_cube_move(mv);
            }

            assert!(cube == ReprCube::SOLVED);
        }

        Ok(())
    }
}

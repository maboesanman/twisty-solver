use crate::{
    cube_ops::{
        cube_move::CubeMove, cube_sym::DominoSymmetry, repr_coord::SymReducedRepr,
        repr_cube::ReprCube,
    },
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

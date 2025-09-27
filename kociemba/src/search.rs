use rand::distr::Distribution;

use crate::{
    cube, cube_ops::{
        coords::{CornerOrientRawCoord, EdgeGroupRawCoord, EdgeOrientRawCoord},
        cube_move::CubeMove,
        phase_1_repr::Phase1InitRepr,
        repr_cube::ReprCube,
    }, tables::{
        lookup_sym_edge_group_orient::LookupSymEdgeGroupOrientTable,
        move_raw_corner_orient::MoveRawCornerOrientTable,
        move_sym_edge_group_orient::MoveSymEdgeGroupOrientTable, prune_phase_1::PrunePhase1Table,
    }
};

pub fn phase_1_solve(
    cube: ReprCube,
    sym_lookup: &LookupSymEdgeGroupOrientTable,
    move_edge_sym: &MoveSymEdgeGroupOrientTable,
    move_corner: &MoveRawCornerOrientTable,
    pruning_table: &PrunePhase1Table,
) -> Vec<CubeMove> {
    let start_cube = Phase1InitRepr::from_cube(cube, move_corner, sym_lookup);

    let (sequence, _) = match pathfinding::directed::dijkstra::dijkstra(
        &start_cube,
        |&cube| {
            let prune = pruning_table.get_value(cube);
            CubeMove::all_iter()
                .map(move |mv| cube.apply_cube_move(mv, move_edge_sym, move_corner).0)
                .filter(move |new_cube| {
                    let new_prune = pruning_table.get_value(*new_cube);
                    (new_prune + 1) % 3 == prune
                })
                .map(|cube| (cube, 1))
        },
        |&start_cube| start_cube == Phase1InitRepr::SOLVED,
    ) {
        Some(x) => x,
        None => {
            println!("failed: {cube:?}");
            return vec![];
        }
    };

    let mut moves = vec![];
    let mut last = cube;

    for solve_c in sequence[1..].iter() {
        let (l, mv) = CubeMove::all_iter()
            .map(|mv| (last.apply_cube_move(mv), mv))
            .filter(|(c, _)| *solve_c == Phase1InitRepr::from_cube(*c, move_corner, sym_lookup))
            .next()
            .unwrap();
        last = l;
        moves.push(mv.try_into().unwrap());
    }

    moves
}

#[test]
fn search_test() -> anyhow::Result<()> {
    let lookup_sym_edge_group_orient = LookupSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_lookup_table.dat",
    ).unwrap();

    let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_move_table.dat",
        &lookup_sym_edge_group_orient,
    ).unwrap();

    let move_raw_corner_orient = MoveRawCornerOrientTable::load("corner_orient_move_table.dat").unwrap();

    let move_sym_edge_group_orient_ref = &move_sym_edge_group_orient;
    let move_raw_corner_orient_ref = &move_raw_corner_orient;

    let prune_phase_1 = PrunePhase1Table::load("phase_1_prune_table.dat", move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).unwrap();



    let moves = phase_1_solve(
        cube![R U Rp Up],
        &lookup_sym_edge_group_orient,
        &move_sym_edge_group_orient,
        &move_raw_corner_orient,
        &prune_phase_1,
    );

    for m in moves {
        print!("{m} ");
    }

    Ok(())
}

#[test]
fn search_test_superflip() -> anyhow::Result<()> {
    let lookup_sym_edge_group_orient = LookupSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_lookup_table.dat",
    ).unwrap();

    let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_move_table.dat",
        &lookup_sym_edge_group_orient,
    ).unwrap();

    let move_raw_corner_orient = MoveRawCornerOrientTable::load("corner_orient_move_table.dat").unwrap();

    let move_sym_edge_group_orient_ref = &move_sym_edge_group_orient;
    let move_raw_corner_orient_ref = &move_raw_corner_orient;

    let prune_phase_1 = PrunePhase1Table::load("phase_1_prune_table.dat", move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).unwrap();



    let moves = phase_1_solve(
        cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
        &lookup_sym_edge_group_orient,
        &move_sym_edge_group_orient,
        &move_raw_corner_orient,
        &prune_phase_1,
    );

    for m in moves {
        print!("{m} ");
    }

    Ok(())
}

#[test]
fn search_test_other() -> anyhow::Result<()> {
    let lookup_sym_edge_group_orient = LookupSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_lookup_table.dat",
    ).unwrap();

    let move_sym_edge_group_orient = MoveSymEdgeGroupOrientTable::load(
        "edge_group_orient_sym_move_table.dat",
        &lookup_sym_edge_group_orient,
    ).unwrap();

    let move_raw_corner_orient = MoveRawCornerOrientTable::load("corner_orient_move_table.dat").unwrap();

    let move_sym_edge_group_orient_ref = &move_sym_edge_group_orient;
    let move_raw_corner_orient_ref = &move_raw_corner_orient;

    let prune_phase_1 = PrunePhase1Table::load("phase_1_prune_table.dat", move_sym_edge_group_orient_ref, move_raw_corner_orient_ref).unwrap();



    let moves = phase_1_solve(
        cube![Fp B2 D2 L D R2 L F2 D Bp U Dp B L2 B2 L Fp Rp Fp Up F2 Up Dp Bp R2],
        &lookup_sym_edge_group_orient,
        &move_sym_edge_group_orient,
        &move_raw_corner_orient,
        &prune_phase_1,
    );

    for m in moves {
        print!("{m} ");
    }

    Ok(())
}

// #[test]
// fn search_test_random() -> anyhow::Result<()> {
//     use rand::{Rng, SeedableRng};
//     let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(18);

//     let phase_1_move_edge_raw_table =
//         crate::tables::move_raw_edge_group_flip::load("edge_group_and_orient_move_table.dat")?;
//     let phase_1_move_corner_raw_table =
//         crate::tables::move_raw_corner_orient::load("corner_orient_move_table.dat")?;
//     let phase_1_lookup_edge_sym_table = crate::tables::sym_lookup_phase_1_edge::load(
//         "phase_1_edge_sym_lookup_table.dat",
//         &phase_1_move_edge_raw_table,
//     )?;
//     let phase_1_move_edge_sym_table = crate::tables::move_table_sym_edge_group_flip::load(
//         "phase_1_edge_sym_move_table.dat",
//         &phase_1_lookup_edge_sym_table,
//         &phase_1_move_edge_raw_table,
//     )?;

//     let phase_1_pruning_table =
//         crate::tables::pruning_table_phase_1_working::load_phase_1_pruning_table(
//             "phase_1_pruning_table.dat",
//             &phase_1_move_edge_sym_table,
//             &phase_1_move_corner_raw_table,
//         )?;

//     // for _ in 0..10 {
//     //     let _: ReprCube = StandardUniform.sample(&mut rng);
//     // }

//     for _ in 0..1000 {
//         let moves = phase_1_solve(
//             {
//                 let mut cube = SOLVED_CUBE;
//                 print!("scramble: ");
//                 for _ in 0..20 {
//                     let m: Move = unsafe { core::mem::transmute(rng.random_range(0..18u8)) };
//                     print!("{m} ");
//                     cube = cube.then(m.into());
//                 }
//                 println!();
//                 cube
//             },
//             &phase_1_lookup_edge_sym_table,
//             &phase_1_move_edge_raw_table,
//             &phase_1_move_edge_sym_table,
//             &phase_1_move_corner_raw_table,
//             &phase_1_pruning_table,
//         );

//         print!("solve: ");
//         for m in moves {
//             print!("{m} ");
//         }
//         println!();
//     }

//     Ok(())
// }

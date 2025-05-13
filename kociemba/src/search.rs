use rand::distr::Distribution;

use crate::{
    coords::{RawCornerOrientCoord, EdgeGroupCoord, EdgeOrientCoord},
    moves::Move,
    repr_cubie::{ReprCube, SOLVED_CUBE},
    tables::{
        move_raw_corner_orient::MoveRawCornerOrientTable,
        move_raw_edge_group_flip::EdgeGroupAndOrientMoveTable,
        move_table_sym_edge_group_flip::Phase1EdgeSymMoveTable,
        pruning_table_phase_1_working::Phase1PruningTable,
        sym_lookup_phase_1_edge::Phase1EdgeSymLookupTable,
    },
};

pub fn phase_1_solve(
    cube: ReprCube,
    sym_lookup: &Phase1EdgeSymLookupTable,
    move_edge_raw: &EdgeGroupAndOrientMoveTable,
    move_edge_sym: &Phase1EdgeSymMoveTable,
    move_corner_raw: &MoveRawCornerOrientTable,
    pruning_table: &Phase1PruningTable,
) -> Vec<Move> {
    let edge_orient_coord = EdgeOrientCoord::from_cubie(cube);
    let edge_group_coord = EdgeGroupCoord::from_cubie(cube);
    let corner_orient_coord = RawCornerOrientCoord::from_cubie(cube);

    let (sym_start, transform) =
        sym_lookup.get_sym_from_raw(move_edge_raw, edge_group_coord, edge_orient_coord);

    let raw_start = move_corner_raw.conjugate_by_transform(corner_orient_coord, transform);

    let start = (sym_start, raw_start);

    let (sequence, _) = match pathfinding::directed::dijkstra::dijkstra(
        &start,
        |(sym, raw)| {
            let sym = *sym;
            let raw = *raw;
            let curr_prune = pruning_table.get_value((sym, raw));
            Move::all_iter().filter_map(move |mv| {
                let (new_sym, transform) = move_edge_sym.apply_move(sym, mv);
                let new_raw = move_corner_raw.apply_move_and_transform(raw, mv, transform);
                let new_prune = pruning_table.get_value((new_sym, new_raw));

                if (new_prune + 1) % 3 == curr_prune {
                    Some(((new_sym, new_raw), 1))
                } else {
                    None
                }
            })
        },
        // |_| 0u8,
        |(a, b)| a.inner() == 0 && b.inner() == 0,
    ) {
        Some(x) => x,
        None => {
            println!("failed: {cube:?}");
            return vec![];
        }
    };

    let mut moves = vec![];
    let mut last = cube;

    for (sym, raw) in sequence[1..].iter() {
        let (l, mv) = Move::all_iter().map(ReprCube::from).map(|mv| (last.then(mv), mv)).filter(|(c, _)| {
            let eo = EdgeOrientCoord::from_cubie(*c);
            let eg = EdgeGroupCoord::from_cubie(*c);
            let co = RawCornerOrientCoord::from_cubie(*c);
        
            let (sym_start, transform) =
                sym_lookup.get_sym_from_raw(move_edge_raw, eg, eo);
        
            if &sym_start != sym {
                return false
            }
            
            let raw_start = move_corner_raw.conjugate_by_transform(co, transform);

            if &raw_start != raw {
                return false
            }

            true
        }).next().unwrap();
        last = l;
        moves.push(mv.try_into().unwrap());
    }

    moves
}

#[test]
fn search_test() -> anyhow::Result<()> {
    let phase_1_move_edge_raw_table =
        crate::tables::move_raw_edge_group_flip::load("edge_group_and_orient_move_table.dat")?;
    let phase_1_move_corner_raw_table =
        crate::tables::move_raw_corner_orient::load("corner_orient_move_table.dat")?;
    let phase_1_lookup_edge_sym_table = crate::tables::sym_lookup_phase_1_edge::load(
        "phase_1_edge_sym_lookup_table.dat",
        &phase_1_move_edge_raw_table,
    )?;
    let phase_1_move_edge_sym_table = crate::tables::move_table_sym_edge_group_flip::load(
        "phase_1_edge_sym_move_table.dat",
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
    )?;

    let phase_1_pruning_table = crate::tables::pruning_table_phase_1_working::load_phase_1_pruning_table(
        "phase_1_pruning_table.dat",
        &phase_1_move_edge_sym_table,
        &phase_1_move_corner_raw_table,
    )?;

    let moves = phase_1_solve(
        cube![R U Rp Up],
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
        &phase_1_move_edge_sym_table,
        &phase_1_move_corner_raw_table,
        &phase_1_pruning_table,
    );

    for m in moves {
        print!("{m} ");
    }

    Ok(())
}


#[test]
fn search_test_superflip() -> anyhow::Result<()> {
    let phase_1_move_edge_raw_table =
        crate::tables::move_raw_edge_group_flip::load("edge_group_and_orient_move_table.dat")?;
    let phase_1_move_corner_raw_table =
        crate::tables::move_raw_corner_orient::load("corner_orient_move_table.dat")?;
    let phase_1_lookup_edge_sym_table = crate::tables::sym_lookup_phase_1_edge::load(
        "phase_1_edge_sym_lookup_table.dat",
        &phase_1_move_edge_raw_table,
    )?;
    let phase_1_move_edge_sym_table = crate::tables::move_table_sym_edge_group_flip::load(
        "phase_1_edge_sym_move_table.dat",
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
    )?;

    let phase_1_pruning_table = crate::tables::pruning_table_phase_1_working::load_phase_1_pruning_table(
        "phase_1_pruning_table.dat",
        &phase_1_move_edge_sym_table,
        &phase_1_move_corner_raw_table,
    )?;

    let moves = phase_1_solve(
        cube![U R2 F B R B2 R U2 L B2 R Up Dp R2 F Rp L B2 U2 F2],
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
        &phase_1_move_edge_sym_table,
        &phase_1_move_corner_raw_table,
        &phase_1_pruning_table,
    );

    for m in moves {
        print!("{m} ");
    }

    Ok(())
}

#[test]
fn search_test_random() -> anyhow::Result<()> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(18);

    let phase_1_move_edge_raw_table =
        crate::tables::move_raw_edge_group_flip::load("edge_group_and_orient_move_table.dat")?;
    let phase_1_move_corner_raw_table =
        crate::tables::move_raw_corner_orient::load("corner_orient_move_table.dat")?;
    let phase_1_lookup_edge_sym_table = crate::tables::sym_lookup_phase_1_edge::load(
        "phase_1_edge_sym_lookup_table.dat",
        &phase_1_move_edge_raw_table,
    )?;
    let phase_1_move_edge_sym_table = crate::tables::move_table_sym_edge_group_flip::load(
        "phase_1_edge_sym_move_table.dat",
        &phase_1_lookup_edge_sym_table,
        &phase_1_move_edge_raw_table,
    )?;

    let phase_1_pruning_table = crate::tables::pruning_table_phase_1_working::load_phase_1_pruning_table(
        "phase_1_pruning_table.dat",
        &phase_1_move_edge_sym_table,
        &phase_1_move_corner_raw_table,
    )?;

    // for _ in 0..10 {
    //     let _: ReprCube = StandardUniform.sample(&mut rng);
    // }

    for _ in 0..1000 {
        let moves = phase_1_solve(
            {
                let mut cube = SOLVED_CUBE;
                print!("scramble: ");
                for _ in 0..20 {
                    let m: Move  = unsafe { core::mem::transmute(rng.random_range(0..18u8)) };
                    print!("{m} ");
                    cube = cube.then(m.into());
                }
                println!();
                cube
            },
            &phase_1_lookup_edge_sym_table,
            &phase_1_move_edge_raw_table,
            &phase_1_move_edge_sym_table,
            &phase_1_move_corner_raw_table,
            &phase_1_pruning_table,
        );
    
        print!("solve: ");
        for m in moves {
            print!("{m} ");
        }
        println!();
    }

    Ok(())
}
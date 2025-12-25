use futures::StreamExt as _;
use rand::SeedableRng as _;
use rand_chacha::ChaCha8Rng;
use three_by_three::{CubeMove, ReprCube, Tables, get_incremental_solutions_stream};

pub fn main() {
    let tables = Box::leak(Box::new(Tables::new("tables").unwrap()));
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for _ in 0..100 {
        let cube: ReprCube =
            rand::distr::Distribution::sample(&rand::distr::StandardUniform, &mut rng);

        cube.pretty_print();

        let mut stream = get_incremental_solutions_stream(cube, tables, Some(20), false);
        let future = stream.next();
        // assert!(futures::executor::block_on(future).is_some());

        let solution = futures::executor::block_on(future).unwrap();
        {
            print!("{:02} ", solution.len());
            for m in solution.into_iter().rev() {
                let m = match m {
                    CubeMove::U1 => CubeMove::U3,
                    CubeMove::U2 => CubeMove::U2,
                    CubeMove::U3 => CubeMove::U1,
                    CubeMove::D1 => CubeMove::D3,
                    CubeMove::D2 => CubeMove::D2,
                    CubeMove::D3 => CubeMove::D1,
                    CubeMove::F1 => CubeMove::F3,
                    CubeMove::F2 => CubeMove::F2,
                    CubeMove::F3 => CubeMove::F1,
                    CubeMove::B1 => CubeMove::B3,
                    CubeMove::B2 => CubeMove::B2,
                    CubeMove::B3 => CubeMove::B1,
                    CubeMove::R1 => CubeMove::R3,
                    CubeMove::R2 => CubeMove::R2,
                    CubeMove::R3 => CubeMove::R1,
                    CubeMove::L1 => CubeMove::L3,
                    CubeMove::L2 => CubeMove::L2,
                    CubeMove::L3 => CubeMove::L1,
                };
                print!("{m} ");
            }
            println!("");
        }
    }
}

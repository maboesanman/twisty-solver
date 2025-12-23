use itertools::Itertools;

use crate::{
    CubeMove, ReprCube, Tables, cube_ops::cube_sym::CubeSymmetry,
    kociemba::search::phase_1_node::Phase1Node, kociemba::tables,
};

// pub fn move_resolver_phase_1<const N: usize>(
//     cube: ReprCube,
//     tables: &Tables,
// ) -> impl Fn(
//     (
//         [SymReducedRepr; 2],
//         [SymReducedRepr; N],
//         Vec<SymReducedReprPhase2>,
//     ),
// ) -> Vec<CubeMove> {
//     move |(a, b, c)| {
//         let move_resolver = move_resolver(cube, tables);
//         let sym_cubes = a[1..].iter().copied().chain(b.iter().copied()).chain(
//             c[1..]
//                 .iter()
//                 .map(|SymReducedReprPhase2([c, d])| SymReducedRepr([0, 0, *c, *d])),
//         );

//         (move_resolver)(sym_cubes)
//     }
// }

pub fn move_resolver_multi_dimension_domino(
    initial_cube: ReprCube,
    cubes: impl Iterator<Item = ReprCube>,
) -> Vec<CubeMove> {
    let mut peekable = cubes.peekable();
    let start_cube = *peekable.peek().unwrap();

    let mut symmetries = (0..3).map(|x| CubeSymmetry(x << 4));
    let sym = symmetries
        .find(|sym| start_cube.conjugate(*sym) == initial_cube)
        .unwrap();

    let adjusted_cubes = peekable.map(|c| c.conjugate(sym));

    move_resolver(adjusted_cubes)
}

pub fn move_resolver(cubes: impl Iterator<Item = ReprCube>) -> Vec<CubeMove> {
    cubes
        .tuple_windows()
        .map(|(cube_a, cube_b)| {
            CubeMove::all_iter()
                .map(|mv| (cube_a.apply_cube_move(mv), mv))
                .find(|(c, _)| *c == cube_b)
                .unwrap()
                .1
        })
        .collect()
}

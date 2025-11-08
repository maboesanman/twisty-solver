use crate::{
    CubeMove, ReprCube, Tables,
    cube_ops::cube_sym::CubeSymmetry,
    kociemba::coords::repr_coord::{SymReducedRepr, SymReducedReprPhase2},
};

pub fn move_resolver_phase_1<const N: usize>(
    cube: ReprCube,
    tables: &Tables,
) -> impl Fn(
    (
        [SymReducedRepr; 2],
        [SymReducedRepr; N],
        Vec<SymReducedReprPhase2>,
    ),
) -> Vec<CubeMove> {
    move |(a, b, c)| {
        let move_resolver = move_resolver(cube, tables);
        let sym_cubes = a[1..].iter().copied().chain(b.iter().copied()).chain(
            c[1..]
                .iter()
                .map(|SymReducedReprPhase2([c, d])| SymReducedRepr([0, 0, *c, *d])),
        );

        (move_resolver)(sym_cubes)
    }
}

pub fn move_resolver<I: Iterator<Item = SymReducedRepr>>(
    cube: ReprCube,
    tables: &Tables,
) -> impl Fn(I) -> Vec<CubeMove> {
    move |sym_cubes| {
        let mut moves = vec![];
        let mut last = cube;

        for solve_c in sym_cubes.map(|c| c.into_cube(tables)) {
            let (_, l, mv) = match CubeMove::all_iter()
                .flat_map(|mv| {
                    let next = last.apply_cube_move(mv);
                    CubeSymmetry::all_iter().map(move |s| (next.conjugate(s), next, mv))
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
}

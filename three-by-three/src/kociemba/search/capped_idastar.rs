use pathfinding::num_traits::Zero;

pub fn idastar_limited<N, C, FN, IN, FH, FS>(
    start: N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
    max_bound: C,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Copy,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut bound = heuristic(&start);
    let mut path = vec![start.clone()];

    loop {
        if bound > max_bound {
            // exceeded user-imposed limit
            return None;
        }

        match search_limited(
            &mut path,
            Zero::zero(),
            bound,
            &mut successors,
            &mut heuristic,
            &mut success,
        ) {
            Path::Found(path, cost) => return Some((path, cost)),
            Path::Minimum(min) => {
                if bound == min {
                    return None;
                }
                bound = min;
            }
            Path::Impossible => return None,
        }
    }
}

enum Path<N, C> {
    Found(Vec<N>, C),
    Minimum(C),
    Impossible,
}

fn search_limited<N, C, FN, IN, FH, FS>(
    path: &mut Vec<N>,
    cost: C,
    bound: C,
    successors: &mut FN,
    heuristic: &mut FH,
    success: &mut FS,
) -> Path<N, C>
where
    N: Eq + Copy,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let current = &path[path.len() - 1];
    let f = cost + heuristic(current);
    if f > bound {
        return Path::Minimum(f);
    }
    if success(current) {
        return Path::Found(path.clone(), cost);
    }

    let mut neighbs = successors(current)
        .into_iter()
        .filter_map(|(n, c)| (!path.contains(&n)).then_some((n, c, cost + c + heuristic(&n))))
        .collect::<Vec<_>>();
    neighbs.sort_unstable_by(|(_, _, f1), (_, _, f2)| f1.cmp(f2));

    let mut min = None;
    for (n, c, _) in neighbs {
        path.push(n);
        match search_limited(path, cost + c, bound, successors, heuristic, success) {
            Path::Found(p, cost) => return Path::Found(p, cost),
            Path::Minimum(m) => {
                if min.map_or(true, |n| m < n) {
                    min = Some(m);
                }
            }
            Path::Impossible => {}
        }
        path.pop();
    }

    min.map_or(Path::Impossible, Path::Minimum)
}


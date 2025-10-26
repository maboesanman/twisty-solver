use std::marker::PhantomData;

use pathfinding::num_traits::{One, Zero};

pub fn idastar<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut bound = heuristic(start);
    let mut path = vec![start.clone()];
    loop {
        match search(
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

fn search<N, C, FN, IN, FH, FS>(
    path: &mut Vec<N>,
    cost: C,
    bound: C,
    successors: &mut FN,
    heuristic: &mut FH,
    success: &mut FS,
) -> Path<N, C>
where
    N: Eq + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let neighbs = {
        let start = &path[path.len() - 1];
        let f = cost + heuristic(start);
        if f > bound {
            return Path::Minimum(f);
        }
        if success(start) {
            return Path::Found(path.clone(), f);
        }
        let mut neighbs = successors(start)
            .into_iter()
            .filter_map(|(n, c)| {
                (!path.contains(&n)).then(|| {
                    let h = heuristic(&n);
                    (n, c, c + h)
                })
            })
            .collect::<Vec<_>>();
        neighbs.sort_unstable_by(|(_, _, c1), (_, _, c2)| c1.cmp(c2));
        neighbs
    };
    let mut min = None;
    for (node, extra, _) in neighbs {
        path.push(node);
        match search(path, cost + extra, bound, successors, heuristic, success) {
            found_path @ Path::Found(_, _) => return found_path,
            Path::Minimum(m) if min.is_none_or(|n| n >= m) => min = Some(m),
            _ => (),
        }
        path.pop();
    }
    min.map_or(Path::Impossible, Path::Minimum)
}

pub fn idastar_solutions<N, C, FN, IN, FH, FS>(
    start: N,
    successors: FN,
    mut heuristic: FH,
    success: FS,
) -> IdastarSolutions<N, C, FN, IN, FH, FS>
where
    N: Eq + Copy,
    C: Zero + Ord + Copy + One,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let bound = heuristic(&start);
    IdastarSolutions {
        path: vec![start],
        successors,
        heuristic,
        success,
        bound,
        finished: false,
        buffer: Vec::new(),
        phantom: PhantomData,
    }
}

pub struct IdastarSolutions<N, C, FN, IN, FH, FS> {
    successors: FN,
    heuristic: FH,
    success: FS,
    path: Vec<N>,
    bound: C,
    finished: bool,
    buffer: Vec<(Vec<N>, C)>,
    phantom: PhantomData<fn() -> IN>,
}

impl<N, C, FN, IN, FH, FS> Iterator for IdastarSolutions<N, C, FN, IN, FH, FS>
where
    N: Eq + Copy,
    C: Zero + Ord + Copy + One,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    type Item = (Vec<N>, C);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sol) = self.buffer.pop() {
            return Some(sol);
        }
        if self.finished {
            return None;
        }

        let mut new_solutions = Vec::new();
        let mut min_exceeded = None;

        search_all(
            &mut self.path,
            Zero::zero(),
            self.bound,
            &mut self.successors,
            &mut self.heuristic,
            &mut self.success,
            &mut new_solutions,
            &mut min_exceeded,
        );

        if !new_solutions.is_empty() {
            // Sort by cost ascending
            new_solutions.sort_by(|a, b| a.1.cmp(&b.1));
            // Reverse so we can pop() in ascending order
            new_solutions.reverse();
            self.buffer = new_solutions;
            return self.buffer.pop();
        }

        // No solutions found at this bound, raise it
        if let Some(next_bound) = min_exceeded {
            if next_bound == self.bound {
                self.finished = true;
                return None;
            }
            self.bound = next_bound;
            // try again
            self.next()
        } else {
            self.finished = true;
            None
        }
    }
}

fn search_all<N, C, FN, IN, FH, FS>(
    path: &mut Vec<N>,
    cost: C,
    bound: C,
    successors: &mut FN,
    heuristic: &mut FH,
    success: &mut FS,
    solutions: &mut Vec<(Vec<N>, C)>,
    min_exceeded: &mut Option<C>,
) where
    N: Eq + Copy,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let start = &path[path.len() - 1];
    let f = cost + heuristic(start);
    if f > bound {
        if min_exceeded.is_none_or(|m| f < m) {
            *min_exceeded = Some(f);
        }
        return;
    }

    if success(start) {
        solutions.push((path.clone(), cost));
        return;
    }

    let mut neighbs = successors(start)
        .into_iter()
        .filter_map(|(n, c)| (!path.contains(&n)).then_some((n, c, cost + c + heuristic(&n))))
        .collect::<Vec<_>>();
    neighbs.sort_unstable_by(|(_, _, f1), (_, _, f2)| f1.cmp(f2));

    for (n, c, _) in neighbs {
        path.push(n);
        search_all(
            path,
            cost + c,
            bound,
            successors,
            heuristic,
            success,
            solutions,
            min_exceeded,
        );
        path.pop();
    }
}

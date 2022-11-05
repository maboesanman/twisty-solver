use std::ops::Mul;

use smallvec::Array;

use crate::scramble::{Scramble, Conjugate, Rotation};

pub trait Puzzle {
    type CoreMove;
    type ExtMove;

    type State: Eq;

    type Rotation: Rotation
        + Conjugate<Self::CoreMove>
        + Conjugate<Self::ExtMove>;

    fn get_solved_state() -> Self::State;
}

pub trait Solvable: Puzzle {
    /// returns the first scramble at or below `move_target`,
    /// the shortest one found before `time_target`,
    /// or the first one found after `time_target`.
    fn solve(
        state: Self::State,
        move_target: usize,
        time_target: core::time::Duration,
        threads: usize,
    ) -> Scramble<Self> {
        Self::solve_into(
            state, 
            Self::get_solved_state(),
            move_target,
            time_target,
            threads
        )
    }

    /// returns the first scramble at or below `move_target`,
    /// the shortest one found before `time_target`,
    /// or the first one found after `time_target`.
    fn solve_into(
        from: Self::State,
        to: Self::State,
        move_target: usize,
        time_target: core::time::Duration,
        threads: usize,
    ) -> Scramble<Self>;

    fn obfuscate(
        scramble: Scramble<Self>,
        move_target: usize,
        time_target: core::time::Duration,
        threads: usize,
    ) -> Scramble<Self>;
}

pub trait RandomStateScrambleable: Solvable {
    fn get_random_state() -> Self::State;

    fn random_state_scramble(
        move_target: usize,
        time_target: core::time::Duration,
        threads: usize,
    ) -> Scramble<Self> {
        Self::solve_into(
            Self::get_solved_state(),
            Self::get_random_state(), 
            move_target,
            time_target,
            threads
        )
    }
}

pub trait OptimallySolvable: Solvable {
    fn solve_optimal(
        state: Self::State,
        move_target: usize,
        threads: usize,
    ) -> Scramble<Self> {
        Self::solve_optimal_into(
            state, 
            Self::get_solved_state(),
            move_target,
            threads
        )
    }

    fn solve_optimal_into(
        from: Self::State,
        to: Self::State,
        move_target: usize,
        threads: usize,
    ) -> Scramble<Self>;
}

pub trait QuicklySolvable: Solvable {
    fn solve_quick(
        state: Self::State,
    ) -> Scramble<Self>;
    fn solve_quick_into(
        from: Self::State,
        to: Self::State,
    ) -> Scramble<Self>;
}

pub trait WCAPuzzle: Puzzle {
    fn official_scramble() -> Scramble<Self>;
}
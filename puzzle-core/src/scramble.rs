use crate::puzzle::Puzzle;

pub trait Identity {
    fn identity() -> Self;
}

pub trait Invertable {
    fn inverse(&self) -> Self;
}

pub trait Conjugate<M>: Invertable {
    // given move M, calculate [S' M S].
    fn conjugate(&self, m: &M) -> M;
}

pub trait Rotation: Sized + Identity + Eq + Invertable + Conjugate<Self> {}

pub struct Scramble<P: Puzzle + ?Sized> {
    // applied left to right
    pub moves: Vec<P::CoreMove>,

    // applied left to right
    pub rotation: P::Rotation,
}

impl<P: Puzzle> Invertable for Scramble<P> {
    fn inverse(&self) -> Self {
        todo!()
    }
}

impl<P: Puzzle> Conjugate<Scramble<P>> for P::Rotation {
    fn conjugate(&self, m: &Scramble<P>) -> Scramble<P> {
        todo!()
    }
}

impl<P: Puzzle> std::ops::Mul for Scramble<P> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        todo!()
    }
}

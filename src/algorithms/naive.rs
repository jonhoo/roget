use crate::{Guess, Guesser};

pub struct Naive;

impl Naive {
    pub fn new() -> Self {
        Naive
    }
}

impl Guesser for Naive {
    fn guess(&mut self, _history: &[Guess]) -> String {
        todo!();
    }
}

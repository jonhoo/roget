use crate::{Correctness, Guess, Guesser, Word, DICTIONARY};
use std::collections::HashMap;

pub struct Allocs {
    remaining: HashMap<Word, usize>,
}

impl Allocs {
    pub fn new() -> Self {
        Self {
            remaining: HashMap::from_iter(DICTIONARY.lines().map(|line| {
                let (word, count) = line
                    .split_once(' ')
                    .expect("every line is word + space + frequency");
                let count: usize = count.parse().expect("every count is a number");
                let word = word
                    .as_bytes()
                    .try_into()
                    .expect("every dictionary word is 5 characters");
                (word, count)
            })),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: Word,
    goodness: f64,
}

impl Guesser for Allocs {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if let Some(last) = history.last() {
            self.remaining.retain(|&word, _| last.matches(word));
        }
        if history.is_empty() {
            return *b"tares";
        }

        let remaining_count: usize = self.remaining.iter().map(|(_, &c)| c).sum();

        let mut best: Option<Candidate> = None;
        for (&word, _) in &self.remaining {
            let mut sum = 0.0;
            for pattern in Correctness::patterns() {
                // considering a world where we _did_ guess `word` and got `pattern` as the
                // correctness. now, compute what _then_ is left.
                let mut in_pattern_total = 0;
                for (&candidate, count) in &self.remaining {
                    let g = Guess {
                        word,
                        mask: pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    continue;
                }
                // TODO: apply sigmoid
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
            }
            let goodness = -sum;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }
        }
        best.unwrap().word
    }
}

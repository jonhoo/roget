use crate::{enumerate_mask, Correctness, Guess, Guesser, DICTIONARY, MAX_MASK_ENUM};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();

pub struct Enumerate {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
}

impl Enumerate {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("every line is word + space + frequency");
                    let count: usize = count.parse().expect("every count is a number");
                    (word, count)
                }))
            })),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
}

impl Guesser for Enumerate {
    fn guess(&mut self, history: &[Guess]) -> String {
        if let Some(last) = history.last() {
            if matches!(self.remaining, Cow::Owned(_)) {
                self.remaining
                    .to_mut()
                    .retain(|(word, _)| last.matches(word));
            } else {
                self.remaining = Cow::Owned(
                    self.remaining
                        .iter()
                        .filter(|(word, _)| last.matches(word))
                        .copied()
                        .collect(),
                );
            }
        }
        if history.is_empty() {
            return "tares".to_string();
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        for &(word, count) in &*self.remaining {
            let mut totals = [0usize; MAX_MASK_ENUM];
            // considering a world where we _did_ guess `word` and got `pattern` as the
            // correctness. now, compute what _then_ is left.
            for (candidate, count) in &*self.remaining {
                let idx = enumerate_mask(&Correctness::compute(candidate, word));
                totals[idx] += count;
            }

            debug_assert_eq!(totals.iter().sum::<usize>(), remaining_count, "{}", word);

            let sum: f64 = totals
                .iter()
                .map(|t| {
                    if *t == 0 {
                        0.0
                    } else {
                        // TODO: apply sigmoid
                        let p_of_this_pattern = *t as f64 / remaining_count as f64;
                        p_of_this_pattern * p_of_this_pattern.log2()
                    }
                })
                .sum();

            let p_word = count as f64 / remaining_count as f64;
            let goodness = p_word * -sum;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }
        }
        best.unwrap().word.to_string()
    }
}

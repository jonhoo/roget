use crate::{Correctness, Guess, Guesser, DICTIONARY, MAX_MASK_ENUM};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Cutoff {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Default for Cutoff {
    fn default() -> Self {
        Self::new()
    }
}

impl Cutoff {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| DICTIONARY.to_vec())),
            patterns: Cow::Borrowed(PATTERNS.get_or_init(|| Correctness::patterns().collect())),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
}

impl Guesser for Cutoff {
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
            self.patterns = Cow::Borrowed(PATTERNS.get().unwrap());
            return "tares".to_string();
        } else {
            assert!(!self.patterns.is_empty());
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        let mut i = 0;
        let stop = (self.remaining.len() / 3).max(20);
        for &(word, count) in &*self.remaining {
            // considering a world where we _did_ guess `word` and got `pattern` as the
            // correctness. now, compute what _then_ is left.

            // Rather than iterate over the patterns sequentially and add up the counts of words
            // that result in that pattern, we can instead keep a running total for each pattern
            // simultaneously by storing them in an array. We can do this since each candidate-word
            // pair deterministically produces only one mask.
            let mut totals = [0usize; MAX_MASK_ENUM];
            for (candidate, count) in &*self.remaining {
                let idx = Correctness::pack(&Correctness::compute(candidate, word));
                totals[usize::from(idx)] += count;
            }

            assert_eq!(totals.iter().sum::<usize>(), remaining_count, "{}", word);

            let sum: f64 = totals
                .into_iter()
                .filter(|t| *t != 0)
                .map(|t| {
                    // TODO: apply sigmoid
                    let p_of_this_pattern = t as f64 / remaining_count as f64;
                    p_of_this_pattern * p_of_this_pattern.log2()
                })
                .sum();

            let p_word = count as f64 / remaining_count as f64;
            let entropy = -sum;
            // TODO: this should be (minimizing):
            // (p_word * (history.len() + 1)) + ((1 - p_word) * estimate_remaining_guesses(remaining_entropy))
            // where remaining_entropy is the existing entropy - entropy
            // and restimate_remaining_guesses is computed by regression over historical data
            let goodness = p_word * entropy;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }

            i += 1;
            if i >= stop {
                break;
            }
        }
        best.unwrap().word.to_string()
    }
}

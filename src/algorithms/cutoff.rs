use crate::{Correctness, Guess, Guesser, DICTIONARY};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Cutoff {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Cutoff {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("every line is word + space + frequency");
                    let count: usize = count.parse().expect("every count is a number");
                    (word, count)
                }));
                words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));
                words
            })),
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
            let mut sum = 0.0;
            let check_pattern = |pattern: &[Correctness; 5]| {
                // considering a world where we _did_ guess `word` and got `pattern` as the
                // correctness. now, compute what _then_ is left.
                let mut in_pattern_total = 0;
                for (candidate, count) in &*self.remaining {
                    let g = Guess {
                        word: Cow::Borrowed(word),
                        mask: *pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    return false;
                }
                // TODO: apply sigmoid
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
                return true;
            };

            if matches!(self.patterns, Cow::Owned(_)) {
                self.patterns.to_mut().retain(check_pattern);
            } else {
                self.patterns = Cow::Owned(
                    self.patterns
                        .iter()
                        .copied()
                        .filter(check_pattern)
                        .collect(),
                );
            }

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

            i += 1;
            if i >= stop {
                break;
            }
        }
        best.unwrap().word.to_string()
    }
}

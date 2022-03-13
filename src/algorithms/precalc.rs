use crate::{Correctness, Guess, Guesser, DICTIONARY};
use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::collections::BTreeMap;

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static MATCH: OnceCell<BTreeMap<(&'static str, &'static str, [Correctness; 5]), bool>> =
    OnceCell::new();

pub struct Precalc {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
}

impl Default for Precalc {
    fn default() -> Self {
        Self::new()
    }
}

impl Precalc {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| DICTIONARY.to_vec())),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
}

impl Guesser for Precalc {
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
        for &(word, _) in &*self.remaining {
            let mut sum = 0.0;
            // TODO: don't consider correctness patterns that had no candidates in the previous
            // iteration
            for pattern in Correctness::patterns() {
                // considering a world where we _did_ guess `word` and got `pattern` as the
                // correctness. now, compute what _then_ is left.
                let mut in_pattern_total = 0;
                for (candidate, count) in &*self.remaining {
                    let matches = MATCH.get_or_init(|| {
                        let words = &INITIAL.get().unwrap()[..512];
                        let mut out = BTreeMap::new();
                        for &(word1, _) in words {
                            for &(word2, _) in words {
                                if word2 < word1 {
                                    break;
                                }
                                for pattern in Correctness::patterns() {
                                    let g = Guess {
                                        word: Cow::Borrowed(word1),
                                        mask: pattern,
                                    };
                                    out.insert((word1, word2, pattern), g.matches(candidate));
                                }
                            }
                        }
                        out
                    });

                    let key = if word < candidate {
                        (&*word, *candidate, pattern)
                    } else {
                        (*candidate, &*word, pattern)
                    };
                    if matches.get(&key).copied().unwrap_or_else(|| {
                        let g = Guess {
                            word: Cow::Borrowed(word),
                            mask: pattern,
                        };
                        g.matches(candidate)
                    }) {
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
            // TODO: weight this by p_word
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
        best.unwrap().word.to_string()
    }
}

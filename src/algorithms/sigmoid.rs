use crate::{enumerate_mask, Correctness, Guess, Guesser, DICTIONARY, MAX_MASK_ENUM};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, f64)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Sigmoid {
    remaining: Cow<'static, Vec<(&'static str, f64)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Default for Sigmoid {
    fn default() -> Self {
        Self::new()
    }
}

const L: f64 = 1.0;
// How steep is the cut-off?
const K: f64 = 30000000.0;
// Where is the cut-off?
const X0: f64 = 0.00000497;
// This combination of settings leads to a fairly sharp cut-off around:
//
//  word  "raw" p      sigmoid p   count
// vying 0.000491% -> 15.999853% (1560905)
// rheum 0.000492% -> 16.735476% (1561474)
// lamas 0.000492% -> 16.827817% (1561544)
// kraal 0.000492% -> 17.389054% (1561963)
// gents 0.000493% -> 25.448008% (1567081)
// hails 0.000494% -> 29.575062% (1569275)
// atria 0.000494% -> 30.602258% (1569792)
// wooed 0.000495% -> 32.304510% (1570628)
// kinks 0.000495% -> 32.904357% (1570917)
// sushi 0.000495% -> 34.261053% (1571561)
// filly 0.000496% -> 39.634216% (1574006)
// lazar 0.000496% -> 43.073524% (1575508)
// lapel 0.000497% -> 48.215265% (1577704)
// cecum 0.000497% -> 48.505414% (1577827)
// kayak 0.000497% -> 49.307915% (1578167)
// fundy 0.000498% -> 55.755065% (1580908)
// haply 0.000498% -> 56.508662% (1581232)
// bigot 0.000498% -> 57.402526% (1581618)
// wisps 0.000498% -> 57.407146% (1581620)
// foals 0.000498% -> 57.875326% (1581823)
fn sigmoid(p: f64) -> f64 {
    L / (1.0 + (-K * (p - X0)).exp())
}
const PRINT_SIGMOID: bool = false;

impl Sigmoid {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                let mut sum = 0;
                let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("every line is word + space + frequency");
                    let count: usize = count.parse().expect("every count is a number");
                    sum += count;
                    (word, count)
                }));

                words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));

                if PRINT_SIGMOID {
                    for &(word, count) in words.iter().rev() {
                        let p = count as f64 / sum as f64;
                        println!(
                            "{} {:.6}% -> {:.6}% ({})",
                            word,
                            100.0 * p,
                            100.0 * sigmoid(p),
                            count
                        );
                    }
                }

                let words: Vec<_> = words
                    .into_iter()
                    .map(|(word, count)| (word, sigmoid(count as f64 / sum as f64)))
                    .collect();

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

impl Guesser for Sigmoid {
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

        let remaining_p: f64 = self.remaining.iter().map(|&(_, p)| p).sum();

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
            let mut totals = [0.0f64; MAX_MASK_ENUM];
            for (candidate, count) in &*self.remaining {
                let idx = enumerate_mask(&Correctness::compute(candidate, word));
                totals[idx] += count;
            }

            let sum: f64 = totals
                .into_iter()
                .filter(|t| *t != 0.0)
                .map(|p| {
                    let p_of_this_pattern = p as f64 / remaining_p as f64;
                    p_of_this_pattern * p_of_this_pattern.log2()
                })
                .sum();

            let p_word = count as f64 / remaining_p as f64;
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

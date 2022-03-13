use crate::{Correctness, Guess, Guesser, DICTIONARY, MAX_MASK_ENUM};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, f64)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Escore {
    remaining: Cow<'static, Vec<(&'static str, f64)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
    entropy: Vec<f64>,
}

impl Default for Escore {
    fn default() -> Self {
        Self::new()
    }
}

// This is an estimation function for how many _more_ guesses are needed given that `entropy`
// entropy remains. It was constructed by iterative regression.
//
// First, I logged the observed remaining entropy + remaining guesses with an implementation that
// just tries to maximize the -sum of the candidates (entropy-initial.dat). I then ran that through
// logistical regression (see `escore-regress.r`). That gave
//
//   E[guesses] = entropy * 0.2592 + 1.3202
//   E[guesses] = ln(entropy * 4.066 + 3.755)
//   E[guesses] = e^(entropy * 0.1346 + 0.2210)
//   E[guesses] = 1/(entropy * -0.07977 + 0.84147)
//   E[guesses] = (entropy * 0.09177 + 1.13241)^2
//   E[guesses] = sqrt(entropy * 1.151 + 1.954)
//
// and an average score of 3.7631.
//
// Then, I ran the E[score] algorithm using the E[guesses] function determined by each of the first
// regressions, which gave the commented-out scores in the fn body below. I then proceeded with the
// best candidate (ln), and re-ran the regression on it, which gave
//
//   E[guesses] = ln(entropy * 3.869 + 3.679)
//
// and an average score of 3.7176 (worse than the first estimate). Further iterations did not
// change the parameters much, so I stuck with that last estimat.
//
// Below are also the formulas and average scores when using different regressions. Interestingly,
// the regression that does the best also tends to overestimate the number of guesses remaining,
// which causes the model to "go for the win" less often, and instead focus on "best information"
// guesses.
fn est_steps_left(entropy: f64) -> f64 {
    // entropy * 0.2592 + 1.3202 // 3.7181
    // (entropy * 4.066 + 3.755).ln() // 3.7172
    // (entropy * 0.1346 + 0.2210).exp() // 3.7237
    // 1.0 / (entropy * -0.07977 + 0.84147) // 3.7246
    // (entropy * 0.09177 + 1.13241).powi(2) // 3.7176
    // (entropy * 1.151 + 1.954).sqrt() // 3.7176
    // (entropy * 3.869 + 3.679).ln() // 3.7176
    (entropy * 3.870 + 3.679).ln() // 3.7176
}
const PRINT_ESTIMATION: bool = false;

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

impl Escore {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                let sum: usize = DICTIONARY.iter().map(|(_, count)| count).sum();

                if PRINT_SIGMOID {
                    for (word, count) in DICTIONARY.iter().rev() {
                        let p = *count as f64 / sum as f64;
                        println!(
                            "{} {:.6}% -> {:.6}% ({})",
                            word,
                            100.0 * p,
                            100.0 * sigmoid(p),
                            count
                        );
                    }
                }

                DICTIONARY
                    .iter()
                    .copied()
                    .map(|(word, count)| (word, sigmoid(count as f64 / sum as f64)))
                    .collect()
            })),
            patterns: Cow::Borrowed(PATTERNS.get_or_init(|| Correctness::patterns().collect())),
            entropy: Vec::new(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    e_score: f64,
}

impl Guesser for Escore {
    fn guess(&mut self, history: &[Guess]) -> String {
        let score = history.len() as f64;

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
            // NOTE: I did a manual run with this commented out and it indeed produced "tares" as
            // the first guess. It slows down the run by a lot though.
            return "tares".to_string();
        } else {
            assert!(!self.patterns.is_empty());
        }

        let remaining_p: f64 = self.remaining.iter().map(|&(_, p)| p).sum();
        let remaining_entropy = -self
            .remaining
            .iter()
            .map(|&(_, p)| {
                let p = p / remaining_p;
                p * p.log2()
            })
            .sum::<f64>();
        self.entropy.push(remaining_entropy);

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
                let idx = Correctness::pack(&Correctness::compute(candidate, word));
                totals[usize::from(idx)] += count;
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
            let e_info = -sum;
            let e_score = p_word * (score + 1.0)
                + (1.0 - p_word) * (score + est_steps_left(remaining_entropy - e_info));
            if let Some(c) = best {
                // Which one gives us a lower (expected) score?
                if e_score < c.e_score {
                    best = Some(Candidate { word, e_score });
                }
            } else {
                best = Some(Candidate { word, e_score });
            }

            i += 1;
            if i >= stop {
                break;
            }
        }
        best.unwrap().word.to_string()
    }

    fn finish(&self, guesses: usize) {
        if PRINT_ESTIMATION {
            for (i, &entropy) in self.entropy.iter().enumerate() {
                // i == 0 is the entropy that was left _after_ guessing the first word.
                // we want to print f(remaining entropy) -> number of guesses needed
                // we know we ended up making `guesses` guesses, and we know this is the entropy after
                // the (i+1)th guess, which means there are
                let guesses_needed = guesses - (i + 1);
                println!("{} {}", entropy, guesses_needed);
            }
        }
    }
}

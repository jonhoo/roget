use crate::{Correctness, Guess, Guesser, PackedCorrectness, DICTIONARY, MAX_MASK_ENUM};
use once_cell::sync::OnceCell;
use once_cell::unsync::OnceCell as UnSyncOnceCell;
use std::borrow::Cow;
use std::cell::Cell;

/// The initial set of words without any smoothing
static INITIAL_COUNTS: OnceCell<Vec<(&'static str, f64, usize)>> = OnceCell::new();
/// The initial set of words after applying sigmoid smoothing.
static INITIAL_SIGMOID: OnceCell<Vec<(&'static str, f64, usize)>> = OnceCell::new();

/// A per-thread cache of cached `Correctness` for each word pair.
///
/// We make this thread-local so that access to it is as cheap as we can get it.
///
/// We store a `Box` because the array is quite large, and we're unlikely to have the stack space
/// needed to store the whole thing on a given thread's stack.
type Cache = [[Cell<Option<PackedCorrectness>>; DICTIONARY.len()]; DICTIONARY.len()];
thread_local! {
    static COMPUTES: UnSyncOnceCell<Box<Cache>> = Default::default();
}

pub struct Solver {
    remaining: Cow<'static, Vec<(&'static str, f64, usize)>>,
    entropy: Vec<f64>,
    options: Options,
    last_guess_idx: Option<usize>,
}

impl Default for Solver {
    fn default() -> Self {
        Options::default().build()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rank {
    /// Just pick the first candidate.
    First,

    /// E[score] = p(word) * (score + 1) + (1 - p(word)) * (score + E[guesses](entropy - E[information]))
    ExpectedScore,

    /// p(word) * E[information]
    WeightedInformation,

    /// p(word) + E[information]
    InfoPlusProbability,

    /// E[information]
    ExpectedInformation,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Options {
    /// If true, counts will be smoothed using a sigmoid.
    pub sigmoid: bool,

    /// If true, candidates will be ranked based on expected score.
    pub rank_by: Rank,

    /// If true, correcness computation will be cached.
    pub cache: bool,

    /// If true, only the most likely 1/3 of candidates are considered at each step.
    pub cutoff: bool,

    /// If true, solver may not guess known-wrong words.
    pub hard_mode: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sigmoid: true,
            rank_by: Rank::ExpectedScore,
            cache: true,
            cutoff: true,
            hard_mode: true,
        }
    }
}

impl Options {
    pub fn build(self) -> Solver {
        let remaining = if self.sigmoid {
            INITIAL_SIGMOID.get_or_init(|| {
                let sum: usize = DICTIONARY.iter().map(|(_, count)| count).sum();

                if PRINT_SIGMOID {
                    for &(word, count) in DICTIONARY.iter().rev() {
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

                DICTIONARY
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(idx, (word, count))| (word, sigmoid(count as f64 / sum as f64), idx))
                    .collect()
            })
        } else {
            INITIAL_COUNTS.get_or_init(|| {
                DICTIONARY
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(idx, (word, count))| (word, count as f64, idx))
                    .collect()
            })
        };

        if self.cache {
            COMPUTES.with(|c| {
                c.get_or_init(|| {
                    // This is really silly.
                    // We'd like to just do `Box::default()`, but that doesn't work since `Default`
                    // isn't implemented for arbitrarily long arrays. We can't use `Box::new` since
                    // that'll create the (huge) array on the _stack_ first before then copying it
                    // to the heap. And support for creation of values directly on the heap (the
                    // `box` keyword) is an unstable nightly-only feature.
                    //
                    // So, we use unsafe.

                    // First, we sanity check that the byte value 0 is equivalent to our `None`
                    // value.
                    let c = &Cell::new(None::<PackedCorrectness>);
                    assert_eq!(std::mem::size_of_val(c), 1);
                    let c = c as *const _;
                    let c = c as *const u8;
                    assert_eq!(unsafe { *c }, 0);

                    // Then, we allocate the number of bytes we need directly on the heap.
                    // And we request that they're all zero, which by the above we know matches the
                    // value we expect for `Cache`.
                    let mem = unsafe {
                        std::alloc::alloc_zeroed(
                            std::alloc::Layout::from_size_align(
                                std::mem::size_of::<Cache>(),
                                std::mem::align_of::<Cache>(),
                            )
                            .unwrap(),
                        )
                    };

                    // And then we cast it to a Box of the appropriate type, which should be safe.
                    unsafe { Box::from_raw(mem as *mut _) }
                });
            });
        }

        Solver {
            remaining: Cow::Borrowed(remaining),
            entropy: Vec::new(),
            last_guess_idx: None,

            options: self,
        }
    }
}

// This inline gives about a 13% speedup.
#[inline]
fn get_packed(
    row: &[Cell<Option<PackedCorrectness>>],
    guess: &str,
    answer: &str,
    answer_idx: usize,
) -> PackedCorrectness {
    let cell = &row[answer_idx];
    match cell.get() {
        Some(a) => a,
        None => {
            let correctness = PackedCorrectness::from(Correctness::compute(answer, guess));
            cell.set(Some(correctness));
            correctness
        }
    }
}

impl Solver {
    pub fn builder() -> Options {
        Options::default()
    }
}

impl Solver {
    fn trim(&mut self, mut cmp: impl FnMut(&str, usize) -> bool) {
        if matches!(self.remaining, Cow::Owned(_)) {
            self.remaining
                .to_mut()
                .retain(|&(word, _, word_idx)| cmp(word, word_idx));
        } else {
            self.remaining = Cow::Owned(
                self.remaining
                    .iter()
                    .filter(|(word, _, word_idx)| cmp(word, *word_idx))
                    .copied()
                    .collect(),
            );
        }
    }
}

impl Guesser for Solver {
    fn guess(&mut self, history: &[Guess]) -> String {
        let score = history.len() as f64;

        if let Some(last) = history.last() {
            if self.options.cache {
                let reference = PackedCorrectness::from(last.mask);
                COMPUTES.with(|c| {
                    let row = &c.get().unwrap()[self.last_guess_idx.unwrap()];
                    self.trim(|word, word_idx| {
                        reference == get_packed(row, &last.word, word, word_idx)
                    });
                });
            } else {
                self.trim(|word, _| last.matches(word));
            }
        }

        if history.is_empty() {
            self.last_guess_idx = Some(
                self.remaining
                    .iter()
                    .find(|(word, _, _)| &**word == "tares")
                    .map(|&(_, _, idx)| idx)
                    .unwrap(),
            );
            // NOTE: I did a manual run with this commented out and it indeed produced "tares" as
            // the first guess. It slows down the run by a lot though.
            return "tares".to_string();
        } else if self.options.rank_by == Rank::First || self.remaining.len() == 1 {
            let w = self.remaining.first().unwrap();
            self.last_guess_idx = Some(w.2);
            return w.0.to_string();
        }
        assert!(!self.remaining.is_empty());

        let remaining_p: f64 = self.remaining.iter().map(|&(_, p, _)| p).sum();
        let remaining_entropy = -self
            .remaining
            .iter()
            .map(|&(_, p, _)| {
                let p = p / remaining_p;
                p * p.log2()
            })
            .sum::<f64>();
        self.entropy.push(remaining_entropy);

        let mut best: Option<Candidate> = None;
        let mut i = 0;
        let stop = (self.remaining.len() / 3).max(20).min(self.remaining.len());
        let consider = if self.options.hard_mode {
            &*self.remaining
        } else if self.options.sigmoid {
            INITIAL_SIGMOID.get().unwrap()
        } else {
            INITIAL_COUNTS.get().unwrap()
        };
        for &(word, count, word_idx) in consider {
            // considering a world where we _did_ guess `word` and got `pattern` as the
            // correctness. now, compute what _then_ is left.

            // Rather than iterate over the patterns sequentially and add up the counts of words
            // that result in that pattern, we can instead keep a running total for each pattern
            // simultaneously by storing them in an array. We can do this since each candidate-word
            // pair deterministically produces only one mask.
            let mut totals = [0.0f64; MAX_MASK_ENUM];

            let mut in_remaining = false;
            if self.options.cache {
                COMPUTES.with(|c| {
                    let row = &c.get().unwrap()[word_idx];
                    for (candidate, count, candidate_idx) in &*self.remaining {
                        in_remaining |= word_idx == *candidate_idx;
                        let idx = get_packed(row, word, candidate, *candidate_idx);
                        totals[usize::from(u8::from(idx))] += count;
                    }
                });
            } else {
                for (candidate, count, candidate_idx) in &*self.remaining {
                    in_remaining |= word_idx == *candidate_idx;
                    let idx = PackedCorrectness::from(Correctness::compute(candidate, word));
                    totals[usize::from(u8::from(idx))] += count;
                }
            }

            let sum: f64 = totals
                .into_iter()
                .filter(|t| *t != 0.0)
                .map(|p| {
                    let p_of_this_pattern = p as f64 / remaining_p as f64;
                    p_of_this_pattern * p_of_this_pattern.log2()
                })
                .sum();

            let p_word = if in_remaining {
                count as f64 / remaining_p as f64
            } else {
                // TODO: penalize further.
                0.0
            };
            let e_info = -sum;
            let goodness = match self.options.rank_by {
                Rank::First => unreachable!("early return above"),
                Rank::ExpectedScore => {
                    // NOTE: Higher is better, so we negate the result.
                    -(p_word * (score + 1.0)
                        + (1.0 - p_word) * (score + est_steps_left(remaining_entropy - e_info)))
                }
                Rank::WeightedInformation => p_word * e_info,
                Rank::InfoPlusProbability => p_word + e_info,
                Rank::ExpectedInformation => e_info,
            };
            if let Some(c) = best {
                // Which one gives us a lower (expected) score?
                if goodness > c.goodness {
                    best = Some(Candidate {
                        word,
                        goodness,
                        idx: word_idx,
                    });
                }
            } else {
                best = Some(Candidate {
                    word,
                    goodness,
                    idx: word_idx,
                });
            }

            if self.options.cutoff && in_remaining {
                i += 1;
                if i >= stop {
                    break;
                }
            }
        }
        let best = best.unwrap();
        assert_ne!(best.goodness, 0.0);
        self.last_guess_idx = Some(best.idx);
        best.word.to_string()
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

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
    idx: usize,
}

use crate::{Guess, Guesser, DICTIONARY};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();

/// A strawman algorithm which simply chooses the most popular word of the
/// words remaining which match the most recent mask
pub struct Popular {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
}

impl Popular {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("every line is word + space + frequency");
                    let count: usize = count.parse().expect("every count is a number");
                    // TODO: apply sigmoid to counts
                    (word, count)
                }));
                words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));
                words
            })),
        }
    }
}

impl Guesser for Popular {
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
            "tares".to_string()
        } else {
            self.remaining.first().unwrap().0.to_string()
        }
    }
}

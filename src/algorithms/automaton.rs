use crate::{Correctness, Guess, Guesser, DICTIONARY};
use fst::{IntoStreamer, Map, Streamer};
use once_cell::sync::OnceCell;
use wordle_automaton::{prepare, Wordle, WordleBuilder};

type Fst = Map<Vec<u8>>;

fn prepare_dict() -> &'static Fst {
    static FST: OnceCell<Fst> = OnceCell::new();

    FST.get_or_init(|| {
        let words = DICTIONARY
            .lines()
            .filter_map(|line| Some(line.split_once(' ')?.0))
            .collect::<Vec<_>>();
        let words = prepare::score_word_list::<_, 5>(words);
        prepare::build_fst(words).expect("Dictionary is utf-8 sorted")
    })
}

pub struct Automaton {
    fst: &'static Fst,
    wordle: Wordle<5>,
    best: [u8; 5],
}

impl Automaton {
    pub fn new() -> Self {
        Self {
            fst: prepare_dict(),
            wordle: WordleBuilder::<5>::new().build(),
            best: [b'z'; 5],
        }
    }
}

impl Guesser for Automaton {
    fn guess(&mut self, history: &[Guess]) -> String {
        let guess = match history.last() {
            None => "tares",
            Some(last) => {
                let wordle = std::mem::replace(&mut self.wordle, Wordle::new());
                let mut wb = WordleBuilder::from(wordle);

                let word = last.word.as_bytes();
                for (pos, (correctness, b)) in last.mask.iter().zip(word).enumerate() {
                    match correctness {
                        Correctness::Correct => wb.correct_pos(pos, *b),
                        Correctness::Misplaced => wb.wrong_pos(pos, *b),
                        Correctness::Wrong => wb.never(*b),
                    };
                }
                self.wordle = wb.build();

                let mut solutions = self.fst.search(&self.wordle).into_stream();
                let mut best_score = 0;

                while let Some((word, score)) = solutions.next() {
                    if score > best_score {
                        best_score = score;
                        self.best.copy_from_slice(word);
                    }
                }

                std::str::from_utf8(&self.best).unwrap()
            }
        };

        guess.to_string()
    }
}

use roget::{Correctness, Guess};
use std::borrow::Cow;

const TESTS: &str = include_str!("../wordle-tests/data/tests.txt");

fn to_correctness(result: &str) -> [Correctness; 5] {
    assert_eq!(result.len(), 5);
    let mut out = [Correctness::Wrong; 5];
    for (c, out) in result.bytes().zip(out.iter_mut()) {
        *out = match c {
            b'c' => Correctness::Correct,
            b'm' => Correctness::Misplaced,
            b'w' => Correctness::Wrong,
            _ => {
                unreachable!("unknown pattern character '{}'", c);
            }
        };
    }
    out
}

#[test]
fn all() {
    for line in TESTS.lines() {
        let mut fields = line.split(',');
        let answer = fields.next().expect("word1");
        let guess = fields.next().expect("word2");
        let result = fields.next().expect("result");
        assert_eq!(fields.count(), 0);
        let result = to_correctness(result);
        assert_eq!(
            Correctness::compute(answer, guess),
            result,
            "guess {} against {}",
            guess,
            answer
        );
        assert!(Guess {
            word: Cow::Borrowed(guess),
            mask: result,
        }
        .matches(answer));
    }
}

use rayon::prelude::*;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::NonZeroUsize;

const DICTIONARY: &str = include_str!("../wordle.txt");

fn main() {
    let files: Vec<_> = std::env::args().skip(1).collect();
    let words: HashMap<_, _> = files
        .into_par_iter()
        .map(|file| {
            let file = std::fs::File::open(&file)
                .unwrap_or_else(|e| panic!("could not open file '{}': {}", file, e));
            let file = BufReader::new(file);
            let file = flate2::bufread::GzDecoder::new(file);
            let mut file = BufReader::new(file);
            let mut words: HashMap<_, _> = DICTIONARY.lines().map(|w| (w.as_bytes(), 0)).collect();
            let mut line = Vec::new();
            loop {
                line.clear();
                if file
                    .read_until(b'\n', &mut line)
                    .expect("reading from stdin should be okay")
                    == 0
                {
                    break;
                }
                let mut fields = line.split_mut(|&c| c == b'\t');
                let word = fields.next().expect("every line should have a word");
                let word = if let Some(w) = word.splitn_mut(2, |&c| c == b'_').next() {
                    w
                } else {
                    word
                };
                if word.len() != 5 {
                    line.clear();
                    continue;
                }
                if !word.iter().all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z')) {
                    continue;
                }
                word.make_ascii_lowercase();
                if let Some(accum) = words.get_mut(&*word) {
                    let count: usize = fields
                        .map(|field| {
                            let mut columns = field.split(|&c| c == b',');
                            let count = columns.nth(1).expect("every row has three fields");
                            let mut v = 0;
                            let mut dec = 1;
                            for &digit in count.iter().rev() {
                                assert!(matches!(digit, b'0'..=b'9'));
                                let digit = digit - b'0';
                                v += digit as usize * dec;
                                dec *= 10;
                            }
                            v
                        })
                        .sum();

                    *accum += count;
                }
            }
            words
        })
        .reduce(HashMap::new, |mut map1, map2| {
            for (word, count) in map2 {
                *map1.entry(word).or_insert(0) += count;
            }
            map1
        });

    for word in DICTIONARY.lines() {
        let count = words
            .get(word.as_bytes())
            .copied()
            .and_then(NonZeroUsize::new)
            .map(|v| v.into())
            .unwrap_or(1);
        println!("{} {}", word, count);
    }
}

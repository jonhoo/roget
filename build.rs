use std::io::prelude::*;

const DICTIONARY: &str = include_str!("dictionary.txt");

fn main() {
    println!("cargo:rerun-if-changed=dictionary.txt");
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut f = std::fs::File::create(out_dir.join("dictionary.rs"))
        .expect("could not create file in OUT_DIR");

    let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
        let (word, count) = line
            .split_once(' ')
            .expect("every line is word + space + frequency");
        let count: usize = count.parse().expect("every count is a number");
        (word, count)
    }));
    words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));

    let mut hm = phf_codegen::OrderedMap::new();
    for (word, count) in words {
        hm.entry(word, &format!("{}", count));
    }

    writeln!(f, "pub const DICT_MAP: phf::OrderedMap<&str, usize> = {};", hm.build()).unwrap();
}

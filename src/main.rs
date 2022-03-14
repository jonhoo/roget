use clap::{ArgEnum, Parser};
use roget::{algorithms, Guesser};

const GAMES: &str = include_str!("../answers.txt");

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, arg_enum, default_value = "cache")]
    implementation: Implementation,

    #[clap(short, long)]
    games: Option<usize>,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Implementation {
    Naive,
    Allocs,
    Vecrem,
    Precalc,
    Weight,
    Enum,
    Cutoff,
    Sigmoid,
    Escore,
    Popular,
    Cache,
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Naive => {
            play::<algorithms::Naive>(args.games);
        }
        Implementation::Allocs => {
            play::<algorithms::Allocs>(args.games);
        }
        Implementation::Vecrem => {
            play::<algorithms::Vecrem>(args.games);
        }
        Implementation::Precalc => {
            play::<algorithms::Precalc>(args.games);
        }
        Implementation::Weight => {
            play::<algorithms::Weight>(args.games);
        }
        Implementation::Enum => {
            play::<algorithms::Enumerate>(args.games);
        }
        Implementation::Cutoff => {
            play::<algorithms::Cutoff>(args.games);
        }
        Implementation::Sigmoid => {
            play::<algorithms::Sigmoid>(args.games);
        }
        Implementation::Escore => {
            play::<algorithms::Escore>(args.games);
        }
        Implementation::Popular => {
            play::<algorithms::Popular>(args.games);
        }
        Implementation::Cache => {
            play::<algorithms::Cached>(args.games);
        }
    }
}

fn play<G>(max: Option<usize>)
where
    G: Guesser + Default,
{
    let w = roget::Wordle::new();
    let mut score = 0;
    let mut games = 0;
    let mut histogram = Vec::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = G::default();
        if let Some(s) = w.play(answer, guesser) {
            games += 1;
            score += s;
            if s >= histogram.len() {
                histogram.extend(std::iter::repeat(0).take(s - histogram.len() + 1));
            }
            histogram[s] += 1;
            // eprintln!("guessed '{}' in {}", answer, s);
        } else {
            eprintln!("failed to guess '{}'", answer);
        }
    }
    let sum: usize = histogram.iter().sum();
    for (score, count) in histogram.into_iter().enumerate().skip(1) {
        let frac = count as f64 / sum as f64;
        let w1 = (30.0 * frac).round() as usize;
        let w2 = (30.0 * (1.0 - frac)).round() as usize;
        eprintln!(
            "{:>2}: {}{} ({})",
            score,
            "#".repeat(w1),
            " ".repeat(w2),
            count
        );
    }
    eprintln!("average score: {:.4}", score as f64 / games as f64);
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_10_games_with_escore() {
        let w = roget::Wordle::new();
        let results: Vec<_> = crate::GAMES
            .split_whitespace()
            .take(10)
            .filter_map(|answer| w.play(answer, roget::algorithms::Escore::new()))
            .collect();

        assert_eq!(results, [4, 3, 4, 4, 3, 4, 4, 3, 4, 3]);
    }
}

use clap::{ArgEnum, Parser};
use roget::Guesser;

const GAMES: &str = include_str!("../answers.txt");

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, arg_enum)]
    implementation: Implementation,

    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Implementation {
    Naive,
    Allocs,
    Vecrem,
    Once,
    Precalc,
    Weight,
    Enum,
    Cutoff,
    Popular,
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Naive => {
            play(roget::algorithms::Naive::new, args.max);
        }
        Implementation::Allocs => {
            play(roget::algorithms::Allocs::new, args.max);
        }
        Implementation::Vecrem => {
            play(roget::algorithms::Vecrem::new, args.max);
        }
        Implementation::Once => {
            play(roget::algorithms::OnceInit::new, args.max);
        }
        Implementation::Precalc => {
            play(roget::algorithms::Precalc::new, args.max);
        }
        Implementation::Weight => {
            play(roget::algorithms::Weight::new, args.max);
        }
        Implementation::Enum => {
            play(roget::algorithms::Enumerate::new, args.max);
        }
        Implementation::Cutoff => {
            play(roget::algorithms::Cutoff::new, args.max);
        }
        Implementation::Popular => {
            play(roget::algorithms::Popular::new, args.max);
        }
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::new();
    let mut score = 0;
    let mut games = 0;
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(s) = w.play(answer, guesser) {
            games += 1;
            score += s;
            println!("guessed '{}' in {}", answer, s);
        } else {
            eprintln!("failed to guess");
        }
    }
    println!("average score: {:.4}", score as f64 / games as f64);
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_10_games_with_cutoff() {
        let w = roget::Wordle::new();
        let results: Vec<_> = crate::GAMES
            .split_whitespace()
            .take(10)
            .filter_map(|answer| w.play(answer, roget::algorithms::Cutoff::new()))
            .collect();

        assert_eq!(results, [4, 4, 4, 4, 4, 5, 4, 5, 4, 2]);
    }
}

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
    Allocs,
    Vecrem,
    Once,
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Allocs => {
            play(roget::algorithms::Allocs::new, args.max);
        }
        Implementation::Vecrem => {
            play(roget::algorithms::Vecrem::new, args.max);
        }
        Implementation::Once => {
            play(roget::algorithms::OnceInit::new, args.max);
        }
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let answer_b: roget::Word = answer
            .as_bytes()
            .try_into()
            .expect("all answers are 5 letters");
        let guesser = (mk)();
        if let Some(score) = w.play(answer_b, guesser) {
            println!("guessed '{}' in {}", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}

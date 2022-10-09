use std::borrow::Cow;

use clap::{ArgEnum, Parser};
use roget::{Guesser, Solver};

const GAMES: &str = include_str!("../answers.txt");

#[global_allocator]
static GLOBAL_ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// By default, counts will be smoothed using a sigmoid. This flag disables that.
    #[clap(long)]
    no_sigmoid: bool,

    /// Set how candidates are ranked at each step of the solver.
    ///
    /// By default, candidates will be ranked based on expected score.
    #[clap(short, long, arg_enum, default_value = "expected-score")]
    rank_by: Rank,

    /// By default, correcness computation are cached. This flag disables that.
    #[clap(long)]
    no_cache: bool,

    /// By default, only the most likely 1/3 of candidates are considered at each step. This flag
    /// disables that pruning behavior.
    #[clap(long)]
    no_cutoff: bool,

    /// By default, all games are played in "hard mode" where known-incorrect guesses are
    /// disallowed. This flag allows arbitrary guesses at every step, which allows for more optimal
    /// guessing, but also increases the search space.
    #[clap(long)]
    easy: bool,

    /// The number of games to run.
    ///
    /// If not passed, all Wordle games are run.
    #[clap(short, long, conflicts_with = "interactive")]
    games: Option<usize>,

    /// Launch in interactive mode.
    ///
    /// This mode is intended for helping you play the game elsewhere. The program will tell you what word to guess next, and ask you for what combination of correct/misplaced/incorrect you got in return.
    #[clap(short, long, conflicts_with = "games")]
    interactive: bool,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Rank {
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

fn main() {
    let args = Args::parse();

    let mut solver = Solver::builder();
    if args.no_cache {
        solver.cache = false;
    }
    if args.no_cutoff {
        solver.cutoff = false;
    }
    if args.no_sigmoid {
        solver.sigmoid = false;
    }
    if args.easy {
        solver.hard_mode = false;
    }
    solver.rank_by = match args.rank_by {
        Rank::First => roget::Rank::First,
        Rank::ExpectedScore => roget::Rank::ExpectedScore,
        Rank::WeightedInformation => roget::Rank::WeightedInformation,
        Rank::InfoPlusProbability => roget::Rank::InfoPlusProbability,
        Rank::ExpectedInformation => roget::Rank::ExpectedInformation,
    };
    if args.interactive {
        play_interactive(solver.build());
    } else {
        play(move || solver.build(), args.games);
    }
}

fn play_interactive(mut guesser: impl Guesser) {
    let mut history = Vec::with_capacity(6);
    println!("C: Correct / Green, M: Misplaced / Yellow, W: Wrong / Gray");
    // Wordle only allows six guesses.
    for _ in 1..=6 {
        let guess = guesser.guess(&history);
        println!("Guess:  {}", guess.to_uppercase());
        let correctness = {
            loop {
                match ask_for_correctness() {
                    Ok(c) => break c,
                    Err(e) => println!("{}", e),
                }
            }
        };
        if correctness == [roget::Correctness::Correct; 5] {
            println!("The answer was {}", guess.to_uppercase());
            return;
        }
        history.push(roget::Guess {
            word: Cow::Owned(guess),
            mask: correctness,
        });
    }
    println!("Game Over, only six guesses are allowed");
}

fn ask_for_correctness() -> Result<[roget::Correctness; 5], Cow<'static, str>> {
    print!("Colors: ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let mut answer = String::with_capacity(7);
    std::io::stdin().read_line(&mut answer).unwrap();
    let answer = answer
        .trim()
        .chars()
        .filter(|v| !v.is_whitespace())
        .map(|v| v.to_ascii_uppercase())
        .collect::<String>();
    if answer.len() != 5 {
        Err("You did not provide exactly 5 colors.")?;
    }
    let parsed = answer
        .chars()
        .map(|c| match c {
            'C' => Ok(roget::Correctness::Correct),
            'M' => Ok(roget::Correctness::Misplaced),
            'W' => Ok(roget::Correctness::Wrong),
            _ => Err(format!(
                "The guess color '{c}' wasn't recognized: use C/M/W"
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parsed
        .try_into()
        .expect("The parsed correctness is checked to be 5 items long"))
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = roget::Wordle::new();
    let mut score = 0;
    let mut games = 0;
    let mut histogram = Vec::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
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
    fn default_solver() {
        let w = roget::Wordle::new();
        let results: Vec<_> = crate::GAMES
            .split_whitespace()
            .take(20)
            .filter_map(|answer| w.play(answer, roget::Solver::default()))
            .collect();

        assert_eq!(
            results,
            [4, 3, 4, 4, 3, 4, 4, 3, 4, 3, 4, 3, 3, 4, 3, 4, 4, 4, 3, 3]
        );
    }
}

/// This one was good fun and not conceptually hard, although the separate logic for X/Y/Z between
/// part one and two was a nice surprise. Doing it "properly" in Rust meant the solution looks more
/// verbose than it likely could be, but I'm not golfing this and more interested in writing
/// idiomatic code, so I'm fine with it.
use std::{error::Error, fmt::Display, str::FromStr};

#[derive(Debug, PartialEq)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug)]
struct ParseMoveError {}

impl Error for ParseMoveError {}

const PARSE_MOVE_ERROR: &'static str = "invalid move key provided";

impl Display for ParseMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", PARSE_MOVE_ERROR)
    }
}

impl FromStr for Move {
    type Err = ParseMoveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // in part 1, X/Y/Z are also moves
            "A" | "X" => Ok(Self::Rock),
            "B" | "Y" => Ok(Self::Paper),
            "C" | "Z" => Ok(Self::Scissors),
            _ => Err(ParseMoveError {}),
        }
    }
}

impl From<char> for Move {
    fn from(ch: char) -> Self {
        match ch {
            'A' | 'X' => Self::Rock,
            'B' | 'Y' => Self::Paper,
            'C' | 'Z' => Self::Scissors,
            _ => panic!("unknown move key"),
        }
    }
}

impl Move {
    pub fn score(&self) -> u8 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    pub fn outcome_with(&self, other: &Self) -> Outcome {
        if *other == *self {
            Outcome::Draw
        } else {
            // Rock beats Scissors
            // Paper beats Rock
            // Scissors beats Paper
            match self {
                Self::Rock => {
                    if *other == Self::Scissors {
                        Outcome::Win
                    } else {
                        Outcome::Loss
                    }
                }
                Self::Paper => {
                    if *other == Self::Rock {
                        Outcome::Win
                    } else {
                        Outcome::Loss
                    }
                }
                Self::Scissors => {
                    if *other == Self::Paper {
                        Outcome::Win
                    } else {
                        Outcome::Loss
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Win,
    Draw,
    Loss,
}

#[derive(Debug)]
struct ParseOutcomeError {}

impl Error for ParseOutcomeError {}

const PARSE_OUTCOME_ERROR: &'static str = "invalid outcome key provided";

impl Display for ParseOutcomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", PARSE_OUTCOME_ERROR)
    }
}

impl FromStr for Outcome {
    type Err = ParseOutcomeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "X" => Ok(Self::Loss),
            "Y" => Ok(Self::Draw),
            "Z" => Ok(Self::Win),
            _ => Err(ParseOutcomeError {}),
        }
    }
}

impl Outcome {
    pub fn score(&self) -> u32 {
        match self {
            Self::Win => 6,
            Self::Draw => 3,
            Self::Loss => 0,
        }
    }
}

#[derive(Debug)]
struct Round {
    them: Move,
    us: Move,
}

impl Round {
    pub fn outcome(&self) -> Outcome {
        self.us.outcome_with(&self.them)
    }
}

#[derive(Debug)]
struct Round2 {
    them: Move,
    desired_outcome: Outcome,
}

// could use a crate to generically iterate over an enum but this is quicker
const MOVES: [Move; 3] = [Move::Rock, Move::Paper, Move::Scissors];

impl Round2 {
    pub fn our_move(&self) -> Move {
        for m in MOVES {
            if m.outcome_with(&self.them) == self.desired_outcome {
                // dbg!(&self.them, &m, &self.desired_outcome);
                return m;
            }
        }

        panic!("unable to find desired move");
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let mut rounds: Vec<Round> = vec![];

    for line in input.lines() {
        let (them, us) = line.split_once(' ').unwrap();

        rounds.push(Round {
            them: them.parse().unwrap(),
            us: us.parse().unwrap(),
        });
    }

    let outcome: u32 = rounds
        .iter()
        .map(|round| round.us.score() as u32 + round.outcome().score())
        .sum();

    Some(outcome)
}

pub fn part_two(input: &str) -> Option<u32> {
    // change the parsing logic so the second key is actually the desired outcome
    let mut rounds: Vec<Round2> = vec![];

    for line in input.lines() {
        let (them, desired_outcome) = line.split_once(' ').unwrap();

        rounds.push(Round2 {
            them: them.parse().unwrap(),
            desired_outcome: desired_outcome.parse().unwrap(),
        });
    }

    let score: u32 = rounds
        .iter()
        .map(|round| round.our_move().score() as u32 + round.desired_outcome.score())
        .sum();

    Some(score)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 2);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 2);
        assert_eq!(part_one(&input), Some(15));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 2);
        assert_eq!(part_two(&input), Some(12));
    }
}

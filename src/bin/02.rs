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

#[derive(Debug)]
enum Outcome {
    Win,
    Draw,
    Loss,
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

fn parse(input: &str) -> Result<Vec<Round>, Box<dyn Error>> {
    let mut rounds: Vec<Round> = vec![];

    for line in input.lines() {
        let (them, us) = line.split_once(' ').unwrap();

        rounds.push(Round {
            them: them.parse().unwrap(),
            us: us.parse().unwrap(),
        });
    }

    Ok(rounds)
}

pub fn part_one(input: &str) -> Option<u32> {
    let rounds = parse(input).unwrap();

    let outcome: u32 = rounds
        .iter()
        .map(|round| round.us.score() as u32 + round.outcome().score())
        .sum();

    Some(outcome)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
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
        assert_eq!(part_two(&input), None);
    }
}

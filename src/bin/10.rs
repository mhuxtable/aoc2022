use std::{borrow::Borrow, error::Error, fmt::Display, str::FromStr};

#[derive(Clone, Debug)]
enum Instruction {
    Noop,
    Addx(isize),
}

#[derive(Debug)]
enum ParseInstructionError {
    InvalidInstruction,
    InsufficientArguments,
    ArgumentParseError(Box<dyn std::error::Error>),
}

impl Error for ParseInstructionError {}

impl Display for ParseInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInstruction => write!(f, "invalid instruction"),
            Self::InsufficientArguments => write!(f, "insufficient arguments for instruction"),
            Self::ArgumentParseError(e) => write!(f, "arguments could not parse: {}", e),
        }
    }
}

impl FromStr for Instruction {
    type Err = ParseInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        match parts[0] {
            "noop" => Ok(Self::Noop),
            "addx" => {
                if parts.len() < 2 {
                    Err(Self::Err::InsufficientArguments)
                } else {
                    Ok(Self::Addx(
                        parts[1]
                            .parse()
                            .map_err(|e| Self::Err::ArgumentParseError(Box::new(e)))?,
                    ))
                }
            }
            _ => Err(Self::Err::InvalidInstruction),
        }
    }
}

impl Instruction {
    pub fn ticks(&self) -> usize {
        match self {
            Self::Noop => 1,
            Self::Addx(_) => 2,
        }
    }
}

fn parse(input: &str) -> Result<Vec<Instruction>, ParseInstructionError> {
    input.lines().map(|l| l.parse()).collect()
}

struct CPU<I: IntoIterator> {
    instrs: I::IntoIter,
    reg_x: isize,
    current_instruction: Option<Instruction>,
    remaining_ticks: usize,
}

impl<I: IntoIterator<Item = Instruction>> CPU<I> {
    pub fn new(instructions: I) -> CPU<I> {
        CPU {
            instrs: instructions.into_iter(),
            reg_x: 1,
            current_instruction: None,
            remaining_ticks: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.remaining_ticks == 0 {
            self.current_instruction = self.instrs.next().and_then(|i| {
                self.remaining_ticks = i.ticks();
                Some(i)
            });
        }

        if self.current_instruction.is_none() {
            return false;
        }

        let i = self.current_instruction.as_ref().unwrap();

        match i {
            Instruction::Noop => {}
            Instruction::Addx(x) => {
                if self.remaining_ticks == 1 {
                    self.reg_x += x;
                }
            }
        }

        self.remaining_ticks -= 1;
        return true;
    }

    pub fn get_x(&self) -> isize {
        self.reg_x
    }
}

fn run_scores<I: IntoIterator<Item = Instruction>>(cpu: &mut CPU<I>) -> i32 {
    let mut score = 0;

    // Cycles don't start counting at 0!
    for cycle in 1..=220 {
        // Signal strength is computed before the CPU ticks
        if cycle % 40 == 20 {
            score += cpu.get_x() as i32 * cycle;
        }

        if !cpu.tick() {
            panic!("ran out of instructions");
        }
    }

    score
}

pub fn part_one(input: &str) -> Option<i32> {
    let instructions = parse(input).expect("error parsing input");
    let mut cpu = CPU::new(instructions);
    Some(run_scores(&mut cpu))
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 10);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 10);
        assert_eq!(part_one(&input), Some(13140));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 10);
        assert_eq!(part_two(&input), None);
    }
}

// This is a really nice exercise, although it has gotchas in terms of off-by-one errors all over
// the place (e.g. cycle count starts ticking at 1, but signal strength in part 1 is computed
// before the tick is over, while pixels start counting at 0).
//
// This is more verbose than perhaps it could be due to (1) use of the std::str::FromStr trait to
// implement parsing, which depends on defining the FromStr::Err type and corresponding
// std::error::Error trait implementation on the defined error type and (2) additional defence in
// depth in each part to check various assertions that can be assumed to pass for the typical AoC
// puzzle inputs.
//
// It's also interesting that vertical position is not tracked on the CRT. That gives momentary
// pause to determine what the various computations to make for drawing the pixel will be.

use std::{error::Error, fmt::Display, str::FromStr};

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

const CRT_SIZE: usize = 240;
const CRT_ROWS: usize = 6;
const CRT_WIDTH: usize = CRT_SIZE / CRT_ROWS;

struct CPU<I: IntoIterator> {
    instrs: I::IntoIter,
    cycle: usize,
    reg_x: isize,
    current_instruction: Option<Instruction>,
    remaining_ticks: usize,
    crt: Vec<bool>,
}

impl<I: IntoIterator<Item = Instruction>> CPU<I> {
    pub fn new(instructions: I) -> CPU<I> {
        CPU {
            instrs: instructions.into_iter(),
            cycle: 0,
            reg_x: 1,
            current_instruction: None,
            remaining_ticks: 0,
            crt: vec![false; CRT_SIZE],
        }
    }

    // returns whether there are any more instructions, the cycle count and the previous value of register X
    pub fn tick(&mut self) -> (bool, usize, isize) {
        if self.remaining_ticks == 0 {
            self.current_instruction = self.instrs.next().and_then(|i| {
                self.remaining_ticks = i.ticks();
                Some(i)
            });
        }

        if self.current_instruction.is_none() {
            // cycle doesn't tick after we run out of instructions? Maybe it should
            return (false, self.cycle, self.reg_x);
        }

        // Pixels are numbered 0 to 39, etc.
        let current_pixel = (self.cycle % CRT_WIDTH) as isize;

        // The sprite is 3 pixels wide, which means it protrudes 1 pixel to the left and 1 pixel to
        // the right relative to the value of register X (which determines the middle pixel).
        let visible = (self.reg_x - 1) <= current_pixel && (self.reg_x + 1) >= current_pixel;

        // The inputs don't seem to provide more instructions than CRT_SIZE, so we don't have to
        // wrap and overwrite previous video output state, but in principle they could wrap.
        self.crt[self.cycle % CRT_SIZE] = visible;

        let x = self.reg_x;

        let i = self.current_instruction.as_ref().unwrap();
        match i {
            Instruction::Noop => {}
            Instruction::Addx(x) => {
                if self.remaining_ticks == 1 {
                    self.reg_x += x;
                }
            }
        }

        self.cycle += 1;
        self.remaining_ticks -= 1;
        return (true, self.cycle, x);
    }

    pub fn get_crt(&self) -> String {
        let mut display = String::new();

        for (i, &pixel) in self.crt.iter().enumerate() {
            if i > 0 && i % (CRT_SIZE / CRT_ROWS) == 0 {
                display.push('\n');
            }

            display.push(if pixel { '#' } else { '.' });
        }

        display
    }
}

// cheating a bit by decoding all instructions ahead of time rather than as a decode stage within
// the CPU. Fine for this :-)
fn parse(input: &str) -> Result<Vec<Instruction>, ParseInstructionError> {
    input.lines().map(|l| l.parse()).collect()
}

const MAX_SCORE_CYCLE: usize = 220;

pub fn part_one(input: &str) -> Option<i32> {
    let instructions = parse(input).expect("error parsing input");
    let mut cpu = CPU::new(instructions);
    let mut score = 0;

    loop {
        let (more, cycle, x) = cpu.tick();

        // Cycles start counting at 1 for purposes of signal strength. The signal strength is
        // computed using the X value at the start of the cycle.
        if cycle % 40 == 20 {
            score += x as i32 * cycle as i32;
        } else if cycle > MAX_SCORE_CYCLE {
            // Don't want to process any instructions beyond cycle 220 as we are not asked for
            // scores beyond that cycle, despite the input not seeming to provide them.
            break;
        }

        if !more {
            if cycle < MAX_SCORE_CYCLE {
                panic!("ran out of instructions!");
            }

            break;
        }
    }

    Some(score)
}

pub fn part_two(input: &str) -> Option<String> {
    let instructions = parse(input).expect("error parsing input");
    let mut cpu = CPU::new(instructions);
    let mut more = true;
    while more {
        (more, _, _) = cpu.tick();
    }
    let crt = cpu.get_crt();
    // println!("{}", crt);

    Some(crt)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 10);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

const PART_TWO: &str = "##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....";

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
        assert_eq!(part_two(&input), Some(PART_TWO.to_string()));
    }
}

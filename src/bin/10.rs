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

const CRT_SIZE: usize = 240;
const CRT_ROWS: usize = 6;
const CRT_WIDTH: usize = CRT_SIZE / CRT_ROWS;
const SPRITE_WIDTH: usize = 3;

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

    pub fn tick(&mut self) -> bool {
        self.cycle += 1;

        if self.remaining_ticks == 0 {
            self.current_instruction = self.instrs.next().and_then(|i| {
                self.remaining_ticks = i.ticks();
                Some(i)
            });
        }

        if self.current_instruction.is_none() {
            return false;
        }

        // Pixels are numbered 0 to 39, etc. but cycle will be 1 to 40 for the first row.
        let current_pixel = ((self.cycle - 1) % CRT_WIDTH) as isize;
        // The sprite is SPRITE_WIDTH wide, which means it protrudes (SPRITE_WIDTH-1)/2 to the left
        // and right relative to the value of the register that determines its middle.
        let visible = (self.reg_x - (SPRITE_WIDTH as isize - 1) / 2) <= current_pixel
            && (self.reg_x + (SPRITE_WIDTH as isize - 1) / 2) >= current_pixel;
        // The inputs don't seem to wrap the CPU state beyond cycle CRT_SIZE, but in principle they
        // could wrap and require us to overwrite previous output frame state.
        self.crt[(self.cycle - 1) % CRT_SIZE] = visible;

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

    pub fn get_cycle(&self) -> usize {
        self.cycle
    }

    pub fn get_x(&self) -> isize {
        self.reg_x
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

pub fn part_one(input: &str) -> Option<i32> {
    let instructions = parse(input).expect("error parsing input");
    let mut cpu = CPU::new(instructions);
    let mut score = 0;

    loop {
        // Cycles start at 1, the CPU will only update cycle after the first tick, but signal
        // strength is computed from before the tick applies to the state.
        let cycle = cpu.get_cycle() + 1;

        if cycle % 40 == 20 {
            score += cpu.get_x() as i32 * cycle as i32;
        }

        if !cpu.tick() {
            if cycle < 220 {
                panic!("ran out of instructions!");
            } else {
                break;
            }
        }
    }

    Some(score)
}

pub fn part_two(input: &str) -> Option<String> {
    let instructions = parse(input).expect("error parsing input");
    let mut cpu = CPU::new(instructions);
    while cpu.tick() {}
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

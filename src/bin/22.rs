use std::{
    fmt::Display,
    str::{FromStr, Lines},
};

use advent_of_code::helpers::{Grid, Point};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Cell {
    Nothingness, // Just here to make storing the map as a uniform rectangular shape easier.
    Open,        // Places we can move, i.e. "."
    Wall,        // Places we cannot move, i.e. "#"
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Nothingness => ' ',
                Self::Open => '.',
                Self::Wall => '#',
            }
        )
    }
}

impl TryFrom<char> for Cell {
    type Error = CellStringParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Open),
            '#' => Ok(Self::Wall),
            ' ' => Ok(Self::Nothingness),
            _ => Err(Self::Error {}),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::Nothingness
    }
}

#[derive(Debug)]
struct CellStringParseError {}

impl Display for CellStringParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing cell string")
    }
}

impl std::error::Error for CellStringParseError {}

impl FromStr for Cell {
    type Err = CellStringParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            " " => Ok(Self::Nothingness),
            "." => Ok(Self::Open),
            "#" => Ok(Self::Wall),
            _ => Err(Self::Err {}),
        }
    }
}

#[derive(Clone, Debug)]
struct CellState {
    cell: Cell,
    visited: std::cell::Cell<Option<Direction>>,
}

impl Default for CellState {
    fn default() -> Self {
        Self {
            cell: Default::default(),
            visited: std::cell::Cell::new(None),
        }
    }
}

impl Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(visited) = self.visited.get() {
            (visited).fmt(f)
        } else {
            (self.cell).fmt(f)
        }
    }
}

#[derive(Clone, Debug)]
enum Instruction {
    Forward(usize),
    Clockwise,
    Anticlockwise,
}

#[derive(Debug)]
struct ParseInstructionError {}

impl Display for ParseInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing instruction")
    }
}

impl std::error::Error for ParseInstructionError {}

impl FromStr for Instruction {
    type Err = ParseInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(steps) = s.parse::<usize>() {
            Ok(Self::Forward(steps))
        } else {
            match s {
                "L" => Ok(Self::Anticlockwise),
                "R" => Ok(Self::Clockwise),
                _ => Err(Self::Err {}),
            }
        }
    }
}

fn parse(input: Lines) -> (Grid<CellState>, Vec<Instruction>) {
    let grid = {
        let (map_lines, longest_line) = {
            let mut longest_line = 0;
            let lines: Vec<&str> = input
                .clone()
                .take_while(|l| !l.is_empty())
                .inspect(|l| {
                    if l.len() > longest_line {
                        longest_line = l.len();
                    }
                })
                .collect();

            (lines, longest_line)
        };

        let mut grid = Grid::new(longest_line, map_lines.len());

        for row in 0..map_lines.len() {
            for col in 0..longest_line {
                *grid.point_mut(&Point { x: col, y: row }) = {
                    let ch = if let Some(ch) = map_lines[row].chars().nth(col) {
                        ch.try_into().unwrap()
                    } else {
                        break;
                    };

                    CellState {
                        cell: ch,
                        visited: std::cell::Cell::new(None),
                    }
                }
            }
        }

        grid
    };

    let instructions = {
        let mut instructions = vec![];
        let mut cur = String::new();

        let mut it = input.last().unwrap().chars();

        loop {
            let ch = it.next();

            if ch.is_none()
                || (!cur.is_empty()
                    && cur.chars().last().unwrap().is_digit(10) != ch.unwrap().is_digit(10))
            {
                // Change of type so record the instruction
                instructions.push(cur.parse().unwrap());
                cur.clear();
            }

            if ch.is_some() {
                cur.push(ch.unwrap());
            } else {
                break;
            }
        }

        instructions
    };

    (grid, instructions)
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    pub fn score(&self) -> u32 {
        match self {
            Self::North => 3,
            Self::East => 0,
            Self::South => 1,
            Self::West => 2,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::North => '^',
                Self::East => '>',
                Self::South => '⌄',
                Self::West => '<',
            }
        )
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (grid, instructions) = parse(input.lines());

    let (mut x, mut y) = (
        (0..grid.width())
            .find(|&x| grid.point(&Point { x, y: 0 }).cell == Cell::Open)
            .unwrap(),
        0,
    );

    let mut direction = Direction::East;

    fn can_wrap<F: Fn(usize) -> Point, I: Iterator<Item = usize>>(
        grid: &Grid<CellState>,
        it: I,
        point_fn: F,
    ) -> Option<usize> {
        for candidate in it {
            match grid.point(&point_fn(candidate)).cell {
                Cell::Nothingness => {
                    continue;
                }
                Cell::Open => return Some(candidate),
                Cell::Wall => return None,
            }
        }

        None
    }

    let find_next = |max: usize, cur: usize, rev, point_fn: Box<dyn Fn(usize) -> Point>| {
        let candidate = if rev { cur.saturating_sub(1) } else { cur + 1 };

        if (rev && cur == 0)
            || (!rev && cur == max)
            || (*grid.point(&point_fn(candidate))).cell == Cell::Nothingness
        {
            let mut range: Box<dyn DoubleEndedIterator<Item = usize>> =
                Box::new((cur + 1..=max).chain(0..cur));
            if rev {
                range = Box::new(range.rev());
            }

            if let Some(next) = can_wrap(&grid, range, point_fn) {
                next
            } else {
                cur
            }
        } else if grid.point(&point_fn(candidate)).cell == Cell::Wall {
            cur
        } else {
            candidate
        }
    };

    for instruction in instructions {
        println!("{:?} ({},{}) {:?}\n{}", instruction, x, y, direction, grid);

        match instruction {
            Instruction::Anticlockwise => {
                direction = match direction {
                    Direction::North => Direction::West,
                    Direction::East => Direction::North,
                    Direction::South => Direction::East,
                    Direction::West => Direction::South,
                };
            }
            Instruction::Clockwise => {
                direction = match direction {
                    Direction::North => Direction::East,
                    Direction::East => Direction::South,
                    Direction::South => Direction::West,
                    Direction::West => Direction::North,
                };
            }
            Instruction::Forward(steps) => {
                (x, y) = (0..steps).fold((x, y), |(x, y), _| {
                    let (next_x, next_y) = match direction {
                        Direction::North => (
                            x,
                            find_next(
                                grid.height() - 1,
                                y,
                                true,
                                Box::new(move |y| Point { x, y }),
                            ),
                        ),
                        Direction::East => (
                            find_next(
                                grid.width() - 1,
                                x,
                                false,
                                Box::new(move |x| Point { x, y }),
                            ),
                            y,
                        ),
                        Direction::South => (
                            x,
                            find_next(
                                grid.height() - 1,
                                y,
                                false,
                                Box::new(move |y| Point { x, y }),
                            ),
                        ),
                        Direction::West => (
                            find_next(grid.width() - 1, x, true, Box::new(move |x| Point { x, y })),
                            y,
                        ),
                    };

                    grid.point(&Point {
                        x: next_x,
                        y: next_y,
                    })
                    .visited
                    .set(Some(direction));

                    (next_x, next_y)
                });
            }
        };
    }

    Some((y as u32 + 1) * 1000 + (x as u32 + 1) * 4 + direction.score())
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 22);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 22);
        assert_eq!(part_one(&input), Some(6032));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 22);
        assert_eq!(part_two(&input), None);
    }
}

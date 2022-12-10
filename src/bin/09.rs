// This exercise took me too long to solve and I needed to compare with @Humpheh's solution to
// figure out where it was going wrong. The issue was something to do with the logic for resolving
// the diagonal resolution; I think I was too overzealous in moving knots. This new solution is
// better and avoids a nested loop to make the knots follow.
//
// I'm still not super impressed with the length of the solution, but it does print pretty pictures
// to show the movement of the rope (windowed on the head) as it runs, reminding me of Conway's
// Game of Life that I implemented in a past University life :-)
//
// I was taken aback by the exercise description when first opening it, and found it challenging to
// decipher the story description, but I realise implementing it that's it's just an adaptation of
// Snake from my first Nokia!

mod day09 {
    use std::{fmt::Display, str::FromStr};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Display for Direction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Up => write!(f, "U"),
                Self::Down => write!(f, "D"),
                Self::Left => write!(f, "L"),
                Self::Right => write!(f, "R"),
            }
        }
    }

    #[derive(Debug)]
    pub struct ParseDirectionError;

    impl std::error::Error for ParseDirectionError {}

    impl Display for ParseDirectionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "ParseDirectionError")
        }
    }

    impl FromStr for Direction {
        type Err = ParseDirectionError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "U" => Ok(Self::Up),
                "D" => Ok(Self::Down),
                "L" => Ok(Self::Left),
                "R" => Ok(Self::Right),
                _ => Err(ParseDirectionError),
            }
        }
    }

    #[derive(Debug)]
    pub struct Move {
        pub dir: Direction,
        pub steps: usize,
    }

    impl Move {
        pub fn new(dir: Direction, steps: usize) -> Move {
            Move { dir, steps }
        }
    }

    impl Display for Move {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} {}", self.dir, self.steps)
        }
    }

    #[derive(Debug)]
    pub struct ParseMoveError;

    impl std::error::Error for ParseMoveError {}

    impl Display for ParseMoveError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "ParseMoveError")
        }
    }

    impl FromStr for Move {
        type Err = ParseMoveError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (direction, steps) = s.split_once(" ").ok_or(Self::Err {})?;

            Ok(Self::new(
                direction.parse().map_err(|_| ParseMoveError {})?,
                steps.parse().map_err(|_| ParseMoveError {})?,
            ))
        }
    }

    #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
    pub struct Point {
        pub x: isize,
        pub y: isize,
    }

    impl AsRef<Point> for Point {
        fn as_ref(&self) -> &Point {
            &self
        }
    }
}

use std::collections::{HashSet, VecDeque};

use day09::{Direction, Move, Point};

struct Grid {
    tail_visits: HashSet<Point>,
    rope: Rope,
}

struct Rope {
    knots: VecDeque<Point>,
}

impl Rope {
    pub fn new(start: &Point, knots: usize) -> Rope {
        Rope {
            knots: (0..knots).map(|_| start.clone()).collect(),
        }
    }

    pub fn head(&self) -> &Point {
        self.knots.front().unwrap()
    }

    pub fn tail(&self) -> &Point {
        self.knots.back().unwrap()
    }

    pub fn move_head(&mut self, dir: Direction) {
        let mut new_rope = VecDeque::new();

        let mut head = self.knots.pop_front().unwrap();

        match dir {
            Direction::Up => head.y -= 1,
            Direction::Down => head.y += 1,
            Direction::Left => head.x -= 1,
            Direction::Right => head.x += 1,
        }

        new_rope.push_front(head);

        for mut knot in self.knots.iter_mut() {
            let last_knot = new_rope.back().unwrap();
            let (dx, dy) = (last_knot.x - knot.x, last_knot.y - knot.y);

            // play catch up with the rest of the rope
            if dx.abs() <= 1 && dy.abs() <= 1 {
                // do nothing
            } else if dx == 0 {
                knot.y = knot.y + dy.signum();
            } else if dy == 0 {
                knot.x = knot.x + dx.signum();
            } else {
                // diagonal
                knot.x = knot.x + dx.signum();
                knot.y = knot.y + dy.signum();
            }

            new_rope.push_back(*knot);
        }

        self.knots = new_rope;
    }

    pub fn has_knot(&self, p: &Point) -> Option<usize> {
        self.knots
            .iter()
            .enumerate()
            .find(|(_, &knot)| knot == *p)
            .map(|(i, _)| i)
    }
}

impl Grid {
    pub fn new(knots: usize) -> Grid {
        let start = Point { x: 0, y: 0 };

        let mut tail_visits = HashSet::new();
        tail_visits.insert(start);

        Grid {
            tail_visits,
            rope: Rope::new(&start, knots),
        }
    }

    fn move_knots(&mut self, dir: Direction) {
        self.rope.move_head(dir);
        self.tail_visits.insert(*self.rope.tail());
    }

    pub fn apply_move(&mut self, m: &Move) {
        (0..m.steps).for_each(|_| self.move_knots(m.dir));
    }

    pub fn total_tail_visits(&self) -> usize {
        self.tail_visits.len()
    }

    pub fn display_around(&self, p: &Point) -> String {
        let mut out = String::new();

        let window_size = 20.max(self.rope.knots.len() as isize);

        for y in p.y - window_size..=p.y + window_size {
            out.push('\n');

            for x in p.x - window_size..=p.x + window_size {
                out.push_str(
                    if let Some(pos) = self.rope.has_knot(&Point { x, y }) {
                        if pos == self.rope.knots.len() - 1 {
                            "T".to_string()
                        } else if pos == 0 {
                            "H".to_string()
                        } else {
                            format!("{}", pos)
                        }
                    } else if self.tail_visits.contains(&Point { x, y }) {
                        "#".to_string()
                    } else {
                        ".".to_string()
                    }
                    .as_str(),
                )
            }
        }

        out.push_str(format!(" tail visits: {}\n", self.total_tail_visits()).as_str());

        out
    }
}

fn parse_input(input: &str) -> Vec<Move> {
    input.lines().map(|line| line.parse().unwrap()).collect()
}

fn print_step(m: &Move, grid: &Grid) {
    if PRINT_STEPS {
        println!(
            "=======\n{}\n\n{}\n",
            &m,
            grid.display_around(&grid.rope.head())
        );
    }
}

static PRINT_STEPS: bool = true;

pub fn part_one(input: &str) -> Option<u32> {
    let moves = parse_input(input);
    let mut grid = Grid::new(2);

    for m in moves {
        grid.apply_move(&m);
        print_step(&m, &grid);
    }

    Some(grid.total_tail_visits() as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    let moves = parse_input(input);
    let mut grid = Grid::new(10);

    for m in moves {
        grid.apply_move(&m);
        print_step(&m, &grid);
    }

    Some(grid.total_tail_visits() as u32)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 9);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 9);
        assert_eq!(part_one(&input), Some(88));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 9);
        assert_eq!(part_two(&input), Some(36));
    }
}

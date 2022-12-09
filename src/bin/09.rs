mod day08 {
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

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Point {
        pub x: isize,
        pub y: isize,
    }

    impl AsRef<Point> for Point {
        fn as_ref(&self) -> &Point {
            &self
        }
    }

    impl Point {
        pub fn adjacent_points(&self, grid_size: usize) -> Vec<Point> {
            let mut adjacencies = vec![];

            for x in -1..=1 {
                for y in -1..=1 {
                    let ax = self.x + x;
                    let ay = self.y + y;

                    if ax < 0
                        || ay < 0
                        || ax as usize > grid_size - 1
                        || ay as usize > grid_size - 1
                    {
                        continue;
                    }

                    adjacencies.push(Point { x: ax, y: ay });
                }
            }

            adjacencies
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn point_eq() {
            assert!(Point { x: 1, y: 1 } == Point { x: 1, y: 1 });
        }
    }
}

use day08::Point;

#[derive(Debug)]
struct ApplyMoveError;

struct Grid {
    // grid is a square represented in a 1-D array. I guess that means I need to write the boundary
    // logic
    tail_visits: Vec<bool>,
    grid_size: usize,
    head: Point,
    tail: Point,
}

impl Grid {
    pub fn new(size: usize) -> Grid {
        assert!(size > 0);

        // head and tail start in the middle
        let start = Point {
            x: size as isize / 2,
            y: size as isize / 2,
        };

        let mut grid = Grid {
            tail_visits: vec![false; size.pow(2)],
            grid_size: size,
            head: start,
            tail: start,
        };

        // first point is visited
        *grid.visit_of_point_mut(start) = true;

        grid
    }

    fn visit_of_point_mut(&mut self, point: impl AsRef<Point>) -> &mut bool {
        let point = point.as_ref();
        let offset = point.y * self.grid_size as isize + point.x;

        &mut self.tail_visits[offset as usize]
    }

    fn is_out_of_bounds(&self, point: impl AsRef<Point>) -> bool {
        let point = point.as_ref();

        point.x < 0
            || point.x >= self.grid_size as isize
            || point.y < 0
            || point.y >= self.grid_size as isize
    }

    fn move_knots(&mut self, dir: day08::Direction) -> Result<(), ApplyMoveError> {
        match dir {
            day08::Direction::Up => self.head.y -= 1,
            day08::Direction::Down => self.head.y += 1,
            day08::Direction::Left => self.head.x -= 1,
            day08::Direction::Right => self.head.x += 1,
        }

        if self.is_out_of_bounds(self.head) {
            return Err(ApplyMoveError {});
        }

        loop {
            // println!("tail resolver\n{}\n", self.display_around(&self.tail));

            // play catch up with the tail
            if self
                .head
                .adjacent_points(self.grid_size)
                .contains(&self.tail)
            {
                break;
            }

            if self.head.x == self.tail.x && self.head.y != self.tail.y {
                // move tail in direction of head
                if self.head.y < self.tail.y {
                    self.tail.y -= 1;
                } else {
                    self.tail.y += 1;
                }
            } else if self.head.x != self.tail.x && self.head.y == self.tail.y {
                if self.head.x < self.tail.x {
                    self.tail.x -= 1;
                } else {
                    self.tail.x += 1;
                }
            } else {
                // diagonal move, which we'll implement by moving in the axis that has a difference
                // with head axis coord of at least 2, then let the next loop iteration move
                // horizontally or vertically. Note that the tail would be considered adjacent if
                // we moved just by one step, but it wouldn't be a diagonal move and we'll possibly
                // miscount due to repeated visits.
                //
                // e.g. in the below (incorrect) example:
                //
                // .....    .....    .....
                // .....    .....    .....
                // ..H.. -> ...H. -> ..TH.
                // .T...    .T...    .....
                // .....    .....    .....
                //
                // if in step 2 of 3 we move the tail one step to the right, we'll consider it to
                // be adjacent to the head and terminate the loop, but this will count the wrong
                // location visited by the tail, which may lead to inaccurate reporting as the
                // exercise input most likely exercises this possible error condition by visiting
                // some places that result from a diagonal move multiple times.
                if self.head.x.abs_diff(self.tail.x) >= 2 {
                    self.tail.y = self.head.y;
                } else if self.head.y.abs_diff(self.tail.y) >= 2 {
                    self.tail.x = self.head.x;
                } else {
                    panic!("tail is too far from head to know how to resolve");
                }
            }
        }

        *self.visit_of_point_mut(self.tail) = true;

        Ok(())
    }

    pub fn apply_move(&mut self, m: &day08::Move) -> Result<(), ApplyMoveError> {
        for i in 0..m.steps {
            self.move_knots(m.dir)?;
            // println!("apply move {}\n{}\n", i, self.display_around(&self.head));
        }

        Ok(())
    }

    pub fn total_tail_visits(&self) -> usize {
        self.tail_visits.iter().filter(|&v| *v).count()
    }

    pub fn display_around(&self, p: &Point) -> String {
        let mut out = String::new();

        const WINDOW_SIZE: isize = 4;

        for y in p.y - WINDOW_SIZE..=p.y + WINDOW_SIZE {
            let mut overlap = false;

            for x in p.x - WINDOW_SIZE..=p.x + WINDOW_SIZE {
                if self.head == self.tail && self.head.x == x && self.head.y == y {
                    out.push('*');
                    overlap = true;
                } else if self.head.x == x && self.head.y == y {
                    out.push('H');
                } else if self.tail.x == x && self.tail.y == y {
                    out.push('T');
                } else if self.tail_visits[(y * self.grid_size as isize + x) as usize] {
                    out.push('#');
                } else {
                    out.push('.');
                }
            }

            if overlap {
                out.push_str(" (*) = head and tail overlap")
            }

            out.push('\n');
        }

        out.pop();
        out.push_str(format!(" tail visits: {}", self.total_tail_visits()).as_str());
        out.push('\n');

        out
    }
}

fn parse_input(input: &str) -> Vec<day08::Move> {
    let mut moves = vec![];

    for line in input.lines() {
        moves.push(line.parse().unwrap());
    }

    moves
}

pub fn part_one(input: &str) -> Option<u32> {
    let moves = parse_input(input);
    let mut grid = Grid::new(1000);

    for m in moves {
        grid.apply_move(&m).expect("applying move");

        println!("=======\n{}\n\n{}\n", &m, grid.display_around(&grid.head));
    }

    Some(grid.tail_visits.iter().filter(|&v| *v).count() as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
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
        assert_eq!(part_one(&input), Some(13));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 9);
        assert_eq!(part_two(&input), None);
    }
}

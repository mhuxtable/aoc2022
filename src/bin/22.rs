use std::{
    collections::HashMap,
    fmt::Display,
    str::{FromStr, Lines},
};

use advent_of_code::helpers::{Grid, Point};
use itertools::Itertools;

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
        // println!("{:?} ({},{}) {:?}", instruction, x, y, direction);

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
    let (grid, instructions) = parse(input.lines());

    // number of cells in net = (side length ** 2) * 6
    let side_length =
        ((grid.iter().filter(|&x| x.cell != Cell::Nothingness).count() / 6) as f64).sqrt() as usize;

    let mut sides = vec![];

    {
        let (mut x, mut y) = (0, 0);

        loop {
            if grid.point(&Point { x, y }).cell != Cell::Nothingness {
                // we have four sides from this point. Sides are tracked over intervals half open,
                // i.e. from is included in the side but to is the first row/column of points of
                // the side on the adjoining face. This makes it easier to match with other sides
                // that are directly connected in the net.
                let (top_left, top_right, bottom_left, bottom_right) = (
                    (x, y),
                    (x + side_length, y),
                    (x, y + side_length),
                    (x + side_length, y + side_length),
                );

                let face_sides = vec![
                    (top_left, top_right),       // top
                    (top_right, bottom_right),   // right
                    (bottom_left, bottom_right), // bottom
                    (top_left, bottom_left),     // left
                ];

                sides.extend(face_sides);
            }

            assert!(sides.len() <= 24);
            if sides.len() == 24 {
                break;
            }

            x += side_length;

            if x >= grid.width() {
                x = 0;
                y += side_length;
            }
        }
    }

    let mut connections = HashMap::new();

    for (idx, coords) in sides.iter().enumerate() {
        let others: Vec<usize> = sides
            .iter()
            .positions(|other| coords == other)
            .filter(|&other| other != idx)
            .collect();

        assert!(others.len() <= 1);

        if others.len() == 1 {
            connections.insert(idx, *others.first().unwrap());
        }
    }

    // fold the net by looking for "L" shapes in the connections already made, starting with the
    // connections that were made by parsing the faces in the provided net
    while connections.len() < 24 {
        println!("{:?}", connections);

        for face in 0..6 {
            let possible_l_shaped_connections: Vec<(usize, usize)> = vec![
                (0, 1), // up-right
                (1, 2), // right-down
                (2, 3), // down-left
                (3, 0), // left-up
            ]
            .into_iter()
            .map(|(side1, side2)| (face * 4 + side1, face * 4 + side2))
            .collect();

            for (side1, side2) in possible_l_shaped_connections {
                let (other_side1, other_side2) = (connections.get(&side1), connections.get(&side2));

                if other_side1.is_none() || other_side2.is_none() {
                    // no L-shaped connection here
                    continue;
                }

                // We have found a connection that can be made. For example, consider two connected
                // faces in an L shape on a planar net. Number the faces as below:
                //
                // +---+
                // |   |
                // | 1 |
                // |   |
                // +---+---+
                // |   |   |
                // | 2 | 3 |
                // |   |   |
                // +---+---+
                //
                // and refer to edges of faces according to their ordinal directions, written
                // face(ordinal), e.g. 1(N) is the top-most edge and 3(E) is the right-most edge.
                // Write a pair of sides (i.e. they are connected) as w(x)/y(z).
                //
                // From face 2, we have found an L shape to face 1 across sides 2(N)/1(S) and face
                // 3 across 2(E)/3(W). To find the sides to connect on faces 1 and 3, invert the
                // ordinal directions used to access the face from the central face (here, face 2).
                // i.e. in this example, the connection will be made between 1(E)/3(N) (because
                // face 1 was accessed by walking north from face 2, and face 3 by walking east
                // from face 2).
                //
                // This connection can be made directly. However, in some cases, we need to apply a
                // rotation factor before selecting the index of the side to connect. This occurs
                // where a net is partially constructed in 3D and a previous connection between
                // faces resulted in the face effectively being rotated by 90º or 180º relative to
                // the initial plane.
                //
                // The rotation factor can be determined by essentially mapping the 3D shape back
                // to a planar net and inspecting whether the connections used to form the L would
                // be expected in the planar net. A representation of a scenario where this occurs
                // cannot be given pictorially as it relies on the net being partially constructed
                // in 3D. However, in the case that a connection is not as expected in the planar
                // net, a rotation factor is applied to make the connection planar. Consider the
                // connected sides of the L shape. In the example above, these are:
                //
                //   Face 1: South
                //   Face 2: North  East
                //   Face 3:        West
                //
                // These ordinal directions are all as expected. However, suppose that when
                // following the connection from face 2 to face 3, we instead walked onto the
                // northern face of face 3:
                //
                //   Face 1: South
                //   Face 2: North  East
                //   Face 3:        North <-- a rotation occurred when face 3 was connected
                //
                // This indicates a rotation has occurred when face 2 was previously connected to
                // face 3. It is clear that we cannot connect 1(E)/3(N) because 3(N) already forms
                // the connection 2(E)/3(N). To determine the side to connect, face 3's sides must
                // be rotated to the expected planar format, i.e. in a plane, walking east across
                // 2(E) would be expected to find 3(W). We've actually found 3(N), so the rotation
                // is 90º anti-clockwise:
                //
                //             +-E-+
                //             |   |
                //  <- face 2  N 3 S
                //             |   |
                //             +-W-+
                //
                // i.e. we have a rotation of 90º anti-clockwise. To undo the rotation, where we
                // would expect to pick 3(N) (because 1(S)/2(N) was used to find the other side of
                // the "L"), we now pick 3(N) + 90º clockwise = 3(E). The connection is thus made
                // between 1(E)/3(E).
                //
                // All of the logic in relation to numeric sides can be determined modulo 4.
                //
                // This algorithm generalises to L shapes in any rotation.

                let (other_side1, other_side2) = (*other_side1.unwrap(), *other_side2.unwrap());

                // Input side1 from the "central" face of the L (side 2 in the example) and side2
                // from the arm (side 3 in the example). This would give in the 90º rotated
                // example. To find the side of the face, divide modulo 4 (4 sides) yielding N=0,
                // E=1, S=2, W=3. The number of 90º rotations required is thus |side1-side3|-2 and
                // the direction is -1 * signum(side1-side3-2).
                //
                // For example, the planar case (no rotation) has 2(E)/3(W). 90º rotations required
                // is |E-W|-2 = |1-3|-2 = |-2|-2 = 0.
                //
                // For the non-planar case of 2(E)/3(N), we're expecting to identify side 3(E) as
                // the "northern" side to connect with. This is obtained as:
                //
                //     rotations: |E-N|-2 = |1-0|-2 = |-1| = 1.
                //     direction: -1 * signum(1-0-2) = 1 (i.e. clockwise)
                //
                // Thus we'll take the "northern" side on face 3 as side 0 + 1 (mod 4) = side 1
                // i.e. we actually want the index to connect with as being the East side.
                //
                // We arrange for the rotation to include the opposite factor of 2 as standard.
                //
                // side1 = central, side2 = arm
                let rotation = |side1: usize, side2: usize| {
                    let diff: isize = (side1 as isize % 4) - (side2 as isize % 4);
                    -1 * ((diff - 2) % 4).signum() * (diff - 2).abs()
                };

                let (rotate1, rotate2) =
                    (rotation(side1, other_side1), rotation(side2, other_side2));

                // make the connection
                let (face1, face2) = (other_side1 / 4, other_side2 / 4);

                let (connect1, connect2) = (
                    (face1 as isize * 4 + ((side2 as isize) + rotate1) % 4) as usize,
                    (face2 as isize * 4 + ((side1 as isize) + rotate2) % 4) as usize,
                );

                assert!(connect1 >= face1 * 4 && connect1 < (face1 + 1) * 4);
                assert!(connect2 >= face2 * 4 && connect2 < (face2 + 1) * 4);

                {
                    let from_result = connections.insert(connect1, connect2);
                    let to_result = connections.insert(connect2, connect1);

                    assert!(from_result.is_none() || from_result.unwrap() == connect2);
                    assert!(to_result.is_none() || to_result.unwrap() == connect1);
                }
            }
        }
    }

    println!("{:?}", connections);

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
        assert_eq!(part_two(&input), Some(5031));
    }
}

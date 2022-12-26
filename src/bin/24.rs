use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum BlizzardDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
struct BlizzardDirectionParseError {}

impl Display for BlizzardDirectionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing blizzard direction")
    }
}

impl std::error::Error for BlizzardDirectionParseError {}

impl TryFrom<char> for BlizzardDirection {
    type Error = BlizzardDirectionParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '<' => Ok(Self::Left),
            '>' => Ok(Self::Right),
            '^' => Ok(Self::Up),
            'v' => Ok(Self::Down),
            _ => Err(Self::Error {}),
        }
    }
}

impl Display for BlizzardDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Up => '^',
                Self::Down => 'v',
                Self::Left => '<',
                Self::Right => '>',
            }
        )
    }
}

fn parse(
    input: &str,
) -> (
    HashMap<(isize, isize), Vec<BlizzardDirection>>,
    (usize, usize),
) {
    let mut result = HashMap::new();
    let (mut width, mut height) = (0, 0);

    for (row, line) in input
        .lines()
        .skip(1)
        .enumerate()
        .take_while(|(_, l)| !l.starts_with("##"))
    {
        for (col, ch) in line.chars().skip(1).enumerate() {
            if ch == '#' {
                break;
            } else if ch == '.' {
                continue;
            } else {
                result.insert((col as isize, row as isize), vec![ch.try_into().unwrap()]);
            }

            width = col + 1;
        }

        height = row + 1;
    }

    (result, (width, height))
}

struct Puzzle {
    blizzards: HashMap<(isize, isize), Vec<BlizzardDirection>>,
    dimensions: (usize, usize), // width x height
}

#[derive(Clone, Copy, Debug)]
enum ValleyPortal {
    TopLeft,
    BottomRight,
}

impl ValleyPortal {
    fn other(&self) -> Self {
        match self {
            Self::TopLeft => Self::BottomRight,
            Self::BottomRight => Self::TopLeft,
        }
    }
}

impl Puzzle {
    pub fn step_blizzards(&mut self) {
        // Update positions of all blizzards
        self.blizzards = {
            let mut next = HashMap::new();
            fn resolve_dimension(cur: isize, max: isize) -> isize {
                if cur < 0 {
                    max - 1
                } else if cur >= max {
                    0
                } else {
                    cur
                }
            }

            let resolve = |(x, y)| {
                // blizzards in part 2 need to account for going to the entrances and exits
                // :scream:
                if (x == 0 && y == -1)
                    || (x == self.dimensions.0 as isize - 1 && y == self.dimensions.1 as isize)
                {
                    (x, y)
                } else if x == 0 && y == self.dimensions.1 as isize {
                    // The blizzard at the left will start from the entrance, not y = 0 (we don't
                    // need to special case the exit blizzard wrapping as that will just start
                    // again at y = 0 in the else case.
                    (0, -1)
                } else {
                    (
                        resolve_dimension(x, self.dimensions.0 as isize),
                        resolve_dimension(y, self.dimensions.1 as isize),
                    )
                }
            };

            for ((x, y), directions) in self.blizzards.clone() {
                for direction in directions {
                    next.entry(resolve(match direction {
                        BlizzardDirection::Up => (x, y - 1),
                        BlizzardDirection::Down => (x, y + 1),
                        BlizzardDirection::Left => (x - 1, y),
                        BlizzardDirection::Right => (x + 1, y),
                    }))
                    .or_insert(vec![])
                    .push(direction);
                }
            }

            next
        };
    }

    pub fn solve(&mut self, start: ValleyPortal) -> Option<u32> {
        let mut reachability: HashMap<(isize, isize), Vec<usize>> = HashMap::new();
        let (width, height) = self.dimensions;

        // We can reach the starting location at step 0.
        reachability.insert(
            match start {
                ValleyPortal::TopLeft => (0, -1),
                ValleyPortal::BottomRight => (width as isize - 1, height as isize),
            },
            vec![0],
        );

        let mut steps = 0;

        loop {
            eprintln!("solve has stepped {} times", steps);
            steps += 1;

            self.step_blizzards();

            // Update reachability for all positions that we could reach in the last position and which
            // do not currently have a blizzard occupying them or the adjacent.
            for row in -1..height as isize + 1 {
                for col in 0..width as isize {
                    let (row, col) = (row as isize, col as isize);

                    if self.blizzards.entry((col, row)).or_default().len() > 0 {
                        // there's a blizzard here, we can't stay
                        continue;
                    }

                    let candidates = vec![
                        (col, row),     // shelter in place
                        (col - 1, row), // move left
                        (col + 1, row), // move right
                        (col, row - 1), // move up
                        (col, row + 1), // move down
                    ];

                    let is_reachable = candidates
                        .iter()
                        // remove points outside the grid that cannot be accessed
                        .filter(|(x, y)| {
                            // The only position permitted outside the grid is the valley entrances
                            // and exists, which are modelled at (0,-1) and (width-1, height)
                            (*x == 0 && *y == -1) // start
                                    || (*x == width as isize - 1 && *y == height as isize) // end
                                    || (*x >= 0
                                        && *x < width as isize
                                        && *y >= 0
                                        && *y < height as isize)
                        })
                        // to access the point we have to have visited at least one of the adjacent
                        // candidates in the previous round or be at the current point and have
                        // stayed here (assuming we can)
                        .any(|(x, y)| {
                            reachability
                                .entry((*x, *y))
                                .or_default()
                                .contains(&(steps - 1))
                        });

                    if is_reachable {
                        reachability.entry((col, row)).or_default().push(steps);
                    }
                }
            }

            // Check if the end was reached
            if reachability
                .entry(
                    // We're going to the opposite side to where we started.
                    match start {
                        ValleyPortal::TopLeft => (width as isize - 1, height as isize),
                        ValleyPortal::BottomRight => (0, -1),
                    },
                )
                .or_default()
                .len()
                > 0
            {
                break;
            }
        }

        Some(steps as u32)
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (blizzards, (width, height)) = parse(input);

    let result = Puzzle {
        blizzards,
        dimensions: (width, height),
    }
    .solve(ValleyPortal::TopLeft);

    Some(result.unwrap())
}

pub fn part_two(input: &str) -> Option<u32> {
    let (blizzards, dimensions) = parse(input);

    let mut steps = 0;
    let mut puzzle = Puzzle {
        blizzards,
        dimensions,
    };

    let mut start = ValleyPortal::TopLeft;

    // 1. there
    // 2. back
    // 3. there again
    for step in 0..3 {
        let this_pass = puzzle.solve(start).unwrap();
        steps += this_pass;
        start = start.other();

        println!(
            "solved step {} in {} minutes (for {} total)",
            step, this_pass, steps
        );
    }

    Some(steps)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 24);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 24);
        assert_eq!(part_one(&input), Some(18));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 24);
        assert_eq!(part_two(&input), Some(54));
    }
}

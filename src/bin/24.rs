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

pub fn part_one(input: &str) -> Option<u32> {
    let (mut blizzards, (width, height)) = parse(input);
    let mut reachability: HashMap<(isize, isize), Vec<usize>> = HashMap::new();

    // We can reach the starting location at step 0.
    reachability.insert((0, -1), vec![0]);

    let mut steps = 0;

    loop {
        steps += 1;

        // Update positions of all blizzards
        blizzards = {
            let mut next = HashMap::new();
            let resolve = |cur, max| {
                if cur < 0 {
                    max - 1
                } else if cur >= max {
                    0
                } else {
                    cur
                }
            };

            for ((x, y), directions) in blizzards.clone() {
                for direction in directions {
                    next.entry({
                        let (x, y) = match direction {
                            BlizzardDirection::Up => (x, y - 1),
                            BlizzardDirection::Down => (x, y + 1),
                            BlizzardDirection::Left => (x - 1, y),
                            BlizzardDirection::Right => (x + 1, y),
                        };

                        (resolve(x, width as isize), resolve(y, height as isize))
                    })
                    .or_insert(vec![])
                    .push(direction);
                }
            }

            next
        };

        // Update reachability for all positions that we could reach in the last position and which
        // do not currently have a blizzard occupying them or the adjacent.
        {
            for row in 0..height {
                for col in 0..width {
                    let (row, col) = (row as isize, col as isize);

                    // if there's a blizzard here, we can't stay
                    if !blizzards.entry((col, row)).or_default().is_empty() {
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
                            // The final position is outside the grid
                            (*x == width as isize - 1 && *y == height as isize + 1)
                                || (*x >= 0
                                    && *x < width as isize
                                    && *y >= -1
                                    && *y < height as isize)
                        })
                        // to access the point we have to have visited at least one of the adjacent
                        // candidates in the previous round
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
        }

        // Check if the end was reached
        if !reachability
            .entry((width as isize - 1, height as isize - 1))
            .or_default()
            .is_empty()
        {
            break;
        }
    }

    Some(steps as u32 + 1)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
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
        assert_eq!(part_two(&input), None);
    }
}

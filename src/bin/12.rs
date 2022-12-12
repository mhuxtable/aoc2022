use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct Map {
    elevations: Vec<PositionType>,
    row_length: usize,
    start: usize,
    end: usize,
}

fn can_move_to(from: u8, to: u8) -> bool {
    // Can move to any location of lower elevation, or to locations precisely one step higher
    // than current.
    to.saturating_sub(from) <= 1
}

impl Map {
    pub fn new(elevations: Vec<PositionType>, row_length: usize) -> Self {
        let (start, end) =
            elevations
                .iter()
                .enumerate()
                .fold((None, None), |(start, end), (i, elevation)| {
                    (
                        if elevation.is_start() {
                            if start.is_none() {
                                Some(i)
                            } else {
                                panic!("multiple starting positions found");
                            }
                        } else {
                            start
                        },
                        if elevation.is_end() {
                            if end.is_none() {
                                Some(i)
                            } else {
                                panic!("multiple ending positions found");
                            }
                        } else {
                            end
                        },
                    )
                });

        Self {
            elevations,
            row_length,
            start: start.expect("missing start"),
            end: end.expect("missing end"),
        }
    }

    pub fn adjacencies(&self) -> HashMap<usize, Vec<usize>> {
        let adjacent_points_for_point = |point: usize| -> Vec<usize> {
            vec![
                // north
                if point >= self.row_length {
                    Some(point - self.row_length)
                } else {
                    None
                },
                // east
                if (point + 1) % self.row_length != 0 {
                    Some(point + 1)
                } else {
                    None
                },
                // south
                if point < self.elevations.len() - self.row_length {
                    Some(point + self.row_length)
                } else {
                    None
                },
                // west
                if point % self.row_length > 0 {
                    Some(point - 1)
                } else {
                    None
                },
            ]
            .into_iter()
            .filter_map(|x| x)
            .collect()
        };

        let mut adjacencies = HashMap::new();

        for (i, position) in self.elevations.iter().enumerate() {
            let adjacent_points = adjacent_points_for_point(i);
            adjacencies.insert(
                i,
                adjacent_points
                    .into_iter()
                    .filter(|&to| {
                        can_move_to(position.elevation(), self.elevations[to].elevation())
                    })
                    .collect(),
            );
        }

        adjacencies
    }

    pub fn point(&self, i: usize) -> (usize, usize) {
        let y = i / self.row_length;
        let x = i % self.row_length;

        (x, y)
    }

    pub fn manhattan_distance(&self, from: usize, to: usize) -> u32 {
        let (from, to) = (self.point(from), self.point(to));

        (from.0.abs_diff(to.0) + from.1.abs_diff(to.1)) as u32
    }
}

#[derive(Debug)]
enum PositionType {
    Start,
    End,
    NotSpecial(u8),
}

impl PositionType {
    pub fn elevation(&self) -> u8 {
        (match self {
            Self::Start => 'a' as u8,
            Self::End => 'z' as u8,
            Self::NotSpecial(x) => *x,
        }) - (
            // Technically don't need to normalise, but it makes parsing the elevations easier by eye
            'a' as u8
        )
    }

    pub fn is_start(&self) -> bool {
        if let Self::Start = self {
            true
        } else {
            false
        }
    }

    pub fn is_end(&self) -> bool {
        if let Self::End = self {
            true
        } else {
            false
        }
    }
}

impl From<char> for PositionType {
    fn from(ch: char) -> Self {
        match ch {
            'S' => PositionType::Start,
            'E' => PositionType::End,
            x => PositionType::NotSpecial(x as u8),
        }
    }
}

impl From<&str> for Map {
    fn from(input: &str) -> Self {
        let row_length = input.lines().nth(1).unwrap().len();

        Map::new(
            input
                .lines()
                .flat_map(|l| {
                    if l.len() != row_length {
                        panic!("input map has lines of non-equal length, which is not supported");
                    }
                    l.chars().map(|ch| PositionType::from(ch))
                })
                .collect(),
            row_length,
        )
    }
}

fn astar_from_point(map: &Map, from: Vec<usize>) -> u32 {
    let graph = map.adjacencies();

    let h = |point: usize| map.manhattan_distance(point, map.end);

    // A*
    let mut fringe = HashSet::new();

    let mut fs = vec![u32::MAX; map.elevations.len()];
    let mut gs = vec![u32::MAX; map.elevations.len()];

    for f in from {
        fs[f] = h(f);
        gs[f] = 0;
        fringe.insert(f);
    }

    while !fringe.is_empty() {
        // If we made the fringe a priority queue this would be easier.
        let mut fscores: Vec<(usize, u32)> =
            fringe.iter().map(|&point| (point, fs[point])).collect();
        fscores.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());

        let (cur, _) = fscores.first().unwrap();
        fringe.remove(cur);

        for &neighbour in &graph[cur] {
            // The cost of all steps is 1
            let gscore = gs[*cur] + 1;
            if gscore < gs[neighbour] {
                gs[neighbour] = gscore;
                fs[neighbour] = gscore + h(neighbour);

                fringe.insert(neighbour);
            }
        }
    }

    gs[map.end] as u32
}

pub fn part_one(input: &str) -> Option<u32> {
    let map = Map::from(input);
    let cost = astar_from_point(&map, vec![map.start]);

    Some(cost)
}

pub fn part_two(input: &str) -> Option<u32> {
    let map = Map::from(input);
    let cost = astar_from_point(
        &map,
        map.elevations
            .iter()
            .enumerate()
            .filter_map(|(point, elevation)| {
                if elevation.is_end() || elevation.elevation() != 0 {
                    None
                } else {
                    Some(point)
                }
            })
            .collect(),
    );

    Some(cost)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 12);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 12);
        assert_eq!(part_one(&input), Some(31));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 12);
        assert_eq!(part_two(&input), Some(29));
    }
}

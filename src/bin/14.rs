// This is a very inefficient (space-wise) solution. It models the entire Grid as a 1-dimension
// vector and simulates the sand falling through this grid (updating only when sand comes to rest).
// This imposes some challenges that constrain the solution for part 2, but it was a nice easy way
// of visualising the problem. This really isn't a great approach today.

use std::fmt::Display;

use advent_of_code::helpers::{Grid, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    Air,
    Rock,
    Sand,
}

impl Default for Space {
    fn default() -> Self {
        Self::Air
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "."),
            Self::Rock => write!(f, "#"),
            Self::Sand => write!(f, "o"),
        }
    }
}

struct Line(Vec<Point>);

impl From<&str> for Line {
    fn from(s: &str) -> Self {
        Line(s.split(" -> ").map(|p| Point::from(p)).collect())
    }
}

fn parse(input: &str) -> Vec<Line> {
    input.lines().map(|l| Line::from(l)).collect()
}

fn format_grid<T: Clone + Default + Display>(grid: &Grid<T>) -> String {
    let mut s = String::new();

    for (i, value) in grid.iter().enumerate() {
        if i > 0 && i % grid.width() == 0 {
            s.push('\n');
        }

        s.push_str(format!("{}", value).as_str());
    }

    s
}

fn min_max<T, R, I, P>(items: I, p: P) -> (Option<R>, Option<R>)
where
    I: IntoIterator<Item = T>,
    R: Ord + Copy,
    P: Fn(&T) -> R,
{
    let resolve = |x: Option<R>, y: &R, ord: std::cmp::Ordering| -> Option<R> {
        if x.is_none() {
            Some(*y)
        } else if y.cmp(&x.unwrap()) == ord {
            Some(*y)
        } else {
            x
        }
    };

    items.into_iter().fold((None, None), |(min, max), item| {
        let x = p(&item);
        (
            resolve(min, &x, std::cmp::Ordering::Less),
            resolve(max, &x, std::cmp::Ordering::Greater),
        )
    })
}

fn draw_grid(
    lines: &Vec<Line>,
    with_floor: bool,
) -> (Grid<Space>, Box<dyn Fn(usize, usize) -> Point>) {
    let (min_x, max_x) = min_max(lines.iter().flat_map(|line| &line.0), |point| point.x);
    let (_, max_y) = min_max(lines.iter().flat_map(|line| &line.0), |point| point.y);

    let max_y = if with_floor {
        max_y.unwrap() + 2
    } else {
        max_y.unwrap()
    };

    let grid_width = min_x.unwrap().abs_diff(max_x.unwrap()) + 1;
    let (grid_width, min_x) = if with_floor {
        // just make the grid mega wide if there's a floor so that we stand a change of the sand
        // blocking the spigot before we run out of space. A better solution would be to be able to
        // expand a grid's width.

        // this was just trial and error to figure out what size grid would give us enough space to
        // fill and block the spigot.
        (grid_width + 1000, min_x.unwrap() - 300)
    } else {
        (grid_width, min_x.unwrap())
    };

    let mut grid = Grid::new(grid_width, max_y);

    // fill in the floor
    for x in 0..grid.width() {
        *grid.point_mut(&Point { x, y: max_y }) = Space::Rock;
    }

    println!("{}", format_grid(&grid));

    let make_point = move |x, y| Point { x: x - min_x, y };

    for line in lines {
        for (from, to) in line.0.iter().zip(line.0.iter().skip(1)) {
            if (from.x == to.x && from.y == to.y) || (from.x != to.x && from.y != to.y) {
                panic!("line is too complicated");
            } else if from.x == to.x {
                let x = from.x;
                for y in from.y.min(to.y)..=from.y.max(to.y) {
                    *grid.point_mut(&make_point(x, y)) = Space::Rock;
                }
            } else if from.y == to.y {
                let y = from.y;
                for x in from.x.min(to.x)..=from.x.max(to.x) {
                    *grid.point_mut(&make_point(x, y)) = Space::Rock;
                }
            }
        }
    }

    (grid, Box::new(make_point))
}

static SPIGOT: Point = Point { x: 500, y: 0 };

fn add_grain<F>(grid: &mut Grid<Space>, make_point: &F) -> Option<Point>
where
    F: Fn(usize, usize) -> Point,
{
    let mut sand = make_point(SPIGOT.x, SPIGOT.y);

    // Can we make something at the spigot?
    if *grid.point(&sand) != Space::Air {
        return None;
    }

    loop {
        let next = vec![
            Some(Point {
                x: sand.x,
                y: sand.y + 1,
            }),
            if sand.x.checked_sub(1).is_none() {
                None
            } else {
                Some(Point {
                    x: sand.x - 1,
                    y: sand.y + 1,
                })
            },
            Some(Point {
                x: sand.x + 1,
                y: sand.y + 1,
            }),
        ];

        let mut found_next = false;

        for candidate in next {
            if candidate.is_none()
                || candidate.unwrap().x >= grid.width()
                || candidate.unwrap().y >= grid.height()
            {
                // The sand would flow out of the grid
                return None;
            }

            if *grid.point(&candidate.unwrap()) == Space::Air {
                // the sand can flow to this candidate
                sand = candidate.unwrap();
                found_next = true;
                break;
            }
        }

        if !found_next {
            // nowhere the sand can go, so it stays here
            break;
        }
    }

    *grid.point_mut(&sand) = Space::Sand;
    return Some(sand);
}

pub fn part_one(input: &str) -> Option<u32> {
    let lines = parse(input);
    let (mut grid, make_point) = draw_grid(&lines, false);

    // Flow the sand
    while add_grain(&mut grid, &make_point).is_some() {
        println!("{}", format_grid(&grid));
    }

    Some(grid.iter().filter(|&space| *space == Space::Sand).count() as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    let lines = parse(input);
    let (mut grid, make_point) = draw_grid(&lines, true);

    // Flow the sand
    while add_grain(&mut grid, &make_point).is_some() {
        // println!("{}", format_grid(&grid));
    }

    Some(grid.iter().filter(|&space| *space == Space::Sand).count() as u32)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 14);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 14);
        assert_eq!(part_one(&input), Some(24));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 14);
        assert_eq!(part_two(&input), Some(93));
    }
}

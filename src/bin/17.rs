use std::collections::VecDeque;

use rstar::RTree;

#[derive(Debug)]
enum JetBlast {
    Left,
    Right,
}

impl From<char> for JetBlast {
    fn from(ch: char) -> Self {
        match ch {
            '<' => Self::Left,
            '>' => Self::Right,
            _ => panic!("unknown jet blast direction"),
        }
    }
}

fn parse(input: &str) -> Vec<JetBlast> {
    input.trim_end().chars().map(|ch| ch.into()).collect()
}

#[derive(Debug)]
enum Shape {
    HorizontalLine,
    Plus,
    Corner,
    VerticalLine,
    Square,
}

impl Shape {
    pub fn starting_geometry(&self, offset: (i64, i64)) -> Vec<(i64, i64)> {
        let point = |x, y| {
            if x < 0 {
                panic!("x is too small");
            } else if x > 6 {
                panic!("x is too big");
            }
            if y < 0 {
                panic!("y is too small");
            }

            (x + offset.0, y + offset.1)
        };

        match self {
            Self::HorizontalLine => (2..=5).map(|x| point(x, 0)).collect(),
            Self::Plus => vec![
                (0..=2).map(|y| point(3, y)).collect::<Vec<(i64, i64)>>(),
                (2..=4).map(|x| point(x, 1)).collect::<Vec<(i64, i64)>>(),
            ]
            .into_iter()
            .flatten()
            .collect(),
            Self::Corner => vec![
                (2..=4).map(|x| point(x, 0)).collect::<Vec<(i64, i64)>>(),
                (0..=2).map(|y| point(4, y)).collect::<Vec<(i64, i64)>>(),
            ]
            .into_iter()
            .flatten()
            .collect(),
            Self::VerticalLine => (0..=3).map(|y| point(2, y)).collect(),
            Self::Square => (2..=3)
                .flat_map(|x| (0..=1).map(|y| point(x, y)).collect::<Vec<(i64, i64)>>())
                .collect(),
        }
    }
}

static FALL_ORDER: [Shape; 5] = [
    Shape::HorizontalLine,
    Shape::Plus,
    Shape::Corner,
    Shape::VerticalLine,
    Shape::Square,
];

fn tree_contains_shape(tree: &RTree<(i64, i64)>, points: &Vec<(i64, i64)>) -> bool {
    points.iter().any(|point| tree.contains(point))
}

fn max_height(points: &Vec<(i64, i64)>) -> i64 {
    *points.iter().map(|(_, y)| y).max().unwrap() + 1
}

fn print_tree(tree: &RTree<(i64, i64)>) -> String {
    let mut s: String = String::new();

    let max_height = tree.iter().map(|(_, y)| y).max().unwrap() + 1;

    for y in (0..max_height + 3).rev() {
        s.push_str(
            format!(
                "{:>5}: |",
                y,
                // width = (max_height as f64).log10().floor() as usize
            )
            .as_str(),
        );

        for x in 0..7 {
            if tree.contains(&(x, y)) {
                s.push('#');
            } else {
                s.push('.');
            }
        }

        s.push_str("|\n");
    }

    s
}

fn drop_object<'a, I>(
    tree: &RTree<(i64, i64)>,
    jet_blasts: &mut I,
    start_height: i64,
    shape: &Shape,
) -> (Vec<(i64, i64)>, i64)
where
    I: Iterator<Item = &'a JetBlast>,
{
    let mut left = 0;
    let mut object = shape.starting_geometry((left, start_height));

    for j in (0..=start_height).rev() {
        let next_object = shape.starting_geometry((left, j));

        if tree_contains_shape(&tree, &next_object) {
            // crash
            let max_height = max_height(&object);
            return (object, max_height);
        }

        // item isn't in tree so do the jet blast
        match jet_blasts.next().unwrap() {
            JetBlast::Left => {
                if !next_object.iter().any(|(x, _)| *x == 0)
                    && !tree_contains_shape(tree, &shape.starting_geometry((left - 1, j)))
                {
                    left -= 1;
                }
            }
            JetBlast::Right => {
                if !next_object.iter().any(|(x, _)| *x == 6)
                    && !tree_contains_shape(tree, &shape.starting_geometry((left + 1, j)))
                {
                    left += 1;
                }
            }
        };

        object = shape.starting_geometry((left, j));
    }

    let max_height = max_height(&object);
    (object, max_height)
}

pub fn part_one(input: &str) -> Option<u64> {
    let directions = parse(input);
    let mut jet_blasts = directions.iter().cycle();
    let mut next_shape = FALL_ORDER.iter().cycle();

    let mut tree: RTree<(i64, i64)> = RTree::new();
    let mut max_height = 0;

    for i in 0..2022 {
        // eprintln!("{}", i);
        let start_height = max_height + 3;

        let shape = next_shape.next().unwrap();
        let (object, new_max_height) = drop_object(&tree, &mut jet_blasts, start_height, shape);

        for point in object {
            tree.insert(point);
        }

        max_height = new_max_height.max(max_height);

        if i % 100 == 0 {
            println!("dropped object {}: max_height={}", i, max_height);
            println!("{}\n==========\n", print_tree(&tree));
        }
    }

    Some(max_height as u64)
}

pub fn part_two(input: &str) -> Option<u64> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 17);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 17);
        assert_eq!(part_one(&input), Some(3068));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 17);
        assert_eq!(part_two(&input), None);
    }
}

use rstar::RTree;
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};

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

#[derive(Debug, PartialEq, Eq)]
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

fn print_tree(tree: &RTree<(i64, i64)>, min_y: i64) -> String {
    let mut s: String = String::new();

    let max_height = tree.iter().map(|(_, y)| y).max().unwrap() + 1;

    for y in (min_y..max_height + 3).rev() {
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
        let start_height = max_height + 3;

        let shape = next_shape.next().unwrap();
        let (object, new_max_height) = drop_object(&tree, &mut jet_blasts, start_height, shape);

        for point in object {
            tree.insert(point);
        }

        max_height = new_max_height.max(max_height);
    }

    Some(max_height as u64)
}

// It is totally infeasible to simulate a trillion rock drops and keep tracking state of the cave,
// even with pruning. However, we can observe that for each rock dropped, we have an event horizon
// in which that rock can settle and a fossil record below a certain point that is immutable
// (because it is entirely covered by rock). Thus, if we can locate a cycle in the inputs (which
// depends only on the mutable fossil record in which new rocks can settle, the jet blasts and the
// shape of the rock being dropped), we can compute the height difference from each cycle and make
// this up to the requisite number of rock drops.
//
// This solution does not function correctly if the input jet blast does not generate cycles in the
// rock record. Also be advised that the cycle does not necessarily start from rock 0, nor is the
// number of cycles from the end of the cycle necessarily a whole cycle's worth of rock drops; it
// is necessary to partially simulate a cycle at the end to retrieve the overall cave height.
pub fn part_two(input: &str) -> Option<u64> {
    let find_first_cycle = || {
        // This is the y height of the top rock in each column. This is used later to generate the
        // minimum possible sized state to find cycles; observe that we can effectively ignore the
        // state of all rocks below the lowest top-most rock across all columns; e.g. if we have
        // three columns and their top-most rock is at y co-ordinates [3,4,7] (y starts from the
        // bottom), we can ignore any rock that exists below y=3 for the purposes of locating
        // cycles in the rock record. Effectively, we window our state function on the interval
        // [min(top_rock_in_column), max_height].
        //
        // We use this to determine the barrier or event horizon beyond which no new rocks can be
        // committed, and which immortalises the state of the cave below that point forever more.
        // This is not a perfect heuristic; indeed, it is highly likely that the resultant state
        // will overestimate the tip of the rock record and hence the cycle, but this is of no
        // consequence beyond potentially requiring additional iterations to develop a cycle.
        //
        // Some sort of proof by contradiction or inductive proof could demonstrate that this is
        // sufficient...
        let mut top_rock_in_column = [0i64; 7];

        // HashMap stores the hash of the state for each iteration (a combination of the current
        // shape index, the current jet cycle index, and all rocks down to the rock horizon – see
        // top_rock_in_column). The value records the shape drop at the point in time when this
        // state occurred, which allows the max height from this drop to be obtained from the
        // heights vector.
        let mut states: HashMap<u64, i32> = HashMap::new();

        let mut tree: RTree<(i64, i64)> = RTree::new();

        let directions = parse(input);
        let jet_blast_count = RefCell::new(0);

        let mut jet_blasts = directions.iter().cycle().inspect(|_| {
            *jet_blast_count.borrow_mut() += 1;
        });
        let mut next_shape = FALL_ORDER.iter().cycle();

        // We need to track the maximum height at each step. When we find a cycle and play this
        // forward to 1 trillion iterations, we may not perfectly reach 1 trillion iterations; the
        // cycle may stop early, and we need to play it forward by part of a cycle to reach that
        // many rocks, requiring us to also know the intra-cycle height change for each step of the
        // cycle.
        let mut step_max_heights = vec![0];

        for shape_drop in 0..10_000 {
            let last_max_height = *step_max_heights.last().unwrap();
            let start_height = last_max_height + 3;

            let shape = next_shape.next().unwrap();
            let (object, new_max_height) = drop_object(&tree, &mut jet_blasts, start_height, shape);

            for point in object {
                tree.insert(point);

                let t = &mut top_rock_in_column[point.0 as usize];
                if *t < point.1 {
                    *t = point.1;
                }
            }

            // Compute a hash from the current state, to enable us to determine whether this state
            // has been observed before and hence is the start of a cycle. The hash is produced
            // from the current rock drop count (modulo total number of shapes), current jet blast
            // (modulo total number of jet blasts) and, for each column 0..7, iterating from top of
            // column to the rock base (the row with the bottom-most exposed rock across all
            // columns, i.e. min(top_rock_in_column).
            {
                let mut hash = DefaultHasher::new();

                let min_y = *top_rock_in_column.iter().min().unwrap();
                for y in min_y..=last_max_height {
                    (0..7)
                        .fold(0u8, |acc, x| {
                            acc | ((if tree.contains(&(x, y)) { 0x01 } else { 0x00 }) << x)
                        })
                        .hash(&mut hash);
                }

                // Separate variable length input to the hash from fixed length input.
                0xFFu8.hash(&mut hash);
                (shape_drop % 5).hash(&mut hash);
                (*jet_blast_count.borrow() % directions.len()).hash(&mut hash);

                let hash = hash.finish();
                if states.contains_key(&hash) {
                    let start_of_cycle = states[&hash];

                    // We found a duplicate state, meaning that there is a cycle. The critical
                    // information is:
                    //
                    // - the current rock drop (i) (this is the start of the next cycle)
                    // - the number of rocks dropped since the cycle started (the periodicity)
                    // - the current height
                    // - the maximum heights at each step, so we can:
                    //     - determine the change in height over this cycle
                    //     - determine a partial change in height in case a partial cycle is
                    //       required to simulate the target number of rock drops.
                    //
                    // The periodicity and the change in height subsequently becomes a simple
                    // calculation to determine the eventual total height, plus a partial cycle
                    // played forward if the target rock drop count is not an integer number of
                    // cycles.
                    return (shape_drop, start_of_cycle, step_max_heights);
                }

                states.insert(hash, shape_drop);
            }

            step_max_heights.push(new_max_height.max(last_max_height));
        }

        panic!("no cycle found - increase the steps");
    };

    // cycle_step_heights indices show the height BEFORE a rock was dropped, i.e.
    // cycle_step_heights[0] is the height before rock 0 was dropped. This makes the logic in the
    // cycle finder easier but care is required in the logic here to ensure the correct heights are
    // retrieved for each step.
    let (cycle_end, cycle_start, cycle_step_heights) = find_first_cycle();

    const TARGET: i64 = 1_000_000_000_000;

    let cycle_length = cycle_end.abs_diff(cycle_start); // Offset by 1 but of no consequence to us as we
                                                        // just care about cycle length.
    let steps_remaining = TARGET - cycle_end as i64; // this is not guaranteed to be an integer
                                                     // number of cycles! Need to clean up a
                                                     // partial cycle later (maybe)

    let cycles_remaining = steps_remaining / cycle_length as i64;

    let cycle_height_change = cycles_remaining
        * (cycle_step_heights[cycle_end as usize] - cycle_step_heights[cycle_start as usize]);

    let partial_cycle = TARGET - (cycles_remaining * cycle_length as i64) - cycle_end as i64;

    let partial_cycle_height_change = cycle_step_heights
        [(cycle_start as i64 + partial_cycle) as usize]
        - cycle_step_heights[cycle_start as usize];

    dbg!(
        cycle_length,
        steps_remaining,
        cycles_remaining,
        partial_cycle,
        partial_cycle_height_change,
    );

    Some(
        // The total height is thus the original height at the end of the cycle, plus the change from
        // playing the cycle forward cycles_remaining times, plus the partial height change from
        // partially playing forward one cycle until we reach the target number of dropped rocks.
        (cycle_step_heights.last().unwrap() + cycle_height_change + partial_cycle_height_change)
            as u64,
    )
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
        assert_eq!(part_two(&input), Some(1514285714288));
    }
}

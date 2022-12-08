use std::borrow::Borrow;

fn parse(input: &str) -> Vec<Vec<u32>> {
    input
        .lines()
        .map(|line| line.chars().map(|x| x.to_digit(10).unwrap()).collect())
        .collect()
}

fn columnise(rows: &Vec<Vec<u32>>) -> Vec<Vec<u32>> {
    (0..rows[0].len())
        .map(|col| rows.iter().map(|row| row[col]).collect())
        .collect()
}

pub fn part_one(input: &str) -> Option<u32> {
    let trees = parse(input);
    let mut visible = 0u32;

    let columns = columnise(&trees);

    for (i, row) in trees.iter().enumerate() {
        for (j, &height) in row.iter().enumerate() {
            // The rules for visibility state there must be trees in both directions along the row
            // and column that are taller than this tree, otherwise it is visible. Trees on edges
            // are automatically visible as nothing can occlude them on that edge.
            //
            // This is an ugly O(N^2) algorithm but it's okay for inputs of this size. We could be
            // more sophisticated by doing some memoisation :shrug:

            // In the iterators that follow all() is documented as returning true on an empty
            // iterator, so we are tracked that all trees in that direction are shorter i.e.
            // whether current tree is visible or not.

            let taller = |&x| x < height;

            let north = columns[j][0..i].iter().all(taller);
            let east = row[j + 1..].iter().all(taller);
            let south = columns[j][i + 1..].iter().all(taller);
            let west = row[0..j].iter().all(taller);

            if north || east || south || west {
                visible += 1;
            }
        }
    }

    Some(visible)
}

pub fn part_two(input: &str) -> Option<u32> {
    let trees = parse(input);
    let columns = columnise(&trees);

    let mut best_score = 0;

    fn visibility<I>(current_tree: u32, heights: I) -> u32
    where
        I: IntoIterator,
        I::Item: Borrow<u32>,
    {
        let (vis, _) = heights.into_iter().fold((0, true), |(trees, cont), tree| {
            (
                // We always count the last tree that terminates the search, even if it is of same
                // or higher height, then we terminate. This is slightly confusing in the puzzle
                // description. Use cont from the invocation of the fold.
                trees + if cont { 1 } else { 0 },
                // And determine whether to continue.
                cont && *tree.borrow() < current_tree,
            )
        });

        vis
    }

    for (i, row) in trees.iter().enumerate() {
        for (j, &height) in row.iter().enumerate() {
            let column = &columns[j];

            let north = visibility(height, column[0..i].iter().rev());
            let east = visibility(height, &row[j + 1..]);
            let south = visibility(height, &column[i + 1..]);
            let west = visibility(height, row[0..j].iter().rev());

            let score = north * east * south * west;

            if score > best_score {
                best_score = score;
            }
        }
    }

    Some(best_score)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 8);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 8);
        assert_eq!(part_one(&input), Some(21));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 8);
        assert_eq!(part_two(&input), Some(8));
    }
}

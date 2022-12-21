fn parse(input: &str) -> Vec<i64> {
    input.lines().map(|x| x.parse().unwrap()).collect()
}

fn mix(file: &Vec<i64>, iterations: usize) -> Vec<i64> {
    let mut intermediate: Vec<(i64, usize)> =
        file.iter().enumerate().map(|(i, x)| (*x, i)).collect();

    for _ in 0..iterations {
        for i in 0..intermediate.len() {
            let (idx, (x, _)) = intermediate
                .iter()
                .enumerate()
                .find(|(_, (_, j))| *j == i)
                .unwrap();

            let x = *x;

            // We need to move it to idx + x places, wrapping if necessary.
            let new_idx = idx as isize + x as isize;
            let removed = intermediate.remove(idx).0;
            assert_eq!(removed, x);

            let new_idx = if new_idx < 0 {
                intermediate.len() as isize + (new_idx % intermediate.len() as isize)
            } else if new_idx == 0 {
                // the example shows that if we move to the beginning, we actually go to the end
                intermediate.len() as isize
            } else if new_idx > intermediate.len() as isize {
                new_idx % intermediate.len() as isize
            } else {
                new_idx
            };

            assert!(new_idx.abs() <= intermediate.len() as isize);

            // insert the element, taking care to adjust the new index if we removed an item before
            // where we are inserting (as that will have shifted all indices down by 1).

            intermediate.insert(new_idx as usize, (x, i));
        }
    }

    intermediate.iter().map(|(x, _)| *x).collect()
}

fn grove_coords(mixed: &Vec<i64>) -> i64 {
    let zero = mixed.iter().position(|&x| x == 0).unwrap();

    let get_elt = |n: usize| {
        let idx = zero + n;
        mixed[idx % mixed.len()]
    };

    get_elt(1000) + get_elt(2000) + get_elt(3000)
}

pub fn part_one(input: &str) -> Option<i64> {
    let file = parse(input);
    let mixed = mix(&file, 1);

    Some(grove_coords(&mixed))
}

pub fn part_two(input: &str) -> Option<i64> {
    let file = parse(input).iter().map(|&x| x * 811589153).collect();
    let mixed = mix(&file, 10);

    Some(grove_coords(&mixed))
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 20);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 20);
        assert_eq!(part_one(&input), Some(3));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 20);
        assert_eq!(part_two(&input), Some(1623178306));
    }
}

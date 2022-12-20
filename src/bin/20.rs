use itertools::Itertools;

fn parse(input: &str) -> Vec<i32> {
    input.lines().map(|x| x.parse().unwrap()).collect()
}

fn mix(file: &Vec<i32>) -> Vec<i32> {
    let mut intermediate: Vec<(i32, bool)> = file.iter().map(|x| (*x, false)).collect();

    while intermediate.iter().any(|(_, touched)| !touched) {
        let (idx, (x, touched)) = intermediate
            .iter()
            .enumerate()
            .find(|(_, (_, touched))| !touched)
            .unwrap();

        assert!(*touched == false);
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

        intermediate.insert(new_idx as usize, (x, true));
    }

    intermediate.iter().map(|(x, _)| *x).collect()
}

pub fn part_one(input: &str) -> Option<i32> {
    let file = parse(input);
    let mixed = mix(&file);

    let zero = mixed.iter().position(|&x| x == 0).unwrap();

    let get_elt = |n: usize| {
        let idx = zero + n;
        mixed[idx % mixed.len()]
    };

    Some(get_elt(1000) + get_elt(2000) + get_elt(3000))
}

pub fn part_two(input: &str) -> Option<u32> {
    None
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
        assert_eq!(part_two(&input), None);
    }
}

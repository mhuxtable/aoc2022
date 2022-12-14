/// This one was good fun and some reasonable assumptions can be made based on an even number of
/// items per bag. Just need to watch out for exclusive top slice indices which caught me out
/// briefly and wasn't caught in the example input.
use std::collections::{HashMap, HashSet};

fn parse(input: &str) -> Vec<String> {
    let sacks: Vec<String> = input.lines().map(|s| s.to_string()).collect();

    sacks
}

fn priority(ch: char) -> u8 {
    if ch.is_uppercase() {
        ch as u8 - 'A' as u8 + 27
    } else if ch.is_lowercase() {
        ch as u8 - 'a' as u8 + 1
    } else {
        panic!("not a suitable item key")
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let sacks = parse(input);

    let priorities = sacks.iter().map(|sack| {
        // we can assume the string is of even length, but check
        assert!(sack.len() % 2 == 0, "expected sack to be of even item size");
        assert!(sack.len() > 0, "sack contains no items");

        let sack = sack.chars().collect::<Vec<char>>();

        let (mut xs, mut ys) = (
            sack[0..sack.len() / 2].to_vec(),
            sack[sack.len() / 2..].to_vec(),
        );

        xs.sort();
        ys.sort();

        let mut i = 0;
        let mut j = 0;

        dbg!(sack, &xs, &ys);

        loop {
            let (x, y) = (xs[i] as u32, ys[j] as u32);

            assert!(i < xs.len());
            assert!(j < ys.len());

            if x < y {
                i += 1;
            } else if x > y {
                j += 1;
            } else {
                // x == y
                break;
            }
        }

        priority(xs[i]) as u32
    });

    Some(priorities.sum())
}

pub fn part_two(input: &str) -> Option<u32> {
    let score = parse(input)
        .chunks(3)
        .map(|bags| {
            assert!(bags.len() == 3);

            let mut map: HashMap<char, u32> = HashMap::new();
            let mut badge: Option<char> = None;

            'bag: for bag in bags {
                let mut set = HashSet::new();

                for ch in bag.chars() {
                    if set.contains(&ch) {
                        continue;
                    }

                    set.insert(ch);

                    let score = map.entry(ch).and_modify(|e| *e += 1).or_insert(1);

                    if *score == 3 {
                        badge = Some(ch);
                        break 'bag;
                    }
                }
            }

            assert!(badge.is_some());

            priority(badge.unwrap()) as u32
        })
        .sum();

    Some(score)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 3);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 3);
        assert_eq!(part_one(&input), Some(157));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 3);
        assert_eq!(part_two(&input), Some(70));
    }
}

/// Nice easy one to start off, summing some groups and chunking where needed. Nothing really to
/// report.

fn parse(input: &str) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let mut elves: Vec<Vec<u32>> = vec![vec![]];

    for line in input.lines() {
        if line.is_empty() {
            elves.push(vec![]);
            continue;
        }

        elves.last_mut().unwrap().push(line.parse()?);
    }

    let sums: Vec<u32> = elves.iter().map(|elf| elf.iter().sum::<u32>()).collect();

    Ok(sums)
}

pub fn part_one(input: &str) -> Option<u32> {
    let elves = parse(input).unwrap();

    Some(*elves.iter().max().unwrap() as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    let elves = parse(input).unwrap();

    Some(elves[elves.len() - 3..elves.len()].iter().sum())
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 1);

    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 1);
        assert_eq!(part_one(&input), Some(24_000));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 1);
        assert_eq!(part_two(&input), Some(45_000));
    }
}

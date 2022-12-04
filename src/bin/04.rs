fn elf_range(range: &str) -> (u32, u32) {
    let (from, to) = range.split_once('-').unwrap();

    (from.parse().unwrap(), to.parse().unwrap())
}

pub fn part_one(input: &str) -> Option<u32> {
    let overlaps = input
        .lines()
        .filter_map(|pair| {
            let (elf1, elf2) = pair.split_once(',').unwrap();

            let (e1f, e1t) = elf_range(elf1);
            let (e2f, e2t) = elf_range(elf2);

            if (e1f <= e2f && e1t >= e2t) || (e2f <= e1f && e2t >= e1t) {
                Some(())
            } else {
                None
            }
        })
        .count();

    Some(overlaps as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    let overlaps = input
        .lines()
        .filter_map(|pair| {
            let (elf1, elf2) = pair.split_once(',').unwrap();

            let (e1f, e1t) = elf_range(elf1);
            let (e2f, e2t) = elf_range(elf2);

            if (e1f <= e2f && e1t >= e2f) || (e2f <= e1f && e2t >= e1f) {
                Some(())
            } else {
                None
            }
        })
        .count();

    Some(overlaps as u32)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 4);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 4);
        assert_eq!(part_one(&input), Some(2));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 4);
        assert_eq!(part_two(&input), Some(4));
    }
}

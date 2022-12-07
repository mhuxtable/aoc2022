use std::{collections::HashMap, path::PathBuf};

fn parse(input: &str) -> HashMap<String, usize> {
    let mut current_path = PathBuf::new();
    let mut tree: HashMap<String, usize> = HashMap::new();

    for line in input.lines() {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();

        // shortest outputs are $ ls (2 parts) or a directory listing with two components
        assert!(parts.len() >= 2);

        // This solution makes some assumptions:
        // 1. The input will explore every directory that it finds. Otherwise we don't have a
        //    complete view of directory sizes.
        // 2. We never explore a directory more than once, otherwise we'll double count files.

        match parts[0] {
            // This is a command input
            "$" => {
                assert!(parts.len() >= 2);

                match parts[1] {
                    "cd" => {
                        assert!(parts.len() == 3);

                        match parts[2] {
                            ".." => _ = current_path.pop(),
                            x => {
                                current_path.push(x);
                            }
                        }
                    }
                    "ls" => {
                        assert!(parts.len() == 2);

                        _ = tree
                            .entry(current_path.to_str().unwrap().to_string())
                            .or_insert(0);
                    }
                    _ => panic!("unknown command"),
                }
            }
            "dir" => {
                // directory listing, don't care about it, we'll explore it later
                continue;
            }
            // It's a file size. We don't care about the file name
            size => {
                let size: usize = size.parse().unwrap();
                let mut here = PathBuf::new();

                // Add the current file size to the cumulative sizes of the current directory and
                // every parent directory.
                for component in current_path.components() {
                    here.push(component);

                    tree.entry(here.to_str().unwrap().to_string())
                        .and_modify(|dir| *dir += size);
                }
            }
        }
    }

    tree
}

pub fn part_one(input: &str) -> Option<usize> {
    let tree = parse(input);

    let candidates: usize = tree.values().filter(|&v| *v <= 100_000).sum();

    Some(candidates)
}

const TOTAL_CAPACITY: usize = 70_000_000;
const SPACE_REQUIRED: usize = 30_000_000;

pub fn part_two(input: &str) -> Option<usize> {
    let tree = parse(input);
    let unused_space = TOTAL_CAPACITY
        .checked_sub(*tree.get("/").unwrap())
        .expect("using more space than total capacity");
    let space_required = SPACE_REQUIRED
        .checked_sub(unused_space)
        .expect("already have enough space!");

    let mut candidates = vec![];

    for (dir, size) in tree.iter() {
        if *size < space_required {
            continue;
        }

        candidates.push((dir, size));
    }

    candidates.sort_by_key(|(_, &size)| size);

    Some(*candidates[0].1)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 7);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 7);
        assert_eq!(part_one(&input), Some(95437));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 7);
        assert_eq!(part_two(&input), Some(24933642));
    }
}

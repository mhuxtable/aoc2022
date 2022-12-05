// Some nice parsing logic to deal with today, but the actual puzzle wasn't all too hard to solve.
// A neat little problem involving a stack and a queue, and quite predictable where it was going to
// go after seeing the first part. Parsing logic could be nicer and would have been slightly easier
// if the number of stacks was known a priori, i.e. put the indices line first.

use std::collections::VecDeque;

#[derive(Debug)]
struct Move {
    quantity: usize,
    from: usize,
    to: usize,
}

fn parse(input: &str) -> (Vec<VecDeque<String>>, Vec<Move>) {
    let mut stacks: Vec<VecDeque<String>> = vec![];
    let mut moves: Vec<Move> = vec![];

    for line in input.lines() {
        if line.starts_with("move") {
            let mut it = line.split_whitespace().skip(1);

            let qty: usize = it.next().unwrap().parse().expect("quantity");
            assert!(it.next().expect("from") == "from");
            let from: usize = it.next().unwrap().parse().expect("from");
            assert!(it.next().expect("to") == "to");
            let to: usize = it.next().unwrap().parse().expect("to");

            moves.push(Move {
                quantity: qty,
                from,
                to,
            });
        } else if line.contains("1") {
            continue;
        } else if line.is_empty() {
            continue;
        } else {
            // dbg!(line);

            let mut stack = 0;
            let chars = line.chars().collect::<Vec<char>>();

            let mut i = 0;

            loop {
                if i >= chars.len() {
                    break;
                }

                if stacks.len() < stack + 1 {
                    // push a new VecDeque as we found a new stack
                    stacks.push(VecDeque::new());
                }

                let crate_id = chars[i + 1];
                i += 4;

                if !crate_id.is_whitespace() {
                    stacks[stack].push_back(crate_id.to_string());
                }

                stack += 1;
            }
        }
    }

    // println!("{:?} {:?}", stacks, moves);

    (stacks, moves)
}

pub fn part_one(input: &str) -> Option<String> {
    let (mut stacks, moves) = parse(input);

    for mv in moves {
        for _ in 0..mv.quantity {
            let crate_id = stacks[mv.from - 1].pop_front().unwrap();
            stacks[mv.to - 1].push_front(crate_id);
        }
    }

    let tops = stacks
        .iter()
        .map(|stack| stack.front().unwrap().to_string())
        .collect::<Vec<String>>()
        .join("");

    Some(tops)
}

pub fn part_two(input: &str) -> Option<String> {
    let (mut stacks, moves) = parse(input);

    // nice, we can make a FIFO out of two stacks

    for mv in moves {
        let mut tmp = VecDeque::new();

        for _ in 0..mv.quantity {
            let crate_id = stacks[mv.from - 1].pop_front().unwrap();
            tmp.push_front(crate_id);
        }

        while !tmp.is_empty() {
            let item = tmp.pop_front().unwrap();
            stacks[mv.to - 1].push_front(item);
        }
    }

    let tops = stacks
        .iter()
        .map(|stack| stack.front().unwrap().to_string())
        .collect::<Vec<String>>()
        .join("");

    Some(tops)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 5);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 5);
        assert_eq!(part_one(&input), Some("CMZ".to_string()));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 5);
        assert_eq!(part_two(&input), Some("MCD".to_string()));
    }
}

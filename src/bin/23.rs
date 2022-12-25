use itertools::Itertools;
use std::collections::HashSet;

fn parse(input: &str) -> HashSet<(isize, isize)> {
    let mut elves = vec![];

    for (y, line) in input.lines().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == '#' {
                elves.push((x as isize, y as isize));
            }
        }
    }

    let mut positions = HashSet::new();
    for elf in elves {
        positions.insert(elf);
    }

    positions
}

fn print_grid(positions: &HashSet<(isize, isize)>) -> (String, (isize, isize), (isize, isize)) {
    let (from, to) = {
        let (min_x, max_x, min_y, max_y) = positions.iter().fold(
            (isize::MAX, isize::MIN, isize::MAX, isize::MIN),
            |(mut min_x, mut max_x, mut min_y, mut max_y), (x, y)| {
                if *x < min_x {
                    min_x = *x;
                }
                if *x > max_x {
                    max_x = *x;
                }
                if *y < min_y {
                    min_y = *y;
                }
                if *y > max_y {
                    max_y = *y;
                }

                (min_x, max_x, min_y, max_y)
            },
        );

        ((min_x, min_y), (max_x, max_y))
    };

    let mut s = String::new();
    for row in from.1..=to.1 {
        for col in from.0..=to.0 {
            if positions.contains(&(col, row)) {
                s.push('#');
            } else {
                s.push('.');
            }
        }

        s.push('\n');
    }

    (s, from, to)
}

fn play_game(
    mut positions: HashSet<(isize, isize)>,
    round_identifier: usize,
) -> (HashSet<(isize, isize)>, usize) {
    let mut moves = vec![];
    let starting_elves = positions.len();

    for (x, y) in positions.iter() {
        if vec![
            (*x - 1, *y - 1),
            (*x, *y - 1),
            (*x + 1, *y - 1),
            (*x - 1, *y),
            (*x + 1, *y),
            (*x - 1, *y + 1),
            (*x, *y + 1),
            (*x + 1, *y + 1),
        ]
        .iter()
        .filter(|&adj| positions.contains(adj))
        .count()
            == 0
        {
            // This elf has no adjacents so does not move on this round.
            continue;
        }

        let mut candidates = vec![
            vec![(*x, *y - 1), (*x + 1, *y - 1), (*x - 1, *y - 1)], // North
            vec![(*x, *y + 1), (*x + 1, *y + 1), (*x - 1, *y + 1)], // South
            vec![(*x - 1, *y), (*x - 1, *y - 1), (*x - 1, *y + 1)], // West
            vec![(*x + 1, *y), (*x + 1, *y - 1), (*x + 1, *y + 1)], // East
        ];
        candidates.rotate_left(round_identifier % 4);

        let (next_x, next_y) = candidates
            .iter()
            .find_map(|candidates| {
                if candidates.iter().all(|c| !positions.contains(c)) {
                    Some(*candidates.first().unwrap())
                } else {
                    None
                }
            })
            .unwrap_or((*x, *y));

        moves.push(((*x, *y), (next_x, next_y)));
    }

    let move_count = moves.iter().counts_by(|(_, to)| *to);

    for (from, to) in &moves {
        if move_count[to] == 1 && !positions.contains(&to) {
            // moves as nobody else proposed to move here
            positions.remove(&from);
            positions.insert(*to);
        }
    }

    assert!(starting_elves == positions.len());

    (
        positions,
        move_count.iter().filter(|(_, &count)| count == 1).count(),
    )
}

pub fn part_one(input: &str) -> Option<u32> {
    let mut positions = parse(input);

    for step in 0..10 {
        let (grid, _, _) = print_grid(&positions);
        println!("===============\nBefore step {}\n\n{}\n", step, grid);

        (positions, _) = play_game(positions, step);
    }

    let (grid, from, to) = print_grid(&positions);
    println!("{}", grid);

    let empty_squares =
        ((from.1.abs_diff(to.1) + 1) * (from.0.abs_diff(to.0) + 1)) - positions.len();

    Some(empty_squares as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    let mut elves = parse(input);
    let mut rounds = 0;

    loop {
        let (grid, _, _) = print_grid(&elves);
        println!("===============\nBefore step {}\n\n{}\n", rounds, grid);

        let (new_elves, moved) = play_game(elves, rounds);

        if moved == 0 {
            break;
        } else {
            elves = new_elves;
        }

        rounds += 1;
    }

    Some(rounds as u32 + 1)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 23);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 23);
        assert_eq!(part_one(&input), Some(110));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 23);
        assert_eq!(part_two(&input), Some(20));
    }
}

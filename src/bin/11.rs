use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use itertools::Itertools;

#[derive(Debug)]
struct Monkey {
    items: RefCell<Vec<u64>>,
    test: u64,
    op: Operation,
    if_true: u32,
    if_false: u32,
}

type Operator = fn(u64, u64) -> u64;

static MUL: Operator = |l: u64, r: u64| l * r;
static ADD: Operator = |l: u64, r: u64| l + r;

#[derive(Debug)]
enum RHS {
    Old,
    Literal(u64),
}

impl RHS {
    pub fn get(&self, old: u64) -> u64 {
        match self {
            Self::Old => old,
            Self::Literal(x) => *x,
        }
    }
}

#[derive(Debug)]
struct Operation {
    op: Operator,
    rhs: RHS,
}

impl Operation {
    pub fn compute(&self, old: u64) -> u64 {
        (self.op)(old, self.rhs.get(old))
    }
}

fn parse_op(s: &str) -> Operation {
    let mut tokens = s.split_whitespace();
    let op = match tokens.next().unwrap() {
        "*" => MUL,
        "+" => ADD,
        x => panic!("unknown operation {}", x),
    };
    let rhs = match tokens.next().unwrap() {
        "old" => RHS::Old,
        x if x.parse::<u64>().is_ok() => RHS::Literal(x.parse().unwrap()),
        x => panic!("unknown right token {}", x),
    };

    Operation { op, rhs }
}

fn parse_test_outcome(s: &str) -> u32 {
    let (_, monkey) = s.split_once("throw to monkey ").unwrap();

    monkey.parse().unwrap()
}

fn parse(input: &str) -> Vec<Monkey> {
    let mut monkeys = vec![];

    for mut chunk in input.lines().chunks(7).into_iter() {
        chunk.next().expect("no monkey"); // Monkey n

        let items: Vec<u64> = chunk
            .next()
            .expect("no starting items")
            .strip_prefix("  Starting items: ")
            .expect("starting items in wrong format")
            .split(", ")
            .map(|x| x.parse().unwrap())
            .collect();

        let operation = parse_op(
            chunk
                .next()
                .expect("no operation")
                .strip_prefix("  Operation: new = old ")
                .expect("operation in wrong format"),
        );

        let test: u64 = chunk
            .next()
            .expect("no test")
            .strip_prefix("  Test: divisible by ")
            .expect("test in wrong format")
            .parse()
            .expect("test not a numeric value");

        let if_true = parse_test_outcome(chunk.next().expect("test true outcome"));
        let if_false = parse_test_outcome(chunk.next().expect("test false outcome"));

        monkeys.push(Monkey {
            items: RefCell::new(items),
            test,
            op: operation,
            if_true,
            if_false,
        })
    }

    monkeys
}

pub fn part_one(input: &str) -> Option<u32> {
    let monkeys = parse(input);
    let mut inspected = vec![0u32; monkeys.len()];

    for _ in 0..20 {
        for i in 0..monkeys.len() {
            let monkey = &monkeys[i];
            inspected[i] += monkey.items.borrow().len() as u32;

            while let Some(item) = monkey.items.borrow_mut().pop() {
                let worry_level = monkey.op.compute(item) / 3;
                let next_monkey = if worry_level % monkey.test == 0 {
                    monkey.if_true
                } else {
                    monkey.if_false
                };

                monkeys[next_monkey as usize]
                    .items
                    .borrow_mut()
                    .push(worry_level);
            }
        }
    }

    inspected.sort();
    Some(inspected[inspected.len() - 2] * inspected[inspected.len() - 1])
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 11);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 11);
        assert_eq!(part_one(&input), Some(10605));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 11);
        assert_eq!(part_two(&input), None);
    }
}

// This was a long exercise to write the parsing logic for (albeit the parsing was much simpler
// than the crane exercise a few days ago!).
//
// It also has a nasty twist in overflowing the u64 in part 2. This isn't readily resolvable using
// a big integer type (a common approach but the size of the numbers blow up far too quickly for
// this to be practicable:
// https://www.reddit.com/r/adventofcode/comments/zioepr/2022_day_11_part_2_ridiculous_worry_levels/).
// I didn't go anywhere near big-ints because it was clear the size of the numbers blew up very
// quickly in the initial experiment, and the puzzle description implied we needed something
// smarter than just smashing a u64 in and hoping for the best.
//
// The solution is some number theory to perform the operations in modular arithmetic, modulo the
// lowest common multiple of the divisors used in each monkey's test. We need only to preserve the
// divisibility of the numbers that result with respect to the various tests used by the monkeys;
// we don't care about the actual numbers themselves! In this case, the test divisors are all
// prime, so their lowest common multiple is simply their product. Thus, for some item with worry
// level A, its score is divisible by its test T if, and only if, A - kT is. If we choose k to be
// the lowest common multiple of all monkey test values, k will always be divisible by the test.
//
// I went for this on a hunch to begin with, based on intuition, and came back to figure the theory
// out once it worked :-)

use itertools::Itertools;
use std::{
    cell::RefCell,
    ops::{Add, Div, Mul},
};

#[derive(Debug)]
struct Monkey {
    items: RefCell<Vec<Modular>>,
    test: u32,
    op: Operation,
    if_true: u32,
    if_false: u32,
}

type Operator = fn(Modular, Modular) -> Modular;

#[derive(Clone, Copy, Debug)]
struct Modular {
    remainder: u32,
    divisor: u32,
}

impl Modular {
    fn new(mut remainder: u32, divisor: u32) -> Modular {
        if remainder > divisor {
            remainder = remainder % divisor;
        }

        Modular { remainder, divisor }
    }

    fn get_remainder(&self) -> u32 {
        self.remainder
    }
}

trait IntoModular {
    fn to_modular(self, divisor: u32) -> Modular;
}

impl IntoModular for u32 {
    fn to_modular(self, divisor: u32) -> Modular {
        Modular::new(self, divisor)
    }
}

impl Add for Modular {
    type Output = Modular;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.divisor, rhs.divisor,
            "cannot add modular numbers of different divisors"
        );

        Modular {
            remainder: ((self.remainder as u64 + rhs.remainder as u64) % (self.divisor as u64))
                as u32,
            divisor: self.divisor,
        }
    }
}

impl Mul for Modular {
    type Output = Modular;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.divisor, rhs.divisor,
            "cannot multiply modular numbers of different divisors"
        );

        Modular {
            remainder: ((self.remainder as u64 * rhs.remainder as u64) % (self.divisor as u64))
                as u32,
            divisor: self.divisor,
        }
    }
}

static MUL: Operator = |l: Modular, r: Modular| l * r;
static ADD: Operator = |l: Modular, r: Modular| l + r;

#[derive(Debug)]
enum RHS {
    Old,
    Literal(Modular),
}

impl RHS {
    pub fn get(&self, old: Modular) -> Modular {
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
    pub fn compute(&self, old: Modular) -> Modular {
        (self.op)(old, self.rhs.get(old))
    }
}

fn parse_op(s: &str, test_divisor: u32) -> Operation {
    let mut tokens = s.split_whitespace();
    let op = match tokens.next().unwrap() {
        "*" => MUL,
        "+" => ADD,
        x => panic!("unknown operation {}", x),
    };
    let rhs = match tokens.next().unwrap() {
        "old" => RHS::Old,
        x if x.parse::<u32>().is_ok() => {
            RHS::Literal(x.parse::<u32>().unwrap().to_modular(test_divisor))
        }
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
    let test_divisor = input
        .lines()
        .skip(3)
        .step_by(7)
        .map(|l| {
            l.strip_prefix("  Test: divisible by ")
                .unwrap()
                .parse::<i32>()
                .unwrap()
        })
        .product::<i32>() as u32;

    for mut chunk in input.lines().chunks(7).into_iter() {
        chunk.next().expect("no monkey"); // Monkey n

        let items: Vec<Modular> = chunk
            .next()
            .expect("no starting items")
            .strip_prefix("  Starting items: ")
            .expect("starting items in wrong format")
            .split(", ")
            .map(|x| x.parse::<u32>().unwrap().to_modular(test_divisor))
            .collect();

        let operation = parse_op(
            chunk
                .next()
                .expect("no operation")
                .strip_prefix("  Operation: new = old ")
                .expect("operation in wrong format"),
            test_divisor,
        );

        let test: u32 = chunk
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

fn play_game<W>(monkeys: Vec<Monkey>, rounds: usize, worry_update: W) -> Vec<u32>
where
    W: Fn(Modular) -> Modular,
{
    let mut inspected = vec![0u32; monkeys.len()];

    for _ in 0..rounds {
        for i in 0..monkeys.len() {
            let monkey = &monkeys[i];
            inspected[i] += monkey.items.borrow().len() as u32;

            while let Some(item) = monkey.items.borrow_mut().pop() {
                let worry_level = worry_update(monkey.op.compute(item));
                let next_monkey = if worry_level.get_remainder() % monkey.test == 0 {
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
    inspected.reverse();
    inspected
}

pub fn part_one(input: &str) -> Option<u32> {
    let monkeys = parse(input);
    let inspected = play_game(monkeys, 20, |x| {
        // division is not in general defined in mod arithmetic. Just hack it because we know that
        // we won't overflow the u32 in part 1 with the division by 3
        Modular {
            remainder: x.remainder / 3,
            divisor: x.divisor,
        }
    });
    Some(inspected[0] * inspected[1])
}

pub fn part_two(input: &str) -> Option<u64> {
    let monkeys = parse(input);
    let inspected = play_game(monkeys, 10_000, |x| x);
    // Yes, even the inspection counts overflow a u32 when multiplied!
    Some(inspected[0] as u64 * inspected[1] as u64)
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
        assert_eq!(part_two(&input), Some(2_713_310_158));
    }
}

use std::{collections::HashMap, fmt::Display, str::FromStr};

enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
struct OpParseError {}

impl Display for OpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing op")
    }
}

impl std::error::Error for OpParseError {}

impl FromStr for Op {
    type Err = OpParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            _ => Err(OpParseError {}),
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",
            }
        )
    }
}

impl Op {
    pub fn compute(&self, lhs: u64, rhs: u64) -> u64 {
        (match self {
            Self::Add => lhs.checked_add(rhs),
            Self::Mul => lhs.checked_mul(rhs),
            Self::Sub => lhs.checked_sub(rhs),
            Self::Div => lhs.checked_div(rhs),
        })
        .expect("overflow")
    }
}

enum Job {
    Yell(u64),
    Operation(Op, String, String),
    Matches(String, String),
}

#[derive(Debug)]
struct JobParseError {}

impl Display for JobParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing job")
    }
}

impl std::error::Error for JobParseError {}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yell(x) => write!(f, "yells {}", x),
            Self::Operation(op, lhs, rhs) => write!(f, " does {} {} {}", lhs, op, rhs),
            Self::Matches(lhs, rhs) => write!(f, "equal {} {}", lhs, rhs),
        }
    }
}

impl FromStr for Job {
    type Err = JobParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(literal) = s.parse::<u64>() {
            Ok(Self::Yell(literal))
        } else {
            let mut parts = s.split_whitespace();

            let dep1 = parts.next().unwrap();
            let op = parts
                .next()
                .unwrap()
                .parse::<Op>()
                .map_err(|_| Self::Err {})?;
            let dep2 = parts.next().unwrap();

            Ok(Self::Operation(op, dep1.to_string(), dep2.to_string()))
        }
    }
}

fn parse(input: &str) -> Result<HashMap<String, Job>, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();

    for line in input.lines() {
        let (monkey, directive) = line.split_once(": ").unwrap();
        map.insert(monkey.to_string(), directive.parse()?);
    }

    Ok(map)
}

pub fn part_one(input: &str) -> Option<u64> {
    let jobs = parse(input).unwrap();

    let mut results: HashMap<String, u64> = HashMap::new();

    for (name, job) in &jobs {
        if let Job::Yell(x) = job {
            results.insert(name.clone(), *x);
        }
    }

    let mut q = vec!["root".to_string()];
    let mut explore = vec!["root".to_string()];

    while !explore.is_empty() {
        let mut next = vec![];

        for item in &explore {
            match &jobs[item] {
                Job::Operation(_, lhs, rhs) => {
                    q.extend(vec![lhs.clone(), rhs.clone()]);
                    next.extend(vec![lhs.clone(), rhs.clone()]);
                }
                _ => {}
            }
        }

        explore = next;
    }

    while !q.is_empty() {
        let item = q.pop().unwrap();

        match &jobs[&item] {
            Job::Yell(x) => {
                results.insert(item, *x);
            }
            Job::Operation(op, lhs, rhs) => {
                let (has_left, has_right) = (results.contains_key(lhs), results.contains_key(rhs));
                assert!(has_left && has_right);
                results.insert(item.to_string(), op.compute(results[lhs], results[rhs]));
            }
            _ => panic!("unsupported operation"),
        }
    }

    Some(results["root"])
}

pub fn part_two(input: &str) -> Option<u64> {
    let mut jobs = parse(input).unwrap();

    if let Job::Operation(_, lhs, rhs) = &jobs["root"] {
        *jobs.get_mut("root").unwrap() = Job::Matches(lhs.clone(), rhs.clone());
    } else {
        panic!("invalid root input");
    }

    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 21);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 21);
        assert_eq!(part_one(&input), Some(152));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 21);
        assert_eq!(part_two(&input), None);
    }
}

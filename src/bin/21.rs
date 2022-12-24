use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Clone, Debug)]
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
    pub fn compute(&self, lhs: i64, rhs: i64) -> i64 {
        (match self {
            Self::Add => lhs.checked_add(rhs),
            Self::Mul => lhs.checked_mul(rhs),
            Self::Sub => lhs.checked_sub(rhs),
            Self::Div => lhs.checked_div(rhs),
        })
        .expect("overflow")
    }
}

#[derive(Clone, Debug)]
enum Job {
    Yell(i64),
    Operation(Op, String, String),
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
        }
    }
}

impl FromStr for Job {
    type Err = JobParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(literal) = s.parse::<i64>() {
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

impl Job {
    pub fn operation(&self) -> (String, Op, String) {
        if let Job::Operation(op, lhs, rhs) = self {
            (lhs.clone(), op.clone(), rhs.clone())
        } else {
            panic!("not an operation");
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

fn explore(jobs: &HashMap<String, Job>, start_at: &str) -> Vec<String> {
    let mut q = vec![start_at.to_string()];
    let mut explore = vec![start_at.to_string()];

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

    q
}

fn reduce(jobs: &HashMap<String, Job>, start: &str) -> i64 {
    let mut results: HashMap<String, i64> = HashMap::new();

    for (name, job) in jobs {
        if let Job::Yell(x) = job {
            results.insert(name.clone(), *x);
        }
    }

    let mut q = explore(&jobs, start);

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
        }
    }

    results[start]
}

pub fn part_one(input: &str) -> Option<i64> {
    let jobs = parse(input).unwrap();
    let result = reduce(&jobs, "root");

    Some(result)
}

pub fn part_two(input: &str) -> Option<i64> {
    let jobs = parse(input).unwrap();

    let (lhs, _, rhs) = &jobs["root"].operation();
    let has_human = |job: &str| explore(&jobs, job).contains(&"humn".to_string());

    #[derive(Debug, PartialEq, Eq)]
    enum Side {
        Left,
        Right,
    }

    let solve = |lhs, rhs| {
        let (side_to_reduce, next, human_side) = if has_human(lhs) {
            (rhs, lhs, Side::Left)
        } else {
            (lhs, rhs, Side::Right)
        };

        eprintln!("reducing {} next {}", side_to_reduce, next);

        (reduce(&jobs, side_to_reduce), next, human_side)
    };

    let (mut result, mut next, _) = solve(lhs, rhs);
    let mut one_over = false;

    while next != "humn" {
        eprintln!("{} = {}", result, &jobs[next]);

        // get the next job, figure out the side with the human, reduce the other side and do the
        // inverse to result
        match &jobs[next] {
            Job::Yell(_) => {
                panic!("unexpectedly reached a terminal state without finding the human!")
            }
            Job::Operation(op, lhs, rhs) => {
                let (this_result, this_next, human_side) = solve(lhs, rhs);
                eprintln!("{} {} {:?}", this_result, this_next, human_side);

                result = match op {
                    Op::Add => result.checked_sub(this_result),
                    Op::Sub => {
                        if human_side == Side::Left {
                            result.checked_add(this_result)
                        } else {
                            // result = this_result - humn
                            // result - this_result = -humn
                            // -result + this_result = humn
                            result
                                .checked_sub(this_result)
                                .expect("overflow")
                                .checked_mul(-1)
                        }
                    }
                    Op::Mul => result.checked_div(this_result),
                    Op::Div => {
                        if human_side == Side::Left {
                            result.checked_mul(this_result)
                        } else {
                            one_over = !one_over;
                            result.checked_div(this_result)
                        }
                    }
                }
                .expect("overflow");

                next = this_next;
            }
        }
    }
    eprintln!("{} = {} ({})", result, &jobs[next], one_over);

    Some(if one_over {
        1i64.checked_div(result).unwrap()
    } else {
        result
    })
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
        assert_eq!(part_two(&input), Some(301));
    }
}

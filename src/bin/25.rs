use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, Debug)]
struct SNAFU(i64);

#[derive(Debug)]
struct SNAFUParseError {
    input: String,
}

impl Display for SNAFUParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error while parsing SNAFU number {}", self.input)
    }
}

impl std::error::Error for SNAFUParseError {}

impl From<SNAFU> for i64 {
    fn from(number: SNAFU) -> Self {
        number.0
    }
}

impl From<i64> for SNAFU {
    fn from(x: i64) -> Self {
        Self(x)
    }
}

impl FromStr for SNAFU {
    type Err = SNAFUParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut place_value = 1;
        let mut cur = 0;

        for ch in s.chars().rev() {
            cur += place_value
                * match ch {
                    '0' => 0,
                    '1' => 1,
                    '2' => 2,
                    '=' => -2,
                    '-' => -1,
                    _ => {
                        return Err(Self::Err {
                            input: s.to_string(),
                        })
                    }
                };

            place_value *= 5;
        }

        Ok(SNAFU(cur))
    }
}

impl Display for SNAFU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut digits = vec![];

        {
            let mut cur = self.0;

            while cur > 0 {
                let bit = cur % 5;
                cur /= 5;

                digits.push(bit);
            }
        }

        digits.reverse();

        let digits: Vec<char> = (0..digits.len())
            .rev()
            .fold(vec![0], |mut chars, x| {
                let this = digits[x] + chars.pop().unwrap();
                let rem = this % 5;

                let carry = if rem == 3 || rem == 4 || this >= 5 {
                    1
                } else {
                    0
                };

                chars.push(rem);
                chars.push(carry);

                chars
            })
            .into_iter()
            .map(|x| match x {
                3 => '=',
                4 => '-',
                x => (x as u8 + b'0') as char,
            })
            .rev()
            .collect();

        write!(f, "{}", String::from_iter(digits).trim_start_matches("0"))
    }
}

#[cfg(test)]
mod snafu_tests {
    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        static ref TESTS: Vec<(&'static str, i64)> = vec![
            ("1", 1),
            ("2", 2),
            ("1=", 3),
            ("1-", 4),
            ("10", 5),
            ("11", 6),
            ("12", 7),
            ("2=", 8),
            ("2-", 9),
            ("20", 10),
            ("1=0", 15),
            ("1-0", 20),
            ("1=11-2", 2022),
            ("1-0---0", 12345),
            ("1121-1110-1=0", 314159265),
        ];
    }

    #[test]
    fn test_from_str() {
        for (input, out) in TESTS.iter() {
            let number: SNAFU = input.parse().expect("unable to parse");

            assert_eq!(i64::from(number), *out);
        }
    }

    #[test]
    fn test_display() {
        for (input, out) in TESTS.iter() {
            let number: SNAFU = input.parse().expect("unable to parse");

            assert_eq!(format!("{}", number), *input, "decimal {}", out);
        }
    }
}

pub fn part_one(input: &str) -> Option<String> {
    let numbers: Vec<i64> = input
        .lines()
        .map(|snafu| snafu.parse::<SNAFU>())
        .collect::<Result<Vec<SNAFU>, SNAFUParseError>>()
        .expect("parse error")
        .iter()
        .map(|&snafu| i64::from(snafu))
        .collect();

    let sum: i64 = numbers.iter().sum();
    Some(format!("{}", SNAFU::from(sum)))
}

pub fn part_two(input: &str) -> Option<i64> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 25);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 25);
        assert_eq!(part_one(&input), Some("2=-1=0".to_string()));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 25);
        assert_eq!(part_two(&input), None);
    }
}

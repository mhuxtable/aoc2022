use itertools::Itertools;
use std::fmt::Display;

#[derive(Clone, Debug)]
enum Packet {
    List(Vec<Packet>),
    Literal(u32),
}

macro_rules! packet_literal {
    ($x: expr) => {
        Packet::Literal($x)
    };
}

macro_rules! packet_list {
    ($x: expr) => {
        Packet::List(Vec::from($x).iter().map(|&x| packet_literal!(x)).collect())
    };
}

impl Packet {
    pub fn push(&mut self, x: Packet) {
        match self {
            Self::List(inner) => inner.push(x),
            _ => panic!("wrong type to push"),
        }
    }

    pub fn is_list(&self) -> bool {
        if let Self::List(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_literal(&self) -> bool {
        if let Self::Literal(_) = self {
            true
        } else {
            false
        }
    }

    pub fn literal(&self) -> &u32 {
        if let Self::Literal(x) = self {
            x
        } else {
            panic!("wrong type");
        }
    }

    pub fn list(&self) -> &Vec<Packet> {
        if let Self::List(lst) = self {
            lst
        } else {
            panic!("wrong type");
        }
    }

    pub fn as_list(&self) -> Packet {
        if self.is_list() {
            self.clone()
        } else {
            Packet::List(vec![self.clone()])
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(x) => write!(f, "{}", x),
            Self::List(lst) => {
                let d = lst.into_iter().map(|x| format!("{}", x)).join(",");

                write!(f, "[{}]", d)
            }
        }
    }
}

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        if self.is_literal() && other.is_literal() {
            self.literal() == other.literal()
        } else if self.is_list() && other.is_list() {
            self.list() == other.list()
        } else {
            false
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(std::cmp::Ordering::Equal)
        } else if self.is_list() && other.is_list() {
            let (x, y) = (self.list(), other.list());
            // We can't check the lengths of the lists ahead of time, because the exercise
            // describes "run out of items". If there is an ordering that matches between the lists
            // before running out of items, that matches and is accepted before we reach the end of
            // one of the lists.
            for (i, item) in x.iter().enumerate() {
                if i >= y.len() {
                    // Ran out of items in the right
                    return Some(std::cmp::Ordering::Greater);
                }

                let cmp = item.partial_cmp(&y[i]);
                if cmp.is_some() && (cmp.unwrap().is_gt() || cmp.unwrap().is_lt()) {
                    return cmp;
                }
            }

            if x.len() == y.len() {
                None
            } else {
                // Ran out of items in the left
                Some(std::cmp::Ordering::Less)
            }
        } else if self.is_literal() && other.is_literal() {
            self.literal().partial_cmp(other.literal())
        } else {
            // one of the sides is a literal and the other is a list
            let (x, y) = (self.as_list(), other.as_list());
            x.partial_cmp(&y)
        }
    }
}

impl From<&str> for Packet {
    fn from(s: &str) -> Self {
        let mut stack: Vec<Packet> = vec![];
        let mut cur_digit = None;

        for ch in s.chars() {
            // Reached a boundary, so tidy up any accrued digits
            if (ch == ',' || ch == ']') && cur_digit.is_some() {
                let literal = packet_literal!(cur_digit.unwrap());
                stack.last_mut().unwrap().push(literal);
                cur_digit = None;
            }

            if ch == ',' {
                continue;
            } else if ch == '[' {
                stack.push(Packet::List(vec![]));
            } else if ch == ']' {
                if stack.len() > 1 {
                    let last = stack.pop().unwrap();
                    stack.last_mut().unwrap().push(last);
                }
            } else {
                cur_digit = Some(10 * cur_digit.unwrap_or_default() + ch.to_digit(10).unwrap());
            }
        }

        stack.last().unwrap().clone()
    }
}

#[cfg(test)]
mod packet_tests {
    use super::*;

    #[test]
    fn test_packets() {
        let tests = vec![
            ("[1]", packet_list!([1])),
            ("[1,2]", packet_list!([1, 2])),
            (
                "[[[[3]]]]",
                Packet::List(vec![Packet::List(vec![Packet::List(vec![Packet::List(
                    vec![packet_literal!(3)],
                )])])]),
            ),
            (
                "[1,[2,[3]]]",
                Packet::List(vec![
                    packet_literal!(1),
                    Packet::List(vec![
                        packet_literal!(2),
                        Packet::List(vec![packet_literal!(3)]),
                    ]),
                ]),
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(Packet::from(input), expect);
        }
    }
}

fn parse(input: &str) -> Vec<Packet> {
    input
        .lines()
        .filter_map(|l| {
            if l.is_empty() {
                None
            } else {
                Some(Packet::from(l))
            }
        })
        .collect()
}

pub fn part_one(input: &str) -> Option<u32> {
    let packets = parse(input);
    let pairs = packets
        .as_slice()
        .chunks(2)
        .map(|chunk| (&chunk[0], &chunk[1]))
        .collect::<Vec<(&Packet, &Packet)>>();

    for (p1, p2) in &pairs {
        println!("{} {} {:?}\n", p1, p2, p1 < p2);
    }

    Some(
        pairs
            .iter()
            .enumerate()
            .map(|(i, (p1, p2))| if p1 < p2 { i + 1 } else { 0 })
            .sum::<usize>() as u32,
    )
}

pub fn part_two(input: &str) -> Option<u32> {
    let mut packets = parse(input);
    let divider_packets: Vec<Packet> = [2u32, 6]
        .iter()
        .map(|&x| Packet::List(vec![packet_list!([x])]))
        .collect();

    packets.extend(divider_packets.clone());
    packets.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let decoder_key = divider_packets
        .iter()
        .map(|pkt| {
            packets
                .iter()
                .enumerate()
                .find_map(|(i, x)| if x == pkt { Some(i as u32 + 1) } else { None })
                .unwrap()
        })
        .product();

    Some(decoder_key)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 13);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 13);
        assert_eq!(part_one(&input), Some(13));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 13);
        assert_eq!(part_two(&input), Some(140));
    }
}

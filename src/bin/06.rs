// Easy peasy today! Nice use of a ring buffer, although I tried to use the ringbuffer crate and
// discovered by part 2 that it needs capacity to be a power of 2, so hacked together my own (most
// probably inefficient and not benchmarked) implementation that suffices for this exercise.

use std::collections::VecDeque;

struct RingBuffer(usize, VecDeque<char>);

impl RingBuffer {
    pub fn with_capacity(c: usize) -> RingBuffer {
        RingBuffer(c, VecDeque::with_capacity(c))
    }

    pub fn push(&mut self, x: char) {
        assert!(
            self.1.len() <= self.0,
            "ring buffer is more full than expected capacity!"
        );

        if self.1.len() == self.0 {
            _ = self.1.pop_front();
        }

        self.1.push_back(x);
    }

    pub fn len(&self) -> usize {
        self.1.len()
    }

    pub fn capacity(&self) -> usize {
        self.0
    }

    pub fn to_vec(&self) -> Vec<char> {
        Vec::from(self.1.clone()).clone()
    }
}

fn solve(input: &str, uniques: usize) -> Option<u32> {
    let mut buf = RingBuffer::with_capacity(uniques);

    for (i, ch) in input.chars().enumerate() {
        buf.push(ch);

        if buf.len() < buf.capacity() {
            // We don't have enough items yet to have detected a start-of-packet marker.
            continue;
        }

        let mut v = buf.to_vec();
        v.sort();

        if v.iter().zip(v.iter().skip(1)).all(|(x, y)| x != y) {
            // got all unique characters
            return Some(i as u32 + 1);
        }
    }

    None
}

pub fn part_one(input: &str) -> Option<u32> {
    solve(input, 4)
}

pub fn part_two(input: &str) -> Option<u32> {
    solve(input, 14)
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 6);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 6);
        assert_eq!(part_one(&input), Some(10));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 6);
        assert_eq!(part_two(&input), Some(29));
    }
}

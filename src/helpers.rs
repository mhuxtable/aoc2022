/*
 * Use this file if you want to extract helpers from your solutions.
 * Example import from this file: `use advent_of_code::helpers::example_fn;`.
 */

use std::slice::Iter;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl From<&str> for Point {
    fn from(s: &str) -> Self {
        let (x, y) = s.split_once(',').unwrap();
        Point {
            x: x.parse().unwrap(),
            y: y.parse().unwrap(),
        }
    }
}

// I really just need to build a library that gives me a 1D grid that models an arbitrary sized
// rectangle, with lookup from (x,y) coordinates into the grid values. The number of times I
// implement this gives me lots and lots of practice. Perhaps today is the day?
//
// Oh look, I did it. Well, I pulled it out of the day's problem anyway
pub struct Grid<T>
where
    T: Clone + Default,
{
    values: Vec<T>,
    width: usize,
}

impl<T> Grid<T>
where
    T: Clone + Default,
{
    pub fn new(width: usize, height: usize) -> Grid<T> {
        Grid {
            values: vec![Default::default(); width * (height + 1)],
            width,
        }
    }

    pub fn point(&self, point: &Point) -> &T {
        &self.values[self.width * point.y + point.x]
    }

    pub fn point_mut(&mut self, point: &Point) -> &mut T {
        &mut self.values[self.width * point.y + point.x]
    }

    pub fn is_out_of_bounds(&self, point: &Point) -> bool {
        self.width * point.y + point.x >= self.values.len()
    }

    pub fn iter(&self) -> Iter<T> {
        self.values.iter()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.values.len() / self.width
    }
}

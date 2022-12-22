use std::{
    hash::Hash,
    num::ParseIntError,
    ops::{Add, Mul},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coord(i8, i8, i8);

impl From<(i8, i8, i8)> for Coord {
    fn from((x, y, z): (i8, i8, i8)) -> Self {
        Coord(x, y, z)
    }
}

impl Coord {
    pub fn dot(self, other: Self) -> i32 {
        self.0 as i32 as i32 * other.0 as i32 as i32
            + self.1 as i32 as i32 * other.1 as i32 as i32
            + self.2 as i32 * other.2 as i32
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.0, self.1, self.2)
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Coord(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Mul<i8> for Coord {
    type Output = Coord;

    fn mul(self, rhs: i8) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

macro_rules! coord {
    ( $x:expr, $y:expr, $z:expr ) => {
        Coord($x, $y, $z)
    };
}

#[derive(Clone, Debug, Eq, Hash)]
struct Plane {
    n: Coord,
    d: i32,
}

impl PartialEq for Plane {
    fn eq(&self, other: &Self) -> bool {
        // we only use unit vectors so no need to normalise magnitude
        let dot = self.n.dot(other.n);
        // need to technically check for negative d but won't happen in our case
        dot.abs() == 1 && self.d.abs() == other.d.abs()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BoundedPlane {
    plane: Plane,
    r0: Coord,
}

impl std::fmt::Display for BoundedPlane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:} {:} {:}", self.plane.n, self.plane.d, self.r0)
    }
}

fn parse(input: &str) -> Result<Vec<Coord>, ParseIntError> {
    let result = input
        .lines()
        .map(|line| {
            let parts: Result<Vec<i8>, ParseIntError> =
                line.splitn(3, ',').map(|x| x.parse()).collect();

            parts.map(|res| coord!(res[0], res[1], res[2]))
        })
        .collect();

    result
}

// (offset of point in plane, normal vector)
//
//    +---------+
//   /         *|
//  /    E    * |
// /         *  |
// +--------+ B |
// |        |   |
// |   A    |  *
// |        | *
// |        |*
// +--------+
const PLANES: [(Coord, Coord); 6] = [
    (coord!(0, 0, 0), coord!(0, 0, -1)), // A
    (coord!(1, 0, 0), coord!(1, 0, 0)),  // B
    (coord!(0, 0, 1), coord!(0, 0, 1)),  // C
    (coord!(0, 0, 0), coord!(-1, 0, 0)), // D
    (coord!(0, 1, 0), coord!(0, 1, 0)),  // E
    (coord!(0, 0, 0), coord!(0, -1, 0)), // F
];

pub fn part_one(input: &str) -> Option<i32> {
    let coords = parse(input).expect("parsing coordinates");

    let mut seen = vec![];
    let mut faces = 0;

    println!(
        "
   +---------+
  /         /|
 /    E    / |
/         /  |
+--------+ B |
|        |   |
|   A    |  /
|        | /
|        |/
+--------+
"
    );

    for coord in coords {
        let planes: Vec<BoundedPlane> = PLANES
            .iter()
            .map(|(offset, normal)| {
                let r0 = coord + *offset;
                let d = (*normal * -1).dot(r0);

                BoundedPlane {
                    plane: Plane { n: *normal, d },
                    r0,
                }
            })
            .collect();

        // println!(
        //     "Cube centred at {}:\n{}",
        //     coord,
        //     planes
        //         .iter()
        //         .map(|p| format!("{}", p))
        //         .collect::<Vec<String>>()
        //         .join("\n")
        // );

        for plane in planes {
            faces += if seen.contains(&plane) {
                // already seen this before, so we're hiding a face
                -1
            } else {
                // new face
                seen.push(plane);
                1
            };
        }
    }

    Some(faces)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 18);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 18);
        assert_eq!(part_one(&input), Some(64));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 18);
        assert_eq!(part_two(&input), None);
    }
}

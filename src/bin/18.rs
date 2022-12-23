use std::{
    borrow::Borrow,
    collections::HashSet,
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
        // we only use unit normal vectors so no need to normalise magnitude
        let dot = self.n.dot(other.n);
        dot.abs() == 1 && self.d == other.d * dot.signum()
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
//   /         /|
//  /    E    / |
// /         /  |
// +--------+ B |
// |        |   |
// |   A    |  /
// |        | /
// |        |/
// +--------+
const PLANES: [(Coord, Coord); 6] = [
    (coord!(0, 0, 0), coord!(0, 0, -1)), // A
    (coord!(1, 0, 0), coord!(1, 0, 0)),  // B
    (coord!(0, 0, 1), coord!(0, 0, 1)),  // C
    (coord!(0, 0, 0), coord!(-1, 0, 0)), // D
    (coord!(0, 1, 0), coord!(0, 1, 0)),  // E
    (coord!(0, 0, 0), coord!(0, -1, 0)), // F
];

fn all_planes<P>(points: P) -> Vec<BoundedPlane>
where
    P: IntoIterator,
    P::Item: Borrow<Coord>,
{
    let mut planes = vec![];

    for coord in points.into_iter() {
        let cube_planes: Vec<BoundedPlane> = PLANES
            .iter()
            .map(|(offset, normal)| {
                let r0 = *coord.borrow() + *offset;
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

        planes.extend(cube_planes);
    }

    planes
}

fn exterior_surface_area<P>(planes: P) -> u32
where
    P: IntoIterator,
    P::Item: Borrow<BoundedPlane>,
{
    let mut seen = vec![];
    let mut faces = 0;

    for plane in planes.into_iter() {
        faces += if seen.contains(plane.borrow()) {
            // already seen this before, so we're hiding a face
            -1
        } else {
            // new face
            seen.push(plane.borrow().clone());
            1
        };
    }

    assert!(faces >= 0);
    faces as u32
}

pub fn part_one(input: &str) -> Option<u32> {
    let coords = parse(input).expect("parsing coordinates");

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

    Some(exterior_surface_area(&all_planes(&coords)))
}

// The general principle for solving part 2 is to find the surface area of all unconnected faces
// (as part 1) and then iteratively remove faces that are exposed to air pockets. The problem
// description initially made me consider methods for solving that involved identifying the
// exterior exposed surface of the shape (e.g. using some variant of convex hull finding that we
// continually refine) and then look for internal planar surfaces that are not connected to other
// rocks that do not form part of these surfaces.
//
// However, the state space is sufficiently small that an alternative algorithm presents itself
// that converges on air pockets by selecting all cubes that are not occupied by rock in the state
// space, then iteratively removing candidates that cannot be air pockets because they are on the
// exterior of the shape:
//
// 1. For all coordinates in the state space, keep a record of whether the corresponding cube is
//    occupied by rock.
// 2. For cubes not occupied by rock, consider them to be air pocket candidates.
// 3. Repeatedly inspect the set of air pocket candidates, removing candidates that are not
//    surrounded on all 6 sides by either:
//    a) rock
//    b) other air pocket candidates
//
//    An air pocket that is surrounded by 6 rocks is a definite 1x1x1 air pocket. An air pocket
//    that is surrounded by n rocks and 6-n other air pocket candidates _might_ be an air pocket or
//    it might be on the exterior of the shape and has not been pruned yet.
//
//    Prune possible air pocket cubes that do not have exactly 6 neighbours that satisfy the above
//    conditions.
// 4. Repeat step 3 until no further changes are made.
//
// Note that it is essential in step 3 that adjacent cubes that are outside the bounds of the
// puzzle (i.e. where a dimension is less than or greater than the min/max values observed for that
// dimension), we do not count these as air but rather as nothingness; this ensures they are
// pruned. Thus successive layers from the outside of the puzzle will be successively removed until
// only air pockets remain.
//
// Once we have identified the definite air pockets, it is trivial to identify those in contact
// with interior faces of the shape and discount these faces.
//
// This algorithm extends to puzzles with any number of disjoint rock sections that may not
// themselves be connected.
pub fn part_two(input: &str) -> Option<u32> {
    let coords = parse(input).expect("parsing coordinates");

    fn find_min_max<F: Fn(&Coord) -> i32>(coords: &Vec<Coord>, f: F) -> (i32, i32) {
        coords.iter().map(|coord| f(coord)).fold(
            (i32::MAX, i32::MIN),
            |(mut min, mut max), item| {
                if item < min {
                    min = item;
                }
                if item > max {
                    max = item;
                }

                (min, max)
            },
        )
    }

    let (x_min, x_max) = find_min_max(&coords, |c| c.0 as i32);
    let (y_min, y_max) = find_min_max(&coords, |c| c.1 as i32);
    let (z_min, z_max) = find_min_max(&coords, |c| c.2 as i32);

    let mut states = vec![false; ((x_max + 1) * (y_max + 1) * (z_max + 1)) as usize];

    let pos = |coord: &Coord| {
        let (x, y, z) = (coord.0 as i32, coord.1 as i32, coord.2 as i32);

        if x as i32 >= x_min
            && x as i32 <= x_max
            && y as i32 >= y_min
            && y as i32 <= y_max
            && z as i32 >= z_min
            && z as i32 <= z_max
        {
            Some(
                ((z as i32 * (x_max + 1) * (y_max + 1)) + (y as i32 * (x_max + 1)) + x as i32)
                    as usize,
            )
        } else {
            // The coordinate provided is out of bounds for the problem space.
            None
        }
    };

    for coord in &coords {
        let pos = pos(coord).unwrap();
        assert!(states[pos] == false);

        states[pos] = true;
    }

    let mut faces = exterior_surface_area(&all_planes(&coords));
    let mut possible_air_gap = HashSet::new();

    let adjacents = |coord: &Coord| {
        let (x, y, z) = (coord.0, coord.1, coord.2);

        vec![
            Coord(x, y, z - 1),
            Coord(x, y, z + 1),
            Coord(x + 1, y, z),
            Coord(x - 1, y, z),
            Coord(x, y - 1, z),
            Coord(x, y + 1, z),
        ]
    };

    for x in x_min..=x_max {
        for y in y_min..=y_max {
            for z in z_min..=z_max {
                let c = Coord(x as i8, y as i8, z as i8);
                let rock_pos = pos(&c).unwrap();

                if states[rock_pos] {
                    // It's already rock, so can't be air.
                    continue;
                } else {
                    possible_air_gap.insert(c);
                }
            }
        }
    }

    println!(
        "Considering {} possible air gap candidates",
        possible_air_gap.len()
    );

    // repeatedly remove all air that is not either adjacent to 6 rocks or to n rocks and 6-n other
    // air pockets. This should leave us with just air pockets but not exterior air.
    {
        let mut changed = true;

        while changed {
            let mut remaining = HashSet::new();

            remaining.extend(possible_air_gap.iter().filter(|coord| {
                // it needs to be surrounded by rock or another air pocket. Thus we gradually remove
                // from the outside in bits of air that are outside the structure until none are left.
                let result = adjacents(coord)
                    .iter()
                    .filter(|adj| pos(adj).is_some())
                    .fold(0, |acc, adj| {
                        if states[pos(adj).unwrap()] || possible_air_gap.contains(adj) {
                            acc + 1
                        } else {
                            acc
                        }
                    });

                result == 6
            }));

            changed = possible_air_gap.len() != remaining.len();
            possible_air_gap = remaining;
        }
    }

    println!("Reduced to {} air gaps", possible_air_gap.len());

    for pocket in &possible_air_gap {
        faces -= adjacents(pocket)
            .iter()
            .filter_map(|adj| pos(adj))
            .filter(|&p| states[p])
            .count() as u32;
    }

    Some(faces)
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
        assert_eq!(part_two(&input), Some(58));
    }
}

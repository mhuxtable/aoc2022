use std::fmt::Display;

#[derive(Debug)]
struct Point {
    x: isize,
    y: isize,
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x={}, y={}", self.x, self.y)
    }
}

#[derive(Debug)]
struct Detection {
    sensor: Point,
    beacon: Point,
}

trait FromCoord {
    fn from_coord(_: &str) -> Point;
}

impl FromCoord for Point {
    fn from_coord(input: &str) -> Point {
        let (x, y) = input.split_once(", ").unwrap();

        Point {
            x: x.strip_prefix("x=").unwrap().parse().unwrap(),
            y: y.strip_prefix("y=").unwrap().parse().unwrap(),
        }
    }
}

impl Point {
    fn manhattan(&self, other: &Point) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    pub fn tuning_frequency(&self) -> isize {
        self.x * 4_000_000 + self.y
    }
}

fn parse(input: &str) -> Vec<Detection> {
    let mut detections = vec![];

    for line in input.lines() {
        let (sensor, beacon) = line.split_once(": ").unwrap();

        let sensor = Point::from_coord(sensor.strip_prefix("Sensor at ").unwrap());
        let beacon = Point::from_coord(beacon.strip_prefix("closest beacon is at ").unwrap());

        detections.push(Detection { sensor, beacon });
    }

    detections
}

pub fn part_one(input: &str) -> Option<u32> {
    let detections = parse(input);

    const SEARCH_Y: isize = if cfg!(test) { 10 } else { 2_000_000 };

    let relevant_sensors: Vec<(&Detection, usize)> = detections
        .iter()
        .zip(detections.iter().map(|d| d.sensor.manhattan(&d.beacon)))
        .filter_map(|(detection, manhattan)| {
            if detection.sensor.y - manhattan as isize <= SEARCH_Y
                && detection.sensor.y + manhattan as isize >= SEARCH_Y
            {
                Some((detection, manhattan))
            } else {
                None
            }
        })
        .collect();

    let mut regions: Vec<(isize, isize)> = vec![];

    for (detection, distance) in relevant_sensors {
        let dy = detection.sensor.y.abs_diff(SEARCH_Y) as isize;
        let (x1, x2) = (
            detection.sensor.x - distance as isize + dy,
            detection.sensor.x + distance as isize - dy,
        );
        regions.push((x1, x2));

        println!(
            "sensor: {}; beacon: {}; d={}, dy={}, xs=({},{})",
            detection.sensor, detection.beacon, distance, dy, x1, x2,
        );
    }
    regions.sort_by_key(|r| r.0);

    let mut beacons_in_row: Vec<isize> = detections
        .iter()
        .filter_map(|d| {
            if d.beacon.y == SEARCH_Y {
                Some(d.beacon.x)
            } else {
                None
            }
        })
        .collect();
    beacons_in_row.sort();
    beacons_in_row.reverse();

    let mut monitored = 0;

    {
        let mut cur_x = isize::MIN;

        for (left, right) in regions.iter() {
            let start = if cur_x > *right {
                continue;
            } else if *left > cur_x {
                cur_x = *left;
                // regions are disjoint so start at cur_x
                cur_x
            } else {
                // regions overlap so the end was already counted
                cur_x + 1
            };

            for i in start..=*right {
                if !beacons_in_row.is_empty() && *beacons_in_row.last().unwrap() == i {
                    // beacon here
                    beacons_in_row.pop();
                    continue;
                }

                monitored += 1;
            }

            cur_x = *right;
        }
    }

    Some(monitored)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 15);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let input = advent_of_code::read_file("examples", 15);
        assert_eq!(part_one(&input), Some(26));
    }

    #[test]
    fn test_part_two() {
        let input = advent_of_code::read_file("examples", 15);
        assert_eq!(part_two(&input), None);
    }
}

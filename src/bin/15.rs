use std::fmt::Display;
#[derive(Debug)]
struct Point {
    x: i64,
    y: i64,
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

impl Point {
    fn from_coord(input: &str) -> Point {
        let (x, y) = input.split_once(", ").unwrap();

        Point {
            x: x.strip_prefix("x=").unwrap().parse().unwrap(),
            y: y.strip_prefix("y=").unwrap().parse().unwrap(),
        }
    }

    fn manhattan(&self, other: &Point) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    pub fn tuning_frequency(&self) -> u64 {
        (self.x as u64 * 4_000_000) + self.y as u64
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

fn with_distances(detections: &Vec<Detection>) -> Vec<(&Detection, u64)> {
    detections
        .iter()
        .zip(detections.iter().map(|d| d.sensor.manhattan(&d.beacon)))
        .collect()
}

fn detections_for_row<'a>(detections: &Vec<(&'a Detection, u64)>, y: i64) -> Vec<(i64, i64)> {
    let relevant_sensors: Vec<(&Detection, u64)> = detections
        .iter()
        .filter_map(|(detection, manhattan)| {
            if detection.sensor.y - *manhattan as i64 <= y
                && detection.sensor.y + *manhattan as i64 >= y
            {
                Some((*detection, *manhattan))
            } else {
                None
            }
        })
        .collect();

    let mut regions = vec![];

    for (detection, distance) in relevant_sensors {
        let dy = detection.sensor.y.abs_diff(y);
        let (x1, x2) = (
            detection.sensor.x - distance as i64 + dy as i64,
            detection.sensor.x + distance as i64 - dy as i64,
        );
        regions.push((x1, x2));

        // println!(
        //     "sensor: {}; beacon: {}; d={}, dy={}, xs=({},{})",
        //     detection.sensor, detection.beacon, distance, dy, x1, x2,
        // );
    }
    regions.sort_by_key(|r| r.0);

    regions
}

pub fn part_one(input: &str) -> Option<u32> {
    let detections = parse(input);
    let with_distances = with_distances(&detections);

    const SEARCH_Y: i64 = if cfg!(test) { 10 } else { 2_000_000 };
    let regions = detections_for_row(&with_distances, SEARCH_Y);

    let mut beacons_in_row: Vec<i64> = detections
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
        let mut cur_x = i64::MIN;

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

pub fn part_two(input: &str) -> Option<u64> {
    let detections = parse(input);
    let with_distances = with_distances(&detections);

    const SEARCH_XY: i64 = if cfg!(test) { 20 } else { 4_000_000 };
    let mut point = None;

    'rows: for row in 0..=SEARCH_XY {
        let ranges = detections_for_row(&with_distances, row);

        let mut cur = 0;

        for (low, high) in ranges {
            if low < cur && high < cur {
                continue;
            } else if low > cur {
                // Found a location that is not monitored
                println!("The point is ({},{})", cur + 1, row);
                dbg!(low, high, cur, row);

                point = Some(Point { x: cur + 1, y: row });
                break 'rows;
            } else {
                cur = high;
            }
        }
    }

    Some(point.unwrap().tuning_frequency() as u64)
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
        assert_eq!(part_two(&input), Some(56_000_011));
    }
}

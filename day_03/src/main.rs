use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::ops::Add;
use std::ops::Mul;

#[derive(Debug)]
struct Wire {
    coordinate_to_delay: HashMap<Coordinate, u64>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Coordinate {
    x: i64,
    y: i64,
}

#[derive(Debug)]
struct Delay {
    coords: Coordinate,
    delay: u64,
}

impl Add for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Self) -> Coordinate {
        Coordinate {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Mul<i64> for Coordinate {
    type Output = Coordinate;

    fn mul(self, rhs: i64) -> Coordinate {
        Coordinate {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Coordinate {
    fn distance_to_origin(&self) -> u64 {
        self.x.abs() as u64 + self.y.abs() as u64
    }
}

impl Wire {
    fn new(spec: &str) -> Wire {
        let mut current = Coordinate { x: 0, y: 0 };
        let mut steps_taken = HashMap::new();
        let mut num_steps_taken: u64 = 0;

        for (dir, stepsize) in spec.split(",").map(|s| s.split_at(1)) {
            let stepsize: i64 = stepsize
                .parse()
                .expect(format!("Invalid stepsize: {}", stepsize).as_str());
            let direction = match dir.to_uppercase().as_str() {
                "R" => Coordinate { x: 1, y: 0 },
                "L" => Coordinate { x: -1, y: 0 },
                "U" => Coordinate { x: 0, y: 1 },
                "D" => Coordinate { x: 0, y: -1 },
                _ => panic!("Invalid direction: {}", dir),
            };

            // eprintln!("Direction: {:?} / stepsize: {}", direction, stepsize);

            for num_steps_local in 1..stepsize + 1 {
                current = current + direction;
                if !steps_taken.contains_key(&current) {
                    steps_taken.insert(current, num_steps_taken + num_steps_local as u64);
                }
            }

            num_steps_taken += stepsize as u64;
        }

        Wire {
            coordinate_to_delay: steps_taken,
        }
    }

    fn coordinates(&self) -> HashSet<Coordinate> {
        let keys = self.coordinate_to_delay.keys();
        let mut key_set = HashSet::with_capacity(keys.len());

        for k in keys {
            key_set.insert(*k);
        }

        key_set
    }

    fn delay(&self, coord: &Coordinate) -> u64 {
        match self.coordinate_to_delay.get(coord) {
            None => std::u64::MAX,
            Some(delay) => *delay,
        }
    }

    fn crossings_with(&self, other: &Self) -> HashSet<Coordinate> {
        let ours = self.coordinates();
        let theirs = other.coordinates();

        let mut returned = HashSet::new();

        for coord in ours.intersection(&theirs)
        {
            returned.insert(*coord);
        }

        returned
    }
}

fn get_closest(wire_1: &Wire, wire_2: &Wire) -> Coordinate {
    let crossings = wire_1.crossings_with(&wire_2);

    let mut closest = Coordinate {
        x: std::i64::MAX,
        y: std::i64::MAX,
    };

    for crossing in crossings {
        if crossing.distance_to_origin() < closest.distance_to_origin() {
            closest = crossing.clone();
        }
    }
    closest
}

fn get_delay_smallest(wire_1: &Wire, wire_2: &Wire) -> Delay {
    let crossings = wire_1.crossings_with(&wire_2);

    let mut delay: u64 = std::u64::MAX;
    let mut coords = Coordinate { x: 0, y: 0 };

    for c in crossings {
        let local_delay = wire_1.delay(&c) + wire_2.delay(&c);
        if local_delay < delay {
            delay = local_delay;
            coords = c;
        }
    }

    Delay { coords, delay }
}

fn main() {
    let mut args = std::env::args().skip(1);

    if args.len() < 3 {
        panic!("Need three arguments!");
    }

    let mode = args.next().unwrap();
    let wire_1 = Wire::new(&args.next().unwrap());
    let wire_2 = Wire::new(&args.next().unwrap());

    match mode.as_str() {
        "closest" => {
            let closest = get_closest(&wire_1, &wire_2);
            println!(
                "Closest crossing: {} (distance: {})",
                closest,
                closest.distance_to_origin()
            );
        }

        "delay" => {
            let delay = get_delay_smallest(&wire_1, &wire_2);
            println!(
                "Smallest delay: {} (delay: {})",
                delay.coords, delay.delay
            );
        }
        _ => {
            panic!("Invalid mode specified: {}\nMust be 'closest' or 'delay'.");
        }
    }
}

use std::collections::HashSet;
use std::fmt;
use std::ops::Add;
use std::ops::Mul;


#[derive(Debug)]
struct Wire {
    coordinates: HashSet<Coordinate>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Coordinate {
    x: i64,
    y: i64,
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
        let mut steps_taken = HashSet::new();

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

            for _ in 0..stepsize {
                current = current + direction;
                steps_taken.insert(current);
            }
        }

        Wire {
            coordinates: steps_taken,
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        panic!("Need two arguments!");
    }

    let wire_1 = Wire::new(&args[1]);
    let wire_2 = Wire::new(&args[2]);

    let crossings = wire_1.coordinates.intersection(&wire_2.coordinates);

    let mut closest = Coordinate {
        x: std::i64::MAX,
        y: std::i64::MAX,
    };

    for crossing in crossings {
        if crossing.distance_to_origin() < closest.distance_to_origin() {
            closest = crossing.clone();
        }
    }

    println!("Closest crossing: {} (distance: {})", closest, closest.distance_to_origin());
}

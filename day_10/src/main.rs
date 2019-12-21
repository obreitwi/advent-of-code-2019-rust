use num_rational::Rational64;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::{Rc, Weak};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Position {
    x: i64,
    y: i64,
}

type WrappedAsteroid = Rc<Asteroid>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum CrudeAngle {
    Above(Rational64),
    Below(Rational64),
    VerticallyAbove,
    VerticallyBelow,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Asteroid {
    pos: Position,
}

#[derive(Debug)]
struct System {
    asteroids: Vec<Asteroid>,
    pos_to_ast: HashMap<Position, Asteroid>,
    width: usize,
    height: usize,
}

impl CrudeAngle {
    fn new(dx: i64, dy: i64) -> CrudeAngle {
        use CrudeAngle::*;
        match (dx, dy) {
            (0, dy) if dy >= 0 => VerticallyAbove,
            (0, dy) if dy < 0 => VerticallyBelow,
            (dx, dy) if dy >= 0 => Above(Rational64::new(dy, dx)),
            _ => Below(Rational64::new(dy, dx)),
        }
    }
}

/// VisibleSet encapsulates the set of astroids that are currently visible
struct VisibleSet {
    angle_to_ast: HashMap<CrudeAngle, Asteroid>,
    origin: Asteroid,
}

impl VisibleSet {
    fn new(origin: Asteroid) -> Self {
        Self {
            origin,
            angle_to_ast: HashMap::new(),
        }
    }

    fn check(&mut self, ast: &Asteroid) {
        if ast.pos == self.origin.pos {
            return;
        }

        let angle = self.origin.angle_to(ast);

        match self.angle_to_ast.get(&angle) {
            None => {
                self.angle_to_ast.insert(angle, *ast);
            }
            Some(other) => {
                if self.origin.sqdist_to(ast) < self.origin.sqdist_to(other) {
                    self.angle_to_ast.insert(angle, *ast);
                }
            }
        }
    }

    fn get_visible(&self) -> Vec<Asteroid> {
        self.angle_to_ast.values().map(|a| *a).collect()
    }
}

impl Asteroid {
    fn new(x: i64, y: i64) -> Asteroid {
        Asteroid {
            pos: Position { x, y },
        }
    }

    fn new_wrapped(x: i64, y: i64) -> WrappedAsteroid {
        Rc::new(Asteroid::new(x, y))
    }

    fn sqdist_to(&self, other: &Self) -> i64 {
        let dx = self.pos.x - other.pos.x;
        let dy = self.pos.y - other.pos.y;

        dx * dx + dy * dy
    }

    fn angle_to(&self, other: &Self) -> CrudeAngle {
        let dx = self.pos.x - other.pos.x;
        let dy = self.pos.y - other.pos.y;

        CrudeAngle::new(dx, dy)
    }
}

impl System {
    pub fn parse<R: Iterator<Item = String>>(iter: R) -> System {
        let mut asteroids = Vec::new();
        let mut width = 0;
        let mut height = 0;

        for (y, line) in iter.enumerate() {
            height = if y + 1 > height { y + 1 } else { height };
            for (x, c) in line.chars().enumerate() {
                width = if x + 1 > width { x + 1 } else { width };
                match c {
                    '#' => asteroids.push(Asteroid::new(x as i64, y as i64)),
                    '.' => {}
                    _ => panic!("Encountered invalid symbol!"),
                }
            }
        }

        Self::new(asteroids, width, height)
    }

    pub fn new(asteroids: Vec<Asteroid>, width: usize, height: usize) -> System {
        let mut pos_to_ast = HashMap::new();

        for a in asteroids.iter() {
            pos_to_ast.insert(a.pos, *a);
        }

        System {
            asteroids,
            pos_to_ast,
            width,
            height,
        }
    }

    pub fn get(&self, x: i64, y: i64) -> Option<Asteroid> {
        match self.pos_to_ast.get(&Position { x, y }) {
            None => None,
            Some(ast) => Some(*ast),
        }
    }

    pub fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!(
                    "{}",
                    match self.get(x as i64, y as i64) {
                        None => '.',
                        Some(_) => '#',
                    }
                );
            }
            println!();
        }
    }

    fn visible_from(&self, root: Asteroid) -> Vec<Asteroid> {
        let mut checker = VisibleSet::new(root);

        for ast in self.asteroids.iter() {
            checker.check(ast);
        }

        checker.get_visible()
    }

    fn visible_from_all(&self) -> HashMap<Asteroid, usize> {
        self.asteroids
            .iter()
            .map(|a| (*a, self.visible_from(*a).len()))
            .collect()
    }
}

fn max_key<K, V: PartialOrd, I: Iterator<Item = (K, V)>>(iter: &mut I) -> (K, V) {
    let (mut arg, mut max) = iter.next().expect("Iterator cannot be empty!");

    for (k, v) in iter {
        if v > max {
            arg = k;
            max = v;
        }
    }
    (arg, max)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() < 1 {
        panic!("Need input filename!");
    }

    let file = File::open(&args[0]).unwrap();
    let reader = BufReader::new(&file);

    let system = System::parse(reader.lines().map(|l| l.unwrap()));
    // eprintln!("{:?}", system);
    system.print();

    let visible_from_all = system.visible_from_all();

    let (location, num_visible) = max_key(&mut visible_from_all.iter());
    println!();
    println!("{}", str::repeat("=", system.width));
    println!();
    println!(
        "Best location: {:?} #Visible: {}",
        location.pos, num_visible+1 // count yourself
    );
    println!();

    let mut visible = system.visible_from(*location);

    visible.push(*location);

    // eprintln!("Visible from {:?}: {:?}", location.pos, visible);
    let subset = System::new(visible, system.width, system.height);
    subset.print();
}

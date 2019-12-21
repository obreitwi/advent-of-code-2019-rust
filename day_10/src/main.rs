use num_rational::Rational64;
use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::f64;
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
    Right(Rational64),
    Left(Rational64),
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
            (dx, dy) if dx > 0 => Right(Rational64::new(dy, dx)),
            _ => Left(Rational64::new(dy, dx)),
        }
    }

    /// Compute the angle to the 12 o'clock position (clockwise) in range [0, 2*pi)
    fn to_rad_12_oclock(&self) -> f64 {
        use CrudeAngle::*;
        match self {
            VerticallyAbove => 0.0,
            VerticallyBelow => f64::consts::PI,
            Right(r) => f64::consts::FRAC_PI_2 - ratio_to_f64(r).atan(),
            Left(r) => f64::consts::FRAC_PI_2 * 3.0 - ratio_to_f64(r).atan(),
        }
    }
}

impl Ord for CrudeAngle {
    /// The order is from small to large:
    /// 1. VerticallyAbove (0 degrees)
    /// 2. Right(compare angles)
    /// 3. VerticallyAbove (180 degrees)
    /// 4. Left(compare angles)
    ///
    /// Since atan is monotonic we do not need to call it.
    fn cmp(&self, other: &Self) -> Ordering {
        use CrudeAngle::*;
        use Ordering::*;
        if self == other {
            Equal
        } else {
            match (self, other) {
                (Right(s), Right(o)) => (-s).cmp(&(-o)),
                (Left(s), Left(o)) => (-s).cmp(&(-o)),
                // equal case is handled above
                (VerticallyAbove, _) => Less,
                (_, VerticallyAbove) => Greater,
                // all cases with values smaller than Right(_) are handled
                (Right(_), _) => Less,
                (_, Right(_)) => Greater,
                // all cases with values smaller than VerticallyBelow(_) are handled
                (VerticallyBelow, _) => Less,
                (_, VerticallyBelow) => Greater,
            }
        }
    }
}

impl PartialOrd for CrudeAngle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn ratio_to_f64(r: &Rational64) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
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

    fn get_visible(&self) -> impl Iterator<Item = &Asteroid> {
        self.angle_to_ast.values()
    }

    fn get_angle_visible(&self) -> impl Iterator<Item = (&CrudeAngle, &Asteroid)> {
        self.angle_to_ast.iter()
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
        let dx = other.pos.x - self.pos.x;
        let dy = other.pos.y - self.pos.y;

        dx * dx + dy * dy
    }

    fn angle_to(&self, other: &Self) -> CrudeAngle {
        let dx = other.pos.x - self.pos.x;
        let dy = self.pos.y - other.pos.y; // NOTE: y gets positive when going to the bottom!

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

        Self::new(&asteroids, width, height)
    }

    pub fn new(starting: &[Asteroid], width: usize, height: usize) -> System {
        let mut pos_to_ast = HashMap::new();
        let mut asteroids = Vec::new();
        asteroids.resize(starting.len(), Asteroid::new(-1, -1));

        asteroids.copy_from_slice(starting);

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

    pub fn get(&self, x: i64, y: i64) -> Option<&Asteroid> {
        match self.pos_to_ast.get(&Position { x, y }) {
            None => None,
            Some(ast) => Some(ast),
        }
    }

    pub fn print(&self) {
        self.print_(None);
    }

    pub fn print_with_root(self, root: &Asteroid) {
        self.print_(Some(root));
    }

    fn print_(&self, root: Option<&Asteroid>) {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(Asteroid {
                    pos: Position { x: rx, y: ry },
                }) = root
                {
                    if x as i64 == *rx && y as i64 == *ry {
                        print!("X");
                        continue;
                    }
                }
                print!(
                    "{}",
                    match self.get(x as i64, y as i64) {
                        None => String::from("."),
                        Some(_) => {
                            if self.asteroids.len() <= 10 {
                                let mut num: Option<String> = None;
                                for (i, ast) in self.asteroids.iter().enumerate() {
                                    let (x, y) = (x as i64, y as i64);
                                    let pos = Position { x, y };
                                    if ast.pos == pos {
                                        num = Some(i.to_string());
                                    }
                                }
                                num.unwrap()
                            } else {
                                String::from("#")
                            }
                        }
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
        checker.get_visible().map(|a| *a).collect()
    }

    fn visible_from_sorted(&self, root: Asteroid) -> Vec<Asteroid> {
        let mut checker = VisibleSet::new(root);

        for ast in self.asteroids.iter() {
            checker.check(ast);
        }
        let mut with_angles: Vec<_> = checker.get_angle_visible().map(|(c, a)| (*c, *a)).collect();

        with_angles.sort_by_key(|t| t.0);

        with_angles.iter().map(|t| t.1).collect()
    }

    fn visible_from_all(&self) -> HashMap<Asteroid, usize> {
        self.asteroids
            .iter()
            .map(|a| (*a, self.visible_from(*a).len()))
            .collect()
    }

    fn remove_asteroids(&mut self, to_remove: &[Asteroid]) {
        for ast in to_remove.iter() {
            if self.pos_to_ast.contains_key(&ast.pos) && self.asteroids.contains(ast) {
                self.pos_to_ast.remove(&ast.pos);
                // very inefficient -> do not care
                for (i, contained) in self.asteroids.iter().enumerate() {
                    if contained == ast {
                        self.asteroids.remove(i);
                        break;
                    }
                }
            } else {
                panic!("Did not find asteroid to remove!");
            }
        }
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

fn banner(system: &System) {
    println!();
    println!("{}", str::repeat("=", system.width));
    println!();
}

fn clear_screen()
{
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() < 1 {
        panic!("Need input filename!");
    }

    let file = File::open(&args[0]).unwrap();
    let reader = BufReader::new(&file);

    let mut system = System::parse(reader.lines().map(|l| l.unwrap()));
    // eprintln!("{:?}", system);
    system.print();

    let visible_from_all = system.visible_from_all();

    let (location, num_visible) = max_key(&mut visible_from_all.iter());

    banner(&system);

    println!(
        "Best location: {:?} #Visible: {}",
        location.pos,
        num_visible
    );
    println!();

    let mut visible = system.visible_from(*location);
    // eprintln!("Visible from {:?}: {:?}", location.pos, visible);
    let subset = System::new(&visible, system.width, system.height);
    subset.print_with_root(location);

    banner(&system);

    let to_remove = 200;
    let mut num_removed = 0;

    loop {
        visible = system.visible_from_sorted(*location);

        // let visible = &visible[..10];

        let mut last_rad = 0.0;

        clear_screen();

        for (i, ast) in visible.iter().enumerate() {
            let current_rad = location.angle_to(ast).to_rad_12_oclock();
            assert!(current_rad >= last_rad);
            last_rad = current_rad;
            println!(
                "Vaporizing #{}: {:?} (angle: {})",
                num_removed + i + 1,
                ast,
                current_rad
            );
        }
        System::new(&visible, system.width, system.height).print_with_root(location);
        std::thread::sleep(std::time::Duration::from_millis(100));

        if visible.len() + num_removed >= to_remove {
            break;
        }

        system.remove_asteroids(&visible);
        num_removed += visible.len();
    }

    banner(&system);

    let target = visible[to_remove - num_removed - 1];

    println!(
        "The {}th asteroid to vaporize is {:?} (answer: {})..",
        to_remove,
        target,
        target.pos.x * 100 + target.pos.y
    );
}

use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug)]
pub struct Grid<T> {
    grid: HashMap<Position, T>,
}

#[derive(Debug)]
struct Dimensions {
    x_min: i64,
    x_max: i64,
    y_min: i64,
    y_max: i64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Turn {
    Left,
    Right,
}

impl Dimensions {
    pub fn width(&self) -> i64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> i64 {
        self.y_max - self.y_min
    }
}

impl Turn {
    pub fn all() -> &'static [Self] {
        use Turn::*;
        static VARIANTS: &'static [Turn] = &[Left, Right];
        VARIANTS
    }
}

impl Into<String> for Turn {
    fn into(self) -> String {
        use Turn::*;
        match self {
            Right => String::from("R"),
            Left => String::from("L"),
        }
    }
}

impl Position {
    pub fn step(&self, dir: &Direction) -> Self {
        use Direction::*;
        let Position { x, y } = self;
        let (dx, dy) = match *dir {
            North => (0, -1),
            South => (0, 1),
            West => (-1, 0),
            East => (1, 0),
        };
        Position {
            x: x + dx,
            y: y + dy,
        }
    }
}

impl Direction {
    pub fn all() -> &'static [Direction] {
        use Direction::*;
        static VARIANTS: &'static [Direction] = &[North, South, West, East];
        VARIANTS
    }

    pub fn invert(&self) -> Self {
        use Direction::*;
        match self {
            North => South,
            South => North,
            West => East,
            East => West,
        }
    }

    pub fn to_turn(&self, other: &Self) -> Turn {
        use Direction::*;
        use Turn::*;
        match (self, other) {
            (North, West) => Right,
            (North, East) => Left,
            (South, East) => Right,
            (South, West) => Left,
            (West, North) => Right,
            (West, South) => Left,
            (East, South) => Right,
            (East, North) => Left,
            (_, _) => panic!("Unsupported turn!"),
        }
    }

    pub fn turn(&self, turn: &Turn) -> Direction {
        use Direction::*;
        use Turn::*;
        match (self, turn) {
            (North, Right) => East,
            (North, Left) => West,
            (South, Right) => West,
            (South, Left) => East,
            (West, Right) => North,
            (West, Left) => South,
            (East, Right) => South,
            (East, Left) => North,
        }
    }
}



impl<T> Grid<T>
where
    T: Default,
    T: fmt::Display,
    T: Copy,
{
    pub fn new() -> Grid<T> {
        Grid {
            grid: HashMap::new(),
        }
    }

    pub fn get(&self, pos: &Position) -> T {
        match self.grid.get(pos) {
            None => Default::default(),
            Some(elem) => *elem,
        }
    }

    pub fn get_existing(&self, pos: &Position) -> Option<T> {
        self.grid.get(pos).map(|e| *e)
    }

    pub fn add(&mut self, pos: Position, tile: T) {
        self.grid.insert(pos, tile);
    }

    fn get_dims(&self) -> Dimensions {
        let mut x_min = std::i64::MAX;
        let mut y_min = std::i64::MAX;
        let mut x_max = -std::i64::MAX;
        let mut y_max = -std::i64::MAX;

        for Position { x, y } in self.grid.keys() {
            x_min = min(x_min, *x);
            y_min = min(y_min, *y);
            x_max = max(x_max, *x);
            y_max = max(y_max, *y);
        }

        Dimensions {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn print<F, I>(&self, f_override: F)
    where
        F: Fn(&Position) -> Option<I>,
        I: fmt::Display,
    {
        let dims = self.get_dims();

        for y in dims.y_min..dims.y_max + 1 {
            for x in dims.x_min..dims.x_max + 1 {
                let pos = Position { x, y };
                let to_print = match f_override(&pos) {
                    None => self.get(&pos).to_string(),
                    Some(special) => special.to_string(),
                };
                print!("{}", to_print);
            }
            println!();
        }
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Position, T> {
        self.grid.iter()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<Position, T> {
        self.grid.values()
    }
}

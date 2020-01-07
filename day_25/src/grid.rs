use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt;
use std::io::prelude::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone)]
pub struct Grid<T> {
    grid: HashMap<Position, T>,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub x_min: i64,
    pub x_max: i64,
    pub y_min: i64,
    pub y_max: i64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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

impl From<(i64, i64)> for Position {
    fn from((x, y): (i64, i64)) -> Self {
        Position { x, y }
    }
}

impl From<(&i64, &i64)> for Position {
    fn from((x, y): (&i64, &i64)) -> Self {
        Position { x: *x, y: *y }
    }
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

    /// manhatten distance
    pub fn mh(&self, other: &Self) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
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
    T: Clone,
{
    pub fn new() -> Grid<T> {
        Grid {
            grid: HashMap::new(),
        }
    }

    pub fn from_dims(dims: &Dimensions, elem: T) -> Grid<T> {
        let Dimensions {
            x_min,
            x_max,
            y_min,
            y_max,
        } = *dims;

        let mut grid = HashMap::new();

        for y in y_min..y_max + 1 {
            for x in x_min..x_max + 1 {
                grid.insert(Position { x, y }, elem.clone());
            }
        }

        Grid { grid }
    }

    /// Expand the grid (and the position if given)
    pub fn expand(&mut self, at: Option<&mut Position>) {
        let mut expanded = HashMap::new();
        let keys: Vec<Position> = self.grid.keys().cloned().collect();
        for pos in keys.iter() {
            let v = self.grid.remove(&pos).unwrap();
            expanded.insert(
                Position {
                    x: 2 * pos.x,
                    y: 2 * pos.y,
                },
                v,
            );
        }
        self.grid = expanded;

        if let Some(at) = at
        {
            at.x *= 2;
            at.y *= 2;
        }
    }

    pub fn get(&self, pos: &Position) -> T {
        match self.grid.get(pos) {
            None => Default::default(),
            Some(elem) => elem.clone(),
        }
    }

    pub fn get_in_direction(
        &self,
        mut pos: Position,
        dir: &Direction,
        max_steps: usize,
    ) -> Option<(Position, T)> {
        let mut num_steps = 0;
        pos = pos.step(dir);
        while let None = self.grid.get(&pos) {
            pos = pos.step(dir);
            num_steps += 1;

            if num_steps >= max_steps {
                return None;
            }
        }
        Some((pos, self.grid.get(&pos).cloned().unwrap()))
    }

    pub fn get_existing(&self, pos: &Position) -> Option<T> {
        self.grid.get(pos).map(|e| e.clone())
    }

    pub fn add(&mut self, pos: Position, tile: T) {
        self.grid.insert(pos, tile);
    }

    pub fn get_dims(&self) -> Dimensions {
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

    pub fn print(&self) {
        self.print_custom(|_: &Position| -> Option<T> { None });
    }

    pub fn print_custom<F, I>(&self, f_override: F)
    where
        F: Fn(&Position) -> Option<I>,
        I: fmt::Display,
    {
        print!("{}", self.write_custom(f_override));
    }

    pub fn write(&self) -> String {
        self.write_custom(|_: &Position| -> Option<T> { None })
    }

    pub fn write_custom<F, I>(&self, f_override: F) -> String
    where
        F: Fn(&Position) -> Option<I>,
        I: fmt::Display,
    {
        let mut output: Vec<u8> = Vec::new();
        let dims = self.get_dims();

        for y in dims.y_min..dims.y_max + 1 {
            for x in dims.x_min..dims.x_max + 1 {
                let pos = Position { x, y };
                let to_print = match f_override(&pos) {
                    None => self.get(&pos).to_string(),
                    Some(special) => special.to_string(),
                };
                write!(output, "{}", to_print);
            }
            writeln!(output);
        }

        String::from_utf8(output).expect("Error formatting grid!")
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Position, T> {
        self.grid.iter()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<Position, T> {
        self.grid.values()
    }
}

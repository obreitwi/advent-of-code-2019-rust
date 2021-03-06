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

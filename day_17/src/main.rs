use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;

mod grid;
mod intcode;

use grid::{Direction, Grid, Position};
use intcode::{Intcode, TapeElem};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Tile(char);

#[derive(Debug)]
struct Robot {
    computer: Intcode,
    grid: Grid<Tile>,
}

impl From<TapeElem> for Tile {
    fn from(i: TapeElem) -> Self {
        Tile(i as u8 as char)
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile('?')
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Tile(c) = self;
        f.write_str(c.to_string().as_str())
    }
}

impl Robot {
    fn new(filename: &str) -> Robot {
        let mut shortest = HashMap::new();
        shortest.insert(Position { x: 0, y: 0 }, 0);
        Robot {
            computer: Intcode::load(filename),
            grid: Grid::new(),
        }
    }

    fn map(&mut self) {
        let (mut x, mut y) = (0, 0);

        while !self.computer.is_finished() {
            self.computer.execute();

            while let Some(output) = self.computer.get_output() {
                match output {
                    10 => {
                        y += 1;
                        x = 0;
                    }
                    _ => {
                        self.grid.add(Position { x, y }, Tile::from(output));
                        x += 1;
                    }
                }
            }
        }
    }

    fn print(&self) {
        let intersections = self.get_intersections();

        self.grid.print(|pos: &Position| -> Option<&str> {
            if intersections.contains(&pos) {
                Some("+")
            } else {
                None
            }
        });
    }

    fn get_intersections(&self) -> Vec<Position> {
        self.grid
            .iter()
            .filter(|(_, Tile(c))| *c == '#')
            .map(|(p, _)| p)
            .filter(|p| {
                Direction::all()
                    .iter()
                    .all(|dir| self.grid.get(&p.step(dir)) == Tile('#'))
            })
            .map(|p| *p)
            .collect()
    }

    fn computer_calibration(&self) -> i64 {
        self.get_intersections()
            .iter()
            .map(|Position { x, y }| x * y)
            .sum()
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    let mut robot = Robot::new("input.txt");
    robot.map();
    robot.print();
    println!();
    println!("Calibration: {}", robot.computer_calibration());
}

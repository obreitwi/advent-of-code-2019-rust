use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::fmt;

mod grid;
mod intcode;

use grid::{Grid, Position};
use intcode::{Intcode, TapeElem};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Stationary,
    Pulled,
}

#[derive(Debug)]
struct Tractor {
    computer: Intcode,
    grid: Grid<Tile>,
}

impl From<TapeElem> for Tile {
    fn from(i: TapeElem) -> Self {
        match i {
            0 => Tile::Stationary,
            1 => Tile::Pulled,
            _ => panic!("Invalid state returned!"),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Stationary
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Tile::Stationary => ".",
            Tile::Pulled => "#",
        })
    }
}

impl Tractor {
    fn new(filename: &str) -> Tractor {
        Tractor {
            computer: Intcode::load(filename),
            grid: Grid::new(),
        }
    }

    fn map(&mut self, size_x: usize, size_y: usize) {
        for y in 0..size_y {
            for x in 0..size_x {
                self.computer.reset();
                self.computer.supply_input(x as TapeElem);
                self.computer.supply_input(y as TapeElem);
                self.computer.execute();
                match self.computer.get_output() {
                    None => panic!("Computer broke during mapping!"),
                    Some(c) => self.grid.add(
                        Position {
                            x: x as TapeElem,
                            y: y as TapeElem,
                        },
                        Tile::from(c),
                    ),
                }
            }
        }
    }

    fn print(&self) {
        self.grid.print(|pos: &Position| -> Option<&str> { None });
    }

    fn get_num_affected(&self) -> usize {
        self.grid
            .iter()
            .filter(|(_, t)| **t == Tile::Pulled)
            .count()
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    let mut tractor = Tractor::new("input.txt");
    tractor.map(50, 50);
    tractor.print();
    println!("Number of affected points: {}", tractor.get_num_affected());
}

use std::cmp::{max, min};
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
    Santa,
}

#[derive(Debug)]
struct Tractor {
    computer: Intcode,
    grid: Grid<Tile>,
    lines_mapped: usize,
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
            Tile::Santa => "O",
        })
    }
}

impl Tractor {
    fn new(filename: &str) -> Tractor {
        Tractor {
            computer: Intcode::load(filename),
            grid: Grid::new(),
            lines_mapped: 0,
        }
    }

    fn _get_pulled_prev_line(&self, y: usize) -> Vec<i64> {
        self.grid
            .iter()
            .filter_map(|(pos, t)| {
                if pos.y == (y as i64) - 1 && *t == Tile::Pulled {
                    Some(pos.x)
                } else {
                    None
                }
            })
            .collect()
    }

    fn map(&mut self, size_x: usize, size_y: usize, thorough: bool) {
        eprintln!("\rMapping {}x{}{}\n", size_x, size_y, " ".repeat(40));
        /*
         * let start_y_at: usize = if let Some(_) = self.grid.iter().next() {
         *     self.grid.iter().map(|(pos, t)| pos.y).max().unwrap_or(0) as usize + 1
         * } else {
         *     0
         * };
         */
        // (min, max)
        let mut prev_line;
        let mut this_line = (0, size_x);
        for scan_y in self.lines_mapped..size_y {
            prev_line = this_line;
            this_line = (0, size_x);
            // eprintln!(
            // "\r[x: {}] this_line/prev_line: {:?}/{:?}",
            // scan_y, this_line, prev_line
            // );
            eprint!("\rLine {} (Mapped {} tiles)", scan_y, self.grid.len());
            let mut beam_found = false;
            let mut scan_x = prev_line.0;
            while scan_x < size_x {
                self.computer.reset();
                self.computer.supply_input(scan_x as TapeElem);
                self.computer.supply_input(scan_y as TapeElem);
                self.computer.execute();
                match self.computer.get_output() {
                    None => panic!("Computer broke during mapping!"),
                    Some(c) => {
                        let tile = Tile::from(c);

                        self.grid.add(
                            Position {
                                x: scan_x as TapeElem,
                                y: scan_y as TapeElem,
                            },
                            tile,
                        );

                        match tile {
                            Tile::Pulled if !beam_found => {
                                beam_found = true;
                                this_line.0 = scan_x;
                                if prev_line.1 < size_x && prev_line.1 > scan_x {
                                    if thorough {
                                        for x_pulled in scan_x..(prev_line.1 + 1) {
                                            self.grid.add(
                                                Position {
                                                    x: x_pulled as TapeElem,
                                                    y: scan_y as TapeElem,
                                                },
                                                tile,
                                            );
                                        }
                                    }
                                    scan_x = prev_line.1;
                                } else {
                                    scan_x += 1;
                                }
                            }
                            Tile::Stationary if beam_found => {
                                this_line.1 = scan_x - 1;
                                break;
                            }
                            _ => {
                                scan_x += 1;
                            }
                        }
                    }
                }
            }
        }
        self.lines_mapped = size_y;
    }

    fn print(&self) {
        self.grid.print(|_: &Position| -> Option<&str> { None });
    }

    fn get_num_affected(&self) -> usize {
        let mut pulled_y_to_x_min: HashMap<i64, i64> = HashMap::new();
        let mut pulled_y_to_x_max: HashMap<i64, i64> = HashMap::new();
        for (Position { x, y }, t) in self.grid.iter() {
            if *t == Tile::Pulled {
                let cur_x_min = pulled_y_to_x_min.get(y).cloned().unwrap_or(std::i64::MAX);
                let cur_x_max = pulled_y_to_x_max.get(y).cloned().unwrap_or(-std::i64::MAX);

                pulled_y_to_x_min.insert(*y, min(*x, cur_x_min));
                pulled_y_to_x_max.insert(*y, max(*x, cur_x_max));
            }
        }
        let mut total = 0;
        for (y, x_min) in pulled_y_to_x_min.iter() {
            let x_max = pulled_y_to_x_max.get(y).unwrap();

            total += *x_max - *x_min + 1;
        }
        total as usize
    }

    fn find_santa_ship(&self, width: i64, height: i64) -> Option<Position> {
        let mut hugh_votes: HashMap<Position, usize> = HashMap::new();
        for Position { x, y } in
            self.grid
                .iter()
                .filter_map(|(pos, t)| if *t == Tile::Pulled { Some(pos) } else { None })
        {
            let top = Position {
                x: x.clone(),
                y: y - height + 1,  // last tile still inside the ship
            };
            let left = Position {
                x: x - width + 1,  // last tile still inside the ship
                y: y.clone(),
            };

            let current = hugh_votes.get(&top).cloned().unwrap_or(0);
            hugh_votes.insert(top, current + 1);

            let current = hugh_votes.get(&left).cloned().unwrap_or(0);
            hugh_votes.insert(left, current + 1);
        }
        for item in hugh_votes.iter().filter(|(_, votes)| **votes == 2) {
            eprintln!("{:?}", item);
        }

        hugh_votes
            .iter()
            .filter(|(_, votes)| **votes == 2)
            .min_by_key(|(pos, _)| pos.x.abs() + pos.y.abs())
            .map(|(pos, _)| pos)
            .cloned()
    }

    fn insert_santas_ship_at(&mut self, pos: &Position, width: i64, height: i64) {
        for y in 0..height {
            for x in 0..width {
                self.grid.add(
                    Position {
                        x: pos.x + x,
                        y: pos.y + y,
                    },
                    Tile::Santa,
                );
            }
        }
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    {
        // part A
        let mut tractor = Tractor::new("input.txt");
        tractor.map(50, 50, true);
        print!("\r");
        println!("{:?}", tractor.grid.get_dims());
        tractor.print();
        println!(
            "\rNumber of affected points: {}",
            tractor.get_num_affected()
        );
    }
    if true {
        let mut tractor = Tractor::new("input.txt");
        let mut mapsize = 2;
        tractor.map(mapsize, mapsize, false);
        tractor.print();

        let ship_size = 100;

        while let None = tractor.find_santa_ship(ship_size, ship_size) {
            mapsize *= 2;
            tractor.map(mapsize, mapsize, false);
            if mapsize < 512 {
                tractor.print();
            }
        }
        let pos_santa = tractor.find_santa_ship(ship_size, ship_size).unwrap();
        // tractor.insert_santas_ship_at(&pos_santa, ship_size, ship_size);
        // tractor.print();
        println!(
            "Topleft corner of ship: {:?} (answer: {})",
            pos_santa,
            pos_santa.x * 10000 + pos_santa.y
        );
    }
}

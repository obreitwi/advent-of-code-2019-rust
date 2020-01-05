use std::cmp::min;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::fs::read_to_string;

mod grid;

use grid::{Dimensions, Direction, Grid, Position};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Empty,
    Bugs,
}

#[derive(Debug)]
struct GameOfEris {
    grid: Grid<Tile>,
    dims: Dimensions,
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        use Tile::*;
        match c {
            '.' => Empty,
            '#' => Bugs,
            _ => panic!("Invalid input for tile."),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tile::*;
        let to_write = match self {
            Empty => '.',
            Bugs => '#',
        };
        f.write_str(&to_write.to_string())
    }
}

impl GameOfEris {
    pub fn new(filename: &str) -> GameOfEris {
        let raw = read_to_string(filename).expect("Could not read input file.");
        let mut grid = Grid::new();

        for (y, line) in raw.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let tile = Tile::from(c);
                let pos = Position {
                    x: x as i64,
                    y: y as i64,
                };
                grid.add(pos, tile);
            }
        }
        grid.print();
        let dims = grid.get_dims();
        assert_eq!(dims.x_min, 0);
        assert_eq!(dims.y_min, 0);
        GameOfEris { grid, dims }
    }

    pub fn step(&mut self) {
        let mut next = self.grid.clone();

        for (pos, prev) in self.grid.iter() {
            let mut num_bugs_neighbouring = 0;
            for dir in Direction::all() {
                num_bugs_neighbouring += match self.grid.get(&pos.step(dir)) {
                    Tile::Empty => 0,
                    Tile::Bugs => 1,
                };
            }

            next.add(
                *pos,
                match (prev, num_bugs_neighbouring) {
                    (Tile::Bugs, 1) => Tile::Bugs,
                    (Tile::Bugs, _) => Tile::Empty,
                    (Tile::Empty, 1) | (Tile::Empty, 2) => Tile::Bugs,
                    _ => *prev,
                },
            );
        }
        self.grid = next;
    }

    pub fn biodiversity(&self) -> u64 {
        let mut res = 0;
        let len_y = self.dims.y_max - self.dims.y_min + 1;
        for (pos, t) in self.grid.iter() {
            if let Tile::Bugs = t {
                res += 2u64.pow((pos.x + len_y * pos.y) as u32);
            }
        }
        res
    }

    pub fn write(&self) -> String {
        self.grid.write()
    }

    pub fn print(&self) {
        self.grid.print();
    }

    pub fn find_first_repeating(&mut self) {
        let mut seen: HashSet<String> = HashSet::new();
        seen.insert(self.write());

        loop {
            self.step();
            let state = self.write();

            println!("\n{}", state);

            std::thread::sleep(std::time::Duration::from_millis(25));

            if seen.contains(&state) {
                break;
            } else {
                seen.insert(state);
            }
        }
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn backspace() {
    print!("\x08");
}

fn main() {
    let mut eris = GameOfEris::new(
        &std::env::args()
            .skip(1)
            .next()
            .expect("Filename not provided."),
    );
    eris.find_first_repeating();
    println!();
    eris.print();
    println!();
    println!("{}", eris.biodiversity());
    println!();

    eprintln!("{:?}", eris.dims);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biodiveristy()
    {
        let eris = GameOfEris::new("example_01.txt");
        assert_eq!(eris.biodiversity(), 2129920);
    }
}

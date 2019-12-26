use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;

mod grid;
mod intcode;

use grid::{Grid, Position};
use intcode::{Intcode, TapeElem};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Unknown,
    Empty,
    Wall,
    Oxygen,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug)]
struct Droid {
    computer: Intcode,
    grid: Grid<Tile>,
    shortest: HashMap<Position, usize>,
    pos: Position,
}

impl Position {
    fn step(&self, dir: &Direction) -> Self {
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
    fn all() -> &'static [Direction] {
        use Direction::*;
        static VARIANTS: &'static [Direction] = &[North, South, West, East];
        VARIANTS
    }

    fn invert(&self) -> Self {
        use Direction::*;
        match self {
            North => South,
            South => North,
            West => East,
            East => West,
        }
    }
}

impl From<TapeElem> for Tile {
    fn from(i: i64) -> Self {
        use Tile::*;
        match i {
            0 => Wall,
            1 => Empty,
            2 => Oxygen,
            _ => panic!("Invalid conversion from int to Tile"),
        }
    }
}

impl Into<TapeElem> for Direction {
    fn into(self) -> TapeElem {
        use Direction::*;
        match self {
            North => 1,
            South => 2,
            West => 3,
            East => 4,
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Unknown
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tile::*;
        f.write_str(match self {
            Unknown => ".",
            Empty => " ",
            Wall => "#",
            Oxygen => "O",
        })
    }
}

impl Droid {
    fn new(filename: &str) -> Droid {
        let mut shortest = HashMap::new();
        shortest.insert(Position { x: 0, y: 0 }, 0);
        Droid {
            computer: Intcode::load(filename),
            grid: Grid::new(),
            pos: Position { x: 0, y: 0 },
            shortest,
        }
    }

    /// step in direction and return Tile
    fn step(&mut self, dir: Direction) -> Tile {
        use Tile::*;
        self.computer.supply_input(dir.into());
        self.computer.execute();
        let tile = Tile::from(
            self.computer
                .get_output()
                .expect("Computer provided no output!"),
        );

        let new_pos = self.pos.step(&dir);

        self.discover(&new_pos, &tile);

        self.pos = match tile {
            Wall => self.pos,
            _ => new_pos,
        };

        tile
    }

    fn get_shortest(&self, pos: &Position) -> Option<usize> {
        self.shortest.get(pos).map(|p| *p)
    }

    fn discover(&mut self, pos: &Position, tile: &Tile) {
        use Tile::*;
        match tile {
            Wall => { /* no shortest paths to compute */ }
            Empty | Oxygen => {
                self.check_update_shortest(pos);
            }
            _ => {
                panic!("Trying to discover unknown tile!");
            }
        }

        self.grid.add(pos.clone(), tile.clone())
    }

    /// Update shortest path for a new tile
    fn check_update_shortest(&mut self, pos: &Position) {
        let mut shortest = std::usize::MAX - 1;
        for dir in Direction::all() {
            let neighbor = pos.step(dir);

            shortest = min(
                shortest,
                self.get_shortest(&neighbor).unwrap_or(std::usize::MAX - 1) + 1,
            )
        }

        self.update_shortest(pos, shortest);
    }

    fn update_shortest(&mut self, pos: &Position, shortest: usize) {
        let old = self
            .shortest
            .insert(*pos, shortest)
            .unwrap_or(std::usize::MAX);
        if old != shortest {
            self.update_shortest_neighbors(pos, shortest)
        }
    }

    /// Update shortest path for neighbors of a newly inserted tile
    fn update_shortest_neighbors(&mut self, pos: &Position, shortest: usize) {
        for dir in Direction::all() {
            let neighbor = pos.step(dir);

            match self.get_shortest(&neighbor) {
                Some(neighbor_shortest) if shortest + 1 < neighbor_shortest => {
                    self.shortest.insert(neighbor, shortest + 1);
                    self.update_shortest(&neighbor, shortest + 1)
                }
                _ => {}
            }
        }
    }

    fn explore(&mut self) {
        let mut path: Vec<Direction> = Vec::new();

        loop {
            match Direction::all()
                .iter()
                .filter(|d| self.grid.get_existing(&self.pos.step(d)).is_none())
                .next()
            {
                Some(d) => match self.step(*d) {
                    Tile::Wall => { /* we did not move */ }
                    _ => {
                        // record where we went
                        path.push(*d);
                    }
                },
                None if self.pos == Position { x: 0, y: 0 } => {
                    // we are back at the root with no paths left to explore
                    break;
                }
                None => {
                    let backtrack = path.pop().expect("Cannot backtrack!").invert();
                    self.step(backtrack);
                }
            }

            /*
             * clear_screen();
             * self.print();
             * std::thread::sleep(std::time::Duration::from_millis(25));
             */
        }
    }

    fn print(&self) {
        self.grid.print(
            |pos: &Position| -> Option<&str> {
                if *pos == self.pos {
                    Some("D")
                } else {
                    None
                }
            },
        );
    }

    fn oxygen(&self) -> Position {
        let (oxygen, _) = self
            .grid
            .iter()
            .filter(|(pos, t)| **t == Tile::Oxygen)
            .next()
            .expect("Oxygen not found");

        *oxygen
    }

    fn shortest_path_to_oxygen(&self) -> usize {
        *self.shortest.get(&self.oxygen()).unwrap()
    }

    fn calc_oxygen_spread(&self) -> usize {
        let oxygen = self.oxygen();
        let mut front: VecDeque<(Position, usize)> = VecDeque::new();
        let mut visited: HashSet<Position> = HashSet::new();
        let num_empty_spaces = self.grid.values().filter(|t| **t == Tile::Empty).count();
        let mut max_time = 0;

        front.push_back((oxygen, 0));
        visited.insert(oxygen);

        while front.len() > 0 {
            let (pos, time) = front.pop_front().unwrap();

            for dir in Direction::all() {
                let neighbor = pos.step(dir);
                match self.grid.get(&neighbor) {
                    Tile::Empty if !visited.contains(&neighbor) => {
                        let advance_time = time + 1;
                        front.push_back((neighbor, advance_time));
                        visited.insert(neighbor);
                        max_time = max(max_time, advance_time);
                    }
                    _ => {}
                }
                /*
                 * clear_screen();
                 * self.grid.print(|pos| {
                 *     if visited.contains(&pos)
                 *     {
                 *         Some("O")
                 *     }
                 *     else
                 *     {
                 *         None
                 *     }
                 * });
                 * std::thread::sleep(std::time::Duration::from_millis(25));
                 */
            }
        }
        assert_eq!(
            visited.len() - 1,
            num_empty_spaces,
            "There are empty spaces without oxygen!"
        );
        max_time
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    let mut robot = Droid::new("input.txt");
    robot.explore();
    robot.print();
    println!(
        "Shortest path to oxygen: {}",
        robot.shortest_path_to_oxygen()
    );
    println!(
        "Time for oxygen to fully fill space: {}",
        robot.calc_oxygen_spread()
    );
}

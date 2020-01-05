use std::cmp::min;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::fs::read_to_string;
use std::io::Write;

mod grid;

use grid::{Dimensions, Direction, Grid, Position};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Empty,
    Bugs,
    Folded,
    OffGrid,
}

#[derive(Debug, Clone)]
struct GameOfEris {
    lvl_to_grid: HashMap<i64, Grid<Tile>>,
    dims: Dimensions,
    pos_folded: Option<Position>,
    empty: Grid<Tile>,
    pos_edges: HashMap<Direction, Vec<Position>>,
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        use Tile::*;
        match c {
            '.' => Empty,
            '#' => Bugs,
            '?' => Folded,
            ' ' => OffGrid,
            _ => panic!("Invalid input for tile."),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::OffGrid
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tile::*;
        let to_write = match self {
            Empty => '.',
            Bugs => '#',
            Folded => '?',
            OffGrid => ' ',
        };
        f.write_str(&to_write.to_string())
    }
}

impl GameOfEris {
    pub fn new(filename: &str) -> GameOfEris {
        let raw = read_to_string(filename).expect("Could not read input file.");
        let mut lvl_to_grid = HashMap::new();
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
        let folded = grid.iter().filter_map(
            |(pos, t)| {
                if *t == Tile::Folded {
                    Some(*pos)
                } else {
                    None
                }
            },
        );
        let pos_folded = if folded.clone().count() > 1 {
            panic!("Can only have one folded instance on grid!");
        } else if folded.clone().count() == 1 {
            Some(folded.clone().next().unwrap())
        } else {
            None
        };

        let mut empty = Grid::from_dims(&dims, Tile::Empty);
        if let Some(pos_folded) = pos_folded
        {
            empty.add(pos_folded, Tile::Folded);
        }

        lvl_to_grid.insert(0, grid);

        let mut pos_edges = HashMap::new();
        for dir in Direction::all() {
            pos_edges.insert(*dir, GameOfEris::compute_edge(&dims, dir));
        }

        GameOfEris {
            lvl_to_grid,
            dims,
            pos_folded,
            empty,
            pos_edges,
        }
    }

    fn height(&self) -> usize {
        (self.dims.y_max - self.dims.y_min + 1) as usize
    }

    fn width(&self) -> usize {
        (self.dims.x_max - self.dims.x_min + 1) as usize
    }

    pub fn steps(&mut self, n: usize) {
        for _ in 0..n {
            self.step();
        }
    }

    /// Return unordered list of levels
    fn levels(&self) -> Vec<i64> {
        self.lvl_to_grid.keys().cloned().collect()
    }

    fn get_bug_count_edges(&self, lvl: i64) -> HashMap<Direction, usize> {
        let grid = self.lvl_to_grid.get(&lvl).unwrap_or(&self.empty);

        let mut retval = HashMap::new();

        for dir in Direction::all() {
            let mut count = 0;
            for pos in self.get_edge(dir).iter() {
                match grid.get(&pos) {
                    Tile::Bugs => {
                        count += 1;
                    }
                    _ => {}
                }
            }
            retval.insert(*dir, count);
        }

        retval
    }

    fn update_grid(&self, lvl: i64) -> Grid<Tile> {
        let prev_grid = self.lvl_to_grid.get(&lvl).unwrap_or(&self.empty);
        let mut next_grid = prev_grid.clone();

        let bugcount_above = self.get_bug_count_edges(lvl + 1);
        let grid_below = self.lvl_to_grid.get(&(lvl - 1)).unwrap_or(&self.empty);

        for (pos, prev) in prev_grid.iter() {
            let mut num_bugs_neighbors = 0;

            for dir in Direction::all() {
                match prev_grid.get(&pos.step(dir)) {
                    Tile::Bugs => {
                        num_bugs_neighbors += 1;
                    }
                    Tile::Folded => {
                        // not that the edge is on the opposite end of our step direction
                        num_bugs_neighbors += bugcount_above.get(&dir.invert()).unwrap();
                    }
                    Tile::OffGrid => {
                        num_bugs_neighbors += match self.pos_folded {
                            // if not folded we treat OffGrid as empty
                            None => 0,
                            Some(pos_folded) => {
                                // replace with position in level below
                                match grid_below.get(&pos_folded.step(dir)) {
                                    Tile::Bugs => 1,
                                    _ => 0,
                                }
                            }
                        }
                    }
                    Tile::Empty => {}
                }
            }

            next_grid.add(
                *pos,
                match (prev, num_bugs_neighbors) {
                    (Tile::Bugs, 1) => Tile::Bugs,
                    (Tile::Bugs, _) => Tile::Empty,
                    (Tile::Empty, 1) | (Tile::Empty, 2) => Tile::Bugs,
                    _ => *prev,
                },
            );
        }
        next_grid
    }

    pub fn step(&mut self) {
        let mut next = HashMap::new();

        let mut levels = self.levels();
        // expand grids one level up and one level below
        let lvl_min = levels.iter().min().cloned().unwrap();
        let lvl_max = levels.iter().max().cloned().unwrap();

        for lvl in levels {
            next.insert(lvl, self.update_grid(lvl));
        }
        let lvl_below = self.update_grid(lvl_min - 1);
        if self.count_bugs_in_grid(&lvl_below) > 0
        {
            next.insert(lvl_min - 1, lvl_below);
        }

        let lvl_above = self.update_grid(lvl_max + 1);
        if self.count_bugs_in_grid(&lvl_above) > 0
        {
            next.insert(lvl_max + 1, lvl_above);
        }

        self.lvl_to_grid = next;
    }

    pub fn biodiversity(&self) -> u64 {
        let mut res = 0;
        let len_y = self.dims.y_max - self.dims.y_min + 1;
        for (pos, t) in self
            .lvl_to_grid
            .get(&0)
            .expect("no lvl 0 grid found")
            .iter()
        {
            if let Tile::Bugs = t {
                res += 2u64.pow((pos.x + len_y * pos.y) as u32);
            }
        }
        res
    }
    /// Get the edge
    fn get_edge(&self, dir: &Direction) -> &[Position] {
        &self.pos_edges.get(dir).unwrap()
    }

    fn compute_edge(dims: &Dimensions, dir: &Direction) -> Vec<Position> {
        let Dimensions {
            x_min,
            x_max,
            y_min,
            y_max,
        } = *dims;

        let height = (y_max - y_min + 1) as usize;
        let width = (x_max - x_min + 1) as usize;

        let vals_x = match dir {
            Direction::West => vec![x_min; width],
            Direction::East => vec![x_max; width],
            Direction::North => (x_min..x_max + 1).collect(),
            Direction::South => (x_min..x_max + 1).collect(),
        };
        let vals_y = match dir {
            Direction::West => (y_min..y_max + 1).collect(),
            Direction::East => (y_min..y_max + 1).collect(),
            Direction::North => vec![y_min; height],
            Direction::South => vec![y_max; height],
        };

        vals_x
            .iter()
            .zip(vals_y.iter())
            .map(|p| Position::from(p))
            .collect()
    }

    pub fn write(&self) -> String {
        let mut output: Vec<u8> = Vec::new();

        let mut levels: Vec<i64> = self.lvl_to_grid.keys().cloned().collect();
        levels.sort();

        for lvl in levels.iter() {
            let grid = self.lvl_to_grid.get(&lvl).unwrap();
            write!(output, "\nLevel: {}\n{}\n", lvl, grid.write());
        }
        String::from_utf8(output).expect("Error encoding state.")
    }

    pub fn print(&self) {
        print!("{}", self.write());
    }

    pub fn find_first_repeating(&mut self) {
        let mut seen: HashSet<String> = HashSet::new();
        seen.insert(self.write());

        loop {
            self.step();
            let state = self.write();

            // println!("\n{}", state);
            // std::thread::sleep(std::time::Duration::from_millis(25));

            if seen.contains(&state) {
                break;
            } else {
                seen.insert(state);
            }
        }
    }

    fn count_bugs_in_grid(&self, grid: &Grid<Tile>) -> usize {
        grid.iter().filter(|(_, t)| **t == Tile::Bugs).count()
    }

    pub fn count_bugs(&self) -> usize {
        let mut count = 0;
        for grid in self.lvl_to_grid.values() {
            count += self.count_bugs_in_grid(grid);
        }
        count
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
    {
        let mut eris = GameOfEris::new("input.txt");
        eris.find_first_repeating();
        println!();
        eris.print();
        println!();
        println!("{}", eris.biodiversity());
        println!();
    }
    {
        let mut eris = GameOfEris::new("input_rec.txt");
        eris.steps(200);
        println!();
        eris.print();
        println!();
        println!("# of bugs: {}", eris.count_bugs());
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biodiveristy() {
        let eris = GameOfEris::new("example_01.txt");
        assert_eq!(eris.biodiversity(), 2129920);
    }

    #[test]
    fn test_bug_count() {
        let mut eris = GameOfEris::new("example_02.txt");
        println!("{:?}", eris);
        eris.steps(10);
        eris.print();
        assert_eq!(eris.count_bugs(), 99);
    }
}

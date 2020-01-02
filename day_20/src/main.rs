use std::cmp::min;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::fs::read_to_string;

mod grid;

use grid::{Direction, Grid, Position};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Empty,
    Way,
    Wall,
    PartialPortal(char),
    Portal(Portal),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Portal {
    pos: Position,
    label: PortalLabel,
    pos_sibling: Position,
    /// direction in which entrance lies
    dir_entrance: Direction,
    dir_entrance_sibling: Direction,
    level_change: LevelChange,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum LevelChange {
    Upwards,
    Downwards,
}

const LEN_PORTAL_LABEL: usize = 2;
type PortalLabel = [char; LEN_PORTAL_LABEL];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Key(char);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Door(char);

#[derive(Debug)]
struct Maze {
    grid: Grid<Tile>,
    portals: HashMap<PortalLabel, [Portal; 2]>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct MazeState {
    pos: Position,
    level: usize,
}

impl MazeState {
    fn step(&self, dir: &Direction) -> Self {
        MazeState {
            pos: self.pos.step(dir),
            level: self.level,
        }
    }

    fn up(&self) -> Self {
        MazeState {
            pos: self.pos,
            level: self.level + 1,
        }
    }

    fn down(&self) -> Self {
        assert!(self.level > 0, "Cannot go lower than level 0");
        MazeState {
            pos: self.pos,
            level: self.level - 1,
        }
    }
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        use Tile::*;
        match c {
            ' ' => Empty,
            '.' => Way,
            '#' => Wall,
            c if c.is_ascii_uppercase() => PartialPortal(c),
            _ => panic!("Invalid input for tile."),
        }
    }
}

impl From<Door> for Key {
    fn from(d: Door) -> Self {
        let Door(c) = d;
        Key(c.to_ascii_lowercase())
    }
}

impl From<Key> for Door {
    fn from(k: Key) -> Self {
        let Key(c) = k;
        Door(c.to_ascii_uppercase())
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
            Empty => ' ',
            Way => '.',
            Wall => '#',
            PartialPortal(c) => *c,
            Portal(portal) => portal.into(),
        };
        f.write_str(&to_write.to_string())
    }
}

impl Into<char> for &Portal {
    fn into(self) -> char {
        match self.dir_entrance {
            Direction::East | Direction::South => self.label[0],
            Direction::West | Direction::North => self.label[1],
        }
    }
}

impl Portal {
    /// Create a new pair of linked portals from the four positions that contain label information
    /// in the grid.
    pub fn new_pair(
        label: PortalLabel,
        positions: &Vec<Position>,
        grid: &Grid<Tile>,
    ) -> [Portal; 2] {
        // find out where the two entrances are
        assert_eq!(
            positions.len(),
            LEN_PORTAL_LABEL * 2,
            "Invalid number of positions to build Portal Pair."
        );

        let mut portal_pos: Vec<(Position, Direction)> = Vec::new();

        for pos in positions {
            for dir in Direction::all() {
                if let Tile::Way = grid.get(&pos.step(dir)) {
                    portal_pos.push((*pos, *dir));
                }
            }
        }
        assert_eq!(
            portal_pos.len(),
            2,
            "Found more than one entrance for Portal!"
        );

        let dims = grid.get_dims();

        // distinguish inner and outter portals
        let distinguish = |pos: Position| -> LevelChange {
            if pos.x - 1 == dims.x_min
                || pos.x + 1 == dims.x_max
                || pos.y - 1 == dims.y_min
                || pos.y + 1 == dims.y_max
            {
                LevelChange::Downwards
            } else {
                LevelChange::Upwards
            }
        };

        let portal_a = Portal {
            label,
            pos: portal_pos[0].0,
            pos_sibling: portal_pos[1].0,
            dir_entrance: portal_pos[0].1,
            dir_entrance_sibling: portal_pos[1].1,
            level_change: distinguish(portal_pos[0].0),
        };
        let portal_b = Portal {
            label,
            pos: portal_pos[1].0,
            pos_sibling: portal_pos[0].0,
            dir_entrance: portal_pos[1].1,
            dir_entrance_sibling: portal_pos[0].1,
            level_change: distinguish(portal_pos[1].0),
        };

        if label != ['A', 'A'] {
            assert_ne!(
                portal_a.level_change, portal_b.level_change,
                "Portals do not point in different directions!"
            );
        }

        [portal_a, portal_b]
    }

    pub fn get_entrance(&self) -> Position {
        self.pos.step(&self.dir_entrance)
    }

    pub fn get_entrance_sibling(&self) -> Position {
        self.pos_sibling.step(&self.dir_entrance_sibling)
    }
}

impl Maze {
    pub fn new(filename: &str) -> Maze {
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
        // grid.print(|_| -> Option<char> {None});
        let portals = Maze::connect_portals(&mut grid);

        Maze { grid, portals }
    }

    fn get_portal_label((pos, label): (&Position, &char), grid: &Grid<Tile>) -> PortalLabel {
        let mut neighbour_label: Option<(&Direction, char)> = None;
        for dir in Direction::all() {
            let pos = pos.step(dir);
            if let Tile::PartialPortal(c) = grid.get(&pos) {
                neighbour_label = Some((dir, c));
                break;
            }
        }

        let (dir, n_label) = neighbour_label.expect("Invalid portal location.");

        // we read left-to-right/top-to-bottom
        match dir {
            Direction::East | Direction::South => [*label, n_label],
            Direction::West | Direction::North => [n_label, *label],
        }
    }

    fn connect_portals(grid: &mut Grid<Tile>) -> HashMap<PortalLabel, [Portal; 2]> {
        let partials = grid.iter().filter_map(|(pos, tile)| {
            if let Tile::PartialPortal(c) = tile {
                Some((pos, c))
            } else {
                None
            }
        });

        let mut collected: HashMap<PortalLabel, Vec<Position>> = HashMap::new();

        for e in partials {
            let label = Maze::get_portal_label(e, grid);
            let mut positions = collected.remove(&label).unwrap_or(Vec::new());
            positions.push(*e.0);
            collected.insert(label, positions);
        }

        let mut portals = HashMap::new();

        let mut entrance: Option<Vec<Position>> = None;
        let mut exit: Option<Vec<Position>> = None;

        for (label, positions) in collected {
            match label {
                ['A', 'A'] => {
                    entrance = Some(positions);
                }
                ['Z', 'Z'] => {
                    exit = Some(positions);
                }
                _ => {
                    let pair = Portal::new_pair(label, &positions, grid);
                    for portal in &pair {
                        grid.add(portal.pos, Tile::Portal(*portal));
                    }

                    portals.insert(label, pair);
                }
            }
        }

        // Make entrance and exit
        let entrance = entrance.expect("Did not find entrance (AA).");
        let exit = exit.expect("Did not find exit (ZZ).");

        let mut pair = Portal::new_pair(['A', 'A'], &[&entrance[..], &exit[..]].concat(), grid);
        // adjust label of exit
        pair[1].label = ['Z', 'Z'];
        for portal in &pair {
            grid.add(portal.pos, Tile::Portal(*portal));
        }
        portals.insert(['A', 'A'], pair);

        portals
    }

    fn get_portal_entrance(&self) -> Portal {
        self.portals.get(&['A', 'A']).unwrap()[0]
    }

    fn get_entrance(&self) -> Position {
        self.get_portal_entrance().get_entrance()
    }

    fn get_exit(&self) -> Position {
        self.get_portal_entrance().get_entrance_sibling()
    }

    fn get_shortest_path(&mut self) -> usize {
        let mut queue: VecDeque<(Position, usize)> = VecDeque::new();

        let mut visited: HashSet<Position> = HashSet::new();

        queue.push_back((self.get_entrance(), 0));
        visited.insert(self.get_entrance());
        // blacklist entrance and exit portals
        visited.insert(self.get_portal_entrance().pos);

        let exit = self.get_exit();

        while let Some((pos, dist)) = queue.pop_front() {
            // eprint!("\rCurrent stack length: {}", queue.len());
            if pos == exit {
                return dist;
            }

            for dir in Direction::all() {
                let pos = pos.step(dir);

                match self.grid.get(&pos) {
                    Tile::Way => {
                        if !visited.contains(&pos) {
                            visited.insert(pos);
                            queue.push_back((pos, dist + 1));
                        }
                    }
                    Tile::Portal(portal) if ![['A', 'A'], ['Z', 'Z']].contains(&portal.label) => {
                        let pos = portal.get_entrance_sibling();
                        if !visited.contains(&pos) {
                            queue.push_back((pos, dist + 1));
                            visited.insert(pos);
                        }
                    }
                    _ => {}
                }
            }
        }
        panic!("Did not reach exit!");
    }

    fn get_shortest_path_recursive(&mut self) -> usize {
        let mut queue: VecDeque<(MazeState, usize)> = VecDeque::new();

        let mut visited: HashSet<MazeState> = HashSet::new();

        let start = MazeState {
            pos: self.get_entrance(),
            level: 0,
        };
        queue.push_back((start, 0));
        visited.insert(start);

        let exit = MazeState {
            pos: self.get_exit(),
            level: 0,
        };

        while let Some((state, dist)) = queue.pop_front() {
            // eprint!("\rCurrent stack length: {}", queue.len());
            if state == exit {
                return dist;
            }

            for dir in Direction::all() {
                let state = state.step(dir);

                match self.grid.get(&state.pos) {
                    Tile::Way => {
                        if !visited.contains(&state) {
                            visited.insert(state);
                            queue.push_back((state, dist + 1));
                        }
                    }
                    // regular portal - not start/exit
                    Tile::Portal(portal) if ![['A', 'A'], ['Z', 'Z']].contains(&portal.label) => {
                        let mut state = state;
                        match (portal.level_change, state.level) {
                            (LevelChange::Upwards, _) => {
                                state = state.up();
                            }
                            (LevelChange::Downwards, l) if l > 0 => {
                                state = state.down();
                            }
                            _ => {
                                continue;
                            }
                        }
                        state.pos = portal.get_entrance_sibling();
                        if !visited.contains(&state) {
                            queue.push_back((state, dist + 1));
                            visited.insert(state);
                        }
                    }
                    _ => {}
                }
            }
        }
        panic!("Did not reach exit!");
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
    let mut maze = Maze::new(
        &std::env::args()
            .skip(1)
            .next()
            .expect("Filename not provided."),
    );
    maze.grid.print(|_: &Position| -> Option<String> { None });
    /*
     * for portals in maze.portals.iter() {
     *     println!("{:?}", portals);
     * }
     */
    println!("\nShortest: {}", maze.get_shortest_path());
    println!("\nShortest (recursive): {}", maze.get_shortest_path_recursive());
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn example_01() {
        let mut maze = Maze::new("example_01.txt");
        assert_eq!(maze.get_shortest_path(), 23);
    }

    #[test]
    fn example_02() {
        let mut maze = Maze::new("example_02.txt");
        assert_eq!(maze.get_shortest_path(), 58);
    }

    #[test]
    fn example_03() {
        let mut maze = Maze::new("example_03.txt");
        assert_eq!(maze.get_shortest_path_recursive(), 396);
    }
}

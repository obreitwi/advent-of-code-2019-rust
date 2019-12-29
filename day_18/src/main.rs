use std::cmp::min;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;
use std::fs::read_to_string;

mod grid;

use grid::{Direction, Grid, Position};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    Entrance,
    Key(Key),
    Door(Door),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Key(char);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
struct Door(char);

#[derive(Debug)]
struct Maze {
    grid: Grid<Tile>,
    entrance: Position,
    keys: HashMap<Key, Position>,
    doors: HashMap<Door, Position>,
    cache_reachable: HashMap<MazeState, HashMap<MazeState, usize>>,
    cache_lower_bound: HashMap<MazeState, usize>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct MazeState {
    /// current position
    pos: Position,
    /// keys in possession
    keys: BTreeSet<Key>,
}

impl MazeState {
    fn from_pos(pos: Position) -> MazeState {
        MazeState {
            pos,
            keys: BTreeSet::new(),
        }
    }
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        use Tile::*;
        if c == '.' {
            Empty
        } else if c == '#' {
            Wall
        } else if c == '@' {
            Entrance
        } else if c.is_ascii_lowercase() {
            Key(self::Key(c))
        } else if c.is_ascii_uppercase() {
            Door(self::Door(c))
        } else {
            panic!("Could not parse Tile!");
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
        Tile::Wall
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Tile::*;
        let to_write = match self {
            Empty => '.',
            Wall => '#',
            Entrance => '@',
            Key(self::Key(c)) => *c,
            Door(self::Door(c)) => *c,
        };
        f.write_str(&to_write.to_string())
    }
}

impl Maze {
    pub fn new(filename: &str) -> Maze {
        use Tile::*;

        let raw = read_to_string(filename).expect("Could not read input file.");
        let mut grid = Grid::new();
        let mut entrance = None;
        let mut keys = HashMap::new();
        let mut doors = HashMap::new();

        for (y, line) in raw.lines().enumerate() {
            for (x, c) in line.trim().chars().enumerate() {
                let tile = Tile::from(c);
                let pos = Position {
                    x: x as i64,
                    y: y as i64,
                };
                grid.add(pos, tile);

                match tile {
                    Entrance => {
                        entrance = Some(pos);
                    }
                    Key(k) => {
                        keys.insert(k, pos);
                    }
                    Door(d) => {
                        doors.insert(d, pos);
                    }
                    _ => {}
                }
            }
        }

        Maze {
            grid,
            entrance: entrance.expect("Maze did not contain entrance."),
            keys,
            doors,
            cache_reachable: HashMap::new(),
            cache_lower_bound: HashMap::new(),
        }
    }

    fn get_reachable(&mut self, state: &MazeState) -> HashMap<MazeState, usize> {
        match self.cache_reachable.get(state) {
            Some(map) => map.to_owned(),
            None => {
                let computed = self.compute_reachable(state);
                self.cache_reachable.insert(state.clone(), computed.clone());
                computed
            }
        }
    }

    fn compute_reachable(&self, state: &MazeState) -> HashMap<MazeState, usize> {
        // eprintln!("Getting reachable state for: {:?}", state);

        let mut state_to_distance = HashMap::new();

        let mut to_explore: VecDeque<(Position, usize)> = VecDeque::new();

        to_explore.push_back((state.pos, 0));
        let mut explored = HashSet::new();

        while let Some((current, dist)) = to_explore.pop_front() {
            explored.insert(current);
            match self.grid.get(&current) {
                Tile::Wall => {
                    continue;
                }
                Tile::Empty | Tile::Entrance => {}
                Tile::Door(d) => {
                    if !state.keys.contains(&Key::from(d)) {
                        continue;
                    }
                }
                Tile::Key(k) => {
                    if !state.keys.contains(&k) {
                        let mut new_state: MazeState = state.clone();
                        new_state.keys.insert(k);
                        new_state.pos = self.keys.get(&k).expect("Key not present!").clone();
                        state_to_distance.insert(new_state, dist);
                        continue; // we don't continue exploring after we encountered a key
                    }
                }
            }

            for d in Direction::all() {
                let new_pos = current.step(d);
                if !explored.contains(&new_pos) {
                    to_explore.push_back((new_pos, dist + 1));
                }
            }
        }

        // eprintln!("Reachable: {:?}", state_to_distance);
        state_to_distance
    }

    fn get_lower_bound(&mut self, state: &MazeState) -> usize {
        match self.cache_lower_bound.get(state) {
            Some(lb) => *lb,
            None => {
                let lb = self.compute_lower_bound(state);
                self.cache_lower_bound.insert(state.clone(), lb);
                lb
            }
        }
    }

    /// Get a lower bound for the remaining keys, ignoring any walls
    fn compute_lower_bound(&self, state: &MazeState) -> usize {
        let mut current = state.pos;
        let mut keys_left = self.get_keys_left(state);
        let mut total = 0;
        for _ in 0..(keys_left.len()) {
            let distances: Vec<usize> = keys_left.iter().map(|p| current.mh(&p)).collect();

            let idx = argmin(&mut distances.iter());
            total += distances[idx];
            current = keys_left.remove(idx);
        }
        total
    }

    fn get_keys_left(&self, state: &MazeState) -> Vec<Position> {
        self.keys
            .iter()
            .filter(|(key, pos)| !state.keys.contains(&key))
            .map(|(_, pos)| pos.clone())
            .collect()
    }

    fn get_shortest_path_keys(&mut self) -> usize {
        let mut stack: Vec<MazeState> = Vec::new();

        let mut shortest = std::usize::MAX;
        let mut visited: HashMap<MazeState, usize> = HashMap::new();

        let ms = MazeState::from_pos(self.entrance);
        stack.push(ms.clone());
        visited.insert(ms, 0);

        let mut uniq_states_visited = HashSet::new();

        while let Some(ms) = stack.pop() {
            let dist = visited.get(&ms).unwrap().clone();
            assert!(uniq_states_visited.insert((ms.clone(), dist)));
            eprint!(
                "\r\rStack size: {} (Cache size reachable/lb|uniq: {}/{}|{})",
                stack.len(),
                self.cache_reachable.len(),
                self.cache_lower_bound.len(),
                uniq_states_visited.len()
            );

            // eprintln!("Current stack length: {}", stack.len());
            for (new_ms, diff_dist) in self.get_reachable(&ms).iter() {
                assert!(new_ms.keys.len() > ms.keys.len());
                // eprintln!("Num found keys: {}/{}", new_ms.keys.len(), self.keys.len());

                let dist = dist + diff_dist;
                if dist > shortest {
                    // stop if we cannot beat the best
                    continue;
                } else if new_ms.keys.len() == self.keys.len() {
                    if dist < shortest {
                        shortest = dist;
                        eprintln!("\rFound new shortest path: {}{}", shortest, " ".repeat(40));
                    }
                    continue;
                } else {
                    match visited.get(new_ms) {
                        Some(old) => {
                            if *old > dist {
                                visited.insert(new_ms.clone(), dist);
                                if !stack.contains(new_ms)
                                {
                                    stack.push(new_ms.clone());
                                }
                            }
                        }
                        None => {
                            if self.get_lower_bound(new_ms) > shortest {
                                continue;
                            }
                            visited.insert(new_ms.clone(), dist);
                            stack.push(new_ms.clone());
                        }
                    }
                }
            }
        }

        shortest
    }
}

fn argmin<R: PartialOrd, T: Iterator<Item = R>>(iter: &mut T) -> usize {
    let mut arg = 0;
    let mut min = iter.next().expect("Iterator cannot be empty!");

    for (i, elem) in iter.enumerate() {
        if elem < min {
            arg = i + 1; // we took the first element in the beginning
            min = elem;
        }
    }
    arg
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
    println!("\nShortest: {}", maze.get_shortest_path_keys());
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn example_01() {
        let mut maze = Maze::new("example_01.txt");
        assert_eq!(maze.get_shortest_path_keys(), 132);
    }

    #[test]
    fn example_02() {
        let mut maze = Maze::new("example_02.txt");
        assert_eq!(maze.get_shortest_path_keys(), 136);
    }

    #[test]
    fn example_03() {
        let mut maze = Maze::new("example_03.txt");
        assert_eq!(maze.get_shortest_path_keys(), 81);
    }
}

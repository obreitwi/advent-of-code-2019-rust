use simple_error::{bail, SimpleError};

use std::collections::BTreeSet;
use std::fmt;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;

mod intcode;
use intcode::{Intcode, TapeElem};

mod grid;
use grid::{Direction, Grid, Position};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Tile {
    Empty,
    Room(Room),
    Hallway(Directions),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Directions(BTreeSet<Direction>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Room {
    name: String,
    doors: Directions,
}

impl FromStr for Room {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().skip_while(|s| !s.starts_with("== "));
        let name = match lines.by_ref().next() {
            None => return Err(SimpleError::new("Did not find name for room.")),
            Some(name) => name.trim_start_matches("== ").trim_end_matches(" =="),
        };

        let mut lines = lines
            .skip_while(|s| !s.starts_with("Doors here lead:"))
            .skip(1);

        let mut doors = BTreeSet::new();
        for dir in lines.by_ref().take_while(|s| s.starts_with("- ")) {
            let dir = str_to_dir(dir.trim_start_matches("- "))?;
            doors.insert(dir);
        }

        if doors.len() == 0 {
            Err(SimpleError::new("Did not find any doors."))
        } else {
            Ok(Room {
                name: name.to_string(),
                doors: Directions(doors),
            })
        }
    }
}

fn str_to_dir(s: &str) -> Result<Direction, SimpleError> {
    use Direction::*;
    match s {
        "north" => Ok(North),
        "south" => Ok(South),
        "west" => Ok(West),
        "east" => Ok(East),
        _ => Err(SimpleError::new(format!(
            "Invalid string for direction: {}",
            s
        ))),
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
        match self {
            Empty => write!(f, " "),
            Room(room) => write!(f, "{}", room),
            Hallway(dirs) => write!(f, "{}", dirs),
        }
    }
}
impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.doors)
    }
}

impl fmt::Display for Directions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Direction::*;
        let Directions(dirs) = self;

        let to_write = match dirs.len() {
            4 => "┼",
            3 => {
                if !dirs.contains(&North) {
                    "┬"
                } else if !dirs.contains(&South) {
                    "┴"
                } else if !dirs.contains(&West) {
                    "├"
                } else {
                    "┤"
                }
            }
            2 => {
                let mut dirs = dirs.iter();
                let first = dirs.next().unwrap();
                let second = dirs.next().unwrap();

                match (first, second) {
                    (North, West) => "┘",
                    (North, East) => "└",
                    (North, South) => "│",
                    (South, West) => "┐",
                    (South, East) => "┌",
                    (West, East) => "─",
                    _ => panic!("Ordering of Directions not as expected!"),
                }
            }
            1 => match dirs.iter().next().unwrap() {
                North => "╵",
                South => "╷",
                West => "╴",
                East => "╶",
            },
            0 => " ",
            _ => panic!("No directions given!"),
        };

        f.write_str(to_write)
    }
}

impl Directions {
    pub fn new() -> Directions {
        Directions(BTreeSet::new())
    }

    pub fn as_set(&self) -> &BTreeSet<Direction> {
        let Self(dirs) = self;
        dirs
    }

    pub fn by_set(&mut self) -> &mut BTreeSet<Direction> {
        let Self(dirs) = self;
        dirs
    }

    pub fn add(&mut self, dir: Direction) {
        self.by_set().insert(dir);
    }
}

#[derive(Debug)]
struct RobotAdventure {
    grid: Grid<Tile>,
    code: Intcode,
    pos: Position,
}

#[derive(Debug)]
struct PotentialRoom {
    /// Room found in the specified direction (with position)
    reachable: Option<(Position, Room)>,

    /// direction in which to go
    dir: Option<Direction>,

    /// first position in the given direction
    origin: Position,
}

impl RobotAdventure {
    pub fn new(filename: &str) -> RobotAdventure {
        let pos = Position { x: 0, y: 0 };
        let mut code = Intcode::load(filename);
        let grid = Grid::new();

        code.execute();

        RobotAdventure { pos, grid, code }
    }

    pub fn is_finished(&self) -> bool {
        self.code.is_finished()
    }

    fn step(&self, dir: &Direction) -> PotentialRoom {
        let reachable = match self
            .grid
            .get_in_direction_until(self.pos, dir, 1024, |tile| {
                if let Tile::Room(_) = tile {
                    true
                } else {
                    false
                }
            }) {
            Some((pos, Tile::Room(room))) => Some((pos, room)),
            None => None,
            _ => panic!("Grid did not return a room!"),
        };

        PotentialRoom {
            reachable,
            dir: Some(dir.clone()),
            origin: self.pos,
        }
    }

    pub fn execute_cmd(&mut self, cmd: &str) {
        let step = if let Ok(dir) = str_to_dir(cmd.trim_end_matches("\n")) {
            Some(self.step(&dir))
        } else {
            None
        };

        // eprintln!("Step: {:?}", step);

        self.supply_cmd(cmd);
        self.code.execute();

        let output = if let Some(step) = step {
            self.get_output_with_step(step)
        } else {
            self.get_output()
        };

        self.grid.print_custom(
            |pos: &Position| -> Option<char> {
                if *pos == self.pos {
                    Some('o')
                } else {
                    None
                }
            },
        );
        println!("{}", output);
    }

    fn supply_cmd(&mut self, cmd: &str) {
        for c in cmd.chars() {
            self.code.supply_input(c as TapeElem);
        }
    }
    fn get_output_with_step(&mut self, potential_room: PotentialRoom) -> String {
        let output = self.get_output_str();

        if let Ok(room) = output.parse() {
            self.update_position_via_room(room, potential_room);
        } else {
            eprintln!("WARN: Did not find valid room!")
        }
        output
    }

    pub fn get_output(&mut self) -> String {
        self.get_output_with_step(PotentialRoom {
            reachable: None,
            dir: None,
            origin: self.pos,
        })
    }

    /// Expand and connect everything by Hallways
    fn expand(&mut self, mut origin: &mut Position) {
        self.grid.expand(Some(&mut origin));

        // self.grid.print();

        // clear hallways
        {
            let hallways: Vec<Position> = self
                .grid
                .iter()
                .filter_map(|(pos, t)| {
                    if let Tile::Hallway(_) = *t {
                        Some(pos.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for hw in hallways.iter() {
                self.grid.remove(hw);
            }
        }

        let rooms: Vec<(Position, Room)> = self
            .grid
            .iter()
            .filter_map(|(pos, t)| {
                if let Tile::Room(r) = t {
                    Some((pos.clone(), r.clone()))
                } else {
                    None
                }
            })
            .collect();

        // eprintln!("Rooms: {:?}", rooms);

        for (origin, room) in rooms.iter() {
            // eprintln!("{:?}", (origin, room));
            for dir in room.doors.as_set().iter() {
                let mut pos = origin.step(dir);
                match self.grid.get_in_direction_until(pos, dir, 1024, |t| {
                    if let Tile::Room(_) = *t {
                        true
                    } else {
                        false
                    }
                }) {
                    None => {
                        // if there is nothing to expand to -> don't try
                        continue;
                    }
                    Some((_, Tile::Room(r))) if !r.doors.as_set().contains(&dir.invert()) => {
                        // if the room is not connected to our room -> don't try
                        continue;
                    }
                    _ => {}
                };
                loop {
                    match self.grid.get_existing_mut(&pos) {
                        Some(Tile::Room(_)) => break,
                        Some(Tile::Hallway(dirs)) => {
                            dirs.add(dir.invert());
                            // eprintln!("Added to hallway {:?} at {:?}", dirs, pos);
                        }
                        None => {
                            let mut dirs = Directions::new();
                            dirs.add(dir.invert());
                            // eprintln!("Added hallway {:?} at {:?}", dirs, pos);
                            self.grid.add(pos, Tile::Hallway(dirs));
                        }
                        Some(Tile::Empty) => {
                            panic!("Grid should not contain explicit empty tiles.")
                        }
                    }
                    pos = pos.step(dir);
                }
            }
        }
    }

    fn update_position_via_room(&mut self, room: Room, potential_room: PotentialRoom) {
        self.pos = if let Some((pos, reachable)) = potential_room.reachable {
            if reachable != room {
                let mut origin = potential_room.origin;
                self.expand(&mut origin);
                origin.step(&potential_room.dir.unwrap())
            } else {
                pos
            }
        } else {
            if let Some(dir) = potential_room.dir {
                potential_room.origin.step(&dir)
            } else {
                potential_room.origin
            }
        };

        self.grid.add(self.pos, Tile::Room(room));
    }

    pub fn get_output_str(&mut self) -> String {
        let mut buf: Vec<u8> = Vec::new();

        while let Some(c) = self.code.get_output() {
            buf.push(c as u8);
        }

        String::from_utf8(buf).unwrap()
    }
}

fn main() {
    let mut robot = RobotAdventure::new("input.txt");

    println!("{}", robot.get_output());

    while !robot.is_finished() {
        let mut s = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");

        robot.execute_cmd(&s);
    }
}

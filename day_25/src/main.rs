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
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Room {
    name: String,
    doors: BTreeSet<Direction>,
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
                doors,
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
        }
    }
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Direction::*;
        let to_write = match self.doors.len() {
            4 => "┼",
            3 => {
                if !self.doors.contains(&North) {
                    "┬"
                } else if !self.doors.contains(&South) {
                    "┴"
                } else if !self.doors.contains(&West) {
                    "├"
                } else {
                    "┤"
                }
            }
            2 => {
                let mut doors = self.doors.iter();
                let first = doors.next().unwrap();
                let second = doors.next().unwrap();

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
            1 => match self.doors.iter().next().unwrap() {
                North => "╵",
                South => "╷",
                West => "╴",
                East => "╶",
            },
            0 => " ",
            _ => panic!("Room has no doors!"),
        };

        f.write_str(to_write)
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

    fn step(&self, dir: &Direction) -> PotentialRoom
    {
        let first_step = self.pos.step(&dir);
        let reachable = match self.grid.get_in_direction(self.pos, dir, 1024)
        {
            Some((pos, Tile::Room(room))) => Some((pos, room)),
            None => None,
            _ => panic!("Grid did not return a room!"),
        };

        PotentialRoom{ reachable, dir: Some(dir.clone()), origin: self.pos }
    }

    pub fn execute_cmd(&mut self, cmd: &str) {
        let step = if let Ok(dir) = str_to_dir(cmd.trim_end_matches("\n")) {
            Some(self.step(&dir))
        }
        else {
            None
        };

        eprintln!("Step: {:?}", step);

        self.supply_cmd(cmd);
        self.code.execute();

        let output = if let Some(step) = step {
            self.get_output_with_step(step)
        }
        else 
        {
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
        self.get_output_with_step(PotentialRoom{ reachable: None, dir: None, origin: self.pos })
    }

    fn update_position_via_room(&mut self, room: Room, potential_room: PotentialRoom)
    {
        self.pos = if let Some((pos, reachable)) = potential_room.reachable
        {
            if reachable != room
            {
                let mut origin = potential_room.origin;
                self.grid.expand(Some(&mut origin));
                origin.step(&potential_room.dir.unwrap())
            }
            else
            {
                pos
            }
        }
        else{
            if let Some(dir) = potential_room.dir
            {
                potential_room.origin.step(&dir)
            }
            else 
            {
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

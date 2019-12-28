use itertools::Itertools;
use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::fmt;

mod grid;
mod intcode;

use grid::{Direction, Grid, Position, Turn};
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
                if output == '\n' as i64 {
                    y += 1;
                    x = 0;
                } else {
                    self.grid.add(Position { x, y }, Tile::from(output));
                    x += 1;
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

    fn get_pos_robot(&self) -> (Position, Direction) {
        let (pos, _) = self
            .grid
            .iter()
            .filter(|(_, t)| **t == Tile('^'))
            .next()
            .expect("Could not find robot!");

        (*pos, Direction::North)
    }

    fn get_stepsize_into(&self, pos: &Position, dir: &Direction) -> usize {
        let mut len = 0;
        let mut pos = pos.step(dir);
        while self.grid.get(&pos) == Tile('#') {
            len += 1;
            pos = pos.step(dir);
        }
        len
    }

    fn get_turn(&self, pos: &Position, dir: &Direction) -> Option<Turn> {
        for t in Turn::all() {
            let new = pos.step(&dir.turn(t));
            if self.grid.get(&new) == Tile('#') {
                return Some(*t);
            }
        }
        None
    }

    fn get_complete_tour(&self) -> Vec<String> {
        let (mut pos, mut dir) = self.get_pos_robot();
        let mut vec = Vec::new();

        while let Some(t) = self.get_turn(&pos, &dir) {
            vec.push(t.into());
            dir = dir.turn(&t);

            let stepsize = self.get_stepsize_into(&pos, &dir);
            vec.push(format!("{}", stepsize));
            for _ in 0..stepsize {
                pos = pos.step(&dir);
            }
        }
        vec
    }

    fn warn_other_robots(&mut self) -> TapeElem {
        let tour = self.get_complete_tour();

        let commands = robot_cmds::encode(&tour[..]);

        self.computer.reset();

        self.computer.set(0, 2);

        for cmd in commands.iter() {
            self.supply_command(cmd);
        }
        self.computer.supply_input('y' as TapeElem);
        self.computer.supply_input('\n' as TapeElem);

        let mut printed_newline = false;
        let mut perfrom_clear_screen = false;

        if false {
            while !self.computer.is_finished() {
                self.computer.execute_n(10000);
                while let Some(c) = self.computer.get_output() {
                    if perfrom_clear_screen {
                        clear_screen();
                        perfrom_clear_screen = false;
                    }

                    print!("{}", Tile::from(c));

                    if c == ('\n' as TapeElem) {
                        if !printed_newline {
                            printed_newline = true;
                        } else {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            printed_newline = false;
                            perfrom_clear_screen = true;
                        }
                    } else {
                        printed_newline = false;
                    }
                }
            }
        }
        self.computer.execute();
        let mut output = None;
        while let Some(c) = self.computer.get_output() {
            match output {
                None => {}
                Some(old) => print!("{}", Tile::from(old)),
            };
            output = Some(c);
        }
        output.expect("Computer did not produce output.")
    }

    fn supply_command(&mut self, cmd: &[String]) {
        let mut num_inputs = 0;
        for c in cmd
            .iter()
            .interleave(std::iter::repeat(&String::from(",")).take(cmd.len() - 1))
        {
            for input in c.chars() {
                num_inputs += 1;
                let input = input as TapeElem;
                // eprintln!("[#{}] Supplying: {} (raw: {})", num_inputs, c, input);
                self.computer.supply_input(input);
            }
        }
        num_inputs += 1;
        // eprintln!("[#{}] Supplying: \\n", num_inputs);
        self.computer.supply_input('\n' as TapeElem);
        assert!(num_inputs <= 20);
        // eprintln!();
    }
}

mod robot_cmds {
    use itertools::Itertools;

    pub fn encode(vec: &[String]) -> [Vec<String>; 4] {
        let get_range = || (2..13).filter(|i| i % 2 == 0);
        let forbidden = [String::from("A"), String::from("B"), String::from("C")];

        for len_a in get_range() {
            let (repl_a, replaced_a) = match check_replacement(vec, len_a, "A", &[]) {
                None => continue,
                Some(x) => x,
            };

            for len_b in get_range() {
                let (repl_b, replaced_b) = match check_replacement(&replaced_a, len_b, "B", &["A"])
                {
                    None => continue,
                    Some(x) => x,
                };

                for len_c in get_range() {
                    let (repl_c, replaced_c) =
                        match check_replacement(&replaced_b, len_c, "C", &["A", "B"]) {
                            None => continue,
                            Some(x) => x,
                        };

                    if !replaced_c.iter().all(|c| forbidden.contains(c)) {
                        // eprintln!("{:?} does not consist of only A B C.", replaced_c);
                        continue;
                    } else {
                        return [
                            replaced_c,
                            repl_a.to_vec(),
                            repl_b.to_vec(),
                            repl_c.to_vec(),
                        ];
                    }
                }
            }
        }
        panic!("Could not find assignment for A B C.")
    }

    pub fn cmd_to_string(cmd: &[String]) -> String {
        let mut retval = String::new();
        for part in cmd
            .iter()
            .interleave(std::iter::repeat(&String::from(",")).take(cmd.len() - 1))
        {
            retval.push_str(part);
        }
        retval
    }

    fn replace_subvector(vec: &[String], to_replace: &[String], label: &str) -> Vec<String> {
        let mut retval = Vec::new();
        let mut idx = 0;

        // eprintln!("Replacing {:?} in {:?}", to_replace, vec);

        while idx < vec.len() {
            if idx + to_replace.len() <= vec.len()
                && vec[idx..idx + to_replace.len()] == *to_replace
            {
                retval.push(String::from(label));
                idx += to_replace.len();
            } else {
                retval.push(String::from(&vec[idx]));
                idx += 1;
            }
        }
        retval
    }

    fn check_replacement(
        original: &[String],
        repl_len: usize,
        label: &str,
        replaced_labels: &[&str],
    ) -> Option<(Vec<String>, Vec<String>)> {
        let check_contains_forbidden = |v: &[String]| -> bool {
            replaced_labels
                .iter()
                .any(|forbidden| v.contains(&String::from(*forbidden)))
        };

        let max_cmd_len = 20;

        // eprintln!("Trying len: {}", repl_len);
        let mut repl: Option<&[String]> = None;

        if original.len() <= repl_len {
            return None;
        }

        for offset in 0..(original.len() - repl_len) {
            let val = &original[offset..repl_len + offset];
            repl = Some(val);
            if !check_contains_forbidden(val) {
                break;
            }
        }
        if let None = repl {
            return None;
        }
        let repl = match repl {
            Some(repl) => repl,
            None => return None,
        };

        if check_contains_forbidden(repl) {
            // eprintln!("{:?} contains {:?}", repl, replaced_labels);
            return None;
        }

        if cmd_to_string(repl).len() > max_cmd_len {
            // eprintln!("Too long: {}", cmd_to_string(repl_b));
            return None;
        }

        let replaced = replace_subvector(&original, repl, label);
        // eprintln!("Programm {}: {:?} (replaced: {:?}", label, repl, replaced);
        return Some((repl.to_vec(), replaced));
    }

    pub fn reconstruct(
        main: &[String],
        prog_a: &[String],
        prog_b: &[String],
        prog_c: &[String],
    ) -> Vec<String> {
        let mut retval: Vec<String> = Vec::new();
        for routine in main.iter() {
            match routine.as_str() {
                "A" => retval.extend_from_slice(prog_a),
                "B" => retval.extend_from_slice(prog_b),
                "C" => retval.extend_from_slice(prog_c),
                _ => panic!("Invalid funciton in main function!"),
            };
        }
        retval
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
    let tour = robot.get_complete_tour();
    println!();
    println!("{:?}", tour);
    println!("Length best tour: {}", tour.len());

    let encoded_tour = robot_cmds::encode(&tour[..]);
    println!("{:?}", encoded_tour);
    let reconstructed = robot_cmds::reconstruct(
        &encoded_tour[0],
        &encoded_tour[1],
        &encoded_tour[2],
        &encoded_tour[3]
    );
    for (i, (l, r)) in tour.iter().zip(reconstructed.iter()).enumerate()
    {
        println!("#{}: ({}, {})", i, l, r);
    }
    assert_eq!(
        tour,
        reconstructed
    );

    println!("{:?}", robot.warn_other_robots());
}

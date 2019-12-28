use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};
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
}

fn consolidate(vec: &[String]) -> [Vec<String>; 4] {
    let get_range = || (2..11);
    let forbidden = [String::from("A"), String::from("B"), String::from("C")];
    let check_contains_abc =
        |v: &[String]| -> bool { forbidden.iter().any(|forbidden| v.contains(forbidden)) };

    for len_a in get_range() {
        let repl_a = &vec[..len_a];
        let replaced_a = replace_subvector(vec, repl_a, "A");

        for len_b in get_range() {
            if replaced_a.len() < len_b + 1 {
                continue;
            }
            let repl_b = &replaced_a[1..len_b + 1];

            if check_contains_abc(repl_b) {
                continue;
            }

            let replaced_b = replace_subvector(&replaced_a, repl_b, "B");

            for len_c in get_range() {
                if replaced_b.len() < len_c + 2 {
                    continue;
                }

                let repl_c = &replaced_b[2..len_c + 2];

                if check_contains_abc(repl_b) {
                    continue;
                }

                let replaced_c = replace_subvector(&replaced_b, repl_c, "C");

                if !replaced_c.iter().all(|c| forbidden.contains(c)) {
                    continue;
                } else {
                    return [
                        repl_a.to_vec(),
                        repl_b.to_vec(),
                        repl_c.to_vec(),
                        replaced_c,
                    ];
                }
            }
        }
    }
    panic!("Could not find assignment for A B C.")
}

fn consolidate_v1(vec: &[String]) -> Vec<String> {
    let retval = vec;

    let (size, start) = find_repeating_best(retval);
    println!("size: {}", size);
    let retval = replace_subvector(retval, &retval[start..start + size], "A");
    println!("After A: {:?}", retval);

    let (size, start) = find_repeating_best(&retval[1..]);
    let start = start + 1;
    println!("size: {}", size);
    let retval = replace_subvector(&retval[..], &retval[start..start + size], "B");
    println!("After B: {:?}", retval);

    let (size, start) = find_repeating_best(&retval[2..]);
    let start = start + 2;
    println!("size: {}", size);
    let retval = replace_subvector(&retval[..], &retval[start..start + size], "C");
    println!("After C: {:?}", retval);

    retval
}

fn replace_subvector(vec: &[String], to_replace: &[String], label: &str) -> Vec<String> {
    let mut retval = Vec::new();
    let mut idx = 0;

    // eprintln!("Replacing {:?} in {:?}", to_replace, vec);

    while idx + to_replace.len() < vec.len() {
        if vec[idx..idx + to_replace.len()] == *to_replace {
            retval.push(String::from(label));
            idx += to_replace.len();
        } else {
            retval.push(String::from(&vec[idx]));
            idx += 1;
        }
    }
    retval
}

fn find_repeating_best(vec: &[String]) -> (usize, usize) {
    let (mut rv_size_replaced, mut rv_len, mut rv_offset) = (std::usize::MAX, 0, 0);

    for offset in 0..vec.len() {
        for len in (2..vec.len() + 1 - offset).rev() {
            if len % 2 != 0 {
                continue;
            }

            let replacement = &vec[offset..offset + len];
            if [String::from("A"), String::from("B"), String::from("C")]
                .iter()
                .any(|forbidden| replacement.contains(forbidden))
            {
                continue;
            }

            if !(replacement[0] == "R" || replacement[0] == "L") {
                continue;
            }

            match find_repeating_step(&vec[1..], replacement) {
                Some(idx) => {
                    let idx = idx + 1;
                    let replaced = replace_subvector(&vec[..], &vec[idx..idx + len], "<replaced>");
                    // eprintln!("Replaced {:?} -> length: {}", replaced, replaced.len());
                    if replaced.len() < rv_size_replaced {
                        rv_size_replaced = replaced.len();
                        rv_len = len;
                        rv_offset = idx;
                    }
                }
                None => {}
            }
        }
    }
    if rv_size_replaced < std::usize::MAX {
        (rv_len, rv_offset)
    } else {
        panic!("Did not find repeating substring!");
    }
}

/// Find repeating sub-vector and return its size
fn find_repeating(vec: &[String]) -> (usize, usize) {
    for len in (2..vec.len() + 1).rev() {
        if len % 2 != 0 {
            continue;
        }

        let replacement = &vec[..len];
        if [String::from("A"), String::from("B"), String::from("C")]
            .iter()
            .any(|forbidden| replacement.contains(forbidden))
        {
            continue;
        }

        if !(replacement[0] == "R" || replacement[0] == "L") {
            continue;
        }

        match find_repeating_step(&vec[1..], replacement) {
            Some(idx) => {
                return (len, idx + 1);
            }
            None => {}
        }
    }
    panic!("Did not find repeating substring!");
}

/// Search for `to_find` in `base` and return its starting position if found
fn find_repeating_step(base: &[String], to_find: &[String]) -> Option<usize> {
    if base.len() < to_find.len() {
        return None;
    }
    let idx_limit = base.len() - to_find.len();
    for idx in 0..idx_limit {
        if base[idx..idx + to_find.len()] == to_find[..] {
            return Some(idx);
        }
    }
    None
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

    println!("{:?}", consolidate(&tour[..]));
}

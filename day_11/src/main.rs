use std::cmp::{max, min};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::fs::read_to_string;

#[derive(Clone)]
pub struct Intcode {
    tape: Vec<TapeElem>,
    pos: usize,
    current: Option<Instruction>,
    input: Option<TapeElem>,
    output: VecDeque<TapeElem>,
    finished: bool,
    relative_base: TapeElem,
}

type TapeElem = i64;

#[derive(Debug, Clone)]
enum Operation {
    Multiply,
    Add,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    SetRelativeBase,
    Break,
}

#[derive(Debug, Clone)]
struct Instruction {
    op: Operation,
    params: Vec<Parameter>,
}

impl Operation {
    fn _code(&self) -> TapeElem {
        match self {
            Operation::Add => 1,
            Operation::Multiply => 2,
            Operation::Input => 3,
            Operation::Output => 4,
            Operation::JumpIfTrue => 5,
            Operation::JumpIfFalse => 6,
            Operation::LessThan => 7,
            Operation::Equals => 8,
            Operation::SetRelativeBase => 9,
            Operation::Break => 99,
        }
    }

    fn num_params(&self) -> usize {
        // don't forget ouput parameter!
        match self {
            Operation::Add => 3,
            Operation::Multiply => 3,
            Operation::Input => 1,
            Operation::Output => 1,
            Operation::JumpIfTrue => 2,
            Operation::JumpIfFalse => 2,
            Operation::LessThan => 3,
            Operation::Equals => 3,
            Operation::SetRelativeBase => 1,
            Operation::Break => 0,
        }
    }

    fn decode(&self, info: i64, pos: usize) -> Vec<Parameter> {
        let mut params = Vec::with_capacity(self.num_params());
        let mut info = info;

        for i in 1..self.num_params() + 1 {
            params.push(match info % 10 {
                0 => Parameter::PositionAt(pos + i),
                1 => Parameter::ImmediateAt(pos + i),
                2 => Parameter::RelativeBy(pos + i),
                mode => panic!("Invalid parameter mode: {}", mode),
            });
            info /= 10;
        }

        params
    }

    fn advance(&self, pos: usize) -> usize {
        pos + self.num_params() + 1
    }
}

#[derive(Debug, Clone)]
enum Parameter {
    PositionAt(usize),
    ImmediateAt(usize),
    RelativeBy(usize),
}

#[derive(Debug)]
enum RawComputeResult {
    Store(TapeElem),
    JumpTo(usize),
    Nothing,
    Pause,
}

#[derive(Debug)]
enum ComputeResult {
    StoreAt(ResultAt),
    JumpTo(usize),
    Nothing,
    Pause,
}

#[derive(Debug)]
struct ResultAt {
    pos: usize,
    value: TapeElem,
    relative: bool,
}

impl Instruction {
    fn compute(&self, tape: &mut Intcode) -> ComputeResult {
        use RawComputeResult::*;

        let params: Vec<TapeElem> = self.params.iter().map(|p| tape.get_parameter(p)).collect();
        // eprintln!("Parameters for {:?}: {:?}", self.op, params);

        let value = match self.op {
            Operation::Add => Store(params[0] + params[1]),
            Operation::Multiply => Store(params[0] * params[1]),
            Operation::Input => match tape.input {
                None => Pause,
                Some(value) => {
                    tape.input = None;
                    Store(value)
                }
            },
            Operation::JumpIfTrue => {
                if params[0] != 0 {
                    JumpTo(params[1] as usize)
                } else {
                    Nothing
                }
            }
            Operation::JumpIfFalse => {
                if params[0] == 0 {
                    JumpTo(params[1] as usize)
                } else {
                    Nothing
                }
            }
            Operation::LessThan => {
                if params[0] < params[1] {
                    Store(1)
                } else {
                    Store(0)
                }
            }
            Operation::Equals => {
                if params[0] == params[1] {
                    Store(1)
                } else {
                    Store(0)
                }
            }
            Operation::SetRelativeBase => {
                tape.relative_base += params[0];
                // eprintln!("Set relative_base to {}", tape.relative_base);
                Nothing
            }
            Operation::Output => {
                tape.output.push_back(params[0]);
                Nothing
            }
            _ => panic!("Computing invalid operation, Break should have been caught earlier!"),
        };

        // eprintln!("Computed value is: {:?}", value);

        match value {
            Nothing => ComputeResult::Nothing,
            Pause => ComputeResult::Pause,
            Store(value) => {
                let (pos, relative) =
                    match self.params.last().expect("Output parameter not present!") {
                        Parameter::PositionAt(idx) => (*idx, false),
                        Parameter::RelativeBy(idx) => (*idx, true),
                        _ => panic!("Output parameter should always be positional!"),
                    };
                ComputeResult::StoreAt(ResultAt {
                    pos,
                    value,
                    relative,
                })
            }
            JumpTo(address) => ComputeResult::JumpTo(address),
        }
    }
}

impl Intcode {
    pub fn new(tape: Vec<i64>) -> Intcode {
        Intcode {
            tape,
            pos: 0,
            current: None,
            input: None,
            output: VecDeque::new(),
            finished: false,
            relative_base: 0,
        }
    }

    pub fn load(filename: &str) -> Intcode {
        let code = read_to_string(filename).expect("Could not load Intcode.");

        let code: Vec<i64> = code.split(',').map(|n| n.trim().parse().unwrap()).collect();

        Self::new(code)
    }

    fn get_parameter(&self, param: &Parameter) -> TapeElem {
        match param {
            Parameter::PositionAt(idx) => self.get(self.get(*idx) as usize),
            Parameter::ImmediateAt(idx) => self.get(*idx),
            Parameter::RelativeBy(idx) => {
                /*
                 * eprintln!(
                 *     "Relative index: {} + {} = {} -> {}",
                 *     self.get(*idx),
                 *     self.relative_base,
                 *     (self.get(*idx) + self.relative_base),
                 *     self.get((self.get(*idx) + self.relative_base) as usize)
                 * );
                 */
                self.get((self.get(*idx) + self.relative_base) as usize)
            }
        }
    }

    fn decode(&self, pos: usize) -> Instruction {
        let opcode_full: i64 = self.get(pos);

        // eprintln!("Decoding opcode: {}", opcode_full);

        let op = match opcode_full % 100 {
            1 => Operation::Add,
            2 => Operation::Multiply,
            3 => Operation::Input,
            4 => Operation::Output,
            5 => Operation::JumpIfTrue,
            6 => Operation::JumpIfFalse,
            7 => Operation::LessThan,
            8 => Operation::Equals,
            9 => Operation::SetRelativeBase,
            99 => Operation::Break,
            code => panic!("Encountered invalid opcode: {}", code),
        };

        let info_params = opcode_full / 100;
        let params = op.decode(info_params, pos);

        Instruction { op, params }
    }

    /// Execute tape and return whether we have finished or not.
    fn execute(&mut self) -> bool {
        while self.step() {
            /*
             * eprintln!("Tape so far:");
             * for i in 0..pos + 1 {
             *     eprintln!("#{}: {}", i, self.tape[i]);
             * }
             */
        }
        self.finished
    }

    /// Supply input that is consumed by input instruction
    fn supply_input(&mut self, input: TapeElem) {
        self.input = Some(input);
    }

    /// Get output fo latest output instruction
    fn get_output(&mut self) -> Option<TapeElem> {
        self.output.pop_front()
    }

    fn ensure_len(&mut self, len: usize) {
        if len >= self.tape.len() {
            self.tape.resize(len, 0);
        }
    }

    fn get(&self, idx: usize) -> TapeElem {
        if idx >= self.tape.len() {
            0
        } else {
            self.tape[idx]
        }
    }

    fn store(&mut self, result: &ResultAt) {
        let ResultAt {
            pos,
            value,
            relative,
        } = result;
        let mut idx_target = self.get(*pos);

        if *relative {
            idx_target += self.relative_base;
        }

        self.ensure_len((idx_target + 1) as usize);
        // eprintln!("Storing {} @ position {}", value, idx_target);
        self.tape[idx_target as usize] = *value;
    }

    /// Perform one step in the program and return the next position
    fn step(&mut self) -> bool {
        use ComputeResult::*;

        let instruction = match self.current.clone() {
            None => self.decode(self.pos),
            Some(instruction) => instruction,
        };

        // eprintln!("[Pos: {}] {:?}", self.pos, instruction);

        if let Operation::Break = instruction.op {
            self.finished = true;
            return false;
        }

        match instruction.compute(self) {
            Pause => {
                self.current = Some(instruction);
                return false;
            }
            JumpTo(address) => self.pos = address,
            StoreAt(result_at) => {
                self.store(&result_at);
                self.pos = instruction.op.advance(self.pos);
            }
            _ => {
                self.pos = instruction.op.advance(self.pos);
            }
        }
        self.current = None;
        true
    }

    fn painting_robot() -> Intcode {
        Self::load("painting_robot.intcode")
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Color {
    Black,
    White,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Robot {
    pos: Position,
    orientation: Orientation,
}

struct PaintingGrid {
    robot: Robot,
    grid: HashMap<Position, Color>,
    computer: Intcode,
}

struct Dimensions {
    x_min: i64,
    x_max: i64,
    y_min: i64,
    y_max: i64,
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Orientation::*;
        f.write_str(match self {
            Up => "^",
            Down => "v",
            Right => ">",
            Left => "<",
        })
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Color::*;
        f.write_str(match self {
            Black => ".",
            White => "#",
        })
    }
}

impl Robot {
    fn turn(&mut self, orientation: &Orientation) {
        use Orientation::*;
        self.orientation = match orientation {
            Right => match self.orientation {
                Up => Right,
                Right => Down,
                Down => Left,
                Left => Up,
            },
            Left => match self.orientation {
                Up => Left,
                Left => Down,
                Down => Right,
                Right => Up,
            },
            Up | Down => panic!("Cannot turn up or down."),
        }
    }

    fn forward(&mut self) {
        use Orientation::*;
        match self.orientation {
            Down => self.pos.y += 1,
            Up => self.pos.y -= 1,
            Right => self.pos.x += 1,
            Left => self.pos.x -= 1,
        }
    }
}

impl PaintingGrid {
    fn new() -> PaintingGrid {
        PaintingGrid {
            robot: Robot {
                pos: Position { x: 0, y: 0 },
                orientation: Orientation::Up,
            },
            grid: HashMap::new(),
            computer: Intcode::load("painting_robot.intcode"),
        }
    }

    fn paint_color(&mut self, color: &Color) {
        self.grid.insert(self.robot.pos, *color);
    }

    fn get_current_color(&self) -> Color {
        self.get_color(&self.robot.pos)
    }

    fn get_color(&self, pos: &Position) -> Color {
        match self.grid.get(pos) {
            Some(c) => *c,
            None => Color::Black,
        }
    }

    fn get_dims(&self) -> Dimensions {
        let mut x_min = std::i64::MAX;
        let mut y_min = std::i64::MAX;
        let mut x_max = -std::i64::MAX;
        let mut y_max = -std::i64::MAX;

        for Position { x, y } in self.grid.keys() {
            x_min = min(x_min, *x);
            y_min = min(y_min, *y);
            x_max = max(x_max, *x);
            y_max = max(y_max, *y);
        }

        Dimensions {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    fn execute(&mut self) {
        use Color::*;
        loop {
            self.computer.supply_input(match self.get_current_color() {
                Black => 0,
                White => 1,
            });
            self.computer.execute();
            if self.computer.finished {
                break;
            }
            let output = self
                    .computer
                    .get_output()
                    .expect("Intcode supplied no output!");
            self.paint_color(
                match output
                {
                    0 => &Black,
                    1 => &White,
                    _ => panic!("Intcode supplied wrong output!"),
                },
            );
            /*
             * clear_screen();
             * self.print();
             * std::thread::sleep(std::time::Duration::from_millis(25));
             */
            match self
                .computer
                .get_output()
                .expect("Intcode supplied no output!")
            {
                0 => self.robot.turn(&Orientation::Left),
                1 => self.robot.turn(&Orientation::Right),
                _ => panic!("Intcode supplied wrong output!"),
            };
            self.robot.forward();
        }
    }

    fn print(&self) {
        let dims = self.get_dims();

        for y in dims.y_min..dims.y_max+1
        {
            for x in dims.x_min..dims.x_max+1
            {
                let pos = Position{ x, y};
                if pos == self.robot.pos
                {
                    print!("{}", self.robot.orientation);
                }
                else {
                    print!("{}", self.get_color(&pos));
                }
            }
            println!();
        }
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    {
        let mut painter = PaintingGrid::new();
        painter.execute();
        let num_panels = painter.grid.keys().count();
        painter.print();
        println!();
        println!("Number of panels: {}", num_panels);
    }
    {
        let mut painter = PaintingGrid::new();
        painter.paint_color(&Color::White);
        painter.execute();
        painter.print();
    }
}

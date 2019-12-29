
use std::collections::VecDeque;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
pub struct Intcode {
    tape: Vec<TapeElem>,
    tape_init: Vec<TapeElem>,
    pos: usize,
    current: Option<Instruction>,
    input: VecDeque<TapeElem>,
    output: VecDeque<TapeElem>,
    finished: bool,
    relative_base: TapeElem,
}

pub type TapeElem = i64;

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
            Operation::Input => match tape.input.pop_front() {
                None => Pause,
                Some(value) => {
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
            tape_init: tape.clone(),
            tape,
            pos: 0,
            current: None,
            input: VecDeque::new(),
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

    pub fn is_finished(&self) -> bool
    {
        self.finished
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

    pub fn execute_n(&mut self, n: usize) -> bool {
        for _ in 0..n {
            if !self.step() {
                break;
            }
        }
        self.finished
    }

    /// Execute tape and return whether we have finished or not.
    pub fn execute(&mut self) -> bool {
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
    pub fn supply_input(&mut self, input: TapeElem) {
        self.input.push_back(input);
    }

    pub fn set(&mut self, idx: usize, value: TapeElem)
    {
        self.tape[idx] = value;
    }

    /// Get output fo latest output instruction
    pub fn get_output(&mut self) -> Option<TapeElem> {
        self.output.pop_front()
    }

    pub fn output_avail(&self) -> bool {
        self.output.len() > 0
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

    pub fn reset(&mut self) 
    {
        self.pos = 0;
        self.finished = false;
        self.tape = self.tape_init.clone();
    }
}

use std::io;

pub struct Tape {
    tape: Vec<TapeElem>,
}

type TapeElem = i64;

#[derive(Debug)]
enum Operation {
    Multiply,
    Add,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Break,
}

#[derive(Debug)]
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
            Operation::Break => 0,
        }
    }

    fn decode(&self, info: i64, pos: usize) -> Vec<Parameter> {
        let mut params = Vec::with_capacity(self.num_params());
        let mut info = info;

        for i in 1..self.num_params() + 1 {
            params.push(match info % 2 {
                0 => Parameter::PositionAt(pos + i),
                1 => Parameter::ImmediateAt(pos + i),
                mode => panic!("Invalid parameter mode: {}", mode),
            });
            info /= 10;
        }

        params
    }
}

#[derive(Debug)]
enum Parameter {
    PositionAt(usize),
    ImmediateAt(usize),
}

#[derive(Debug)]
enum RawComputeResult {
    Store(TapeElem),
    JumpTo(usize),
    Nothing,
}

#[derive(Debug)]
enum ComputeResult {
    StoreAt(ResultAt),
    JumpTo(usize),
    Nothing,
}

#[derive(Debug)]
struct ResultAt {
    pos: usize,
    value: TapeElem,
}

impl Instruction {
    fn compute(&self, tape: &Tape) -> ComputeResult {
        use RawComputeResult::*;

        let params: Vec<TapeElem> = self.params.iter().map(|p| tape.get_parameter(p)).collect();
        eprintln!("Parameters for {:?}: {:?}", self.op, params);

        let value = match self.op {
            Operation::Add => Store(params[0] + params[1]),
            Operation::Multiply => Store(params[0] * params[1]),
            Operation::Input => {
                println!("Please provide input:");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to get input");
                let value: TapeElem = input
                    .trim()
                    .parse()
                    .expect("Could not cast input to integer.");
                Store(value)
            }
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
            Operation::Output => {
                println!("{}", params[0]);
                Nothing
            }
            _ => panic!("Computing invalid operation, Break should have been caught earlier!"),
        };

        eprintln!("Computed value is: {:?}", value);

        match value {
            Nothing => ComputeResult::Nothing,
            Store(value) => {
                let pos = match self.params.last().expect("Output parameter not present!") {
                    Parameter::PositionAt(idx) => *idx,
                    _ => panic!("Output parameter should always be positional!"),
                };
                ComputeResult::StoreAt(ResultAt{pos, value})
            }
            JumpTo(address) => ComputeResult::JumpTo(address),
        }
    }
}

impl Tape {
    pub fn new(tape: Vec<i64>) -> Tape {
        Tape { tape }
    }

    fn get_parameter(&self, param: &Parameter) -> TapeElem {
        match param {
            Parameter::PositionAt(idx) => self.tape[self.tape[*idx] as usize],
            Parameter::ImmediateAt(idx) => self.tape[*idx],
        }
    }

    fn decode(&self, pos: usize) -> Instruction {
        let opcode_full: i64 = self.tape[pos];

        eprintln!("Decoding opcode: {}", opcode_full);

        let op = match opcode_full % 100 {
            1 => Operation::Add,
            2 => Operation::Multiply,
            3 => Operation::Input,
            4 => Operation::Output,
            5 => Operation::JumpIfTrue,
            6 => Operation::JumpIfFalse,
            7 => Operation::LessThan,
            8 => Operation::Equals,
            99 => Operation::Break,
            code => panic!("Encountered invalid opcode: {}", code),
        };

        let info_params = opcode_full / 100;
        let params = op.decode(info_params, pos);

        Instruction { op, params }
    }

    fn execute(&mut self) {
        let mut pos = 0;

        loop {
            /*
             * eprintln!("Tape so far:");
             * for i in 0..pos + 1 {
             *     eprintln!("#{}: {}", i, self.tape[i]);
             * }
             */

            match self.step(pos) {
                Some(new_pos) => pos = new_pos,
                None => break,
            }
        }
    }

    fn store(&mut self, result: &ResultAt) {
        let ResultAt { pos, value } = result;

        let idx_target = self.tape[*pos] as usize;
        eprintln!("Storing {} @ position {}", value, idx_target);
        self.tape[idx_target] = *value;
    }

    /// Perform one step in the program and return the next position
    fn step(&mut self, pos: usize) -> Option<usize> {
        use ComputeResult::*;

        let instruction = self.decode(pos);

        eprintln!("[Pos: {}] Instruction: {:?}", pos, instruction);

        if let Operation::Break = instruction.op {
            return None;
        }

        let result = instruction.compute(self);

        if let JumpTo(address) = result {
            Some(address)
        }
        else {
            if let StoreAt(result_at) = result
            {
                self.store(&result_at);
            }
            Some(pos + instruction.op.num_params() + 1)
        }
    }
}

fn main() {
    let mut tape = Tape::new(vec![
        3, 225, 1, 225, 6, 6, 1100, 1, 238, 225, 104, 0, 101, 67, 166, 224, 1001, 224, -110, 224,
        4, 224, 102, 8, 223, 223, 1001, 224, 4, 224, 1, 224, 223, 223, 2, 62, 66, 224, 101, -406,
        224, 224, 4, 224, 102, 8, 223, 223, 101, 3, 224, 224, 1, 224, 223, 223, 1101, 76, 51, 225,
        1101, 51, 29, 225, 1102, 57, 14, 225, 1102, 64, 48, 224, 1001, 224, -3072, 224, 4, 224,
        102, 8, 223, 223, 1001, 224, 1, 224, 1, 224, 223, 223, 1001, 217, 90, 224, 1001, 224, -101,
        224, 4, 224, 1002, 223, 8, 223, 1001, 224, 2, 224, 1, 223, 224, 223, 1101, 57, 55, 224,
        1001, 224, -112, 224, 4, 224, 102, 8, 223, 223, 1001, 224, 7, 224, 1, 223, 224, 223, 1102,
        5, 62, 225, 1102, 49, 68, 225, 102, 40, 140, 224, 101, -2720, 224, 224, 4, 224, 1002, 223,
        8, 223, 1001, 224, 4, 224, 1, 223, 224, 223, 1101, 92, 43, 225, 1101, 93, 21, 225, 1002,
        170, 31, 224, 101, -651, 224, 224, 4, 224, 102, 8, 223, 223, 101, 4, 224, 224, 1, 223, 224,
        223, 1, 136, 57, 224, 1001, 224, -138, 224, 4, 224, 102, 8, 223, 223, 101, 2, 224, 224, 1,
        223, 224, 223, 1102, 11, 85, 225, 4, 223, 99, 0, 0, 0, 677, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1105, 0, 99999, 1105, 227, 247, 1105, 1, 99999, 1005, 227, 99999, 1005, 0, 256, 1105, 1,
        99999, 1106, 227, 99999, 1106, 0, 265, 1105, 1, 99999, 1006, 0, 99999, 1006, 227, 274,
        1105, 1, 99999, 1105, 1, 280, 1105, 1, 99999, 1, 225, 225, 225, 1101, 294, 0, 0, 105, 1, 0,
        1105, 1, 99999, 1106, 0, 300, 1105, 1, 99999, 1, 225, 225, 225, 1101, 314, 0, 0, 106, 0, 0,
        1105, 1, 99999, 1107, 226, 226, 224, 102, 2, 223, 223, 1006, 224, 329, 1001, 223, 1, 223,
        1007, 226, 677, 224, 1002, 223, 2, 223, 1005, 224, 344, 101, 1, 223, 223, 108, 677, 677,
        224, 1002, 223, 2, 223, 1006, 224, 359, 101, 1, 223, 223, 1008, 226, 226, 224, 1002, 223,
        2, 223, 1005, 224, 374, 1001, 223, 1, 223, 108, 677, 226, 224, 1002, 223, 2, 223, 1006,
        224, 389, 101, 1, 223, 223, 7, 226, 226, 224, 102, 2, 223, 223, 1006, 224, 404, 101, 1,
        223, 223, 7, 677, 226, 224, 1002, 223, 2, 223, 1005, 224, 419, 101, 1, 223, 223, 107, 226,
        226, 224, 102, 2, 223, 223, 1006, 224, 434, 1001, 223, 1, 223, 1008, 677, 677, 224, 1002,
        223, 2, 223, 1005, 224, 449, 101, 1, 223, 223, 108, 226, 226, 224, 102, 2, 223, 223, 1005,
        224, 464, 1001, 223, 1, 223, 1108, 226, 677, 224, 1002, 223, 2, 223, 1005, 224, 479, 1001,
        223, 1, 223, 8, 677, 226, 224, 102, 2, 223, 223, 1006, 224, 494, 1001, 223, 1, 223, 1108,
        677, 677, 224, 102, 2, 223, 223, 1006, 224, 509, 1001, 223, 1, 223, 1007, 226, 226, 224,
        1002, 223, 2, 223, 1005, 224, 524, 1001, 223, 1, 223, 7, 226, 677, 224, 1002, 223, 2, 223,
        1005, 224, 539, 1001, 223, 1, 223, 8, 677, 677, 224, 102, 2, 223, 223, 1005, 224, 554,
        1001, 223, 1, 223, 107, 226, 677, 224, 1002, 223, 2, 223, 1006, 224, 569, 101, 1, 223, 223,
        1107, 226, 677, 224, 102, 2, 223, 223, 1005, 224, 584, 1001, 223, 1, 223, 1108, 677, 226,
        224, 102, 2, 223, 223, 1006, 224, 599, 1001, 223, 1, 223, 1008, 677, 226, 224, 102, 2, 223,
        223, 1006, 224, 614, 101, 1, 223, 223, 107, 677, 677, 224, 102, 2, 223, 223, 1006, 224,
        629, 1001, 223, 1, 223, 1107, 677, 226, 224, 1002, 223, 2, 223, 1005, 224, 644, 101, 1,
        223, 223, 8, 226, 677, 224, 102, 2, 223, 223, 1005, 224, 659, 1001, 223, 1, 223, 1007, 677,
        677, 224, 102, 2, 223, 223, 1005, 224, 674, 1001, 223, 1, 223, 4, 223, 99, 226,
    ]);

    tape.execute();
}

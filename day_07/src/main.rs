use permutohedron::heap_recursive;

#[derive(Clone)]
pub struct Intcode {
    tape: Vec<TapeElem>,
    pos: usize,
    current: Option<Instruction>,
    input: Option<TapeElem>,
    output: Option<TapeElem>,
    finished: bool,
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

    fn advance(&self, pos: usize) -> usize {
        pos + self.num_params() + 1
    }
}

#[derive(Debug, Clone)]
enum Parameter {
    PositionAt(usize),
    ImmediateAt(usize),
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
            Operation::Output => {
                tape.output = Some(params[0]);
                Nothing
            }
            _ => panic!("Computing invalid operation, Break should have been caught earlier!"),
        };

        // eprintln!("Computed value is: {:?}", value);

        match value {
            Nothing => ComputeResult::Nothing,
            Pause => ComputeResult::Pause,
            Store(value) => {
                let pos = match self.params.last().expect("Output parameter not present!") {
                    Parameter::PositionAt(idx) => *idx,
                    _ => panic!("Output parameter should always be positional!"),
                };
                ComputeResult::StoreAt(ResultAt { pos, value })
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
            output: None,
            finished: false,
        }
    }

    fn get_parameter(&self, param: &Parameter) -> TapeElem {
        match param {
            Parameter::PositionAt(idx) => self.tape[self.tape[*idx] as usize],
            Parameter::ImmediateAt(idx) => self.tape[*idx],
        }
    }

    fn decode(&self, pos: usize) -> Instruction {
        let opcode_full: i64 = self.tape[pos];

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
    fn get_output(&self) -> Option<TapeElem> {
        self.output
    }

    fn store(&mut self, result: &ResultAt) {
        let ResultAt { pos, value } = result;

        let idx_target = self.tape[*pos] as usize;
        // eprintln!("Storing {} @ position {}", value, idx_target);
        self.tape[idx_target] = *value;
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

    fn amplifier() -> Intcode {
        Intcode::new(vec![
            3, 8, 1001, 8, 10, 8, 105, 1, 0, 0, 21, 42, 67, 88, 101, 114, 195, 276, 357, 438,
            99999, 3, 9, 101, 3, 9, 9, 1002, 9, 4, 9, 1001, 9, 5, 9, 102, 4, 9, 9, 4, 9, 99, 3, 9,
            1001, 9, 3, 9, 1002, 9, 2, 9, 101, 2, 9, 9, 102, 2, 9, 9, 1001, 9, 5, 9, 4, 9, 99, 3,
            9, 102, 4, 9, 9, 1001, 9, 3, 9, 102, 4, 9, 9, 101, 4, 9, 9, 4, 9, 99, 3, 9, 101, 2, 9,
            9, 1002, 9, 3, 9, 4, 9, 99, 3, 9, 101, 4, 9, 9, 1002, 9, 5, 9, 4, 9, 99, 3, 9, 102, 2,
            9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9,
            4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3,
            9, 1002, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 99, 3, 9,
            102, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 1002,
            9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 2,
            9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4,
            9, 99, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9,
            3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9,
            1001, 9, 1, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1001, 9,
            1, 9, 4, 9, 99, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 1001, 9, 1,
            9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9,
            3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9,
            1001, 9, 1, 9, 4, 9, 99, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9,
            1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002,
            9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 2, 9,
            9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 99,
        ])
    }
}

fn run_amplifiers(phase_settings: Vec<TapeElem>) -> TapeElem {
    let mut current = 0;
    for (idx, phase) in phase_settings.iter().enumerate() {
        let mut amplifier = Intcode::amplifier();
        amplifier.supply_input(*phase);
        amplifier.execute();
        amplifier.supply_input(current);
        if !amplifier.execute() {
            panic!("Amplifier did not finish!");
        }
        current = amplifier
            .get_output()
            .expect(format!("Amplifier #{} did not produce any output", idx + 1).as_str());
    }

    current
}
fn run_amplifiers_loop(phase_settings: Vec<TapeElem>) -> TapeElem {
    let mut current = 0;
    let mut amplifiers: Vec<Intcode> = Vec::new();
    for phase in phase_settings.iter()
    {
        let mut amp = Intcode::amplifier();
        amp.supply_input(*phase);
        amp.execute();
        amplifiers.push(amp);
    }

    while !amplifiers[phase_settings.len()-1].finished
    {
        for (idx, amp) in amplifiers.iter_mut().enumerate()
        {
            amp.supply_input(current);
            amp.execute();
            current = amp.get_output().expect(format!("Amplifier #{} did not produce any output", idx + 1).as_str());
        }
    }
    current
}

fn main() {
    {
        let mut data = [0, 1, 2, 3, 4];
        let mut max_value = 0;
        heap_recursive(&mut data, |permutation| {
            max_value = std::cmp::max(run_amplifiers(permutation.to_vec()), max_value);
        });
        println!("Max value: {}", max_value);
    }

    {
        let mut data = [5, 6, 7, 8, 9];
        let mut max_value = 0;
        heap_recursive(&mut data, |permutation| {
            max_value = std::cmp::max(run_amplifiers_loop(permutation.to_vec()), max_value);
        });
        println!("Max value (loop): {}", max_value);
    }
}

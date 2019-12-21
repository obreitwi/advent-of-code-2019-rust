#[derive(Clone)]
pub struct Intcode {
    tape: Vec<TapeElem>,
    pos: usize,
    current: Option<Instruction>,
    input: Option<TapeElem>,
    output: Option<TapeElem>,
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
                eprintln!("Set relative_base to {}", tape.relative_base);
                Nothing
            }
            Operation::Output => {
                tape.output = Some(params[0]);
                println!("{}", tape.output.unwrap());
                Nothing
            }
            _ => panic!("Computing invalid operation, Break should have been caught earlier!"),
        };

        // eprintln!("Computed value is: {:?}", value);

        match value {
            Nothing => ComputeResult::Nothing,
            Pause => ComputeResult::Pause,
            Store(value) => {
                let (pos, relative) = match self.params.last().expect("Output parameter not present!") {
                    Parameter::PositionAt(idx) => (*idx, false),
                    Parameter::RelativeBy(idx) => (*idx, true),
                    _ => panic!("Output parameter should always be positional!"),
                };
                ComputeResult::StoreAt(ResultAt { pos, value, relative })
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
            relative_base: 0,
        }
    }

    fn get_parameter(&self, param: &Parameter) -> TapeElem {
        match param {
            Parameter::PositionAt(idx) => self.get(self.get(*idx) as usize),
            Parameter::ImmediateAt(idx) => self.get(*idx),
            Parameter::RelativeBy(idx) => {
                eprintln!(
                    "Relative index: {} + {} = {} -> {}",
                    self.get(*idx),
                    self.relative_base,
                    (self.get(*idx) + self.relative_base),
                    self.get((self.get(*idx) + self.relative_base) as usize)
                );
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
    fn get_output(&self) -> Option<TapeElem> {
        self.output
    }

    fn ensure_len(&mut self, len: usize)
    {
        if len >= self.tape.len()
        {
            self.tape.resize(len, 0);
        }
    }

    fn get(&self, idx: usize) -> TapeElem
    {
        if idx >= self.tape.len()
        {
            0
        }
        else 
        {
            self.tape[idx]
        }
    }

    fn store(&mut self, result: &ResultAt) {
        let ResultAt { pos, value, relative } = result;
        let mut idx_target = self.get(*pos);

        if *relative
        {
            idx_target += self.relative_base;
        }

        self.ensure_len((idx_target+1) as usize);
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

        eprintln!("[Pos: {}] {:?}", self.pos, instruction);

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

    fn boost() -> Intcode {
        Intcode::new(vec![
            1102, 34463338, 34463338, 63, 1007, 63, 34463338, 63, 1005, 63, 53, 1101, 0, 3, 1000,
            109, 988, 209, 12, 9, 1000, 209, 6, 209, 3, 203, 0, 1008, 1000, 1, 63, 1005, 63, 65,
            1008, 1000, 2, 63, 1005, 63, 904, 1008, 1000, 0, 63, 1005, 63, 58, 4, 25, 104, 0, 99,
            4, 0, 104, 0, 99, 4, 17, 104, 0, 99, 0, 0, 1102, 1, 32, 1016, 1102, 326, 1, 1029, 1102,
            1, 26, 1009, 1102, 1, 753, 1024, 1102, 1, 1, 1021, 1102, 35, 1, 1000, 1102, 1, 0, 1020,
            1101, 25, 0, 1012, 1102, 36, 1, 1011, 1101, 0, 33, 1013, 1102, 1, 667, 1022, 1102, 1,
            38, 1014, 1102, 1, 24, 1017, 1101, 0, 31, 1004, 1102, 443, 1, 1026, 1101, 37, 0, 1015,
            1101, 27, 0, 1007, 1101, 0, 748, 1025, 1102, 1, 23, 1008, 1102, 1, 34, 1002, 1101, 28,
            0, 1006, 1102, 1, 22, 1003, 1101, 0, 29, 1005, 1101, 0, 39, 1018, 1101, 21, 0, 1019,
            1102, 30, 1, 1001, 1102, 660, 1, 1023, 1102, 1, 331, 1028, 1101, 0, 440, 1027, 1101, 0,
            20, 1010, 109, 18, 1206, 2, 195, 4, 187, 1105, 1, 199, 1001, 64, 1, 64, 1002, 64, 2,
            64, 109, -12, 1208, 0, 28, 63, 1005, 63, 217, 4, 205, 1105, 1, 221, 1001, 64, 1, 64,
            1002, 64, 2, 64, 109, 3, 2101, 0, -5, 63, 1008, 63, 31, 63, 1005, 63, 247, 4, 227,
            1001, 64, 1, 64, 1106, 0, 247, 1002, 64, 2, 64, 109, -7, 2101, 0, 6, 63, 1008, 63, 26,
            63, 1005, 63, 267, 1105, 1, 273, 4, 253, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 10,
            21108, 40, 40, 4, 1005, 1016, 295, 4, 279, 1001, 64, 1, 64, 1106, 0, 295, 1002, 64, 2,
            64, 109, -9, 2107, 23, 0, 63, 1005, 63, 315, 1001, 64, 1, 64, 1105, 1, 317, 4, 301,
            1002, 64, 2, 64, 109, 30, 2106, 0, -5, 4, 323, 1105, 1, 335, 1001, 64, 1, 64, 1002, 64,
            2, 64, 109, -19, 1202, -9, 1, 63, 1008, 63, 26, 63, 1005, 63, 355, 1106, 0, 361, 4,
            341, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -5, 21107, 41, 42, 6, 1005, 1015, 379, 4,
            367, 1105, 1, 383, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -6, 21108, 42, 43, 8, 1005,
            1011, 403, 1001, 64, 1, 64, 1105, 1, 405, 4, 389, 1002, 64, 2, 64, 109, 11, 21102, 43,
            1, 1, 1008, 1015, 42, 63, 1005, 63, 425, 1106, 0, 431, 4, 411, 1001, 64, 1, 64, 1002,
            64, 2, 64, 109, 13, 2106, 0, 0, 1105, 1, 449, 4, 437, 1001, 64, 1, 64, 1002, 64, 2, 64,
            109, 1, 1205, -7, 463, 4, 455, 1106, 0, 467, 1001, 64, 1, 64, 1002, 64, 2, 64, 109,
            -14, 1206, 7, 479, 1105, 1, 485, 4, 473, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -6,
            1202, 0, 1, 63, 1008, 63, 23, 63, 1005, 63, 507, 4, 491, 1106, 0, 511, 1001, 64, 1, 64,
            1002, 64, 2, 64, 109, 13, 1205, -1, 523, 1106, 0, 529, 4, 517, 1001, 64, 1, 64, 1002,
            64, 2, 64, 109, -23, 2107, 22, 10, 63, 1005, 63, 551, 4, 535, 1001, 64, 1, 64, 1106, 0,
            551, 1002, 64, 2, 64, 109, 14, 21101, 44, 0, 6, 1008, 1018, 44, 63, 1005, 63, 577, 4,
            557, 1001, 64, 1, 64, 1106, 0, 577, 1002, 64, 2, 64, 109, -12, 2108, 32, 0, 63, 1005,
            63, 597, 1001, 64, 1, 64, 1105, 1, 599, 4, 583, 1002, 64, 2, 64, 109, 7, 1201, -4, 0,
            63, 1008, 63, 20, 63, 1005, 63, 619, 1106, 0, 625, 4, 605, 1001, 64, 1, 64, 1002, 64,
            2, 64, 109, -11, 1201, 6, 0, 63, 1008, 63, 34, 63, 1005, 63, 647, 4, 631, 1106, 0, 651,
            1001, 64, 1, 64, 1002, 64, 2, 64, 109, 20, 2105, 1, 7, 1001, 64, 1, 64, 1106, 0, 669,
            4, 657, 1002, 64, 2, 64, 109, -4, 21101, 45, 0, 6, 1008, 1018, 46, 63, 1005, 63, 689,
            1106, 0, 695, 4, 675, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, -16, 2108, 22, 7, 63,
            1005, 63, 717, 4, 701, 1001, 64, 1, 64, 1105, 1, 717, 1002, 64, 2, 64, 109, 10, 1207,
            0, 27, 63, 1005, 63, 733, 1105, 1, 739, 4, 723, 1001, 64, 1, 64, 1002, 64, 2, 64, 109,
            8, 2105, 1, 10, 4, 745, 1105, 1, 757, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 1, 21102,
            46, 1, -2, 1008, 1013, 46, 63, 1005, 63, 779, 4, 763, 1106, 0, 783, 1001, 64, 1, 64,
            1002, 64, 2, 64, 109, -2, 1208, -7, 29, 63, 1005, 63, 799, 1105, 1, 805, 4, 789, 1001,
            64, 1, 64, 1002, 64, 2, 64, 109, -19, 2102, 1, 10, 63, 1008, 63, 32, 63, 1005, 63, 829,
            1001, 64, 1, 64, 1106, 0, 831, 4, 811, 1002, 64, 2, 64, 109, 14, 1207, -2, 29, 63,
            1005, 63, 849, 4, 837, 1105, 1, 853, 1001, 64, 1, 64, 1002, 64, 2, 64, 109, 8, 21107,
            47, 46, -6, 1005, 1010, 873, 1001, 64, 1, 64, 1106, 0, 875, 4, 859, 1002, 64, 2, 64,
            109, -17, 2102, 1, 6, 63, 1008, 63, 29, 63, 1005, 63, 901, 4, 881, 1001, 64, 1, 64,
            1106, 0, 901, 4, 64, 99, 21102, 1, 27, 1, 21102, 1, 915, 0, 1106, 0, 922, 21201, 1,
            27817, 1, 204, 1, 99, 109, 3, 1207, -2, 3, 63, 1005, 63, 964, 21201, -2, -1, 1, 21101,
            0, 942, 0, 1105, 1, 922, 21202, 1, 1, -1, 21201, -2, -3, 1, 21102, 1, 957, 0, 1105, 1,
            922, 22201, 1, -1, -2, 1106, 0, 968, 22102, 1, -2, -2, 109, -3, 2105, 1, 0,
        ])
    }

    fn copy_myself() -> Intcode {
        Intcode::new(vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ])
    }

    fn print_16_digits() -> Intcode {
        Intcode::new(vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0])
    }
}

fn main() {
    {
        let mut copy = Intcode::copy_myself();
        println!("Copy: {:?}", copy.tape);
        copy.execute();
    }
    {
        let mut copy = Intcode::print_16_digits();
        println!("16 digits");
        copy.execute();
    }
    {
        let mut boost = Intcode::boost();

        println!("BOOST");
        boost.supply_input(1);
        boost.execute();
    }
    {
        let mut boost = Intcode::boost();

        println!("BOOST");
        boost.supply_input(2);
        boost.execute();
    }
}

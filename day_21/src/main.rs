use std::fmt;

mod intcode;

use intcode::{Intcode, TapeElem};

#[derive(Debug)]
struct Springdroid {
    computer: Intcode,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    op: Operation,
    src: Register,
    tgt: Register,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Copy)]
enum Register {
    Temp,
    Jump,
    Read1,
    Read2,
    Read3,
    Read4,
    Read5,
    Read6,
    Read7,
    Read8,
    Read9,
}

impl Instruction {
    fn new(op: Operation, src: Register, tgt: Register) -> Self {
        Instruction { op, src, tgt }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{} {} {}", self.op, self.src, self.tgt))
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Operation::*;
        f.write_str(match self {
            And => "AND",
            Or => "OR",
            Not => "NOT",
        })
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Register::*;
        f.write_str(match self {
            Temp => "T",
            Jump => "J",
            Read1 => "A",
            Read2 => "B",
            Read3 => "C",
            Read4 => "D",
            Read5 => "E",
            Read6 => "F",
            Read7 => "G",
            Read8 => "H",
            Read9 => "I",
        })
    }
}

impl Springdroid {
    fn new(filename: &str) -> Self {
        Springdroid {
            computer: Intcode::load(filename),
            instructions: Vec::new(),
        }
    }

    fn walk(&mut self) {
        let check_register = |r: &Register| -> bool {
            use Register::*;
            match r {
                Read5 | Read6 | Read7 | Read8 | Read9 => false,
                _ => true,
            }
        };
        let check_instruction = |instr: &Instruction| {
            let Instruction { src, tgt, .. } = instr;
            check_register(src) && check_register(tgt)
        };

        self.operate("WALK\n", check_instruction);
    }

    fn run(&mut self) {
        self.operate("RUN\n", |_: &Instruction| -> bool { true });
    }

    fn operate<F>(&mut self, start_cmd: &str, validator: F)
    where
        F: Fn(&Instruction) -> bool,
    {
        eprintln!("Number of instructions: {}", self.instructions.len());

        for instr in self.instructions.iter() {
            assert!(validator(instr), "Invalid instruction supplied.");
            for c in format!("{}\n", instr).chars() {
                self.computer.supply_input(c as TapeElem)
            }
        }

        for c in start_cmd.chars() {
            self.computer.supply_input(c as TapeElem)
        }

        self.computer.execute();

        while let Some(output) = self.computer.get_output() {
            if output < 127 {
                print!("{}", output as u8 as char);
            } else {
                println!("{}", output);
            }
        }
    }

    fn add(&mut self, instr: Instruction) {
        assert!(
            self.instructions.len() < 15,
            "Cannot store more instructions!"
        );
        self.instructions.push(instr);
    }
}

fn clear_screen() {
    // print!("{}[2J", 27 as char);
    print!("\x1B[2J");
}

fn main() {
    // part A
    {
        let mut jumper = Springdroid::new("input.txt");
        use Operation::*;
        use Register::*;

        // big gap
        jumper.add(Instruction::new(Not, Read1, Jump));
        jumper.add(Instruction::new(And, Read2, Jump));
        jumper.add(Instruction::new(Not, Read3, Temp));
        jumper.add(Instruction::new(And, Temp, Jump));
        jumper.add(Instruction::new(Not, Read4, Temp));
        jumper.add(Instruction::new(And, Temp, Jump));

        // small gap at the end
        jumper.add(Instruction::new(Not, Read3, Temp));
        jumper.add(Instruction::new(And, Read4, Temp));
        jumper.add(Instruction::new(Or, Temp, Jump));

        // small gap at the beginningj
        jumper.add(Instruction::new(Not, Read1, Temp));
        jumper.add(Instruction::new(And, Read4, Temp));
        jumper.add(Instruction::new(Or, Temp, Jump));
        jumper.walk();
    }
    // part B
    {
        let mut jumper = Springdroid::new("input.txt");
        use Operation::*;
        use Register::*;

        jumper.add(Instruction::new(Or, Read5, Jump));
        jumper.add(Instruction::new(And, Read9, Jump));
        jumper.add(Instruction::new(Or, Read8, Jump));
        jumper.add(Instruction::new(Or, Read5, Temp));
        jumper.add(Instruction::new(And, Read6, Temp));
        jumper.add(Instruction::new(Or, Temp, Jump));
        jumper.add(Instruction::new(And, Read4, Jump));

        jumper.add(Instruction::new(Not, Read1, Temp));
        jumper.add(Instruction::new(Not, Temp, Temp));
        jumper.add(Instruction::new(And, Read2, Temp));
        jumper.add(Instruction::new(And, Read3, Temp));
        jumper.add(Instruction::new(Not, Temp, Temp));

        jumper.add(Instruction::new(And, Temp, Jump));

        jumper.run();
    }
}

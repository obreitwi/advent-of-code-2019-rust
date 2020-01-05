use std::collections::{HashSet, VecDeque};

mod intcode;

use intcode::{Intcode, TapeElem};

#[derive(Debug)]
struct NIC {
    computer: Intcode,
    address: TapeElem,
    input: VecDeque<Message>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Message {
    from: TapeElem,
    to: TapeElem,
    x: TapeElem,
    y: TapeElem,
}

#[derive(Debug)]
struct Network {
    nics: Vec<NIC>,
    nat: Message,
}

impl NIC {
    pub fn new(filename: &str, address: TapeElem) -> NIC {
        let mut computer = Intcode::load(filename);
        computer.supply_input(address);
        NIC {
            computer: computer,
            address: address,
            input: VecDeque::new(),
        }
    }

    pub fn deliver(&mut self, msg: Message) {
        assert_eq!(msg.to, self.address, "Wrong to-address in message!");
        self.input.push_back(msg);
    }

    pub fn retrieve(&mut self) -> Option<Message> {
        match self.computer.get_output() {
            None => None,
            Some(to) => {
                let x = self.computer.get_output().expect("X missing from message!");
                let y = self.computer.get_output().expect("X missing from message!");
                Some(Message {
                    x,
                    y,
                    to,
                    from: self.address,
                })
            }
        }
    }

    pub fn execute(&mut self) {
        if self.input.len() > 0 {
            while let Some(msg) = self.input.pop_front() {
                self.computer.supply_input(msg.x);
                self.computer.supply_input(msg.y);
            }
        } else {
            self.computer.supply_input(-1);
        }
        self.computer.execute();
    }
}

impl Network {
    fn new(filename: &str, num: usize) -> Network {
        let mut nics = Vec::with_capacity(num);

        for addr in 0..num {
            nics.push(NIC::new(filename, addr as TapeElem));
        }

        Network {
            nics,
            nat: Message {
                from: 255,
                to: 0,
                x: -1,
                y: -1,
            },
        }
    }

    /// needed for part A
    fn execute_till_condition<F>(&mut self, mut condition: F)
    where
        F: FnMut(&Message) -> bool,
    {
        let mut potentially_idle = false;

        loop {
            let mut messages = VecDeque::new();
            for nic in self.nics.iter_mut() {
                nic.execute();
                while let Some(msg) = nic.retrieve() {
                    messages.push_back(msg);
                }
            }

            let mut break_after_delivery = false;

            if potentially_idle && messages.len() == 0 {
                // we are idle -> deliver nat message
                messages.push_back(self.nat.clone());
            } else if messages.len() == 0 {
                potentially_idle = true;
            } else {
                potentially_idle = false;
            }

            while let Some(msg) = messages.pop_front() {
                if condition(&msg) {
                    println!("{:?}", msg);
                    break_after_delivery = true;
                }
                if msg.to == 255 {
                    self.set_nat(msg);
                } else {
                    self.nics[msg.to as usize].deliver(msg);
                }
            }
            if break_after_delivery {
                break;
            }
        }
    }

    fn set_nat(&mut self, mut msg: Message) {
        msg.to = 0;
        msg.from = 255;
        self.nat = msg;
    }
}

fn main() {
    {
        let mut net = Network::new(
            &std::env::args()
                .skip(1)
                .next()
                .expect("No filename supplied."),
            50,
        );

        net.execute_till_condition(|msg| msg.to == 255);
    }
    {
        let mut net = Network::new(
            &std::env::args()
                .skip(1)
                .next()
                .expect("No filename supplied."),
            50,
        );

        let mut seen = HashSet::new();
        let condition = |msg: &Message| -> bool {
            if msg.from == 255 {
                if seen.contains(msg) {
                    true
                } else {
                    seen.clear();
                    seen.insert(msg.clone());
                    false
                }
            } else {
                false
            }
        };
        net.execute_till_condition(condition);
        eprintln!("{:?}", seen);
    }
}

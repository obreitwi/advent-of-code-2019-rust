use std::cmp::{max,min};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::ops::Mul;
use std::str::FromStr;

#[derive(Debug)]
struct Reaction {
    input: Vec<Chemical>,
    output: Chemical,
}

#[derive(Debug, Clone)]
struct Chemical {
    name: String,
    quantity: usize,
}

#[derive(Debug)]
struct Workbench {
    producers: HashMap<String, Reaction>,
}

impl FromStr for Chemical {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.trim().split(' ');
        let quantity = str::parse(split.next().expect("quantity missing"))?;
        let name = String::from(split.next().expect("name missing"));

        Ok(Chemical { name, quantity })
    }
}

impl FromStr for Reaction {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reactions: Vec<&str> = s.split("=>").collect();
        assert_eq!(reactions.len(), 2);

        let mut input = Vec::new();

        for s_input in reactions[0].split(",") {
            input.push(str::parse(s_input).expect("invalid input"));
        }

        let output = str::parse(reactions[1]).expect("invalid output");

        Ok(Reaction { input, output })
    }
}

impl Mul<usize> for Chemical {
    type Output = Self;

    fn mul(self, scalar: usize) -> Self::Output {
        Chemical {
            quantity: self.quantity * scalar,
            name: self.name,
        }
    }
}

impl Workbench {
    fn new(filename: &str) -> Workbench {
        let input = read_to_string(filename).expect("Could not read input file.");
        let producers = input
            .lines()
            .map(str::parse)
            .filter_map(Result::ok)
            .map(|r: Reaction| (r.output.name.clone(), r))
            .collect();

        Workbench {
            producers,
        }
    }

    fn compute_fuel_for_ore(&self, ore_available: usize) -> usize {
        let mut fuel_min = 0;
        let mut fuel_max = ore_available;
        let mut fuel_current = 1;

        loop {
            let ore = self.compute_fuel(fuel_current);

            if ore > ore_available
            {
                fuel_max = fuel_current;
                let diff = (fuel_current - fuel_min)/2;
                fuel_current -= max(1, diff);

            }
            else if ore < ore_available
            {
                fuel_min = fuel_current;
                let diff = (fuel_max - fuel_current)/2;
                fuel_current += max(1, diff);

            }
            else
            {
                // we accicdentally found the answer
                break;
            }
            if fuel_current == fuel_min
            {
                break;
            }
        }
        fuel_current
    }

    fn compute_fuel(&self, num_fuel: usize) -> usize {
        let mut stack = HashMap::new();
        let mut surplus = HashMap::new();

        stack.insert(String::from("FUEL"), num_fuel);
        while stack.len() > 1 || stack.keys().next().unwrap_or(&String::from("")) != "ORE" {
            /*
             * eprintln!("Stack (len: {}):", stack.len());
             * for elem in stack.iter() {
             *     println!("{:?}", elem);
             * }
             */
            let name = stack
                .keys()
                .filter(|k| *k != "ORE") // can't produce new ore
                .next()
                .unwrap()
                .clone();
            let q_needed = stack.remove(&name).unwrap();

            // eprintln!("Computing {}", name);

            let reaction = self.producers.get(&name).unwrap();

            let num_reactions =
                q_needed / reaction.output.quantity + min(1, q_needed % reaction.output.quantity);

            let q_produced = num_reactions * reaction.output.quantity;

            if q_produced > q_needed {
                let q_surplus = surplus.get(&name).unwrap_or(&0).clone();
                surplus.insert(name.clone(), q_surplus + (q_produced - q_needed));
            }

            for input in reaction.input.iter() {
                let needed = input.quantity * num_reactions;
                let needed_total = stack.get(&input.name).unwrap_or(&0) + needed;

                let q_surplus = surplus.remove(&input.name).unwrap_or(0);

                if q_surplus > needed_total {
                    surplus.insert(input.name.clone(), q_surplus - needed_total);
                } else if q_surplus < needed_total {
                    stack.insert(input.name.clone(), needed_total - q_surplus);
                }
            }
        }

        // eprintln!("{:?}", surplus);

        stack.remove("ORE").unwrap()
    }
}

fn run_task_1(filename: &str) {
    let workbench = Workbench::new(filename);
    /*
     * for prod in workbench.producers.values() {
     *     println!("{:?}", prod);
     * }
     */
    let ore_per_fuel = workbench.compute_fuel(1);
    println!("{}: ORE-per-FUEL: {}", filename, ore_per_fuel,);
}

fn run_task_2(filename: &str) {
    let workbench = Workbench::new(filename);

    let ore_available = 1000000000000usize;
    let fuel_possible = workbench.compute_fuel_for_ore(ore_available);

    println!("{}: {} ORE could produce {} FUEL", filename, ore_available, fuel_possible);
}

fn main() {
    run_task_1("example_01.txt");
    run_task_1("example_02.txt");
    run_task_1("large_01.txt");
    run_task_1("large_02.txt");
    run_task_1("large_03.txt");
    run_task_1("input.txt");

    run_task_2("example_01.txt");
    run_task_2("example_02.txt");
    run_task_2("large_01.txt");
    run_task_2("large_02.txt");
    run_task_2("large_03.txt");
    run_task_2("input.txt");
}

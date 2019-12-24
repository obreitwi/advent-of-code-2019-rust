use std::cmp::min;
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

        Workbench { producers }
    }

    fn compute_fuel_requirements(&self) -> usize {
        let mut stack: HashMap<String, usize> = HashMap::new();
        let mut surplus: HashMap<String, usize> = HashMap::new();
        stack.insert(String::from("FUEL"), 1);

        while stack.len() > 1 || stack.keys().next().unwrap() != "ORE" {
            /*
             * eprintln!("Stack:");
             * for elem in stack.iter() {
             *     println!("{:?}", elem);
             * }
             */
            // can't produce new ore
            let name = stack.keys().filter(|k| *k != "ORE").next().unwrap().clone();
            let q_needed = stack.remove(&name).unwrap();

            // eprintln!("Computing {}", name);

            let reaction = self.producers.get(&name).unwrap();

            let num_reactions =
                q_needed / reaction.output.quantity + min(1, q_needed % reaction.output.quantity);

            let q_produced = num_reactions * reaction.output.quantity;

            if q_produced > q_needed {
                let prev = surplus.get(&name).unwrap_or(&0).clone();
                surplus.insert(name.clone(), prev + (q_produced - q_needed));
            }

            for input in reaction.input.iter() {
                let needed = input.quantity * num_reactions;
                let needed_total = stack.get(&input.name).unwrap_or(&0) + needed;

                let present = surplus.remove(&input.name).unwrap_or(0);

                if present > needed_total {
                    surplus.insert(input.name.clone(), present - needed_total);
                } else {
                    stack.insert(input.name.clone(), needed_total - present);
                }
            }
        }

        eprintln!("{:?}", surplus);

        *stack.get("ORE").unwrap()
    }
}

fn run(filename: &str) {
    let workbench = Workbench::new(filename);
    /*
     * for prod in workbench.producers.values() {
     *     println!("{:?}", prod);
     * }
     */
    let ore_per_fuel = workbench.compute_fuel_requirements();
    println!(
        "{}: {} FUEL needed",
        filename,
        ore_per_fuel,
    );

    println!("Could produce {} FUEL units", 1000000000000usize / ore_per_fuel);
}

fn main() {
    run("example_01.txt");
    run("example_02.txt");
    run("large_01.txt");
    run("large_02.txt");
    run("large_03.txt");
    run("input.txt");
}

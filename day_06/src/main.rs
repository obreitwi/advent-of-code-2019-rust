use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Orbit {
    big: String,
    small: String,
}

type WrappedCraft = Rc<RefCell<Craft>>;

#[derive(Debug)]
struct Craft {
    name: String,
    orbits: Option<WrappedCraft>,
    // trabants: Vec<Weak<RefCell<Craft>>>,
    trabants: Vec<WrappedCraft>,
}

#[derive(Debug)]
struct System {
    crafts: HashMap<String, WrappedCraft>,
    root: WrappedCraft,
}

impl Craft {
    pub fn _new(name: &str, orbits: &WrappedCraft) -> WrappedCraft {
        Rc::new(RefCell::new(Craft {
            name: String::from(name),
            orbits: Some(orbits.clone()),
            trabants: Vec::new(),
        }))
    }

    pub fn new_root(name: &str) -> WrappedCraft {
        Rc::new(RefCell::new(Craft {
            name: String::from(name),
            orbits: None,
            trabants: Vec::new(),
        }))
    }
}

// pub fn parse<'a, R: Iterator<Item=&'a str> >(iter: R) -> Option<Orbit>
impl Orbit {
    pub fn parse(s: &str) -> Option<Orbit> {
        let mut split: Vec<String> = s.trim().split(")").map(String::from).collect();

        if split.len() != 2 {
            eprintln!("Invalid orbit specification: {}", s);
            None
        } else {
            let small = split.pop().unwrap();
            let big = split.pop().unwrap();
            Some(Orbit { big, small })
        }
    }
}

impl System {
    pub fn parse<R: Iterator<Item = String>>(iter: R) -> System {
        let mut crafts = HashMap::new();
        let root = Craft::new_root("COM");
        crafts.insert(String::from("COM"), root.clone());

        let mut system = System { crafts, root };

        for o in iter
            .map(|s| Orbit::parse(&s))
            .filter(Option::is_some)
            .map(|x| x.unwrap())
        {
            // eprintln!("Found orbit: {:?}", o);
            system.insert(o);
        }

        system
    }

    fn insert_craft(&mut self, craft: &WrappedCraft) {
        // eprintln!("Inserting craft: {:?}", craft);
        self.crafts
            .insert(craft.borrow().name.clone(), craft.clone());
    }

    /// Get or create a craft by name
    fn get_or_create_craft(&mut self, name: &str) -> WrappedCraft {
        match self.crafts.get(name) {
            Some(craft) => craft.clone(),
            None => {
                let big = Craft::new_root(name);
                self.insert_craft(&big);
                big
            }
        }
    }

    fn insert(&mut self, Orbit { big, small }: Orbit) {
        // eprintln!("===> {} orbits {}", small, big);

        let big = self.get_or_create_craft(&big);
        let small = self.get_or_create_craft(&small);

        small.borrow_mut().orbits = Some(big.clone());
        self.insert_craft(&small);

        {
            let mut big = big.borrow_mut();
            big.trabants.push(small.clone());
        }

        // self.verify();
    }

    fn _verify(&self) {
        for (name, ptr) in self.crafts.iter() {
            eprintln!("Pepare to check pointer for {}", name);
            eprintln!("Pointer {} -> {:?}", name, ptr);
        }
    }

    fn num_crafts(&self) -> usize {
        self.crafts.len()
    }

    fn count_orbits(&self) -> usize {
        let mut num_orbits = 0;

        for craft in self.crafts.values() {
            let mut walker: WrappedCraft = craft.clone();
            // eprintln!("Current walker: {}", print_wrapped_craft(&walker));
            loop {
                let next = match &walker.borrow().orbits {
                    Some(parent) => parent.clone(),
                    None => {
                        assert!(walker.borrow().name == "COM");
                        break;
                    }
                };
                walker = next;
                // eprintln!("Current walker: {}", print_wrapped_craft(&walker));
                num_orbits += 1;
            }
        }

        num_orbits
    }

    /// Find shortest path from parent of from to parent of to.
    fn find_shortest_hops(&self, from: &str, to: &str) -> Option<usize> {
        let from = self.crafts.get(from).expect("Did not find start.");
        let to = self.crafts.get(to).expect("Did not find target.");

        {
            assert!(from.borrow().orbits.is_some());
            assert!(to.borrow().orbits.is_some());
        }

        let name_target = to.borrow().orbits.clone().unwrap().borrow().name.clone();
        let start = from.borrow().orbits.clone().unwrap();
        let mut traveller = Traveller::new(start);

        loop {
            match traveller.pop() {
                Some(current) => {
                    let current_b = current.borrow();
                    let name = current_b.name.clone();
                    let hops: usize = *traveller
                        .get_hops(&name)
                        .expect("Error: No hops defined - should not happen!");
                    if name == name_target {
                        return Some(hops);
                    }

                    traveller.add_if_not_visited(current_b.orbits.clone(), hops);
                    for trab in current_b.trabants.iter() {
                        traveller.add_if_not_visited(Some(trab.clone()), hops);
                    }
                }
                None => break,
            }
        }
        None
    }
}

struct Traveller {
    visited: HashMap<String, usize>,
    queue: Vec<WrappedCraft>,
}

impl Traveller {
    fn new(start: WrappedCraft) -> Traveller {
        let mut traveller = Traveller {
            visited: HashMap::new(),
            queue: Vec::new(),
        };
        traveller.visited.insert(start.borrow().name.clone(), 0);
        traveller.queue.push(start);
        traveller
    }

    fn add_if_not_visited(&mut self, craft: Option<WrappedCraft>, hops: usize) {
        if let Some(craft) = craft {
            let craft_b = craft.borrow();

            let name = &craft_b.name;

            let one_more_hop = hops + 1;

            if !self.visited.contains_key(name) {
                self.visited.insert(String::from(name), one_more_hop);
                self.queue.push(craft.clone());
            } else if one_more_hop < *self.visited.get(name).unwrap() {
                self.visited.insert(String::from(name), one_more_hop);
                panic!("FOUND A CIRCLE");
            }
        }
    }

    fn get_hops<'a>(&'a self, name: &str) -> Option<&'a usize> {
        self.visited.get(name)
    }

    fn pop(&mut self) -> Option<WrappedCraft> {
        self.queue.pop()
    }
}

fn _print_wrapped_craft(craft: &WrappedCraft) -> String {
    let craft = craft.borrow();
    format!(
        "{} ({})",
        craft.name,
        match &craft.orbits {
            Some(parent) => format!("parent: {}", parent.borrow().name),
            None => String::from("no parent!"),
        }
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() < 1 {
        panic!("Need input filename!");
    }

    let file = File::open(&args[0]).unwrap();
    let reader = BufReader::new(&file);

    let system = System::parse(reader.lines().map(|l| l.unwrap()));

    println!("Complete system contains {} crafts", system.num_crafts());
    println!("Number of orbits: {}", system.count_orbits());

    println!(
        "Shortest path from YOU to SAN: {:?}",
        system.find_shortest_hops("YOU", "SAN")
    );
}

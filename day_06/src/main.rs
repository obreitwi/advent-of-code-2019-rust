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
    pub fn new(name: &str, orbits: &WrappedCraft) -> WrappedCraft {
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
}

fn print_wrapped_craft(craft: &WrappedCraft) -> String {
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
}

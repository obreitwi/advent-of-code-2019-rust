use std::collections::HashSet;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Triple {
    x: i64,
    y: i64,
    z: i64,
}

impl Triple {
    fn zero() -> Triple {
        Triple { x: 0, y: 0, z: 0 }
    }

    fn new(x: i64, y: i64, z: i64) -> Triple {
        Triple { x, y, z }
    }
}

impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<x={}, y={}, z={}>", self.x, self.y, self.z)
    }
}

struct Moons {
    vel: Vec<Triple>,
    pos: Vec<Triple>,
}

impl Moons {
    fn gravity(&mut self, left: usize, right: usize) {
        let pos_left = self.pos[left];
        let pos_right = self.pos[right];

        if pos_left.x < pos_right.x {
            self.vel[left].x += 1;
            self.vel[right].x -= 1;
        } else if pos_left.x > pos_right.x {
            self.vel[left].x -= 1;
            self.vel[right].x += 1;
        }

        if pos_left.y < pos_right.y {
            self.vel[left].y += 1;
            self.vel[right].y -= 1;
        } else if pos_left.y > pos_right.y {
            self.vel[left].y -= 1;
            self.vel[right].y += 1;
        }

        if pos_left.z < pos_right.z {
            self.vel[left].z += 1;
            self.vel[right].z -= 1;
        } else if pos_left.z > pos_right.z {
            self.vel[left].z -= 1;
            self.vel[right].z += 1;
        }
    }

    fn forward(&mut self, idx: usize) {
        let pos = &mut self.pos[idx];
        let vel = &mut self.vel[idx];
        pos.x += vel.x;
        pos.y += vel.y;
        pos.z += vel.z;
    }

    fn size(&self) -> usize {
        self.pos.len()
    }

    fn new() -> Self {
        Self {
            vel: Vec::new(),
            pos: Vec::new(),
        }
    }

    fn debug() -> Self {
        let mut moons = Self::new();

        moons.add(-8, -10, 0);
        moons.add(5, 5, 10);
        moons.add(2, -7, 3);
        moons.add(9, -8, -3);

        moons
    }

    fn input() -> Self {
        let mut moons = Self::new();

        moons.add(19, -10, 7);
        moons.add(1, 2, -3);
        moons.add(14, -4, 1);
        moons.add(8, 7, -6);

        moons
    }

    fn add(&mut self, x: i64, y: i64, z: i64) {
        self.pos.push(Triple { x, y, z });
        self.vel.push(Triple::zero());
    }

    fn step(&mut self) {
        for idx_left in 0..self.size() {
            for idx_right in idx_left + 1..self.size() {
                self.gravity(idx_left, idx_right);
            }
        }
        for idx in 0..self.pos.len() {
            self.forward(idx);
        }
    }

    fn print(&self) {
        for idx in 0..self.pos.len() {
            println!("#{} pos={} vel={}", idx, self.pos[idx], self.vel[idx]);
        }
    }

    fn energy(&self, idx: usize) -> i64 {
        let pos = &self.pos[idx];
        let vel = &self.vel[idx];

        let pot = pos.x.abs() + pos.y.abs() + pos.z.abs();
        let kin = vel.x.abs() + vel.y.abs() + vel.z.abs();

        pot * kin
    }

    fn total_energy(&self) -> i64 {
        let mut total = 0;
        for idx in 0..self.size() {
            total += self.energy(idx);
        }
        total
    }

    fn current_state(&self) -> MoonState {
        (self.pos.clone(), self.vel.clone())
    }

    fn find_previous_state(&mut self) -> usize {
        let mut record: HashSet<MoonState> = HashSet::new();

        let start = self.current_state();
        record.insert(start);

        let mut iterations = 1;
        self.step();

        while !record.contains(&self.current_state()) {
            record.insert(self.current_state());
            if (iterations + 1) % 100000 == 0 {
                banner(format!("Iteration #{}:", iterations + 1).as_str());
                self.print();
            }
            self.step();
            iterations += 1;
        }
        iterations
    }
}

type MoonState = (Vec<Triple>, Vec<Triple>);

fn banner(banner: &str) {
    println!();
    println!("=== {} {}", banner, str::repeat("=", 75 - banner.len()));
    println!();
}

fn main() {
    {
        let mut hs: HashSet<(Vec<Triple>, Vec<Triple>)> = HashSet::new();

        hs.insert((
            vec![Triple::new(1, 2, 3), Triple::new(3, 4, 5)],
            vec![Triple::zero(), Triple::zero()],
        ));

        assert!(hs.contains(&(
            vec![Triple::new(1, 2, 3), Triple::new(3, 4, 5)],
            vec![Triple::zero(), Triple::zero()]
        )));
        assert!(!hs.contains(&(
            vec![Triple::new(1, 2, 3), Triple::new(3, 4, 6)],
            vec![Triple::zero(), Triple::zero()]
        )));
    }
    {
        let mut moons = Moons::input();

        for i in 0..1000 {
            // banner(format!("Iteration #{}:", i + 1).as_str());
            moons.step();
            moons.print();
        }
        println!();
        println!();
        println!("Total energy: {}", moons.total_energy());
    }
    {
        let mut moons = Moons::debug();
        let initial_state = moons.current_state();
        let target = 4686774924_i64;
        for i in 0..target {
            if (i + 1) % 100000 == 0 {
                banner(format!("Iteration #{} ({:.2}%)", i + 1, i as f64 / target as f64 * 100.0).as_str());
                moons.print();
            }
            moons.step();
        }
        assert_eq!(initial_state, moons.current_state());
        return
    }
    {
        let mut moons = Moons::input();
        println!(
            "Iterations after which the state repeats: {}",
            moons.find_previous_state()
        );
    }
}

use std::collections::HashSet;
use num::Integer;
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

        moons.add(-1, 0, 2);
        moons.add(2, -10, -7);
        moons.add(4, -8, 8);
        moons.add(3, 5, -1);

        moons
    }

    fn debug_second() -> Self {
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
            println!("#{} pos={} vel={}", idx + 1, self.pos[idx], self.vel[idx]);
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

    fn reset_state(&mut self, state: MoonState) {
        self.pos = state.0;
        self.vel = state.1;
    }

    fn find_previous_state(&mut self) -> usize {
        let start = self.current_state();
        let mut iterations = [0; 3];
        iterations[0] = self.find_previous_state_in(|t| t.x);
        self.reset_state(start.clone());
        iterations[1] = self.find_previous_state_in(|t| t.y);
        self.reset_state(start.clone());
        iterations[2] = self.find_previous_state_in(|t| t.z);
        self.reset_state(start.clone());

        println!("Iterations in each dimension: {:?}", iterations);

        iterations[0].lcm(&iterations[1]).lcm(&iterations[2])
    }

    fn find_previous_state_in<F>(&mut self, getter: F) -> usize
    where
        F: Fn(&Triple) -> i64,
    {
        // let mut record: HashSet<MoonState> = HashSet::new();

        let start = self.current_state();
        // record.insert(start.clone());

        let mut iterations = 1;
        self.step();

        // while !record.contains(&self.current_state()) {
        while !Self::check_equal_in(&start, &self.current_state(), &getter) {
            // record.insert(self.current_state());
            /*
             * if (iterations + 1) % report_every == 0 {
             *     banner(format!("Iteration #{}:", iterations + 1).as_str());
             *     self.print();
             * }
             */
            self.step();
            iterations += 1;
        }
        iterations
    }

    fn check_equal_in<F>(left: &MoonState, right: &MoonState, getter: &F) -> bool
    where
        F: Fn(&Triple) -> i64,
    {
        let (left_pos, left_vel) = left;
        let (right_pos, right_vel) = right;

        left_pos
            .iter()
            .zip(right_pos.iter())
            .all(|(l, r)| getter(l) == getter(r))
            && left_vel
                .iter()
                .zip(right_vel.iter())
                .all(|(l, r)| getter(l) == getter(r))
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
        let mut hs: HashSet<MoonState> = HashSet::new();

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
            // moons.print();
        }
        println!();
        println!("Total energy: {}", moons.total_energy());
    }
    {
        let mut moons = Moons::debug();
        println!();
        let num_iterations = moons.find_previous_state();
        println!(
            "Iterations after which the state repeats: {}",
            num_iterations
        );
    }
    {
        let mut moons = Moons::debug_second();
        // let initial_state = moons.current_state();
        let target = 4686774924_usize;
        assert_eq!(target, moons.find_previous_state())
        /*
         * for i in 0..target {
         *     if (i + 1) % 100000000 == 0 {
         *         banner(
         *             format!(
         *                 "Iteration #{} ({:.2}%)",
         *                 i + 1,
         *                 i as f64 / target as f64 * 100.0
         *             )
         *             .as_str(),
         *         );
         *         moons.print();
         *     }
         *     moons.step();
         * }
         */
        // assert_eq!(initial_state, moons.current_state());
    }
    {
        let mut moons = Moons::input();
        println!();
        let num_iterations = moons.find_previous_state();
        println!(
            "Iterations after which the state repeats: {}",
            num_iterations
        );
    }
}

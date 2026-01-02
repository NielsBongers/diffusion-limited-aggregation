use crate::simulation::CellState;
use crate::simulation::Simulation;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

impl Simulation {
    pub fn new(x_max: i32, y_max: i32, max_iterations: i32) -> Self {
        let mut thread_rng = rand::rng();
        let rng = SmallRng::from_rng(&mut thread_rng);

        let lattice: HashMap<(i32, i32), CellState> = HashMap::new();

        let mut simulation = Simulation {
            lattice,
            rng,
            x_max,
            y_max,
            max_iterations,
        };

        simulation.set_seed();

        simulation
    }

    pub fn random_direction(&mut self) -> (i32, i32) {
        let vertical = if self.rng.random::<bool>() { 1 } else { -1 };
        let horizontal = if self.rng.random::<bool>() { 1 } else { -1 };

        (vertical, horizontal)
    }

    pub fn check_cell(&self, x: i32, y: i32) -> &CellState {
        if x < 0 || x >= self.x_max || y < 0 || y >= self.y_max {
            return &CellState::Blocked;
        }

        match self.lattice.get(&(x, y)) {
            Some(cell_state) => cell_state,
            None => &CellState::Empty,
        }
    }

    pub fn check_occupied_neighbors(&self, x: i32, y: i32) -> bool {
        let neighbors = [
            (x, y + 1),
            (x, y - 1),
            (x + 1, y),
            (x - 1, y),
            (x + 1, y + 1),
            (x - 1, y - 1),
            (x + 1, y - 1),
            (x - 1, y + 1),
        ];

        for (x_neighbor, y_neighbor) in neighbors {
            let neighbor_state = self.check_cell(x_neighbor, y_neighbor);

            if let CellState::Occupied(_) = neighbor_state {
                return true;
            }
        }

        false
    }

    pub fn set_cell(&mut self, x: i32, y: i32, iteration: i32) {
        self.lattice.insert((x, y), CellState::Occupied(iteration));
    }

    pub fn set_seed(&mut self) {
        self.lattice
            .insert((self.x_max / 2, self.y_max / 2), CellState::Occupied(0));
    }

    pub fn export_lattice(&self) {
        fs::create_dir_all("data").expect("failed to create directory");

        let mut file = File::create("data/data.csv").expect("failed to create file");

        writeln!(file, "x,y,state").expect("failed to write header");

        for ((x, y), state) in self.lattice.iter() {
            let s = match state {
                CellState::Empty => 0,
                CellState::Occupied(iteration) => *iteration,
                CellState::Blocked => -1,
            };

            writeln!(file, "{},{},{}", x, y, s).expect("failed to write row");
        }
    }

    pub fn step(&mut self) {
        for iteration in 0..self.max_iterations {
            let x_init = self.rng.random_range(0..=self.x_max);
            let y_init = self.rng.random_range(0..=self.y_max);

            let mut x = x_init;
            let mut y = y_init;

            if let CellState::Occupied(_) = self.check_cell(x, y) {
                continue;
            }

            loop {
                let occupied_neighbors = self.check_occupied_neighbors(x, y);

                // println!("({}, {}): {:?}", x, y, occupied_neighbors);

                if occupied_neighbors {
                    self.set_cell(x, y, iteration);

                    // println!("Stopping!");

                    break;
                }

                loop {
                    let (i, j) = self.random_direction();

                    let x_new = x + i;
                    let y_new = y + j;

                    // println!(
                    //     "({}, {}) - ({}, {}) - {:?}",
                    //     x_new,
                    //     y_new,
                    //     i,
                    //     j,
                    //     self.check_cell(x_new, y_new)
                    // );

                    if self.check_cell(x_new, y_new) != &CellState::Blocked {
                        x = x_new;
                        y = y_new;

                        break;
                    }
                }
            }
        }
    }
}

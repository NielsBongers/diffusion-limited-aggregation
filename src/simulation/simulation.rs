use crate::simulation::CellState;
use crate::simulation::SeedType;
use crate::simulation::Simulation;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

impl Simulation {
    pub fn new(x_max: i32, y_max: i32, max_iterations: i32, seed_type: &SeedType) -> Self {
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

        simulation.set_seed(seed_type);

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

    pub fn set_seed(&mut self, seed_type: &SeedType) {
        match seed_type {
            SeedType::Random => {
                let x = self.rng.random_range(0..=self.x_max);
                let y = self.rng.random_range(0..=self.y_max);

                self.lattice.insert((x, y), CellState::Occupied(0));
            }
            SeedType::RandomMultiple(seed_count) => {
                for _ in 0..*seed_count {
                    let x = self.rng.random_range(0..=self.x_max);
                    let y = self.rng.random_range(0..=self.y_max);

                    self.lattice.insert((x, y), CellState::Occupied(0));
                }
            }
            SeedType::Single((x, y)) => {
                assert!(
                    *x > 0 && *x < self.x_max && *y > 0 && *y < self.y_max,
                    "Bounds for initialization not satisfied! Required: x: [0, {}]. y: [0, {}]",
                    self.x_max,
                    self.y_max
                );

                self.lattice.insert((*x, *y), CellState::Occupied(0));
            }
            SeedType::LineAtX(x) => {
                assert!(
                    *x > 0 && *x < self.x_max,
                    "Bounds for initialization not satisfied! Required: x: [0, {}]. y: [0, {}]",
                    self.x_max,
                    self.y_max
                );

                for y in 0..self.y_max {
                    self.lattice.insert((*x, y), CellState::Occupied(0));
                }
            }
            SeedType::LineAtY(y) => {
                assert!(
                    *y > 0 && *y < self.y_max,
                    "Bounds for initialization not satisfied! Required: x: [0, {}]. y: [0, {}]",
                    self.x_max,
                    self.y_max
                );

                for x in 0..self.x_max {
                    self.lattice.insert((x, *y), CellState::Occupied(0));
                }
            }
            SeedType::Ring(radius, width) => {
                let center_x = self.x_max / 2;
                let center_y = self.y_max / 2;

                for x in 0..self.x_max {
                    for y in 0..self.y_max {
                        let distance_to_center =
                            (((x - center_x).pow(2) + (y - center_y).pow(2)) as f64).sqrt();

                        if distance_to_center >= *radius && distance_to_center <= radius + width {
                            self.lattice.insert((x, y), CellState::Occupied(0));
                        }
                    }
                }
            }
        }
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
        let progress_bar = ProgressBar::new(self.max_iterations as u64);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta_precise})",
            )
            .unwrap()
            .progress_chars("=>-"),
        );
        progress_bar.set_message("Simulating diffusion");

        for iteration in 0..self.max_iterations {
            let x_init = self.rng.random_range(0..=self.x_max);
            let y_init = self.rng.random_range(0..=self.y_max);

            let mut x = x_init;
            let mut y = y_init;

            if let CellState::Occupied(_) = self.check_cell(x, y) {
                continue;
            }

            loop {
                if self.check_occupied_neighbors(x, y) {
                    self.set_cell(x, y, iteration);

                    break;
                }

                loop {
                    let (i, j) = self.random_direction();

                    let x_new = x + i;
                    let y_new = y + j;

                    if self.check_cell(x_new, y_new) != &CellState::Blocked {
                        x = x_new;
                        y = y_new;

                        break;
                    }
                }
            }

            progress_bar.inc(1);
        }
    }
}

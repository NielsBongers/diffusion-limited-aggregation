use crate::simulation::CellState;
use crate::simulation::SeedType;
use crate::simulation::Simulation;
use crate::utils::utils::MovingAverage;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

#[allow(unused_imports)]
use log::{debug, info, warn};

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
        let vertical = self.rng.random_range(-1..=1);
        let horizontal = self.rng.random_range(-1..=1);

        (vertical, horizontal)
    }

    pub fn check_cell(&self, x: i32, y: i32) -> CellState {
        if x < 0 || x >= self.x_max || y < 0 || y >= self.y_max {
            return CellState::Blocked;
        }

        match self.lattice.get(&(x, y)) {
            Some(cell_state) => *cell_state,
            None => CellState::Empty,
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
                let x = self.rng.random_range(1..self.x_max);
                let y = self.rng.random_range(1..self.y_max);

                self.lattice.insert((x, y), CellState::Occupied(0));
            }
            SeedType::RandomMultiple(seed_count) => {
                for _ in 0..*seed_count {
                    let x = self.rng.random_range(1..self.x_max);
                    let y = self.rng.random_range(1..self.y_max);

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

    #[cfg(feature = "regular_implementation")]
    pub fn run(&mut self) {
        let progress_bar = ProgressBar::new(self.max_iterations as u64);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] \
         {pos:>7}/{len:7} ({eta_precise}) | {msg}",
            )
            .unwrap()
            .progress_chars("=>-"),
        );

        let mut moving_average_brownian = MovingAverage::new(100);
        let mut moving_average_hashmap = MovingAverage::new(100);

        let mut enable_hashmap_optimization = true;

        const NEIGHBORHOOD_SIZE: i32 = 8;

        for iteration in 0..self.max_iterations {
            let x_init = self.rng.random_range(1..self.x_max);
            let y_init = self.rng.random_range(1..self.y_max);

            let mut x = x_init;
            let mut y = y_init;

            if let CellState::Occupied(_) = self.check_cell(x, y) {
                continue;
            }

            let mut last_check_x = x_init;
            let mut last_check_y = y_init;

            let mut minimum_distance_to_seed = 0;

            let mut brownian_motion_iteration_counter = 0;
            let mut hashmap_iteration_counter = 0;

            loop {
                if enable_hashmap_optimization {
                    let distance_to_last_check =
                        (last_check_x - x).abs() + (last_check_y - y).abs();

                    brownian_motion_iteration_counter += NEIGHBORHOOD_SIZE;

                    // Only if we've travelled near to the radius do we check adjacency!
                    if distance_to_last_check >= minimum_distance_to_seed - 2 {
                        hashmap_iteration_counter +=
                            self.lattice.keys().len() as i32 + NEIGHBORHOOD_SIZE;

                        minimum_distance_to_seed = self
                            .lattice
                            .keys()
                            .map(|(particle_x, particle_y)| {
                                (x - particle_x).abs() + (y - particle_y).abs()
                            })
                            .min()
                            .expect("No minimum distance found");

                        last_check_x = x;
                        last_check_y = y;

                        if self.check_occupied_neighbors(x, y) {
                            self.set_cell(x, y, iteration);
                            break;
                        }
                    }
                } else {
                    // Otherwise: always check adjacency
                    if self.check_occupied_neighbors(x, y) {
                        self.set_cell(x, y, iteration);
                        break;
                    }
                }

                // Rejection sampling non-blocked cells.
                // Could be done more effectively but very few are blocked (i.e. map edge) so this is fine-ish.
                loop {
                    let (i, j) = self.random_direction();

                    let x_new = x + i;
                    let y_new = y + j;

                    if self.check_cell(x_new, y_new) != CellState::Blocked {
                        x = x_new;
                        y = y_new;

                        break;
                    }
                }
            }

            // Keeping track of the current moving averages.
            if enable_hashmap_optimization {
                moving_average_brownian.add(brownian_motion_iteration_counter);
                moving_average_hashmap.add(hashmap_iteration_counter);
            }

            if iteration % 100 == 0 && iteration > 0 {
                let msg = if enable_hashmap_optimization {
                    format!(
                        "Mean hashmap: {:.0} | Mean Brownian: {:.0}",
                        moving_average_hashmap.mean(),
                        moving_average_brownian.mean()
                    )
                } else {
                    format!("Optimization disabled")
                };

                // Switching behavior.
                if moving_average_brownian.mean() < moving_average_hashmap.mean()
                    && enable_hashmap_optimization
                {
                    enable_hashmap_optimization = false;
                }

                progress_bar.set_message(msg);
            }

            progress_bar.inc(1);
        }

        progress_bar.finish();
    }

    #[cfg(feature = "clt")]
    pub fn run(&mut self) {
        let progress_bar = ProgressBar::new(self.max_iterations as u64);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] \
         {pos:>7}/{len:7} ({eta_precise}) | {msg}",
            )
            .unwrap()
            .progress_chars("=>-"),
        );

        const DISTRIBUTION_MEAN: f64 = 0.0;
        const DISTRIBUTION_STD: f64 = 2.0 / 3.0;

        const STEP_COUNT: f64 = 100.0;

        let normal_distribution = Normal::new(DISTRIBUTION_MEAN, STEP_COUNT * DISTRIBUTION_STD)
            .expect("Failed to initialize normal distribution");

        for iteration in 0..self.max_iterations {
            let x_init = self.rng.random_range(1..self.x_max);
            let y_init = self.rng.random_range(1..self.y_max);

            let mut last_check_x = x_init;
            let mut last_check_y = y_init;

            let mut minimum_distance_to_seed = 0;

            let mut x = x_init;
            let mut y = y_init;

            if let CellState::Occupied(_) = self.check_cell(x, y) {
                continue;
            }

            loop {
                let distance_to_last_check = (last_check_x - x).abs() + (last_check_y - y).abs();

                let minimum_distance_to_border = *[x, self.x_max - x, y, self.y_max - y]
                    .iter()
                    .min()
                    .expect("Somehow no minimum?!");

                debug!(
                    "Distance to last check: {}. Minimum distance to seed: {}",
                    distance_to_last_check, minimum_distance_to_seed
                );

                let mut check_neighborhood = false;

                // Either initializing or near the edge of the ball!
                if distance_to_last_check >= minimum_distance_to_seed - 2 {
                    minimum_distance_to_seed = self
                        .lattice
                        .keys()
                        .map(|(particle_x, particle_y)| {
                            (x - particle_x).abs() + (y - particle_y).abs()
                        })
                        .min()
                        .expect("No minimum distance found");

                    last_check_x = x;
                    last_check_y = y;

                    check_neighborhood = true;
                }

                let maximum_clt_jump_distance =
                    i32::min(minimum_distance_to_seed, minimum_distance_to_border);

                debug!(
                    "Minimum distance to seed: {}. Minimum distance to border: {}. Maximum CLT jump distance: {}",
                    minimum_distance_to_seed, minimum_distance_to_border, maximum_clt_jump_distance
                );

                if maximum_clt_jump_distance > 50 {
                    let mut loop_counter = 0;

                    loop {
                        loop_counter += 1;

                        let x_shift = normal_distribution.sample(&mut self.rng).round() as i32;
                        let y_shift = normal_distribution.sample(&mut self.rng).round() as i32;

                        let total_distance = x_shift.abs() + y_shift.abs();

                        if total_distance < maximum_clt_jump_distance {
                            debug!(
                                "Used CLT! Moved from ({}, {}) to ({}, {}) (shift of {}, {}). Loop counter: {}",
                                x,
                                y,
                                x + x_shift,
                                y + y_shift,
                                x_shift,
                                y_shift,
                                loop_counter,
                            );

                            x += x_shift;
                            y += y_shift;

                            break;
                        }
                    }
                } else {
                    loop {
                        let (i, j) = self.random_direction();

                        let x_new = x + i;
                        let y_new = y + j;

                        if self.check_cell(x_new, y_new) != CellState::Blocked {
                            x = x_new;
                            y = y_new;

                            break;
                        }
                    }
                }

                if check_neighborhood {
                    debug!("Checking neighborhood");
                    if self.check_occupied_neighbors(x, y) {
                        self.set_cell(x, y, iteration);
                        debug!("Found adjacent seed: halting!");
                        break;
                    }
                }
            }

            progress_bar.inc(1);
        }
        progress_bar.finish();
    }
}

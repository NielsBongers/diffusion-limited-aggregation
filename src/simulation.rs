use rand::rngs::SmallRng;
use std::collections::HashMap;
pub mod simulation;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CellState {
    Empty,
    Occupied(i32),
    Blocked,
}

pub enum SeedType {
    Random,
    RandomMultiple(i32), // Counts
    Single((i32, i32)),  // Location
    LineAtX(i32),        // x-coordinate
    LineAtY(i32),        // y-coordinate
    Ring(f64, f64),      // Radius and width.
}

pub struct Simulation {
    pub lattice: HashMap<(i32, i32), CellState>,
    pub rng: SmallRng,

    pub x_max: i32,
    pub y_max: i32,

    pub max_iterations: i32,
}

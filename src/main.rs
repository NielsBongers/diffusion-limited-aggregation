use dendrite_model::simulation::{SeedType, Simulation};
use env_logger::{Builder, Env};
#[allow(unused_imports)]
use log::{debug, info, warn};

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the simulation!");

    let x_max = 1000;
    let y_max = 1000;
    let max_iterations = 100_000;

    let seed_type = SeedType::Single((x_max / 2, y_max / 2));

    let mut simulation = Simulation::new(x_max, y_max, max_iterations, &seed_type);

    simulation.run();
    simulation.export_lattice();
}

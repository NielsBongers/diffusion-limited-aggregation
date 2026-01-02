use dendrite_model::simulation::{SeedType, Simulation};

fn main() {
    let x_max = 500;
    let y_max = 500;
    let max_iterations = 10_000;

    let seed_type = SeedType::Single((x_max / 2, y_max / 2));

    let mut simulation = Simulation::new(x_max, y_max, max_iterations, &seed_type);

    simulation.step();
    simulation.export_lattice();
}

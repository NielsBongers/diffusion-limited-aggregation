use dendrite_model::simulation::Simulation;

fn main() {
    let x_max = 1000;
    let y_max = 1000;
    let max_iterations = 10_000;

    let mut simulation = Simulation::new(x_max, y_max, max_iterations);

    simulation.step();
    simulation.export_lattice();
}

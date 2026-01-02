# Diffusion-Limited Aggregation in Rust 

<img src="figures/20260102 - Diffusion Limited Aggregation - colorful picture.png" width="400" alt="Single seed position in the center.">

## Overview 

[Diffusion-Limited Aggregation](https://en.wikipedia.org/wiki/Diffusion-limited_aggregation) is a type of simulation that can be used to create pretty dendrite-like structures. In the simplest case (i.e. this implementation), we just have a lattice with a seed, and we iteratively move particles around with Brownian motion until it hits that seed, at which point it sticks to it. We iteratively repeat that until we grow some dendrites. 


We can play with the initial seed positions and counts and simulation time to get different behaviors. 

## Features 

The entire code is very simple - it's single-threaded, and has practically no optimizations; we just place a particle, then iterate until it hits something. That means the first few iterations can be very slow, especially for large grid sizes. 

One implementation detail I'm happy with is the use of a hash map to store the states: given the degree of sparsity, I think that was the correct choice. 

This was just an afternoon project though, so I'm not very interested in trying to optimize the entire thing - but I'm curious about any ideas! 

All the settings are located in `main.rs`: the size of the lattice, the maximum number of iterations, and the seed type. 

```rust
let x_max = 500;
let y_max = 500;
let max_iterations = 10_000;

let seed_type = SeedType::Single((x_max / 2, y_max / 2));
```

The seed type has a number of pre-defined options: 

```rust
pub enum SeedType {
    Random,
    RandomMultiple(i32), // Counts
    Single((i32, i32)),  // Location
    LineAtX(i32),        // x-coordinate
    LineAtY(i32),        // y-coordinate
    Ring(f64, f64),      // Radius and width.
}
```

### Examples 

#### Line along the top. 

<img src="figures/20260102 - Diffusion Limited Aggregation - growing from the bottom.png" width="600" alt="Seeds along the top.">

#### Circular seed 

<img src="figures/20260102 - Diffusion Limited Aggregation - rings.png" width="600" alt="Circular seeds.">

## Getting started 

Clone the repo, then simply: 

```bash 
cargo run --release
```

I would advise against `cargo run` - it all takes a long time to finish depending on the settings. 


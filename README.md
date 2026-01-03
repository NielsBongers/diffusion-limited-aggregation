# Diffusion-Limited Aggregation in Rust 

<img src="figures/20260102 - Diffusion Limited Aggregation - colorful picture.png" width="400" alt="Single seed position in the center.">

## Overview 

[Diffusion-Limited Aggregation](https://en.wikipedia.org/wiki/Diffusion-limited_aggregation) is a type of simulation that can be used to create pretty dendrite-like structures. In the simplest case (i.e. this implementation), we just have a lattice with a seed, and we iteratively move particles around with Brownian motion until it hits that seed, at which point it sticks to it. We iteratively repeat that until we grow some dendrites. 

We can play with the initial seed positions and counts and simulation time to get different behaviors. 

## Getting started 

Clone the repo, then simply: 

```bash 
cargo run --release
```

I would advise against `cargo run` - it all takes a long time to finish depending on the settings. 

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

### Update #1 

I wasn't satisfied with the runtime, so contrary to my promise, I couldn't help but apply some basic optimizations. 

For a 2D random walk,

$$ \|X_N - X_0\|_2 \sim \sqrt{N} $$

so reaching a distance $L$ typically costs

$$ N \sim L^2. $$

On a $1000\times1000$ grid, particles we initialize far from the seed take a long time to reach any seed. So - we check the Manhattan distance between the closest point, requiring $M$ hashmap accesses for the particles added so far, then walk until we have moved that distance from that reference position - at which point we recalculate the minimum position. 

This takes $\mathcal O(M)$ iterations for all the hashmap checks, but saves eight accesses for the Moore neighborhood - which, looking at the results, adds up to a giant amount of savings. 

As our dendrites grow, $M$ increases linearly (we add one particle at a time), while the required $N$ drops, because our lattice fills up more and more, so at some point, $M \gg N$, and our optimization becomes counterproductive. To dynamically switch, we track moving averages for the roughly $M$ hashmap accesses with our optimization, and the $8N$ accesses the the naive method would have used, and when the mean for the naive method over 100 iterations exceeds the optimized count, we switch. 

This allows for a massive initial speed-up which automatically adjusts to different lattice scales. For example: running 100.000 iterations (i.e. 100.001 particles in total) on a $1000 \times 1000$ lattice switches around 28.000 iterations in, and takes in total around 3 minutes to run; around 555 it/s for the _outer_ loop: not bad. 

### Examples 

The entire reason for writing this code was to create some pretty diagrams, so here they are: 

#### Line along the top. 

<img src="figures/20260102 - Diffusion Limited Aggregation - growing from the bottom.png" width="600" alt="Seeds along the top.">

#### Circular seed 

<img src="figures/20260102 - Diffusion Limited Aggregation - rings.png" width="600" alt="Circular seeds.">
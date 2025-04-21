# wanglandau

A minimal, model-agnostic **Wang–Landau** Monte Carlo sampling implementation in Rust.

[![Crates.io](https://img.shields.io/crates/v/wanglandau)](https://crates.io/crates/wanglandau)
[![Documentation](https://docs.rs/wanglandau/badge.svg)](https://docs.rs/wanglandau)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/b-vitamins/wanglandau/actions/workflows/rust.yml/badge.svg)](https://github.com/b-vitamins/wanglandau/actions/workflows/rust.yml)

## Overview

The Wang-Landau algorithm is a powerful Monte Carlo technique for estimating the density of states in complex systems with rugged energy landscapes. This crate provides a generic, type-safe implementation that can be adapted to any state space and move set.

Key features:

- **Generic implementation** works with any state type
- **Modular design** with interchangeable components
- **Flexible configuration** of schedules and flatness criteria
- **Type-safe API** leveraging Rust's type system

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wanglandau = "0.0.1"
```

## Quick Example

```rust
use wanglandau::prelude::*;

#[derive(Clone)]
struct Spin(i8);               // ±1 Ising spin
impl State for Spin {}

struct Flip;
impl<R: rand::RngCore> Move<Spin, R> for Flip {
    fn propose(&mut self, s: &mut Spin, rng: &mut R) {
        use rand::Rng;
        s.0 = if rng.gen() { 1 } else { -1 };
    }
}

struct Energy;
impl Macrospace<Spin> for Energy {
    type Bin = usize;
    fn locate(&self, s: &Spin) -> usize { (s.0 + 1) as usize / 2 }
    fn bins(&self) -> &[usize] { &[0, 1] }
}

fn main() {
    let mut drv = WLDriver::new(
        Spin(1),
        Flip,
        Energy,
        Params::default(),
        Geometric { alpha: 0.5, tol: 1e-8 },
        Fraction,
        rng::seeded(2025),
    );
    drv.run(1_000_000);
    println!("{:?}", drv.ln_g());
}
```

## Algorithm Background

The Wang-Landau algorithm (introduced by Fugao Wang and David P. Landau in 2001) estimates the density of states g(E) by performing a random walk in energy space with a dynamically modified probability. It's particularly useful for:

- Calculating thermodynamic properties across a wide temperature range
- Overcoming energy barriers in complex systems
- Sampling rare events and configurations
- Estimating free energy differences

The core principle is to build a histogram of visited states while continuously updating an estimate of the density of states, adjusting a modification factor to ensure even sampling across all energy levels.

## Usage

### Core Traits

The library is built around several core traits:

1. `State`: Represents a microscopic configuration
2. `Move`: Defines Monte Carlo move proposals
3. `Macrospace`: Maps states to macroscopic bins (typically energy levels)
4. `Schedule`: Controls how the modification factor changes
5. `Flatness`: Determines when a histogram is "flat enough"

### Implementing a New System

To apply Wang-Landau sampling to a new system, implement these traits for your model:

```rust
// 1. Define your system state
#[derive(Clone)]
struct MySystem { /* ... */ }
impl State for MySystem {}

// 2. Define your move set
struct MyMoves;
impl<R: rand::RngCore> Move<MySystem, R> for MyMoves {
    fn propose(&mut self, state: &mut MySystem, rng: &mut R) {
        // Propose a random modification to the state
    }
}

// 3. Define your energy binning
struct MyBins;
impl Macrospace<MySystem> for MyBins {
    type Bin = usize;
    
    fn locate(&self, state: &MySystem) -> usize {
        // Calculate which bin this state belongs to
    }
    
    fn bins(&self) -> &[usize] {
        // Return all possible bins
    }
}
```

### Running a Simulation

Once your system is defined:

```rust
// Create a driver with your system components
let mut driver = WLDriver::new(
    initial_state,
    moves,
    bins,
    params,
    schedule,
    flatness_criterion,
    rng,
);

// Run the simulation for a fixed number of steps
driver.run(max_steps);

// Get the resulting ln(density of states)
let ln_g = driver.ln_g();
```

## Available Components

### Schedules

- `Geometric`: Reduces ln_f by a constant factor (original Wang-Landau)
- `OneOverT`: Belardinelli-Pereyra 1/t schedule for optimal convergence

### Flatness Criteria

- `Fraction`: Checks if min(H) ≥ flat × mean(H)
- `RMS`: Uses relative standard deviation σ/μ ≤ (1-flat)

## Advanced Example: 2D Ising Model

Here's a sketch of how you might implement a 2D Ising model:

```rust
#[derive(Clone)]
struct IsingLattice {
    spins: Vec<bool>,  // true for up, false for down
    size: usize,       // lattice size (N×N)
}

impl State for IsingLattice {}

// Define a random spin flip
struct FlipSite;
impl<R: rand::RngCore> Move<IsingLattice, R> for FlipSite {
    fn propose(&mut self, state: &mut IsingLattice, rng: &mut R) {
        let idx = rng.gen_range(0..state.spins.len());
        state.spins[idx] = !state.spins[idx];
    }
}

// Define energy bins
struct EnergyBins {
    n_bins: usize,
}

impl Macrospace<IsingLattice> for EnergyBins {
    type Bin = usize;
    
    fn locate(&self, state: &IsingLattice) -> usize {
        let energy = calculate_ising_energy(state);
        // Map energy to bin index
        // ...
    }
    
    fn bins(&self) -> &[usize] {
        // Return array of bin indices
        // ...
    }
}

fn calculate_ising_energy(state: &IsingLattice) -> i32 {
    // Calculate energy for the current configuration
    // ...
}
```

## Documentation

For detailed documentation, visit [docs.rs/wanglandau](https://docs.rs/wanglandau).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.

## References

1. Wang, F. & Landau, D.P. (2001). "Efficient, Multiple-Range Random Walk Algorithm to Calculate the Density of States". *Physical Review Letters*, 86(10), 2050–2053.
2. Belardinelli, R.E. & Pereyra, V.D. (2007). "Fast algorithm to calculate density of states". *Physical Review E*, 75(4), 046701.
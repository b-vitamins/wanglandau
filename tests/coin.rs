//! Test the Wang-Landau algorithm on a simple coin flip model.
//!
//! This test verifies that the Wang-Landau algorithm correctly estimates
//! the density of states for a system with two equally probable states.

use wanglandau::{flatness, prelude::*, rng, schedule};

/// A simple two-state system representing a coin (heads or tails)
#[derive(Clone)]
struct Coin(bool);
impl State for Coin {}

/// A move that randomly flips the coin
struct Flip;
impl<R: rand::RngCore> Move<Coin, R> for Flip {
    fn propose(&mut self, s: &mut Coin, rng: &mut R) {
        use rand::Rng;
        s.0 = rng.random();
    }
}

/// Maps the coin state to one of two bins (0 for tails, 1 for heads)
struct Mapper;
impl Macrospace<Coin> for Mapper {
    type Bin = usize;
    fn locate(&self, s: &Coin) -> usize {
        if s.0 {
            1
        } else {
            0
        }
    }
    fn bins(&self) -> &[usize] {
        &[0, 1]
    }
}

/// Test that the Wang-Landau algorithm correctly estimates ln(g) for a coin.
///
/// Since both states are equally probable, we expect ln(g) values to be
/// approximately equal for both states.
#[test]
fn coin_ln_g_flat() {
    // Configure parameters with single-proposal sweeps
    let params = Params {
        sweep_len: 1,
        ..Default::default()
    };

    // Initialize Wang-Landau driver with geometric schedule
    let mut drv = WLDriver::new(
        Coin(false), // Initial state (tails)
        Flip,        // Random flip moves
        Mapper,      // Maps states to bins
        params,      // Algorithm parameters
        schedule::Geometric {
            alpha: 0.5,
            tol: 1e-8,
        }, // Schedule
        flatness::Fraction, // Flatness criterion
        rng::seeded(42), // Seeded RNG for reproducibility
    );

    // Run for a large number of steps to ensure convergence
    drv.run(1_000_000);

    // Check that ln(g) values are approximately equal for both states
    let d = (drv.ln_g()[0] - drv.ln_g()[1]).abs();
    assert!(d < 2.0, "ln g difference is too large: {}", d);
}

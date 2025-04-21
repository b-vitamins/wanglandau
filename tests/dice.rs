//! Test the Wang-Landau algorithm on a six-sided die model.
//!
//! This test verifies that the Wang-Landau algorithm correctly estimates
//! the density of states for a system with six equally probable states.

use wanglandau::{flatness, prelude::*, rng, schedule};

/// A system representing a six-sided die with values 1-6
#[derive(Clone)]
struct Dice(u8);
impl State for Dice {}

/// A move that randomly rolls the die to a new value
struct Roll;
impl<R: rand::RngCore> Move<Dice, R> for Roll {
    fn propose(&mut self, s: &mut Dice, rng: &mut R) {
        use rand::Rng;
        s.0 = rng.random_range(1..=6);
    }
}

/// Maps die values to bins (0-5 for values 1-6)
struct Face;
impl Macrospace<Dice> for Face {
    type Bin = usize;
    fn locate(&self, s: &Dice) -> usize {
        (s.0 - 1) as usize
    }
    fn bins(&self) -> &[usize] {
        &[0, 1, 2, 3, 4, 5]
    }
}

/// Test that the Wang-Landau algorithm correctly estimates ln(g) for a die.
///
/// Since all six faces are equally probable, we expect ln(g) values to be
/// approximately equal for all states.
#[test]
fn dice_entropy_constant() {
    // Configure parameters with single-proposal sweeps
    let params = Params {
        sweep_len: 1,
        ..Default::default()
    };

    // Initialize Wang-Landau driver with geometric schedule
    let mut drv = WLDriver::new(
        Dice(1), // Initial state (face 1)
        Roll,    // Random roll moves
        Face,    // Maps faces to bins
        params,  // Algorithm parameters
        schedule::Geometric {
            alpha: 0.5,
            tol: 1e-9,
        }, // Schedule with tight tolerance
        flatness::Fraction, // Flatness criterion
        rng::seeded(2025), // Seeded RNG for reproducibility
    );

    // Run for a large number of steps to ensure convergence
    drv.run(2_000_000);

    // Check that all ln(g) values are approximately equal
    let ln_g = drv.ln_g();
    let (min, max) = ln_g
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &x| {
            (a.min(x), b.max(x))
        });

    // The spread of ln(g) values should be small
    assert!(
        (max - min) < 2.0,
        "Spread of ln(g) values too large: {}",
        max - min
    );
}

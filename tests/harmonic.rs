//! Test the Wang-Landau algorithm on a harmonic oscillator model.
//!
//! This test verifies that the Wang-Landau algorithm correctly converges
//! for a continuous system (harmonic oscillator) with 100 energy bins.
//! Unlike the discrete systems (coin and dice), this test exercises the
//! algorithm's ability to handle continuous state spaces and energy landscapes.

use wanglandau::{flatness, prelude::*, rng, schedule};

/// A system representing a one-dimensional harmonic oscillator
/// with position as the state variable
#[derive(Clone)]
struct Harmonic(f64);
impl State for Harmonic {}

/// A move that randomly displaces the oscillator position
struct Displace;
impl<R: rand::RngCore> Move<Harmonic, R> for Displace {
    fn propose(&mut self, s: &mut Harmonic, rng: &mut R) {
        use rand::Rng;
        // Propose a random displacement in the range [-0.5, 0.5]
        s.0 += rng.random_range(-0.5..=0.5);
    }
}

/// Maps oscillator states to energy bins with width ΔE = 0.1 for 0 ≤ E < 10
struct EnergyBins;
impl Macrospace<Harmonic> for EnergyBins {
    type Bin = usize;

    /// Calculate energy E = 0.5*x² and map to the appropriate bin
    fn locate(&self, s: &Harmonic) -> usize {
        let e = 0.5 * s.0 * s.0;
        let idx = (e / 0.1).floor() as usize;
        idx.min(99) // Clamp to the last bin (for E ≥ 10)
    }

    /// Return all possible bin indices (0-99)
    fn bins(&self) -> &[usize] {
        // 0,1,2,…,99 — using explicit array for clarity
        const B: &[usize] = &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
            68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
            90, 91, 92, 93, 94, 95, 96, 97, 98, 99,
        ];
        B
    }
}

/// Test that the Wang-Landau algorithm successfully converges for a harmonic oscillator.
///
/// This test verifies that the modification factor (ln_f) falls below the
/// specified tolerance, indicating successful convergence of the algorithm.
#[test]
fn harmonic_converges() {
    // Set convergence tolerance
    let ln_f_tol = 1e-3;

    // Initialize Wang-Landau driver with default parameters
    let mut drv = WLDriver::new(
        Harmonic(0.0), // Initial state (equilibrium position)
        Displace,      // Random displacement moves
        EnergyBins,    // Maps states to energy bins
        Params {
            ..Default::default()
        }, // Default algorithm parameters
        schedule::Geometric {
            alpha: 0.5,
            tol: ln_f_tol,
        }, // Schedule with specified tolerance
        flatness::Fraction, // Flatness criterion
        rng::seeded(7), // Seeded RNG for reproducibility
    );

    // Run for a large number of steps to ensure convergence
    drv.run(100_000_000);

    // Verify that ln_f has fallen below the tolerance
    assert!(
        drv.ln_f() < ln_f_tol,
        "Algorithm failed to converge: ln_f = {}",
        drv.ln_f()
    );
}

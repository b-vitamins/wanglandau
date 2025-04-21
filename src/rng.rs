//! # Random number generation utilities
//!
//! This module provides utilities for random number generation in
//! Wang-Landau simulations, with a focus on reproducibility and
//! high-quality randomness.
//!
//! The PCG-64 algorithm is used as the default RNG due to its excellent
//! statistical properties and performance.

use rand::SeedableRng;

/// Default random number generator used by the Wang-Landau driver.
///
/// PCG-64 is a high-quality, fast random number generator with excellent
/// statistical properties, making it suitable for Monte Carlo simulations.
pub type Rng64 = rand_pcg::Pcg64;

/// Creates a seeded PCG-64 random number generator.
///
/// Using a fixed seed allows for reproducible simulations, which is
/// crucial for testing and validation.
///
/// # Parameters
///
/// * `seed` - The seed value to initialize the RNG
///
/// # Returns
///
/// A seeded PCG-64 random number generator
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
/// use wanglandau::rng::seeded;
/// use rand::SeedableRng;
///
/// // Create a seeded RNG for reproducible results
/// let rng = seeded(42);
///
/// // For non-reproducible results, use entropy-based seeding:
/// // let rng = Rng64::from_entropy();
/// ```
pub fn seeded(seed: u64) -> Rng64 {
    Rng64::seed_from_u64(seed)
}

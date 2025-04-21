//! # Wang-Landau Monte Carlo Sampling
//!
//! `wanglandau` is a model-agnostic implementation of the Wang-Landau algorithm
//! for efficiently sampling systems with rugged energy landscapes and estimating
//! density of states.
//!
//! ## Overview
//!
//! The Wang-Landau algorithm is an adaptive importance sampling technique that
//! helps overcome energy barriers in complex systems by dynamically modifying
//! the acceptance probabilities during simulation. This ensures uniform sampling
//! across the entire energy range, making it particularly useful for:
//!
//! - Calculating the density of states
//! - Estimating thermodynamic properties
//! - Sampling complex systems with energy barriers
//! - Rare event sampling
//!
//! ## Features
//!
//! - Generic implementation that works with any system
//! - Configurable modification factor schedules
//! - Multiple histogram flatness criteria
//! - Deterministic seeding for reproducibility
//!
//! ## Example
//!
//! ```no_run
//! use wanglandau::prelude::*;
//! use rand::SeedableRng;
//!
//! // Define your system state
//! #[derive(Clone)]
//! struct MyState {}
//! impl State for MyState {}
//!
//! // Define a move proposal
//! struct MyMoves;
//! impl<R: rand::RngCore> Move<MyState, R> for MyMoves {
//!     fn propose(&mut self, state: &mut MyState, rng: &mut R) {
//!         // Propose a move to state using rng
//!     }
//! }
//!
//! // Define how states map to energy bins
//! struct MyMapper;
//! impl Macrospace<MyState> for MyMapper {
//!     type Bin = usize;
//!     fn locate(&self, state: &MyState) -> usize {
//!         // Return the bin index for this state
//!         0
//!     }
//!     fn bins(&self) -> &[usize] {
//!         // Return all possible bins
//!         &[0]
//!     }
//! }
//!
//! // Create and run a Wang-Landau simulation
//! let params = Params::default();
//! let mut driver = WLDriver::new(
//!     MyState {},
//!     MyMoves,
//!     MyMapper,
//!     params,
//!     Geometric { alpha: 0.5, tol: 1e-8 },
//!     Fraction,
//!     Rng64::seed_from_u64(42),  // For reproducibility
//! );
//!
//! // Run for a fixed number of steps
//! driver.run(1_000_000);
//!
//! // Get the estimated ln(density of states)
//! let ln_g = driver.ln_g();
//! ```

pub mod driver;
pub mod flatness;
pub mod rng;
pub mod schedule;
pub mod traits;

/// Commonly used items, exported for convenience.
pub mod prelude {
    pub use crate::driver::{Params, WLDriver};
    pub use crate::flatness::{Fraction, RMS};
    pub use crate::rng::Rng64;
    pub use crate::schedule::{Geometric, OneOverT};
    pub use crate::traits::*;
}

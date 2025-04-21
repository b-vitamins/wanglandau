//! # Core abstractions for Wang-Landau sampling
//!
//! This module defines the core traits that power the Wang-Landau algorithm:
//!
//! - [`State`]: Represents a microscopic configuration of the system
//! - [`Move`]: Defines Monte Carlo move proposals that modify states
//! - [`Macrospace`]: Maps microscopic states to macroscopic energy/parameter bins
//! - [`Schedule`]: Controls how the modification factor (ln_f) decays over time
//! - [`Flatness`]: Determines when a histogram is considered "flat enough"
//!
//! Implementing these traits for your specific system allows the generic
//! [`crate::driver::WLDriver`] to perform Wang-Landau sampling on any model.

use rand::RngCore;

/// Represents a microscopic configuration of the system being simulated.
///
/// This trait marks types that can be used as system states in Wang-Landau
/// sampling. The only requirement is that states must be cloneable, as the
/// algorithm sometimes needs to revert to previous states.
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// #[derive(Clone)]
/// struct MyState;
///
/// impl State for MyState {}
/// ```
pub trait State: Clone {}

/// Defines how states are modified during Monte Carlo sampling.
///
/// Implementations propose moves by mutating a state in-place. The acceptance
/// or rejection of moves is handled separately by the Wang-Landau driver.
///
/// # Type Parameters
///
/// * `S` - The state type that this move operates on
/// * `R` - The random number generator type used for stochastic moves
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
/// use rand::Rng;
///
/// #[derive(Clone)]
/// struct Particle { position: f64 }
/// impl State for Particle {}
///
/// struct Displace;
/// impl<R: rand::RngCore> Move<Particle, R> for Displace {
///     fn propose(&mut self, state: &mut Particle, rng: &mut R) {
///         // Randomly displace the particle
///         state.position += rng.gen_range(-0.5..0.5);
///     }
/// }
/// ```
pub trait Move<S: State, R: RngCore> {
    /// Proposes a new move by modifying `state` in-place using the random number generator.
    ///
    /// # Parameters
    ///
    /// * `state` - The current system state, which will be modified in-place
    /// * `rng` - A random number generator for stochastic move proposals
    fn propose(&mut self, state: &mut S, rng: &mut R);
}

/// Maps microscopic states to macroscopic bins (typically energy levels).
///
/// This trait defines how system states are categorized into discrete bins
/// for histogram building. In Wang-Landau sampling, these bins are used to
/// construct the density of states estimate.
///
/// # Type Parameters
///
/// * `S` - The state type this mapper can categorize
///
/// # Associated Types
///
/// * `Bin` - The type that represents a bin identifier
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// #[derive(Clone)]
/// struct Particle { position: f64 }
/// impl State for Particle {}
///
/// struct EnergyBins {
///     bin_edges: Vec<f64>,
/// }
///
/// impl Macrospace<Particle> for EnergyBins {
///     type Bin = usize;
///
///     fn locate(&self, state: &Particle) -> usize {
///         // Calculate harmonic oscillator energy: E = 0.5 * x^2
///         let energy = 0.5 * state.position * state.position;
///         
///         // Find the appropriate bin for this energy
///         // (Simple version for example)
///         (energy / 0.1).floor() as usize
///     }
///
///     fn bins(&self) -> &[usize] {
///         // Return a slice of all possible bin indices
///         static BINS: &[usize] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
///         BINS
///     }
/// }
/// ```
pub trait Macrospace<S: State> {
    /// The type that identifies a specific macroscopic bin.
    /// Must be convertible to `usize` for array indexing.
    type Bin: Copy + Into<usize>;

    /// Maps a given state to its corresponding macroscopic bin.
    ///
    /// # Parameters
    ///
    /// * `state` - The system state to categorize
    ///
    /// # Returns
    ///
    /// The bin identifier for the provided state
    fn locate(&self, state: &S) -> Self::Bin;

    /// Returns a slice containing all possible bin identifiers.
    ///
    /// This is used to initialize data structures and verify bin assignments.
    ///
    /// # Returns
    ///
    /// A slice containing all bin identifiers that could be returned by `locate`
    fn bins(&self) -> &[Self::Bin];
}

/// Controls how the modification factor (ln_f) changes during simulation.
///
/// The schedule determines when to consider the Wang-Landau algorithm
/// "converged" by progressively reducing the modification factor according
/// to some strategy.
///
/// # Example
///
/// ```rust
/// use wanglandau::prelude::*;
///
/// struct CustomSchedule {
///     step: u64,
///     tol: f64,
/// }
///
/// impl Schedule for CustomSchedule {
///     fn update(&mut self, ln_f: &mut f64) -> bool {
///         self.step += 1;
///         *ln_f = 1.0 / (self.step as f64).sqrt();
///         *ln_f < self.tol
///     }
/// }
/// ```
pub trait Schedule {
    /// Updates the modification factor and checks for convergence.
    ///
    /// This method is called whenever the histogram is deemed flat enough
    /// to warrant reducing the modification factor.
    ///
    /// # Parameters
    ///
    /// * `ln_f` - The current modification factor (ln f), which will be updated in-place
    ///
    /// # Returns
    ///
    /// `true` if the algorithm should be considered converged, `false` otherwise
    fn update(&mut self, ln_f: &mut f64) -> bool; // return true if converged
}

/// Defines a criterion for histogram flatness.
///
/// In Wang-Landau sampling, the simulation proceeds in stages where
/// the modification factor is reduced once the histogram of visited states
/// is considered "flat enough" according to some criterion.
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// struct MaxMinRatio;
///
/// impl Flatness for MaxMinRatio {
///     fn is_flat(&self, hist: &[u64], flatness: f64) -> bool {
///         if hist.is_empty() { return false; }
///
///         let min = *hist.iter().min().unwrap() as f64;
///         let max = *hist.iter().max().unwrap() as f64;
///
///         if min == 0.0 { return false; }
///         (max / min) <= (1.0 / flatness)
///     }
/// }
/// ```
pub trait Flatness {
    /// Determines if a histogram is "flat enough" according to some criterion.
    ///
    /// # Parameters
    ///
    /// * `hist` - The current histogram of visited states
    /// * `flatness` - A parameter controlling how strict the flatness criterion is,
    ///                typically between 0.0 and 1.0
    ///
    /// # Returns
    ///
    /// `true` if the histogram is considered flat enough, `false` otherwise
    fn is_flat(&self, hist: &[u64], flatness: f64) -> bool;
}

//! # Wang-Landau algorithm implementation
//!
//! This module provides the core implementation of the Wang-Landau algorithm,
//! a powerful Monte Carlo technique for estimating the density of states in
//! systems with complex energy landscapes.
//!
//! The key component is the [`WLDriver`] struct, which orchestrates the
//! sampling process using the traits defined in the crate.

use rand::{Rng, RngCore};

use crate::rng::Rng64;
use crate::traits::{Flatness, Macrospace, Move, Schedule, State};

/// Configurable parameters for Wang-Landau sampling.
///
/// These parameters control the behavior and convergence of the algorithm.
///
/// # Fields
///
/// * `ln_f0` - The initial value of the modification factor (ln f)
/// * `ln_f_min` - The minimum value of ln_f for convergence (not used directly by the driver)
/// * `flatness` - The flatness parameter (typically between 0.0 and 1.0)
/// * `sweep_len` - The number of move proposals per Wang-Landau step
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// // Use default parameters
/// let default_params = Params::default();
///
/// // Or customize parameters
/// let custom_params = Params {
///     ln_f0: 1.0,
///     ln_f_min: 1e-8,
///     flatness: 0.9, // Stricter flatness criterion
///     sweep_len: 10, // More move proposals per step
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Params {
    /// Initial modification factor value (ln f)
    pub ln_f0: f64,

    /// Target minimum value for ln_f (exposed for convenience)
    pub ln_f_min: f64,

    /// Flatness criterion parameter (typically between 0.0 and 1.0)
    pub flatness: f64,

    /// Number of move proposals per Wang-Landau step
    pub sweep_len: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            ln_f0: 1.0,
            ln_f_min: 1e-8,
            flatness: 0.8,
            sweep_len: 1,
        }
    }
}

/// Generic single-walker Wang-Landau sampling engine.
///
/// This struct implements the Wang-Landau algorithm for arbitrary state spaces
/// and move sets. It builds a histogram of visited states and dynamically
/// modifies acceptance probabilities to achieve uniform sampling across all
/// energy levels.
///
/// # Type Parameters
///
/// * `S` - The system state type
/// * `Mv` - The move proposal type
/// * `Map` - The state-to-bin mapping type
/// * `R` - The random number generator type (defaults to PCG-64)
/// * `Sch` - The modification factor schedule type (defaults to geometric)
/// * `F` - The histogram flatness criterion type (defaults to fraction-based)
///
/// # Example
///
/// ```no_run
/// use wanglandau::prelude::*;
/// use rand::{SeedableRng, Rng};
///
/// // Define a simple two-state system (coin flip)
/// #[derive(Clone)]
/// struct Coin(bool);
/// impl State for Coin {}
///
/// // Define a move that flips the coin
/// struct Flip;
/// impl<R: rand::RngCore> Move<Coin, R> for Flip {
///     fn propose(&mut self, s: &mut Coin, rng: &mut R) {
///         s.0 = rng.gen();
///     }
/// }
///
/// // Define mapping from coin state to bins
/// struct CoinMapper;
/// impl Macrospace<Coin> for CoinMapper {
///     type Bin = usize;
///     fn locate(&self, s: &Coin) -> usize { if s.0 { 1 } else { 0 } }
///     fn bins(&self) -> &[usize] { &[0, 1] }
/// }
///
/// // Create and run a Wang-Landau simulation
/// let params = Params::default();
/// let mut driver = WLDriver::new(
///     Coin(false),                         // Initial state
///     Flip,                                // Move proposals
///     CoinMapper,                          // State-to-bin mapping
///     params,                              // Algorithm parameters
///     Geometric { alpha: 0.5, tol: 1e-8 }, // Modification factor schedule
///     Fraction,                            // Flatness criterion
///     Rng64::seed_from_u64(42),            // Seeded random number generator
/// );
///
/// // Run for 10,000 steps
/// driver.run(10_000);
///
/// // The resulting ln_g approximates the density of states
/// let ln_g = driver.ln_g();
/// assert_eq!(ln_g.len(), 2);  // Two states: heads and tails
/// ```
#[allow(clippy::type_complexity)]
pub struct WLDriver<
    S,
    Mv,
    Map,
    R = Rng64,
    Sch = crate::schedule::Geometric,
    F = crate::flatness::Fraction,
> where
    S: State,
    Mv: Move<S, R>,
    Map: Macrospace<S, Bin = usize>,
    R: RngCore,
    Sch: Schedule,
    F: Flatness,
{
    /// Current system state
    state: S,

    /// Move proposal generator
    moves: Mv,

    /// Maps states to macroscopic bins
    mapper: Map,

    /// Current estimate of ln(density of states)
    ln_g: Vec<f64>,

    /// Histogram of visited states
    hist: Vec<u64>,

    /// Current modification factor (ln f)
    ln_f: f64,

    /// Algorithm parameters
    params: Params,

    /// Random number generator
    rng: R,

    /// Modification factor update schedule
    sched: Sch,

    /// Histogram flatness criterion
    flat: F,

    /// Current step count
    step: u64,
}

impl<S, Mv, Map, R, Sch, F> WLDriver<S, Mv, Map, R, Sch, F>
where
    S: State,
    Mv: Move<S, R>,
    Map: Macrospace<S, Bin = usize>,
    R: RngCore,
    Sch: Schedule,
    F: Flatness,
{
    /// Creates a new Wang-Landau driver with the specified components.
    ///
    /// # Parameters
    ///
    /// * `state` - The initial system state
    /// * `moves` - The move proposal generator
    /// * `mapper` - The state-to-bin mapping
    /// * `params` - The algorithm parameters
    /// * `sched` - The modification factor update schedule
    /// * `flat` - The histogram flatness criterion
    /// * `rng` - The random number generator
    ///
    /// # Returns
    ///
    /// A new `WLDriver` instance initialized and ready to run
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: S,
        moves: Mv,
        mapper: Map,
        params: Params,
        sched: Sch,
        flat: F,
        rng: R,
    ) -> Self {
        let n_bins = mapper.bins().len();
        Self {
            state,
            moves,
            mapper,
            ln_g: vec![0.0; n_bins],
            hist: vec![0; n_bins],
            ln_f: params.ln_f0,
            params,
            rng,
            sched,
            flat,
            step: 0,
        }
    }

    /// Performs one Wang-Landau step, consisting of multiple move proposals and histogram updates.
    ///
    /// A single step consists of:
    /// 1. Proposing `sweep_len` moves and applying Wang-Landau acceptance
    /// 2. Checking histogram flatness
    /// 3. Updating the modification factor if the histogram is flat
    ///
    /// # Returns
    ///
    /// `true` if the algorithm has converged (ln_f below tolerance), `false` otherwise
    pub fn step(&mut self) -> bool {
        for _ in 0..self.params.sweep_len {
            // --- propose move & evaluate bins --------------------
            let bin_old: usize = self.mapper.locate(&self.state);
            let prev_state = self.state.clone();

            self.moves.propose(&mut self.state, &mut self.rng);
            let bin_new: usize = self.mapper.locate(&self.state);

            // --- WL acceptance -----------------------------------
            let accept = if bin_new == bin_old {
                true
            } else {
                let delta = self.ln_g[bin_old] - self.ln_g[bin_new];
                self.rng.random::<f64>() < delta.exp()
            };
            let bin_final = if accept {
                bin_new
            } else {
                self.state = prev_state;
                bin_old
            };

            // --- WL bookkeeping ----------------------------------
            self.ln_g[bin_final] += self.ln_f;
            self.hist[bin_final] += 1;
        }

        if self.flat.is_flat(&self.hist, self.params.flatness) {
            self.hist.fill(0);
            if self.sched.update(&mut self.ln_f) {
                return true;
            }
        }

        self.step += 1;
        false
    }

    /// Runs the Wang-Landau simulation for up to `max_steps` steps or until convergence.
    ///
    /// The simulation will stop early if the modification factor falls below
    /// the tolerance specified in the schedule.
    ///
    /// # Parameters
    ///
    /// * `max_steps` - The maximum number of Wang-Landau steps to perform
    pub fn run(&mut self, max_steps: u64) {
        for _ in 0..max_steps {
            if self.step() {
                break;
            }
        }
    }

    /// Returns the current estimate of ln(density of states).
    ///
    /// # Returns
    ///
    /// A slice containing the ln(g) values for each bin
    pub fn ln_g(&self) -> &[f64] {
        &self.ln_g
    }

    /// Returns the current histogram of visited states.
    ///
    /// # Returns
    ///
    /// A slice containing the visit counts for each bin
    pub fn histogram(&self) -> &[u64] {
        &self.hist
    }

    /// Returns the current modification factor (ln f).
    ///
    /// # Returns
    ///
    /// The current ln_f value
    pub fn ln_f(&self) -> f64 {
        self.ln_f
    }

    /// Returns the number of Wang-Landau steps performed so far.
    ///
    /// # Returns
    ///
    /// The current step count
    pub fn step_count(&self) -> u64 {
        self.step
    }

    /// Returns a reference to the current system state.
    ///
    /// # Returns
    ///
    /// A reference to the current state
    pub fn state(&self) -> &S {
        &self.state
    }
}

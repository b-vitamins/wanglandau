//! # Modification factor schedules
//!
//! This module provides implementations of the [`Schedule`] trait for
//! controlling how the modification factor (`ln_f`) changes during
//! Wang-Landau sampling.
//!
//! Two common schedules are provided:
//!
//! - [`Geometric`]: Reduces ln_f by a constant factor (e.g., ln_f *= 0.5)
//! - [`OneOverT`]: Uses the Belardinelli-Pereyra 1/t schedule
//!
//! Custom schedules can be implemented by implementing the [`Schedule`] trait.

use crate::traits::Schedule;

/// A geometric schedule that multiplies `ln_f` by a constant factor.
///
/// This is the original schedule proposed in the Wang-Landau algorithm,
/// where the modification factor is reduced by a constant factor (typically 0.5)
/// whenever the histogram becomes flat.
///
/// # Fields
///
/// * `alpha` - The factor by which ln_f is multiplied (0 < alpha < 1)
/// * `tol` - The convergence tolerance for ln_f
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// let mut ln_f = 1.0;
/// let mut schedule = Geometric { alpha: 0.5, tol: 1e-8 };
///
/// // Update ln_f geometrically
/// let converged = schedule.update(&mut ln_f);
/// assert_eq!(ln_f, 0.5); // ln_f *= 0.5
/// assert_eq!(converged, false); // Not yet below tolerance
///
/// // After many updates...
/// // converged will be true when ln_f < 1e-8
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Geometric {
    /// Factor by which ln_f is multiplied (typically 0.5)
    pub alpha: f64,

    /// Convergence tolerance for ln_f
    pub tol: f64,
}

impl Schedule for Geometric {
    fn update(&mut self, ln_f: &mut f64) -> bool {
        *ln_f *= self.alpha;
        *ln_f < self.tol
    }
}

/// A 1/t schedule for ln_f, following the Belardinelli-Pereyra algorithm.
///
/// This schedule sets ln_f = 1/t, where t is the number of updates performed.
/// This provides provably optimal convergence for Wang-Landau sampling.
///
/// # Fields
///
/// * `t` - The current time step (internal counter)
/// * `tol` - The convergence tolerance for ln_f
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// let mut ln_f = 1.0;
/// let mut schedule = OneOverT::default();
///
/// // Update ln_f using 1/t schedule
/// let converged = schedule.update(&mut ln_f);
/// assert_eq!(ln_f, 0.5); // ln_f = 1/2
/// assert_eq!(converged, false);
///
/// // Update again
/// let converged = schedule.update(&mut ln_f);
/// assert_eq!(ln_f, 1.0/3.0); // ln_f = 1/3
/// assert_eq!(converged, false);
///
/// // After many updates...
/// // converged will be true when ln_f < tol (default 1e-8)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct OneOverT {
    /// Internal time step counter
    t: u64,

    /// Convergence tolerance for ln_f
    pub tol: f64,
}

impl Default for OneOverT {
    fn default() -> Self {
        Self { t: 1, tol: 1e-8 }
    }
}

impl Schedule for OneOverT {
    fn update(&mut self, ln_f: &mut f64) -> bool {
        self.t += 1;
        *ln_f = 1.0 / self.t as f64;
        *ln_f < self.tol
    }
}

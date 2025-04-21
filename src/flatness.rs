//! # Histogram flatness criteria
//!
//! This module provides implementations of the [`Flatness`] trait for
//! determining when a histogram is sufficiently "flat" during Wang-Landau
//! sampling.
//!
//! Two common criteria are provided:
//!
//! - [`Fraction`]: Checks if the minimum visit count is at least some fraction
//!   of the mean visit count
//! - [`RMS`]: Checks if the relative standard deviation is below a threshold
//!
//! Custom criteria can be implemented by implementing the [`Flatness`] trait.

use crate::traits::Flatness;

/// Considers a histogram flat when `min(H) ≥ flat × mean(H)`.
///
/// This popular criterion checks if the minimum visit count across all bins is
/// at least some fraction of the mean visit count. The `flat` parameter typically
/// ranges from 0.7 to 0.9, with higher values enforcing stricter flatness.
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// let hist = vec![80, 95, 103, 88, 90];
/// let flatness = Fraction;
///
/// // Check if visits are at least 80% of the mean
/// let is_flat = flatness.is_flat(&hist, 0.8);
///
/// // Mean is 91.2, minimum is 80, ratio is 0.877
/// // So this would return true
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Fraction;

impl Flatness for Fraction {
    fn is_flat(&self, hist: &[u64], flat: f64) -> bool {
        if hist.is_empty() {
            return false;
        }

        let min = *hist.iter().min().unwrap() as f64;
        let avg = hist.iter().sum::<u64>() as f64 / hist.len() as f64;

        min >= flat * avg
    }
}

/// Considers a histogram flat when the relative standard deviation `σ/μ ≤ (1 - flat)`.
///
/// This criterion uses the coefficient of variation (relative standard deviation)
/// to measure flatness. When `flat` is close to 1.0, this requires the histogram
/// to have a very small spread relative to its mean.
///
/// # Example
///
/// ```
/// use wanglandau::prelude::*;
///
/// let hist = vec![95, 105, 98, 102, 100];
/// let flatness = RMS;
///
/// // Check if relative std dev is <= 0.1
/// let is_flat = flatness.is_flat(&hist, 0.9);
///
/// // Mean is 100, std dev is ~3.74, so ratio is ~0.0374
/// // This is less than 0.1, so would return true
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RMS;

impl Flatness for RMS {
    fn is_flat(&self, hist: &[u64], flat: f64) -> bool {
        if hist.is_empty() {
            return false;
        }

        let mean = hist.iter().sum::<u64>() as f64 / hist.len() as f64;

        // Calculate variance
        let var = hist
            .iter()
            .map(|&h| {
                let d = h as f64 - mean;
                d * d
            })
            .sum::<f64>()
            / hist.len() as f64;

        // Coefficient of variation (σ/μ)
        let rel_std_dev = var.sqrt() / mean;

        rel_std_dev <= 1.0 - flat
    }
}

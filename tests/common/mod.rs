//! Helpers shared between the integration tests.

// Test binaries might not use all helpers defined here.
#![allow(dead_code)]

/// The solvers measure well inside `1e-15` relative, so this leaves room for platform differences.
const SOLVED_TOLERANCE: f64 = 1e-12;

/// Asserts that a solved value matches what it was solved from.
#[track_caller]
pub fn assert_solved(solved: f64, expected: f64) {
    // The floor keeps an expected zero from demanding bit-exact equality.
    let tolerance = (expected.abs() * SOLVED_TOLERANCE).max(SOLVED_TOLERANCE);
    let error = (solved - expected).abs();
    assert!(
        error <= tolerance,
        "solved {solved} but expected {expected}, which is {error:e} apart and outside the \
         tolerance of {tolerance:e}"
    );
}

/// Significant digits kept when snapshotting, leaving room for the platform's libm to differ in
/// the last bits of `exp`, `ln`, and `powf`.
const SNAPSHOT_SIGNIFICANT_DIGITS: usize = 12;

/// Asserts an inline snapshot of a floating-point value, rounded so it is stable across platforms.
macro_rules! assert_sig_snapshot {
    ($value:expr, @$snapshot:literal) => {
        insta::assert_snapshot!($crate::common::sig($value), @$snapshot)
    };
}

pub(crate) use assert_sig_snapshot;

/// Formats a value for snapshotting, rounded so it is stable across platforms.
pub fn sig(value: f64) -> String {
    if !value.is_finite() || value == 0. {
        // Normalize `-0` to `0`; neither case has digits to round.
        return format!("{}", if value == 0. { 0. } else { value });
    }

    let magnitude = value.abs().log10().floor() as i32;
    let decimals = (SNAPSHOT_SIGNIFICANT_DIGITS as i32 - 1 - magnitude).max(0) as usize;
    format!("{}", format!("{value:.decimals$}").parse::<f64>().unwrap())
}

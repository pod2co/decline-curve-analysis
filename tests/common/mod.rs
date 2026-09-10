//! Helpers shared between the integration tests.

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

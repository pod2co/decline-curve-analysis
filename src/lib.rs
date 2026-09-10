use std::marker::PhantomData;
use thiserror::Error;

mod decline_rate;
mod delay;
mod exponential;
mod flat;
mod harmonic;
mod hyperbolic;
mod linear;

pub use decline_rate::*;
pub use delay::*;
pub use exponential::*;
pub use flat::*;
pub use harmonic::*;
pub use hyperbolic::*;
pub use linear::*;

/// Absolute tolerance for floating-point comparisons and "effectively zero" checks.
pub(crate) const EPSILON: f64 = 1e-12;

/// Maximum allowed duration in years. This is extremely long and just meant to catch obvious
/// errors that could cause numerical instability.
pub(crate) const MAX_DURATION_YEARS: f64 = 1000.;

/// Returns true if `value` is approximately zero, otherwise false.
pub(crate) fn is_effectively_zero(value: f64) -> bool {
    value.abs() <= EPSILON
}

/// Returns true if two finite values are approximately equal, otherwise false.
pub(crate) fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPSILON
}

/// Returns true if `a >= b`, otherwise false.
pub(crate) fn approx_gte(a: f64, b: f64) -> bool {
    a >= b - EPSILON
}

/// Validates that a floating-point value is finite (not NaN or infinity).
pub(crate) fn validate_finite(
    value: f64,
    name: &'static str,
) -> Result<(), DeclineCurveAnalysisError> {
    if value.is_finite() {
        return Ok(());
    }

    Err(DeclineCurveAnalysisError::InvalidInput {
        reason: if value.is_nan() {
            format!("{name} is not-a-number, but expected a finite number")
        } else {
            format!("{name} is infinity, but expected a finite number")
        },
    })
}

/// Validates that a value is positive and finite.
pub(crate) fn validate_positive(
    value: f64,
    name: &'static str,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_finite(value, name)?;
    if value.is_sign_negative() {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: format!("{name} is negative, but expected a positive number"),
        });
    }
    Ok(())
}

/// Validates that a rate value is non-zero, positive, and finite.
pub(crate) fn validate_non_zero_positive_rate(
    value: f64,
    name: &'static str,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_finite(value, name)?;
    if value.is_sign_negative() || is_effectively_zero(value) {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: format!("{name} is negative or zero, but expected a positive number"),
        });
    }
    Ok(())
}

/// Validates that a decline rate value is non-zero and finite.
pub(crate) fn validate_non_zero_decline_rate(
    value: f64,
    name: &'static str,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_finite(value, name)?;
    if is_effectively_zero(value) {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: format!("{name} is approximately zero, but expected it to be non-zero"),
        });
    }
    Ok(())
}

/// Validates that a duration is positive, finite, and doesn't exceed the maximum duration.
pub(crate) fn validate_duration<Time: DeclineTimeUnit>(
    duration: Time,
) -> Result<(), DeclineCurveAnalysisError> {
    let duration_value = duration.value();

    validate_positive(duration_value, "duration")?;

    let duration_years = duration.to_unit::<AverageYearsTime>().value();

    if duration_years > MAX_DURATION_YEARS {
        return Err(DeclineCurveAnalysisError::DurationTooLong);
    }
    Ok(())
}

/// Validates that a volume is positive and finite.
pub(crate) fn validate_incremental_volume(volume: f64) -> Result<(), DeclineCurveAnalysisError> {
    validate_finite(volume, "incremental volume")?;
    if volume.is_sign_negative() {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: "incremental volume is negative, but expected a positive number".to_string(),
        });
    }
    Ok(())
}

/// Validates that a volume is positive, finite, and non-zero.
pub(crate) fn validate_non_zero_positive_volume(
    volume: f64,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_incremental_volume(volume)?;
    if is_effectively_zero(volume) {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: "incremental volume is approximately zero, but expected it to be non-zero"
                .to_string(),
        });
    }
    Ok(())
}

/// Validates a derived duration, reporting an unusable one as an unsolvable decline and
/// normalizing the `-0.0` that deriving a zero might result in.
pub(crate) fn validate_derived_duration<Time: DeclineTimeUnit>(
    duration: Time,
) -> Result<Time, DeclineCurveAnalysisError> {
    // `-0.0` is zero reached from below rather than a negative duration.
    if duration.value() == 0. {
        return Ok(Time::from(0.));
    }

    match validate_duration(duration) {
        Ok(()) => Ok(duration),
        Err(DeclineCurveAnalysisError::InvalidInput { .. }) => {
            Err(DeclineCurveAnalysisError::CannotSolveDecline)
        }
        Err(other) => Err(other),
    }
}

/// Validates a derived decline rate, reporting an unusable one as an unsolvable decline.
pub(crate) fn validate_derived_decline_rate(
    decline_rate: f64,
) -> Result<(), DeclineCurveAnalysisError> {
    if !decline_rate.is_finite() {
        return Err(DeclineCurveAnalysisError::CannotSolveDecline);
    }
    Ok(())
}

/// Validates that a derived decline rate is non-zero, on top of
/// [`validate_derived_decline_rate`].
pub(crate) fn validate_non_zero_derived_decline_rate(
    decline_rate: f64,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_derived_decline_rate(decline_rate)?;
    if is_effectively_zero(decline_rate) {
        return Err(DeclineCurveAnalysisError::CannotSolveDecline);
    }
    Ok(())
}

/// Validates that a duration is non-zero, on top of [`validate_duration`].
pub(crate) fn validate_non_zero_duration<Time: DeclineTimeUnit>(
    duration: Time,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_duration(duration)?;
    if is_effectively_zero(duration.value()) {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: "duration is approximately zero, but expected it to be non-zero".to_string(),
        });
    }
    Ok(())
}

/// Validates that a segment avoids the singularity in the Arps rate equation.
///
/// `q_i * (1 + b * d * t) ^ (-1 / b)` is only defined while its base stays positive.
pub(crate) fn validate_arps_singularity<Time: DeclineTimeUnit>(
    exponent: f64,
    decline_rate: NominalDeclineRate<Time>,
    duration: Time,
) -> Result<(), DeclineCurveAnalysisError> {
    let base_slope = exponent * decline_rate.value();
    if base_slope < 0. && duration.value() >= -1. / base_slope {
        return Err(DeclineCurveAnalysisError::DurationTooLong);
    }
    Ok(())
}

/// An error type for invalid parameters.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DeclineCurveAnalysisError {
    #[error("decline rate too high")]
    DeclineRateTooHigh,
    #[error("decline rate has wrong sign")]
    DeclineRateWrongSign,
    #[error("cannot solve decline: no finite solution exists for the given parameters")]
    CannotSolveDecline,
    #[error("exponent too large")]
    ExponentTooLarge,
    #[error("duration too long")]
    DurationTooLong,
    #[error("{reason}")]
    InvalidInput { reason: String },
}

/// The production rate for a specific time unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProductionRate<Time: DeclineTimeUnit> {
    value: f64,
    _time: PhantomData<Time>,
}

impl<Time: DeclineTimeUnit> ProductionRate<Time> {
    pub const fn new(value: f64) -> Self {
        Self {
            value,
            _time: PhantomData,
        }
    }

    pub const fn value(&self) -> f64 {
        self.value
    }
}

impl From<ProductionRate<AverageYearsTime>> for ProductionRate<AverageDaysTime> {
    fn from(val: ProductionRate<AverageYearsTime>) -> Self {
        ProductionRate::new(val.value * AverageDaysTime::LENGTH / AverageYearsTime::LENGTH)
    }
}

impl From<ProductionRate<AverageDaysTime>> for ProductionRate<AverageYearsTime> {
    fn from(val: ProductionRate<AverageDaysTime>) -> Self {
        ProductionRate::new(val.value * AverageYearsTime::LENGTH / AverageDaysTime::LENGTH)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeclineRateSignValidation {
    Continue,
    ZeroDuration,
}

/// Validates decline rate sign vs. rate direction.
fn validate_decline_rate_sign(
    decline_rate: f64,
    initial_rate: f64,
    final_rate: f64,
) -> Result<DeclineRateSignValidation, DeclineCurveAnalysisError> {
    if initial_rate < final_rate {
        if decline_rate > 0. {
            return Err(DeclineCurveAnalysisError::DeclineRateWrongSign);
        }
    } else if initial_rate > final_rate {
        if decline_rate < 0. {
            return Err(DeclineCurveAnalysisError::DeclineRateWrongSign);
        }
    } else {
        // If the rates are equal, the duration is zero.
        return Ok(DeclineRateSignValidation::ZeroDuration);
    }

    Ok(DeclineRateSignValidation::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_effectively_zero_range() {
        assert!(is_effectively_zero(0.));
        assert!(is_effectively_zero(EPSILON * 0.1));
        assert!(is_effectively_zero(-EPSILON * 0.1));
        assert!(is_effectively_zero(1e-15));
        assert!(!is_effectively_zero(1.));
        assert!(!is_effectively_zero(-1.));
        assert!(!is_effectively_zero(EPSILON * 10.));
        assert!(is_effectively_zero(EPSILON));
        assert!(!is_effectively_zero(EPSILON * 1.01));
        assert!(!is_effectively_zero(f64::NAN));
    }

    #[test]
    fn approx_eq_exactly_range() {
        assert!(approx_eq(100., 100.));
        assert!(approx_eq(0., 0.));
        assert!(!approx_eq(100., 200.));
        assert!(!approx_eq(0., 1.));
        assert!(approx_eq(0., EPSILON * 0.5));
        assert!(approx_eq(EPSILON * 0.5, 0.));
        assert!(!approx_eq(f64::NAN, 100.));
        assert!(!approx_eq(100., f64::NAN));
        assert!(!approx_eq(f64::NAN, f64::NAN));
    }

    #[test]
    fn validate_decline_rate_sign_range() {
        insta::assert_debug_snapshot!(validate_decline_rate_sign(0.1, 100., 100.).unwrap(), @"ZeroDuration");
        insta::assert_debug_snapshot!(validate_decline_rate_sign(0.1, 100., 50.).unwrap(), @"Continue");
        insta::assert_debug_snapshot!(validate_decline_rate_sign(-0.1, 50., 100.).unwrap(), @"Continue");
        insta::assert_debug_snapshot!(validate_decline_rate_sign(-0.1, 100., 50.).unwrap_err(), @"DeclineRateWrongSign");
        insta::assert_debug_snapshot!(validate_decline_rate_sign(0.1, 50., 100.).unwrap_err(), @"DeclineRateWrongSign");
    }

    #[test]
    fn subnormal_values_are_effectively_zero() {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        assert!(is_effectively_zero(subnormal));
        assert!(approx_eq(subnormal, 0.));
    }

    #[test]
    fn negative_zero_is_effectively_zero() {
        assert!(is_effectively_zero(-0.));
        assert!(approx_eq(-0., 0.));
    }

    #[test]
    fn validate_non_zero_positive_rate_rejects_negative_zero() {
        let result = validate_non_zero_positive_rate(-0., "value");
        insta::assert_snapshot!(result.unwrap_err(), @"value is negative or zero, but expected a positive number");
    }

    #[test]
    fn validate_positive_rejects_negative_zero() {
        let result = validate_positive(-0., "value");
        insta::assert_snapshot!(result.unwrap_err(), @"value is negative, but expected a positive number");
    }

    #[test]
    fn validate_non_zero_positive_volume_rejects_zero() {
        insta::assert_snapshot!(
            validate_non_zero_positive_volume(0.).unwrap_err(),
            @"incremental volume is approximately zero, but expected it to be non-zero"
        );
        insta::assert_snapshot!(
            validate_non_zero_positive_volume(EPSILON * 0.5).unwrap_err(),
            @"incremental volume is approximately zero, but expected it to be non-zero"
        );
        insta::assert_snapshot!(
            validate_non_zero_positive_volume(-1.).unwrap_err(),
            @"incremental volume is negative, but expected a positive number"
        );
        assert!(validate_non_zero_positive_volume(EPSILON * 10.).is_ok());
    }

    #[test]
    fn validate_non_zero_duration_rejects_zero() {
        insta::assert_snapshot!(
            validate_non_zero_duration(AverageDaysTime { days: 0. }).unwrap_err(),
            @"duration is approximately zero, but expected it to be non-zero"
        );
        insta::assert_snapshot!(
            validate_non_zero_duration(AverageDaysTime { days: -1. }).unwrap_err(),
            @"duration is negative, but expected a positive number"
        );
        assert!(validate_non_zero_duration(AverageDaysTime { days: 1. }).is_ok());
    }

    #[test]
    fn validate_derived_duration_hides_a_parameter_the_caller_never_passed() {
        insta::assert_debug_snapshot!(
            validate_derived_duration(AverageYearsTime { years: -1. }).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_derived_duration(AverageYearsTime { years: f64::NAN }).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_derived_duration(AverageYearsTime { years: f64::INFINITY }).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_derived_duration(AverageYearsTime { years: MAX_DURATION_YEARS + 1. })
                .unwrap_err(),
            @"DurationTooLong"
        );
        assert_eq!(
            validate_derived_duration(AverageYearsTime { years: 10. }).unwrap(),
            AverageYearsTime { years: 10. }
        );

        let normalized = validate_derived_duration(AverageYearsTime { years: -0. }).unwrap();
        assert_eq!(normalized.years, 0.);
        assert!(normalized.years.is_sign_positive());
    }

    #[test]
    fn validate_derived_decline_rate_hides_a_parameter_the_caller_never_passed() {
        insta::assert_debug_snapshot!(
            validate_derived_decline_rate(f64::NAN).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_derived_decline_rate(f64::INFINITY).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_derived_decline_rate(f64::NEG_INFINITY).unwrap_err(),
            @"CannotSolveDecline"
        );

        // Zero is a flat segment, which is a curve a linear decline can represent.
        assert!(validate_derived_decline_rate(0.).is_ok());
        assert!(validate_derived_decline_rate(0.1).is_ok());
        assert!(validate_derived_decline_rate(-0.1).is_ok());
    }

    #[test]
    fn validate_non_zero_derived_decline_rate_also_rejects_zero() {
        insta::assert_debug_snapshot!(
            validate_non_zero_derived_decline_rate(0.).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_non_zero_derived_decline_rate(-0.).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_non_zero_derived_decline_rate(EPSILON * 0.5).unwrap_err(),
            @"CannotSolveDecline"
        );
        insta::assert_debug_snapshot!(
            validate_non_zero_derived_decline_rate(f64::NAN).unwrap_err(),
            @"CannotSolveDecline"
        );
        assert!(validate_non_zero_derived_decline_rate(0.1).is_ok());
        assert!(validate_non_zero_derived_decline_rate(-0.1).is_ok());
    }

    #[test]
    fn validate_arps_singularity_only_limits_a_shrinking_base() {
        let incline = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
        let decline = NominalDeclineRate::<AverageYearsTime>::new(0.1);

        // `b * d < 0`, so the base reaches zero at `t_max = -1 / (b * d)` = 20 years.
        let exponent = 0.5;
        assert!(
            validate_arps_singularity(exponent, incline, AverageYearsTime { years: 19.9 }).is_ok()
        );
        insta::assert_debug_snapshot!(
            validate_arps_singularity(exponent, incline, AverageYearsTime { years: 20. }).unwrap_err(),
            @"DurationTooLong"
        );
        insta::assert_debug_snapshot!(
            validate_arps_singularity(-exponent, decline, AverageYearsTime { years: 20. }).unwrap_err(),
            @"DurationTooLong"
        );

        // `b * d > 0`, so the base only grows and any duration is reachable.
        assert!(
            validate_arps_singularity(exponent, decline, AverageYearsTime { years: 1e6 }).is_ok()
        );
        assert!(
            validate_arps_singularity(-exponent, incline, AverageYearsTime { years: 1e6 }).is_ok()
        );
    }
}

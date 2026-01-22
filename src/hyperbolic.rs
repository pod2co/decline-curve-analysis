use crate::{
    DeclineCurveAnalysisError, DeclineRateSignValidation, DeclineTimeUnit, NominalDeclineRate,
    ProductionRate, approx_gte, is_effectively_zero, validate_decline_rate_sign, validate_duration,
    validate_finite, validate_incremental_volume, validate_non_zero_decline_rate,
    validate_non_zero_positive_rate,
};

/// Maximum allowed exponent magnitude for hyperbolic decline.
///
/// Exponents typically range between -2 and 2, so this is an extreme limit to catch obvious
/// errors.
const MAX_EXPONENT: f64 = 100.;

/// Validates that a hyperbolic exponent is valid.
fn validate_hyperbolic_exponent(
    exponent: f64,
    initial_decline_rate: f64,
) -> Result<(), DeclineCurveAnalysisError> {
    validate_finite(exponent, "exponent")?;
    if is_effectively_zero(exponent) {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: "exponent was approximately zero, so an exponential should be used instead"
                .to_string(),
        });
    }

    if is_effectively_zero(exponent - 1.) {
        return Err(DeclineCurveAnalysisError::InvalidInput {
            reason: "exponent was approximately one, so a harmonic should be used instead"
                .to_string(),
        });
    }

    if exponent.abs() > MAX_EXPONENT {
        return Err(DeclineCurveAnalysisError::ExponentTooLarge);
    }

    if exponent.is_sign_positive() != initial_decline_rate.is_sign_positive() {
        return Err(DeclineCurveAnalysisError::DeclineRateWrongSign);
    }

    Ok(())
}

/// A hyperbolic decline segment.
///
/// This is derived from the Arps equation when the exponent is not equal to 0 or 1.
#[derive(Debug, Clone, PartialEq)]
pub struct HyperbolicParameters<Time: DeclineTimeUnit> {
    initial_rate: ProductionRate<Time>,
    initial_decline_rate: NominalDeclineRate<Time>,
    incremental_duration: Time,
    exponent: f64,
}

impl<Time: DeclineTimeUnit> HyperbolicParameters<Time> {
    pub fn initial_rate(&self) -> ProductionRate<Time> {
        self.initial_rate
    }

    pub fn initial_decline_rate(&self) -> NominalDeclineRate<Time> {
        self.initial_decline_rate
    }

    pub fn incremental_duration(&self) -> Time {
        self.incremental_duration
    }

    pub fn exponent(&self) -> f64 {
        self.exponent
    }

    pub fn from_incremental_duration(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        incremental_duration: Time,
        exponent: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        let initial_decline_rate_value = initial_decline_rate.value();

        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(initial_decline_rate_value, "initial decline rate")?;
        validate_duration(incremental_duration)?;
        validate_hyperbolic_exponent(exponent, initial_decline_rate_value)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
            exponent,
        })
    }

    pub fn from_incremental_volume(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        incremental_volume: f64,
        exponent: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        let initial_decline_rate_value = initial_decline_rate.value();

        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(initial_decline_rate_value, "initial decline rate")?;
        validate_incremental_volume(incremental_volume)?;
        validate_hyperbolic_exponent(exponent, initial_decline_rate_value)?;

        let one_minus_exponent = 1. - exponent;

        // For hyperbolic declines with a positive decline rate, and 0 < exponent < 1, the maximum
        // volume possible (as time approaches infinity) is given by:
        //
        //   q_i / ((1 - b) * d)
        //
        // If the incremental volume is greater or equal to this, then we can't solve the decline.
        //
        // There should be no maximum volume for all other cases (inclines and/or other exponent
        // ranges).
        if initial_decline_rate_value > 0. && exponent > 0. && exponent < 1. {
            let max_volume = initial_rate.value / (one_minus_exponent * initial_decline_rate_value);
            if approx_gte(incremental_volume, max_volume) {
                return Err(DeclineCurveAnalysisError::CannotSolveDecline);
            }
        }

        let base = 1.
            - (incremental_volume * initial_decline_rate_value * one_minus_exponent)
                / initial_rate.value;
        let duration_denom = exponent * initial_decline_rate_value;
        let incremental_duration =
            Time::from((base.powf(-exponent / one_minus_exponent) - 1.) / duration_denom);
        validate_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
            exponent,
        })
    }

    pub fn from_final_decline_rate(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        final_decline_rate: NominalDeclineRate<Time>,
        exponent: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        let initial_decline_rate_value = initial_decline_rate.value();
        let final_decline_rate_value = final_decline_rate.value();

        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(initial_decline_rate_value, "initial decline rate")?;
        validate_non_zero_decline_rate(final_decline_rate_value, "final decline rate")?;
        validate_hyperbolic_exponent(exponent, initial_decline_rate_value)?;

        if initial_decline_rate_value.is_sign_positive()
            != final_decline_rate_value.is_sign_positive()
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        if exponent > 0. {
            if final_decline_rate_value > initial_decline_rate_value {
                return Err(DeclineCurveAnalysisError::CannotSolveDecline);
            }
        } else if final_decline_rate_value < initial_decline_rate_value {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let incremental_duration = Time::from(
            (initial_decline_rate_value / final_decline_rate_value - 1.)
                / (exponent * initial_decline_rate_value),
        );
        validate_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
            exponent,
        })
    }

    pub fn from_final_rate(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        final_rate: ProductionRate<Time>,
        exponent: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        let initial_decline_rate_value = initial_decline_rate.value();

        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(initial_decline_rate_value, "initial decline rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;
        validate_hyperbolic_exponent(exponent, initial_decline_rate_value)?;

        match validate_decline_rate_sign(
            initial_decline_rate_value,
            initial_rate.value,
            final_rate.value,
        )? {
            DeclineRateSignValidation::Continue => {}
            DeclineRateSignValidation::ZeroDuration => {
                return Ok(Self {
                    initial_rate,
                    initial_decline_rate,
                    incremental_duration: Time::from(0.),
                    exponent,
                });
            }
        }

        let incremental_duration = Time::from(
            ((initial_rate.value / final_rate.value).powf(exponent) - 1.0)
                / (exponent * initial_decline_rate_value),
        );
        validate_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
            exponent,
        })
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        let initial_decline_rate_value = self.initial_decline_rate.value();

        let factor_denom = self
            .initial_decline_rate
            .value()
            .mul_add(-self.exponent, initial_decline_rate_value);

        // `q_i / (a_i * (1 - b))`
        let factor = self.initial_rate.value() / factor_denom;

        // `1 - 1 / b`
        let power = 1. - 1. / self.exponent;

        // `b * a_i`
        let exponent_times_initial_decline_rate = self.exponent * initial_decline_rate_value;

        let base = time
            .value()
            .mul_add(exponent_times_initial_decline_rate, 1.);

        base.powf(power).mul_add(-factor, factor)
    }

    pub fn incremental_volume_at_time(&self, time: Time) -> f64 {
        if time.value() > self.incremental_duration.value() {
            self.incremental_volume()
        } else {
            self.incremental_volume_at_time_without_clamping(time)
        }
    }

    pub fn incremental_volume(&self) -> f64 {
        self.incremental_volume_at_time_without_clamping(self.incremental_duration)
    }

    fn rate_at_time_without_clamping(&self, time: Time) -> ProductionRate<Time> {
        ProductionRate::new(
            self.initial_rate.value
                / (time
                    .value()
                    .mul_add(self.exponent * self.initial_decline_rate.value(), 1.))
                .powf(1. / self.exponent),
        )
    }

    pub fn final_rate(&self) -> ProductionRate<Time> {
        self.rate_at_time_without_clamping(self.incremental_duration)
    }

    pub fn rate_at_time(&self, time: Time) -> ProductionRate<Time> {
        if time.value() > self.incremental_duration.value() {
            self.final_rate()
        } else {
            self.rate_at_time_without_clamping(time)
        }
    }
}

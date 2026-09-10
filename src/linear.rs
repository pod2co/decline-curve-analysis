use crate::{
    DeclineCurveAnalysisError, DeclineTimeUnit, NominalDeclineRate, ProductionRate, approx_eq,
    is_effectively_zero, validate_derived_decline_rate, validate_derived_duration,
    validate_duration, validate_finite, validate_incremental_volume, validate_non_zero_duration,
    validate_non_zero_positive_rate, validate_non_zero_positive_volume,
};

/// A linear decline segment.
///
/// The rate falls at a constant absolute pace, so the nominal decline rate `d / (1 - d * t)` grows
/// over the segment and the one held here is its initial value.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearParameters<Time: DeclineTimeUnit> {
    initial_rate: ProductionRate<Time>,
    initial_decline_rate: NominalDeclineRate<Time>,
    incremental_duration: Time,
}

impl<Time: DeclineTimeUnit> LinearParameters<Time> {
    pub fn initial_rate(&self) -> ProductionRate<Time> {
        self.initial_rate
    }

    pub fn initial_decline_rate(&self) -> NominalDeclineRate<Time> {
        self.initial_decline_rate
    }

    pub fn incremental_duration(&self) -> Time {
        self.incremental_duration
    }

    pub fn from_incremental_duration(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        incremental_duration: Time,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_finite(initial_decline_rate.value(), "initial decline rate")?;
        validate_duration(incremental_duration)?;

        let result = Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
        };

        let final_rate = result.rate_at_time_without_clamping(incremental_duration);
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;

        Ok(result)
    }

    pub fn from_incremental_volume(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_finite(initial_decline_rate.value(), "initial decline rate")?;
        validate_incremental_volume(incremental_volume)?;

        if is_effectively_zero(incremental_volume) {
            return Ok(Self {
                initial_rate,
                initial_decline_rate,
                incremental_duration: Time::from(0.),
            });
        }

        // Solve quadratic equation for incremental duration.
        let a = -0.5 * initial_decline_rate.value() * initial_rate.value;
        let b = initial_rate.value;
        let c = -incremental_volume;

        let discriminant = b * b - 4. * a * c;

        if discriminant < 0. {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        // Only take the positive root. The negative root would be the time at which the rate
        // becomes negative and causes the cumulative volume to be reached again, but that's not a
        // valid solution for this case.
        //
        // `2c / (-b - sqrt(d))` is that same root written to avoid the cancellation in
        // `-b + sqrt(d)`, which otherwise loses the small durations entirely. It also stays
        // defined when a zero decline rate takes `a` to zero.
        let incremental_duration = Time::from((2. * c) / (-b - discriminant.sqrt()));
        let incremental_duration = validate_derived_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
        })
    }

    pub fn from_final_rate(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        final_rate: ProductionRate<Time>,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_finite(initial_decline_rate.value(), "initial decline rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;

        if is_effectively_zero(initial_decline_rate.value()) {
            // A flat segment holds its rate, so it reaches an equal one at once and a different
            // one never.
            if approx_eq(initial_rate.value, final_rate.value) {
                return Ok(Self {
                    initial_rate,
                    initial_decline_rate,
                    incremental_duration: Time::from(0.),
                });
            }

            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let incremental_duration = Time::from(
            (initial_rate.value - final_rate.value)
                / (initial_rate.value * initial_decline_rate.value()),
        );
        let incremental_duration = validate_derived_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
        })
    }

    /// Builds a segment from a duration and a final rate, solving the initial decline rate that
    /// reaches both at the same point.
    pub fn from_incremental_duration_and_final_rate(
        initial_rate: ProductionRate<Time>,
        incremental_duration: Time,
        final_rate: ProductionRate<Time>,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;
        validate_non_zero_duration(incremental_duration)?;

        // `q_f = q_i * (1 - d * t)`, rearranged for `d`.
        let initial_decline_rate = NominalDeclineRate::new(
            (1. - final_rate.value / initial_rate.value) / incremental_duration.value(),
        );
        validate_derived_decline_rate(initial_decline_rate.value())?;

        Self::from_incremental_duration(initial_rate, initial_decline_rate, incremental_duration)
    }

    /// Builds a segment from a final rate and an incremental volume, solving the initial decline
    /// rate that reaches both at the same point.
    ///
    /// See [`Self::from_incremental_duration_and_final_rate`].
    pub fn from_final_rate_and_incremental_volume(
        initial_rate: ProductionRate<Time>,
        final_rate: ProductionRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;
        validate_non_zero_positive_volume(incremental_volume)?;

        // Substituting `t = (1 - q_f / q_i) / d` into `n_p = q_i * t - d * q_i * t ^ 2 / 2` leaves
        // `d` as the only unknown.
        let fraction_declined = 1. - final_rate.value / initial_rate.value;
        let initial_decline_rate = NominalDeclineRate::new(
            initial_rate.value * fraction_declined * (2. - fraction_declined)
                / (2. * incremental_volume),
        );
        validate_derived_decline_rate(initial_decline_rate.value())?;

        // A decline rate that rounds to zero is a flat segment, so the volume sets the length
        // instead of the final rate.
        if is_effectively_zero(initial_decline_rate.value()) {
            return Self::from_incremental_volume(
                initial_rate,
                NominalDeclineRate::new(0.),
                incremental_volume,
            );
        }

        Self::from_final_rate(initial_rate, initial_decline_rate, final_rate)
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        let time_value = time.value();

        self.initial_rate.value * time_value
            - 0.5 * self.initial_decline_rate.value() * self.initial_rate.value * time_value.powi(2)
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
        ProductionRate::new(self.initial_rate.value.mul_add(
            -self.initial_decline_rate.value() * time.value(),
            self.initial_rate.value,
        ))
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

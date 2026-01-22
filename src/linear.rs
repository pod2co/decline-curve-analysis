use crate::{
    DeclineCurveAnalysisError, DeclineTimeUnit, NominalDeclineRate, ProductionRate, approx_eq,
    is_effectively_zero, validate_duration, validate_incremental_volume,
    validate_non_zero_decline_rate, validate_non_zero_positive_rate,
};

/// A linear decline segment.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearParameters<Time: DeclineTimeUnit> {
    initial_rate: ProductionRate<Time>,
    decline_rate: NominalDeclineRate<Time>,
    incremental_duration: Time,
}

impl<Time: DeclineTimeUnit> LinearParameters<Time> {
    pub fn initial_rate(&self) -> ProductionRate<Time> {
        self.initial_rate
    }

    pub fn decline_rate(&self) -> NominalDeclineRate<Time> {
        self.decline_rate
    }

    pub fn incremental_duration(&self) -> Time {
        self.incremental_duration
    }

    pub fn from_incremental_duration(
        initial_rate: ProductionRate<Time>,
        decline_rate: NominalDeclineRate<Time>,
        incremental_duration: Time,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(decline_rate.value(), "decline rate")?;
        validate_duration(incremental_duration)?;

        let result = Self {
            initial_rate,
            decline_rate,
            incremental_duration,
        };

        let final_rate = result.rate_at_time_without_clamping(incremental_duration);
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;

        Ok(result)
    }

    pub fn from_incremental_volume(
        initial_rate: ProductionRate<Time>,
        decline_rate: NominalDeclineRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(decline_rate.value(), "decline rate")?;
        validate_incremental_volume(incremental_volume)?;

        if is_effectively_zero(incremental_volume) {
            return Ok(Self {
                initial_rate,
                decline_rate,
                incremental_duration: Time::from(0.),
            });
        }

        // Solve quadratic equation for incremental duration.
        let a = -0.5 * decline_rate.value() * initial_rate.value;
        let b = initial_rate.value;
        let c = -incremental_volume;

        let discriminant = b * b - 4. * a * c;

        if discriminant < 0. {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        // Only take the positive root. The negative root would be the time at which the rate
        // becomes negative and causes the cumulative volume to be reached again, but that's not a
        // valid solution for this case.
        let incremental_duration = Time::from((-b + discriminant.sqrt()) / (2. * a));
        validate_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            decline_rate,
            incremental_duration,
        })
    }

    pub fn from_final_rate(
        initial_rate: ProductionRate<Time>,
        decline_rate: NominalDeclineRate<Time>,
        final_rate: ProductionRate<Time>,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(decline_rate.value(), "decline rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;

        if is_effectively_zero(decline_rate.value()) {
            if approx_eq(initial_rate.value, final_rate.value) {
                return Ok(Self {
                    initial_rate,
                    decline_rate,
                    incremental_duration: Time::from(0.),
                });
            }
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let incremental_duration = Time::from(
            (initial_rate.value - final_rate.value) / (initial_rate.value * decline_rate.value()),
        );
        validate_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            decline_rate,
            incremental_duration,
        })
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        let time_value = time.value();

        self.initial_rate.value * time_value
            - 0.5 * self.decline_rate.value() * self.initial_rate.value * time_value.powi(2)
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
            -self.decline_rate.value() * time.value(),
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

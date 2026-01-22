use crate::{
    DeclineCurveAnalysisError, DeclineRateSignValidation, DeclineTimeUnit, NominalDeclineRate,
    ProductionRate, approx_gte, validate_decline_rate_sign, validate_duration,
    validate_incremental_volume, validate_non_zero_decline_rate, validate_non_zero_positive_rate,
};

/// An exponential decline segment that represents a decline with a constant nominal decline rate.
///
/// This is derived from the Arps equation for the case when the exponent is 0.
#[derive(Debug, Clone, PartialEq)]
pub struct ExponentialParameters<Time: DeclineTimeUnit> {
    initial_rate: ProductionRate<Time>,
    decline_rate: NominalDeclineRate<Time>,
    incremental_duration: Time,
}

impl<Time: DeclineTimeUnit> ExponentialParameters<Time> {
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

        Ok(Self {
            initial_rate,
            decline_rate,
            incremental_duration,
        })
    }

    pub fn from_incremental_volume(
        initial_rate: ProductionRate<Time>,
        decline_rate: NominalDeclineRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(decline_rate.value(), "decline rate")?;
        validate_incremental_volume(incremental_volume)?;

        // For exponential declines with a positive decline rate, the maximum volume possible
        // (as time approaches infinity) is given by:
        //
        //   q_i / d
        //
        // If the incremental volume is greater or equal to this, then we can't solve the decline.
        //
        // For negative decline rates, there is no maximum volume.
        if decline_rate.value() > 0. {
            let max_volume = initial_rate.value / decline_rate.value();
            if approx_gte(incremental_volume, max_volume) {
                return Err(DeclineCurveAnalysisError::CannotSolveDecline);
            }
        }

        let incremental_duration = Time::from(
            -((-incremental_volume * decline_rate.value()) / initial_rate.value).ln_1p()
                / decline_rate.value(),
        );
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

        match validate_decline_rate_sign(
            decline_rate.value(),
            initial_rate.value,
            final_rate.value,
        )? {
            DeclineRateSignValidation::Continue => {}
            DeclineRateSignValidation::ZeroDuration => {
                return Ok(Self {
                    initial_rate,
                    decline_rate,
                    incremental_duration: Time::from(0.),
                });
            }
        }

        let incremental_duration =
            Time::from((initial_rate.value / final_rate.value).ln() / decline_rate.value());
        validate_duration(incremental_duration)?;

        Ok(Self {
            initial_rate,
            decline_rate,
            incremental_duration,
        })
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        let exp_part = -(-self.decline_rate.value() * time.value()).exp_m1();
        (exp_part * self.initial_rate.value) / self.decline_rate.value()
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
            self.initial_rate.value * (-self.decline_rate.value() * time.value()).exp(),
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

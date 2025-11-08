use crate::{
    DeclineCurveAnalysisError, DeclineRateSignValidation, DeclineTimeUnit, NominalDeclineRate,
    ProductionRate, validate_decline_rate_sign,
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
        if initial_rate.value <= 0.
            || decline_rate.value() == 0.
            || incremental_duration.value() < 0.
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

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
        if initial_rate.value <= 0. || decline_rate.value() == 0. || incremental_volume < 0. {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let incremental_duration =
            -((-incremental_volume * decline_rate.value()) / initial_rate.value).ln_1p()
                / decline_rate.value();

        Ok(Self {
            initial_rate,
            decline_rate,
            incremental_duration: Time::from(incremental_duration),
        })
    }

    pub fn from_final_rate(
        initial_rate: ProductionRate<Time>,
        decline_rate: NominalDeclineRate<Time>,
        final_rate: ProductionRate<Time>,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        let decline_rate_value = decline_rate.value();

        if initial_rate.value <= 0. || decline_rate_value == 0. || final_rate.value <= 0. {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        match validate_decline_rate_sign(decline_rate_value, initial_rate.value, final_rate.value)?
        {
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
            (initial_rate.value / final_rate.value).ln() / decline_rate_value;

        Ok(Self {
            initial_rate,
            decline_rate,
            incremental_duration: Time::from(incremental_duration),
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

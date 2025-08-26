use crate::{
    DeclineCurveAnalysisError, DeclineRateSignValidation, DeclineTimeUnit, NominalDeclineRate,
    ProductionRate, validate_decline_rate_sign,
};

/// A hyperbolic decline segment.
///
/// This is derived from the Arps equation when the exponent is not equal to 0 or 1.
#[derive(Debug, Clone)]
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
        if initial_rate.value <= 0.
            || initial_decline_rate.value() == 0.
            || incremental_duration.value() < 0.
            || exponent == 0.
            || exponent == 1.
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

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
        if initial_rate.value <= 0.
            || initial_decline_rate.value() == 0.
            || incremental_volume < 0.
            || exponent == 0.
            || exponent == 1.
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let one_minus_exponent = 1. - exponent;
        let base = 1.
            - (incremental_volume * initial_decline_rate.value() * one_minus_exponent)
                / initial_rate.value;
        let denom = exponent * initial_decline_rate.value();
        let incremental_duration = (base.powf(-exponent / one_minus_exponent) - 1.) / denom;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration: Time::from(incremental_duration),
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

        if initial_rate.value <= 0.
            || initial_decline_rate_value == 0.
            || final_decline_rate_value == 0.
            || exponent == 0.
            || exponent == 1.
            || initial_decline_rate_value.is_sign_positive()
                != final_decline_rate_value.is_sign_positive()
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        if exponent > 0. {
            if final_decline_rate_value > initial_decline_rate_value {
                return Err(DeclineCurveAnalysisError::CannotSolveDecline);
            }
        } else {
            if final_decline_rate_value < initial_decline_rate_value {
                return Err(DeclineCurveAnalysisError::CannotSolveDecline);
            }
        }

        let incremental_duration = (initial_decline_rate_value / final_decline_rate_value - 1.)
            / (exponent * initial_decline_rate_value);

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration: Time::from(incremental_duration),
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

        if initial_rate.value <= 0.
            || initial_decline_rate_value == 0.
            || final_rate.value <= 0.
            || exponent == 0.
            || exponent == 1.
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

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

        let incremental_duration = ((initial_rate.value / final_rate.value).powf(exponent) - 1.)
            / (exponent * initial_decline_rate_value);

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration: Time::from(incremental_duration),
            exponent,
        })
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        let factor_denom = self
            .initial_decline_rate
            .value()
            .mul_add(-self.exponent, self.initial_decline_rate.value());

        // `q_i / (a_i * (1 - b))`
        let factor = self.initial_rate.value() / factor_denom;

        // `1 - 1 / b`
        let power = 1. - 1. / self.exponent;

        // `b * a_i`
        let exponent_times_initial_decline_rate = self.exponent * self.initial_decline_rate.value();

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

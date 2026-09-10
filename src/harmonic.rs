use crate::{
    DeclineCurveAnalysisError, DeclineRateSignValidation, DeclineTimeUnit, NominalDeclineRate,
    ProductionRate, validate_arps_singularity, validate_decline_rate_sign,
    validate_derived_duration, validate_duration, validate_incremental_volume,
    validate_non_zero_decline_rate, validate_non_zero_derived_decline_rate,
    validate_non_zero_duration, validate_non_zero_positive_rate, validate_non_zero_positive_volume,
};

/// A harmonic decline is the Arps equation at an exponent of one, which puts its singularity at
/// `t_max = 1 / |d|`.
const EXPONENT: f64 = 1.;

/// A harmonic decline segment.
///
/// This is derived from the Arps equation for the case when the exponent is 1.
#[derive(Debug, Clone, PartialEq)]
pub struct HarmonicParameters<Time: DeclineTimeUnit> {
    initial_rate: ProductionRate<Time>,
    initial_decline_rate: NominalDeclineRate<Time>,
    incremental_duration: Time,
}

impl<Time: DeclineTimeUnit> HarmonicParameters<Time> {
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
        validate_non_zero_decline_rate(initial_decline_rate.value(), "initial decline rate")?;
        validate_duration(incremental_duration)?;
        validate_arps_singularity(EXPONENT, initial_decline_rate, incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
        })
    }

    pub fn from_incremental_volume(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(initial_decline_rate.value(), "initial decline rate")?;
        validate_incremental_volume(incremental_volume)?;

        let incremental_duration = Time::from(
            (((incremental_volume * initial_decline_rate.value()) / initial_rate.value).exp_m1())
                / initial_decline_rate.value(),
        );
        let incremental_duration = validate_derived_duration(incremental_duration)?;

        validate_arps_singularity(EXPONENT, initial_decline_rate, incremental_duration)?;

        Ok(Self {
            initial_rate,
            initial_decline_rate,
            incremental_duration,
        })
    }

    pub fn from_final_decline_rate(
        initial_rate: ProductionRate<Time>,
        initial_decline_rate: NominalDeclineRate<Time>,
        final_decline_rate: NominalDeclineRate<Time>,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_decline_rate(initial_decline_rate.value(), "initial decline rate")?;
        validate_non_zero_decline_rate(final_decline_rate.value(), "final decline rate")?;

        let initial_decline_rate_value = initial_decline_rate.value();
        let final_decline_rate_value = final_decline_rate.value();

        if initial_decline_rate_value.is_sign_positive()
            != final_decline_rate_value.is_sign_positive()
        {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let incremental_duration =
            Time::from(1. / final_decline_rate_value - 1. / initial_decline_rate_value);
        let incremental_duration = validate_derived_duration(incremental_duration)?;

        validate_arps_singularity(EXPONENT, initial_decline_rate, incremental_duration)?;

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
        validate_non_zero_decline_rate(initial_decline_rate.value(), "initial decline rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;

        match validate_decline_rate_sign(
            initial_decline_rate.value(),
            initial_rate.value,
            final_rate.value,
        )? {
            DeclineRateSignValidation::Continue => {}
            DeclineRateSignValidation::ZeroDuration => {
                return Ok(Self {
                    initial_rate,
                    initial_decline_rate,
                    incremental_duration: Time::from(0.),
                });
            }
        }

        let incremental_duration = Time::from(
            (initial_rate.value - final_rate.value)
                / (initial_decline_rate.value() * final_rate.value),
        );
        let incremental_duration = validate_derived_duration(incremental_duration)?;

        validate_arps_singularity(EXPONENT, initial_decline_rate, incremental_duration)?;

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

        // `q_f = q_i / (1 + d * t)`, rearranged for `d`.
        let initial_decline_rate = NominalDeclineRate::new(
            (initial_rate.value / final_rate.value - 1.) / incremental_duration.value(),
        );
        validate_non_zero_derived_decline_rate(initial_decline_rate.value())?;

        Self::from_incremental_duration(initial_rate, initial_decline_rate, incremental_duration)
    }

    /// Builds a segment from a final rate and an incremental volume, solving the initial decline
    /// rate that reaches both at the same point.
    pub fn from_final_rate_and_incremental_volume(
        initial_rate: ProductionRate<Time>,
        final_rate: ProductionRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_non_zero_positive_rate(initial_rate.value, "initial rate")?;
        validate_non_zero_positive_rate(final_rate.value, "final rate")?;
        validate_non_zero_positive_volume(incremental_volume)?;

        // `n_p = q_i * ln(q_i / q_f) / d`, rearranged for `d`.
        let initial_decline_rate = NominalDeclineRate::new(
            initial_rate.value * (initial_rate.value / final_rate.value).ln() / incremental_volume,
        );
        validate_non_zero_derived_decline_rate(initial_decline_rate.value())?;

        Self::from_final_rate(initial_rate, initial_decline_rate, final_rate)
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        (self.initial_rate.value * (time.value() * self.initial_decline_rate.value()).ln_1p())
            / self.initial_decline_rate.value()
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
            self.initial_rate.value / (time.value().mul_add(self.initial_decline_rate.value(), 1.)),
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

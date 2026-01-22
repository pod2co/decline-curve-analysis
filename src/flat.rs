use crate::{
    DeclineCurveAnalysisError, DeclineTimeUnit, ProductionRate, is_effectively_zero,
    validate_duration, validate_incremental_volume, validate_positive,
};

/// A flat segment that represents a constant production rate.
#[derive(Debug, Clone, PartialEq)]
pub struct FlatParameters<Time: DeclineTimeUnit> {
    rate: ProductionRate<Time>,
    incremental_duration: Time,
}

impl<Time: DeclineTimeUnit> FlatParameters<Time> {
    pub fn rate(&self) -> ProductionRate<Time> {
        self.rate
    }

    pub fn incremental_duration(&self) -> Time {
        self.incremental_duration
    }

    pub fn from_incremental_duration(
        rate: ProductionRate<Time>,
        incremental_duration: Time,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_positive(rate.value, "rate")?;
        validate_duration(incremental_duration)?;

        Ok(Self {
            rate,
            incremental_duration,
        })
    }

    pub fn from_incremental_volume(
        rate: ProductionRate<Time>,
        incremental_volume: f64,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_positive(rate.value, "rate")?;
        validate_incremental_volume(incremental_volume)?;

        if is_effectively_zero(incremental_volume) {
            return Ok(Self {
                rate,
                incremental_duration: Time::from(0.),
            });
        }

        if is_effectively_zero(rate.value) {
            return Err(DeclineCurveAnalysisError::CannotSolveDecline);
        }

        let incremental_duration = Time::from(incremental_volume / rate.value);
        validate_duration(incremental_duration)?;

        Ok(Self {
            rate,
            incremental_duration,
        })
    }

    fn incremental_volume_at_time_without_clamping(&self, time: Time) -> f64 {
        self.rate.value * time.value()
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

    pub fn final_rate(&self) -> ProductionRate<Time> {
        self.rate
    }

    pub fn rate_at_time(&self, _time: Time) -> ProductionRate<Time> {
        self.rate
    }
}

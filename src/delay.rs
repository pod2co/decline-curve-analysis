use crate::{DeclineCurveAnalysisError, DeclineTimeUnit, ProductionRate, validate_duration};

/// A no-op delay segment that represents a delay with no volume. It can be useful to represent an
/// arbitrary delay in forecasts.
#[derive(Debug, Clone, PartialEq)]
pub struct DelayParameters<Time: DeclineTimeUnit> {
    incremental_duration: Time,
}

impl<Time: DeclineTimeUnit> DelayParameters<Time> {
    const ZERO_PRODUCTION_RATE: ProductionRate<Time> = ProductionRate::new(0.);

    pub const fn rate(&self) -> ProductionRate<Time> {
        Self::ZERO_PRODUCTION_RATE
    }

    pub fn incremental_duration(&self) -> Time {
        self.incremental_duration
    }

    pub fn from_incremental_duration(
        incremental_duration: Time,
    ) -> Result<Self, DeclineCurveAnalysisError> {
        validate_duration(incremental_duration)?;

        Ok(Self {
            incremental_duration,
        })
    }

    pub const fn incremental_volume_at_time(&self, _time: Time) -> f64 {
        0.
    }

    pub const fn incremental_volume(&self) -> f64 {
        0.
    }

    pub const fn final_rate(&self) -> ProductionRate<Time> {
        Self::ZERO_PRODUCTION_RATE
    }

    pub const fn rate_at_time(&self, _time: Time) -> ProductionRate<Time> {
        Self::ZERO_PRODUCTION_RATE
    }
}

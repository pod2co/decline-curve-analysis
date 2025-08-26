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

/// An error type for invalid parameters.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DeclineCurveAnalysisError {
    #[error("decline rate too high")]
    DeclineRateTooHigh,
    #[error("decline rate has wrong sign")]
    DeclineRateWrongSign,
    #[error("cannot solve decline")]
    CannotSolveDecline,
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

impl Into<ProductionRate<AverageDaysTime>> for ProductionRate<AverageYearsTime> {
    fn into(self) -> ProductionRate<AverageDaysTime> {
        ProductionRate::new(self.value * AverageDaysTime::LENGTH / AverageYearsTime::LENGTH)
    }
}

impl Into<ProductionRate<AverageYearsTime>> for ProductionRate<AverageDaysTime> {
    fn into(self) -> ProductionRate<AverageYearsTime> {
        ProductionRate::new(self.value * AverageYearsTime::LENGTH / AverageDaysTime::LENGTH)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeclineRateSignValidation {
    Continue,
    ZeroDuration,
}

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

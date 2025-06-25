use crate::DeclineCurveAnalysisError;
use std::marker::PhantomData;

/// A time unit for decline parameters. The base unit is defined in terms of average days, where an
/// average year is 365.25 days. This allows for conversions between different time units, even
/// with different average year lengths (e.g., 365 days in some software).
pub trait DeclineTimeUnit: Copy + Clone + std::fmt::Debug + PartialEq + From<f64> {
    const LENGTH: f64;

    fn value(&self) -> f64;

    fn to_unit<OtherTimeUnit: DeclineTimeUnit>(self) -> OtherTimeUnit {
        OtherTimeUnit::from((self.value() * Self::LENGTH) / OtherTimeUnit::LENGTH)
    }

    fn length(&self) -> f64 {
        Self::LENGTH
    }
}

/// Average year length of 365.25 days.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AverageYearsTime {
    pub years: f64,
}

impl From<f64> for AverageYearsTime {
    fn from(years: f64) -> Self {
        Self { years }
    }
}

impl DeclineTimeUnit for AverageYearsTime {
    const LENGTH: f64 = 365.25;

    fn value(&self) -> f64 {
        self.years
    }
}

/// Average day length of 1 day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AverageDaysTime {
    pub days: f64,
}

impl From<f64> for AverageDaysTime {
    fn from(days: f64) -> Self {
        Self { days }
    }
}

impl DeclineTimeUnit for AverageDaysTime {
    const LENGTH: f64 = 1.;

    fn value(&self) -> f64 {
        self.days
    }
}

/// The nominal decline rate as a fraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NominalDeclineRate<Time: DeclineTimeUnit> {
    value: f64,
    _time: PhantomData<Time>,
}

impl<Time: DeclineTimeUnit> NominalDeclineRate<Time> {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            _time: PhantomData,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn to_secant_effective(
        &self,
        exponent: f64,
    ) -> Result<SecantEffectiveDeclineRate<Time>, DeclineCurveAnalysisError> {
        if exponent == 0. {
            // Handle as an exponential segment, so use the tangent effective conversion.
            let tangent_effective = self.to_tangent_effective()?;

            // Then just call it a secant effective.
            Ok(SecantEffectiveDeclineRate::new(tangent_effective.value))
        } else {
            let secant_effective = 1. - (self.value.mul_add(exponent, 1.)).powf(-1. / exponent);

            Ok(SecantEffectiveDeclineRate::new(secant_effective))
        }
    }

    pub fn to_tangent_effective(
        &self,
    ) -> Result<TangentEffectiveDeclineRate<Time>, DeclineCurveAnalysisError> {
        let tangent_effective = 1. - (-self.value).exp();

        Ok(TangentEffectiveDeclineRate::new(tangent_effective))
    }

    fn to_time<ToTimeUnit: DeclineTimeUnit>(&self) -> NominalDeclineRate<ToTimeUnit> {
        NominalDeclineRate {
            value: (self.value * ToTimeUnit::LENGTH) / Time::LENGTH,
            _time: PhantomData,
        }
    }
}

impl From<NominalDeclineRate<AverageDaysTime>> for NominalDeclineRate<AverageYearsTime> {
    fn from(value: NominalDeclineRate<AverageDaysTime>) -> Self {
        value.to_time()
    }
}

impl From<NominalDeclineRate<AverageYearsTime>> for NominalDeclineRate<AverageDaysTime> {
    fn from(value: NominalDeclineRate<AverageYearsTime>) -> Self {
        value.to_time()
    }
}

/// The secant effective decline rate as a fraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecantEffectiveDeclineRate<Time: DeclineTimeUnit> {
    value: f64,
    _time: PhantomData<Time>,
}

impl<Time: DeclineTimeUnit> SecantEffectiveDeclineRate<Time> {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            _time: PhantomData,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    fn to_nominal_inner(
        &self,
        exponent: f64,
    ) -> Result<NominalDeclineRate<Time>, DeclineCurveAnalysisError> {
        if self.value >= 1. {
            return Err(DeclineCurveAnalysisError::DeclineRateTooHigh);
        }

        Ok(NominalDeclineRate::new(
            (((1. - self.value).powf(-exponent)) - 1.) / exponent,
        ))
    }

    pub fn to_nominal(
        &self,
        exponent: f64,
    ) -> Result<NominalDeclineRate<Time>, DeclineCurveAnalysisError> {
        if exponent == 0. {
            // Handle as an exponential segment, so treat the decline rate as a tangent effective
            // conversion.
            TangentEffectiveDeclineRate::new(self.value).to_nominal()
        } else {
            self.to_nominal_inner(exponent)
        }
    }

    pub fn to_tangent_effective(
        &self,
        exponent: f64,
    ) -> Result<TangentEffectiveDeclineRate<Time>, DeclineCurveAnalysisError> {
        if exponent == 0. {
            // It's an exponential, so secant effective and tangent effective are the same.
            Ok(TangentEffectiveDeclineRate::new(self.value))
        } else {
            let nominal = self.to_nominal_inner(exponent)?;
            nominal.to_tangent_effective()
        }
    }
}

/// The tangent effective decline rate as a fraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TangentEffectiveDeclineRate<Time: DeclineTimeUnit> {
    value: f64,
    _time: PhantomData<Time>,
}

impl<Time: DeclineTimeUnit> TangentEffectiveDeclineRate<Time> {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            _time: PhantomData,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    fn to_nominal_inner(&self) -> Result<NominalDeclineRate<Time>, DeclineCurveAnalysisError> {
        if self.value >= 1. {
            return Err(DeclineCurveAnalysisError::DeclineRateTooHigh);
        }

        Ok(NominalDeclineRate::new(-(-self.value).ln_1p()))
    }

    pub fn to_nominal(&self) -> Result<NominalDeclineRate<Time>, DeclineCurveAnalysisError> {
        self.to_nominal_inner()
    }

    pub fn to_secant_effective(
        &self,
        exponent: f64,
    ) -> Result<SecantEffectiveDeclineRate<Time>, DeclineCurveAnalysisError> {
        if exponent == 0. {
            // It's an exponential, so secant effective and tangent effective are the same.
            Ok(SecantEffectiveDeclineRate::new(self.value))
        } else {
            let nominal = self.to_nominal_inner()?;
            nominal.to_secant_effective(exponent)
        }
    }
}

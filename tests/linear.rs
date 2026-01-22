use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, LinearParameters, NominalDeclineRate, ProductionRate,
};
use proptest::prelude::*;

#[test]
fn linear_from_incremental_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.01).into();
    let incremental_duration = AverageDaysTime { days: 4. * 365.25 };

    let calculated_duration = LinearParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        incremental_duration,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"1461");
}

#[test]
fn linear_from_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_volume = 43830.;

    let calculated_duration =
        LinearParameters::from_incremental_volume(initial_rate, decline_rate, incremental_volume)
            .unwrap()
            .incremental_duration()
            .days;

    insta::assert_snapshot!(calculated_duration, @"1461");

    // Try with a positive decline_rate to ensure we can reach the same point in time. This ensures we
    // handle both positive and negative decline_rates.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(10.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-1.).into();
    let incremental_volume = 43830.;

    let calculated_duration =
        LinearParameters::from_incremental_volume(initial_rate, decline_rate, incremental_volume)
            .unwrap()
            .incremental_duration()
            .days;

    insta::assert_snapshot!(calculated_duration, @"1461");
}

#[test]
fn linear_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let calculated_duration =
        LinearParameters::from_final_rate(initial_rate, decline_rate, final_rate)
            .unwrap()
            .incremental_duration()
            .days;

    insta::assert_snapshot!(calculated_duration, @"1461");
}

#[test]
fn linear_incremental_volume_at_time() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_duration = AverageDaysTime { days: 1461. };

    let parameters = LinearParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        incremental_duration,
    )
    .unwrap();

    // Calculate past the end to check the total.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 1470. }), @"43830");

    // Check a point somewhere in the middle.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 1470. }), @"29354.722792607805");
}

#[test]
fn linear_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_duration = AverageDaysTime { days: 1461. };

    let parameters = LinearParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        incremental_duration,
    )
    .unwrap();

    insta::assert_snapshot!(parameters.final_rate().value(), @"10.000000000000004");
}

#[test]
fn prevent_negative_rates() {
    // Use a long duration that would cause the rate to become negative at some point.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_duration = AverageDaysTime { days: 10_000. };

    let parameters = LinearParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        incremental_duration,
    );

    insta::assert_snapshot!(parameters.unwrap_err(), @"final rate is negative or zero, but expected a positive number");
}

#[test]
fn rejects_zero_decline_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.);
    let volume = 1000.;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

    insta::assert_snapshot!(result.unwrap_err(), @"decline rate is approximately zero, but expected it to be non-zero");
}

#[test]
fn zero_duration_from_zero_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.1);

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, 0.);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn zero_duration_from_extremely_small_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.1);
    let tiny_volume = 1e-300;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, tiny_volume);
    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"0");
}

#[test]
fn large_rate_and_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(150_000.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let volume = 10_000_000.;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"70.02271045856541");
}

#[test]
fn rejects_different_rates_when_decline_rate_is_zero() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.);
    let final_rate = ProductionRate::<AverageDaysTime>::new(50.);

    let result = LinearParameters::from_final_rate(initial_rate, decline_rate, final_rate);

    insta::assert_snapshot!(result.unwrap_err(), @"decline rate is approximately zero, but expected it to be non-zero");
}

#[test]
fn rejects_infinity_volume() {
    let result = LinearParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");

    let result = LinearParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::NEG_INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");
}

#[test]
fn zero_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.1);
    let zero_time = AverageDaysTime { days: 0. };

    let result = LinearParameters::from_incremental_duration(initial_rate, decline_rate, zero_time);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn rejects_infinity_decline_rate() {
    let result = LinearParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"decline rate is infinity, but expected a finite number");
}

#[test]
fn incline_from_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.001);
    let volume = 1005.;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"10");
}

#[test]
fn incline_from_small_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.01);
    let volume = 100.;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"0.9950493836207812");
}

#[test]
fn rejects_non_finite_final_rate() {
    let result = LinearParameters::<AverageDaysTime>::from_final_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        ProductionRate::new(f64::INFINITY),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"final rate is infinity, but expected a finite number");

    let result = LinearParameters::<AverageDaysTime>::from_final_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        ProductionRate::new(f64::NAN),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"final rate is not-a-number, but expected a finite number");
}

#[test]
fn no_positive_root() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(1.);
    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, 60.);

    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn linear_from_final_rate_roundtrip() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let target_final_rate = ProductionRate::<AverageDaysTime>::new(50.);

    let params =
        LinearParameters::from_final_rate(initial_rate, decline_rate, target_final_rate).unwrap();

    insta::assert_snapshot!(params.final_rate().value(), @"50");
}

#[test]
fn precision_loss_in_duration_calculation() {
    // Because of the difference in order of magnitude between initial rate and volume, the
    // calculated duration can become extremely small. The precision loss can lead to durations so
    // small they might end up as -0.0.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(1e10);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.001);
    let tiny_volume = 1e-5;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, tiny_volume);
    insta::assert_snapshot!(result.unwrap_err(), @"duration is negative, but expected a positive number");
}

#[test]
fn discriminant_near_zero() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let volume = 4999.9999;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"99.98585786436196");
}

#[test]
fn rejects_approximately_zero_initial_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(f64::MIN_POSITIVE);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let duration = AverageDaysTime { days: 1. };

    let result = LinearParameters::from_incremental_duration(initial_rate, decline_rate, duration);

    insta::assert_snapshot!(result.unwrap_err(), @"initial rate is negative or zero, but expected a positive number");

    let subnormal = f64::MIN_POSITIVE / 2.0;
    assert!(subnormal > 0., "Sanity check: subnormal is positive");
    assert!(
        subnormal < f64::MIN_POSITIVE,
        "Sanity check: subnormal is subnormal"
    );

    let initial_rate = ProductionRate::<AverageDaysTime>::new(subnormal);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let duration = AverageDaysTime { days: 1. };

    let result = LinearParameters::from_incremental_duration(initial_rate, decline_rate, duration);

    insta::assert_snapshot!(result.unwrap_err(), @"initial rate is negative or zero, but expected a positive number");
}

#[test]
fn avoids_volume_overflow() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let volume = f64::MAX;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn cannot_reach_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.5);
    let over_max_volume = 100.1;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, decline_rate, over_max_volume);
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn handles_calculated_not_a_number_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(1e308);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(1e-10);
    let volume = 1e300;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);
    insta::assert_snapshot!(result.unwrap_err(), @"duration is not-a-number, but expected a finite number");
}

#[test]
fn incline_from_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.1);
    let duration = AverageDaysTime { days: 5.0 };

    let result = LinearParameters::from_incremental_duration(initial_rate, decline_rate, duration);
    insta::assert_snapshot!(result.unwrap().final_rate().value(), @"150");
}

#[test]
fn incline_large_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.01);
    let volume = 1e6;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"1317.7446878757826");

    let computed_volume = params.incremental_volume();
    insta::assert_snapshot!(computed_volume, @"1000000.0000000001");
}

#[test]
fn incline_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.05);
    let final_rate = ProductionRate::<AverageDaysTime>::new(200.);

    let result = LinearParameters::from_final_rate(initial_rate, decline_rate, final_rate);
    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"20");
}

#[test]
fn incline_with_extremely_small_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(1e12);
    let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.001);
    let tiny_volume = 1.;

    let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, tiny_volume);
    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0.0000000000009765625");
    insta::assert_snapshot!(params.incremental_volume(), @"0.9765625000000006");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn from_incremental_duration(
        rate in prop::num::f64::ANY,
        decline in prop::num::f64::ANY,
        duration in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let duration = AverageDaysTime { days: duration };
        let result = LinearParameters::from_incremental_duration(initial_rate, decline_rate, duration);

        if let Ok(params) = result {
            let computed_volume = params.incremental_volume();
            prop_assert!(computed_volume >= 0. || computed_volume.is_nan() || computed_volume.is_infinite(),
                "Computed volume should be non-negative, got {}", computed_volume);
        }
    }

    #[test]
    fn from_incremental_volume(
        rate in prop::num::f64::ANY,
        decline in prop::num::f64::ANY,
        volume in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let result = LinearParameters::from_incremental_volume(initial_rate, decline_rate, volume);

        if let Ok(params) = result {
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_rate(
        rate in prop::num::f64::ANY,
        decline in prop::num::f64::ANY,
        final_rate in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let final_rate = ProductionRate::<AverageDaysTime>::new(final_rate);
        let result = LinearParameters::from_final_rate(initial_rate, decline_rate, final_rate);

        if let Ok(params) = result {
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }
}

mod common;
use common::assert_solved;
use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, LinearParameters, NominalDeclineRate, ProductionRate,
};
use proptest::prelude::*;

#[test]
fn linear_from_incremental_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.01).into();
    let incremental_duration = AverageDaysTime { days: 4. * 365.25 };

    let calculated_duration = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
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
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_volume = 43830.;

    let calculated_duration = LinearParameters::from_incremental_volume(
        initial_rate,
        initial_decline_rate,
        incremental_volume,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"1461");

    // Try with a positive initial decline rate to ensure we can reach the same point in time. This
    // ensures we handle both positive and negative decline rates.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(10.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-1.).into();
    let incremental_volume = 43830.;

    let calculated_duration = LinearParameters::from_incremental_volume(
        initial_rate,
        initial_decline_rate,
        incremental_volume,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"1461");
}

#[test]
fn linear_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let calculated_duration =
        LinearParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate)
            .unwrap()
            .incremental_duration()
            .days;

    insta::assert_snapshot!(calculated_duration, @"1461");
}

#[test]
fn linear_incremental_volume_at_time() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_duration = AverageDaysTime { days: 1461. };

    let parameters = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
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
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_duration = AverageDaysTime { days: 1461. };

    let parameters = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    insta::assert_snapshot!(parameters.final_rate().value(), @"10.000000000000004");
}

#[test]
fn prevent_negative_rates() {
    // Use a long duration that would cause the rate to become negative at some point.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.2).into();
    let incremental_duration = AverageDaysTime { days: 10_000. };

    let parameters = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    );

    insta::assert_snapshot!(parameters.unwrap_err(), @"final rate is negative or zero, but expected a positive number");
}

#[test]
fn zero_initial_decline_rate_is_a_flat_segment() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.);
    let volume = 1000.;

    let params =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume)
            .unwrap();

    insta::assert_snapshot!(params.incremental_duration().days, @"10");
    insta::assert_snapshot!(params.final_rate().value(), @"100");
    insta::assert_snapshot!(params.incremental_volume(), @"1000");

    let params = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        AverageDaysTime { days: 10. },
    )
    .unwrap();

    insta::assert_snapshot!(params.rate_at_time(AverageDaysTime { days: 5. }).value(), @"100");
    insta::assert_snapshot!(params.incremental_volume(), @"1000");
}

#[test]
fn zero_duration_from_zero_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.1);

    let result = LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, 0.);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn zero_duration_from_extremely_small_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.1);
    let tiny_volume = 1e-300;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, tiny_volume);
    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"0");
}

#[test]
fn large_rate_and_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(150_000.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let volume = 10_000_000.;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"70.02271045856543");
}

#[test]
fn rejects_different_rates_when_initial_decline_rate_is_zero() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.);
    let final_rate = ProductionRate::<AverageDaysTime>::new(50.);

    let result = LinearParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate);

    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
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
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.1);
    let zero_time = AverageDaysTime { days: 0. };

    let result =
        LinearParameters::from_incremental_duration(initial_rate, initial_decline_rate, zero_time);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn rejects_infinity_initial_decline_rate() {
    let result = LinearParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is infinity, but expected a finite number");
}

#[test]
fn incline_from_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.001);
    let volume = 1005.;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);

    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"10");
}

#[test]
fn incline_from_small_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.01);
    let volume = 100.;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);

    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"0.9950493836207795");
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
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(1.);
    let result = LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, 60.);

    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn linear_from_final_rate_roundtrip() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let target_final_rate = ProductionRate::<AverageDaysTime>::new(50.);

    let params =
        LinearParameters::from_final_rate(initial_rate, initial_decline_rate, target_final_rate)
            .unwrap();

    insta::assert_snapshot!(params.final_rate().value(), @"50");
}

#[test]
fn precision_loss_in_duration_calculation() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(1e10);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.001);
    let tiny_volume = 1e-5;

    let params =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, tiny_volume)
            .unwrap();

    insta::assert_snapshot!(params.incremental_duration().days, @"0.000000000000001");
    assert_solved(params.incremental_volume(), tiny_volume);
}

#[test]
fn discriminant_near_zero() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let volume = 4999.9999;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"99.98585786436196");
}

#[test]
fn rejects_approximately_zero_initial_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(f64::MIN_POSITIVE);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let duration = AverageDaysTime { days: 1. };

    let result =
        LinearParameters::from_incremental_duration(initial_rate, initial_decline_rate, duration);

    insta::assert_snapshot!(result.unwrap_err(), @"initial rate is negative or zero, but expected a positive number");

    let subnormal = f64::MIN_POSITIVE / 2.0;
    assert!(subnormal > 0., "Sanity check: subnormal is positive");
    assert!(
        subnormal < f64::MIN_POSITIVE,
        "Sanity check: subnormal is subnormal"
    );

    let initial_rate = ProductionRate::<AverageDaysTime>::new(subnormal);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let duration = AverageDaysTime { days: 1. };

    let result =
        LinearParameters::from_incremental_duration(initial_rate, initial_decline_rate, duration);

    insta::assert_snapshot!(result.unwrap_err(), @"initial rate is negative or zero, but expected a positive number");
}

#[test]
fn avoids_volume_overflow() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.01);
    let volume = f64::MAX;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn cannot_reach_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(0.5);
    let over_max_volume = 100.1;

    let result = LinearParameters::from_incremental_volume(
        initial_rate,
        initial_decline_rate,
        over_max_volume,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn handles_calculated_not_a_number_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(1e308);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(1e-10);
    let volume = 1e300;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn incline_from_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.1);
    let duration = AverageDaysTime { days: 5.0 };

    let result =
        LinearParameters::from_incremental_duration(initial_rate, initial_decline_rate, duration);
    insta::assert_snapshot!(result.unwrap().final_rate().value(), @"150");
}

#[test]
fn incline_large_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.01);
    let volume = 1e6;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);

    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"1317.7446878757826");

    let computed_volume = params.incremental_volume();
    insta::assert_snapshot!(computed_volume, @"1000000.0000000001");
}

#[test]
fn incline_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.05);
    let final_rate = ProductionRate::<AverageDaysTime>::new(200.);

    let result = LinearParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate);
    insta::assert_snapshot!(result.unwrap().incremental_duration().days, @"20");
}

#[test]
fn incline_with_extremely_small_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(1e12);
    let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(-0.001);
    let tiny_volume = 1.;

    let result =
        LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, tiny_volume);
    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0.0000000000009999999999999996");
    assert_solved(params.incremental_volume(), tiny_volume);
}

#[test]
fn linear_from_incremental_duration_and_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 365. };

    let reference = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = LinearParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        reference.final_rate(),
    )
    .unwrap();

    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());
}

#[test]
fn linear_from_final_rate_and_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 365. };

    let reference = LinearParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = LinearParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        reference.final_rate(),
        reference.incremental_volume(),
    )
    .unwrap();

    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(
        solved.incremental_duration().days,
        reference.incremental_duration().days,
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());
    assert_solved(solved.incremental_volume(), reference.incremental_volume());
}

#[test]
fn linear_solving_an_incline_keeps_the_sign() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    let incremental_duration = AverageDaysTime { days: 5. * 365. };

    let reference = LinearParameters::from_incremental_duration(
        initial_rate,
        incline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = LinearParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        reference.final_rate(),
    )
    .unwrap();

    assert!(solved.initial_decline_rate().value() < 0.);
    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());

    let solved = LinearParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        reference.final_rate(),
        reference.incremental_volume(),
    )
    .unwrap();

    assert!(solved.initial_decline_rate().value() < 0.);
    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(solved.incremental_volume(), reference.incremental_volume());
}

#[test]
fn linear_solving_rejects_cutoffs_that_imply_no_decline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    // The decline rate rounds to nothing, leaving a flat segment longer than we support.
    let result = LinearParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        ProductionRate::<AverageDaysTime>::new(10.),
        1e300,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");

    // Linear divides the other way around from the Arps families, so it's an incline that
    // overflows here.
    let result = LinearParameters::from_incremental_duration_and_final_rate(
        ProductionRate::<AverageDaysTime>::new(1e-11),
        incremental_duration,
        ProductionRate::<AverageDaysTime>::new(1e300),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn linear_solving_equal_cutoffs_gives_a_flat_segment() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. };

    let solved = LinearParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        initial_rate,
    )
    .unwrap();

    insta::assert_snapshot!(solved.initial_decline_rate().value(), @"0");
    insta::assert_snapshot!(solved.final_rate().value(), @"50");
    insta::assert_snapshot!(solved.incremental_volume(), @"500");

    // With no decline, the volume rather than the final rate is what sets the length.
    let solved =
        LinearParameters::from_final_rate_and_incremental_volume(initial_rate, initial_rate, 500.)
            .unwrap();

    insta::assert_snapshot!(solved.initial_decline_rate().value(), @"0");
    insta::assert_snapshot!(solved.incremental_duration().days, @"10");
    insta::assert_snapshot!(solved.final_rate().value(), @"50");
}

#[test]
fn linear_solving_near_flat_cutoffs_gives_a_flat_segment() {
    // The decline rate rounds to zero while the rates stay outside the rate tolerance.
    let solved = LinearParameters::from_final_rate_and_incremental_volume(
        ProductionRate::<AverageDaysTime>::new(100.),
        ProductionRate::<AverageDaysTime>::new(99.9999999999),
        1000.,
    )
    .unwrap();

    insta::assert_snapshot!(solved.incremental_duration().days, @"10");
    insta::assert_snapshot!(solved.initial_decline_rate().value(), @"0");
}

#[test]
fn linear_solving_keeps_the_volume_when_the_rates_nearly_match() {
    // The rates sit inside the rate tolerance, but the decline rate is well outside its own.
    let solved = LinearParameters::from_final_rate_and_incremental_volume(
        ProductionRate::<AverageDaysTime>::new(100.),
        ProductionRate::<AverageDaysTime>::new(100. - 1e-12),
        0.001,
    )
    .unwrap();

    // Cutoffs this close keep only a couple of digits through `1 - q_f / q_i`.
    assert!(solved.incremental_duration().days > 0.);
    let relative_error = (solved.incremental_volume() - 0.001).abs() / 0.001;
    assert!(
        relative_error < 0.01,
        "relative error was {relative_error:e}"
    );
}

#[test]
fn linear_equal_rates_take_no_time_whichever_way_the_segment_runs() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);

    for decline in [0.5, -0.5, 0., -0.] {
        let params = LinearParameters::from_final_rate(
            initial_rate,
            NominalDeclineRate::<AverageDaysTime>::new(decline),
            initial_rate,
        )
        .unwrap();

        assert_eq!(
            params.incremental_duration().days,
            0.,
            "decline rate {decline} should give a zero duration"
        );
    }
}

#[test]
fn linear_solving_rejects_degenerate_cutoffs() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let result = LinearParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        AverageDaysTime { days: 0. },
        final_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration is approximately zero, but expected it to be non-zero");

    let result =
        LinearParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, 0.);
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is approximately zero, but expected it to be non-zero");
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
        let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let duration = AverageDaysTime { days: duration };
        let result = LinearParameters::from_incremental_duration(initial_rate, initial_decline_rate, duration);

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
        let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let result = LinearParameters::from_incremental_volume(initial_rate, initial_decline_rate, volume);

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
        let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let final_rate = ProductionRate::<AverageDaysTime>::new(final_rate);
        let result = LinearParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate);

        if let Ok(params) = result {
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    /// Nothing the solver returns may carry an infinity or a NaN out with it.
    #[test]
    fn from_incremental_duration_and_final_rate(
        rate in prop::num::f64::ANY,
        duration in prop::num::f64::ANY,
        final_rate in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let final_rate = ProductionRate::<AverageDaysTime>::new(final_rate);
        let duration = AverageDaysTime { days: duration };
        let result = LinearParameters::from_incremental_duration_and_final_rate(initial_rate, duration, final_rate);

        if let Ok(params) = result {
            let decline = params.initial_decline_rate().value();
            prop_assert!(decline.is_finite(), "Decline rate should be finite, got {}", decline);
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_rate_and_incremental_volume(
        rate in prop::num::f64::ANY,
        final_rate in prop::num::f64::ANY,
        volume in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let final_rate = ProductionRate::<AverageDaysTime>::new(final_rate);
        let result = LinearParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, volume);

        if let Ok(params) = result {
            let decline = params.initial_decline_rate().value();
            prop_assert!(decline.is_finite(), "Decline rate should be finite, got {}", decline);
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn solving_round_trips_through_both_cutoffs(
        rate in 1.0f64..1e5,
        decline in prop_oneof![-1.0f64..-0.01, 0.01f64..1.0],
        duration in 0.1f64..50.,
        zero_rate_fraction in 0.01f64..0.95,
    ) {
        // Stay short of the zero rate at `1 / d`.
        let duration = if decline > 0. {
            (zero_rate_fraction / decline).min(duration)
        } else {
            duration
        };

        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let incremental_duration = AverageYearsTime { years: duration };

        let reference = LinearParameters::from_incremental_duration(
            initial_rate,
            initial_decline_rate,
            incremental_duration,
        )
        .unwrap();

        let solved = LinearParameters::from_incremental_duration_and_final_rate(
            initial_rate,
            incremental_duration,
            reference.final_rate(),
        )
        .unwrap();
        assert_solved(solved.initial_decline_rate().value(), decline);
        assert_solved(solved.final_rate().value(), reference.final_rate().value());

        let solved = LinearParameters::from_final_rate_and_incremental_volume(
            initial_rate,
            reference.final_rate(),
            reference.incremental_volume(),
        )
        .unwrap();
        assert_solved(solved.initial_decline_rate().value(), decline);
        assert_solved(solved.incremental_duration().years, duration);
        assert_solved(solved.incremental_volume(), reference.incremental_volume());
    }
}

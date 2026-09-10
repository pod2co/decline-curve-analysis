mod common;
use common::{assert_sig_snapshot, assert_solved};
use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, ExponentialParameters, NominalDeclineRate, ProductionRate,
};
use proptest::prelude::*;

#[test]
fn exponential_from_incremental_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let calculated_duration = ExponentialParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap()
    .incremental_duration()
    .days;

    assert_sig_snapshot!(calculated_duration, @"3650");
}

#[test]
fn exponential_from_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_volume = 29220.;

    let calculated_duration = ExponentialParameters::from_incremental_volume(
        initial_rate,
        initial_decline_rate,
        incremental_volume,
    )
    .unwrap()
    .incremental_duration()
    .days;

    assert_sig_snapshot!(calculated_duration, @"1175.69439503");
}

#[test]
fn exponential_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let calculated_duration =
        ExponentialParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate)
            .unwrap()
            .incremental_duration()
            .days;

    assert_sig_snapshot!(calculated_duration, @"1175.69439503");
}

#[test]
fn exponential_incremental_volume_at_time() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 1175.6943 };

    let parameters = ExponentialParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    // Calculate past the end to check the total.
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 1180. }), @"29219.9990497");

    // Check a point somewhere in the middle.
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 1180. }), @"20238.5906888");
}

#[test]
fn exponential_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 1175.6943 };

    let parameters = ExponentialParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    assert_sig_snapshot!(parameters.final_rate().value(), @"10.0000013009");
}

#[test]
fn exponential_incline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = ExponentialParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    assert_sig_snapshot!(parameters.incremental_duration().days, @"3650");
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 4000. }), @"5365745.69992");
    assert_sig_snapshot!(parameters.final_rate().value(), @"7395.30554404");
}

#[test]
fn exponential_decline_rate_wrong_sign() {
    // Incline with a negative decline rate.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(60.);

    let parameters =
        ExponentialParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate);

    insta::assert_snapshot!(parameters.unwrap_err(), @"decline rate has wrong sign");
}

#[test]
fn volume_range() {
    // For exponential declines, the maximum possible volume is the point immediately before
    // volume = q_i / d.
    //
    // max volume as time approaches infinity = 3000 / 0.1 = 30000
    let initial_rate = ProductionRate::<AverageYearsTime>::new(3000.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let incremental_volume_greater_than_max = 1_000_000_000.;

    let result = ExponentialParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        incremental_volume_greater_than_max,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let incremental_volume_equal_to_max = 3000. / 0.1;
    let result = ExponentialParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        incremental_volume_equal_to_max,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let incremental_volume_slightly_less_than_max = (3000. / 0.1) * 0.9999;
    let params = ExponentialParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        incremental_volume_slightly_less_than_max,
    )
    .unwrap();
    assert_sig_snapshot!(params.incremental_duration().years, @"92.1034037198");
}

#[test]
fn extremely_small_volume() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let tiny_volume = 1e-10;

    let params =
        ExponentialParameters::from_incremental_volume(initial_rate, decline_rate, tiny_volume)
            .unwrap();
    assert_sig_snapshot!(params.incremental_duration().years, @"0.000000000001");
}

#[test]
fn incline_large_volume() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let large_volume = 1_000_000.;

    let params =
        ExponentialParameters::from_incremental_volume(initial_rate, decline_rate, large_volume)
            .unwrap();
    assert_sig_snapshot!(params.incremental_duration().years, @"69.0875477932");
}

#[test]
fn finite_initial_rate() {
    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(f64::NAN),
        NominalDeclineRate::new(0.1),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial rate is not-a-number, but expected a finite number");

    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(f64::INFINITY),
        NominalDeclineRate::new(0.1),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial rate is infinity, but expected a finite number");
}

#[test]
fn finite_decline_rate() {
    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::NAN),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"decline rate is not-a-number, but expected a finite number");

    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"decline rate is infinity, but expected a finite number");
}

#[test]
fn finite_volume() {
    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::NAN,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is not-a-number, but expected a finite number");

    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");

    let result = ExponentialParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::NEG_INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");
}

#[test]
fn decline_rate_wrong_sign() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let final_rate = ProductionRate::<AverageYearsTime>::new(150.);
    let result = ExponentialParameters::from_final_rate(initial_rate, decline_rate, final_rate);
    insta::assert_snapshot!(result.unwrap_err(), @"decline rate has wrong sign");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let final_rate = ProductionRate::<AverageYearsTime>::new(50.);
    let result = ExponentialParameters::from_final_rate(initial_rate, decline_rate, final_rate);
    insta::assert_snapshot!(result.unwrap_err(), @"decline rate has wrong sign");
}

#[test]
fn zero_volume() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let params =
        ExponentialParameters::from_incremental_volume(initial_rate, decline_rate, 0.).unwrap();
    assert_sig_snapshot!(params.incremental_duration().years, @"0");
}

#[test]
fn zero_duration() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let zero_time = AverageYearsTime { years: 0. };
    let params =
        ExponentialParameters::from_incremental_duration(initial_rate, decline_rate, zero_time)
            .unwrap();
    let volume = params.incremental_volume();
    assert_sig_snapshot!(volume, @"0");
}

#[test]
fn final_rate_roundtrip() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5);
    let target_final_rate = ProductionRate::<AverageYearsTime>::new(50.);
    let params =
        ExponentialParameters::from_final_rate(initial_rate, decline_rate, target_final_rate)
            .unwrap();
    let actual_final_rate = params.final_rate().value();
    assert_sig_snapshot!(actual_final_rate, @"50");
}

#[test]
fn duration_range() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let extreme_duration = AverageYearsTime { years: 10_000. };
    let result = ExponentialParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        extreme_duration,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let reasonable_duration = AverageYearsTime { years: 100. };
    let result = ExponentialParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        reasonable_duration,
    );
    assert_sig_snapshot!(result.unwrap().incremental_duration().years, @"100");
}

#[test]
fn exponential_from_incremental_duration_and_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let reference = ExponentialParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = ExponentialParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        reference.final_rate(),
    )
    .unwrap();

    assert_solved(
        solved.decline_rate().value(),
        reference.decline_rate().value(),
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());
}

#[test]
fn exponential_from_final_rate_and_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let reference = ExponentialParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = ExponentialParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        reference.final_rate(),
        reference.incremental_volume(),
    )
    .unwrap();

    assert_solved(
        solved.decline_rate().value(),
        reference.decline_rate().value(),
    );
    assert_solved(
        solved.incremental_duration().days,
        reference.incremental_duration().days,
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());
    assert_solved(solved.incremental_volume(), reference.incremental_volume());
}

#[test]
fn exponential_solving_an_incline_keeps_the_sign() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    let incremental_duration = AverageDaysTime { days: 5. * 365. };

    let reference = ExponentialParameters::from_incremental_duration(
        initial_rate,
        incline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = ExponentialParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        reference.final_rate(),
    )
    .unwrap();

    assert!(solved.decline_rate().value() < 0.);
    assert_solved(
        solved.decline_rate().value(),
        reference.decline_rate().value(),
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());

    let solved = ExponentialParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        reference.final_rate(),
        reference.incremental_volume(),
    )
    .unwrap();

    assert!(solved.decline_rate().value() < 0.);
    assert_solved(
        solved.decline_rate().value(),
        reference.decline_rate().value(),
    );
    assert_solved(solved.incremental_volume(), reference.incremental_volume());
}

#[test]
fn exponential_solving_rejects_cutoffs_that_imply_no_decline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let result = ExponentialParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        initial_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let result = ExponentialParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        ProductionRate::<AverageDaysTime>::new(10.),
        1e300,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // The rate ratio overflows to infinity.
    let result = ExponentialParameters::from_incremental_duration_and_final_rate(
        ProductionRate::<AverageDaysTime>::new(1e300),
        incremental_duration,
        ProductionRate::<AverageDaysTime>::new(1e-11),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn exponential_solving_rejects_degenerate_cutoffs() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let result = ExponentialParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        AverageDaysTime { days: 0. },
        final_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration is approximately zero, but expected it to be non-zero");

    let result =
        ExponentialParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, 0.);
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
        let decline_rate = NominalDeclineRate::<AverageDaysTime>::new(decline);
        let duration = AverageDaysTime { days: duration };
        let result = ExponentialParameters::from_incremental_duration(initial_rate, decline_rate, duration);

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
        let result = ExponentialParameters::from_incremental_volume(initial_rate, decline_rate, volume);

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
        let result = ExponentialParameters::from_final_rate(initial_rate, decline_rate, final_rate);

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
        let result = ExponentialParameters::from_incremental_duration_and_final_rate(initial_rate, duration, final_rate);

        if let Ok(params) = result {
            let decline = params.decline_rate().value();
            prop_assert!(decline.is_finite(), "Decline rate should be finite, got {}", decline);
            prop_assert!(decline != 0., "Decline rate should be non-zero");
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
        let result = ExponentialParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, volume);

        if let Ok(params) = result {
            let decline = params.decline_rate().value();
            prop_assert!(decline.is_finite(), "Decline rate should be finite, got {}", decline);
            prop_assert!(decline != 0., "Decline rate should be non-zero");
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
        decline_span in 0.01f64..10.,
    ) {
        // Cap how far the segment declines, so the final rate stays above the validation floor.
        let duration = (decline_span / decline.abs()).min(duration);

        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let incremental_duration = AverageYearsTime { years: duration };

        let reference = ExponentialParameters::from_incremental_duration(
            initial_rate,
            decline_rate,
            incremental_duration,
        )
        .unwrap();

        let solved = ExponentialParameters::from_incremental_duration_and_final_rate(
            initial_rate,
            incremental_duration,
            reference.final_rate(),
        )
        .unwrap();
        assert_solved(solved.decline_rate().value(), decline);
        assert_solved(solved.final_rate().value(), reference.final_rate().value());

        let solved = ExponentialParameters::from_final_rate_and_incremental_volume(
            initial_rate,
            reference.final_rate(),
            reference.incremental_volume(),
        )
        .unwrap();
        assert_solved(solved.decline_rate().value(), decline);
        assert_solved(solved.incremental_duration().years, duration);
        assert_solved(solved.incremental_volume(), reference.incremental_volume());
    }
}

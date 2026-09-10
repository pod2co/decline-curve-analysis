mod common;
use common::{assert_sig_snapshot, assert_solved};
use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, HarmonicParameters, NominalDeclineRate, ProductionRate,
};
use proptest::prelude::*;

#[test]
fn harmonic_from_incremental_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let calculated_duration = HarmonicParameters::from_incremental_duration(
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
fn harmonic_from_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_volume = 58784.7197516555;

    let calculated_duration = HarmonicParameters::from_incremental_volume(
        initial_rate,
        initial_decline_rate,
        incremental_volume,
    )
    .unwrap()
    .incremental_duration()
    .days;

    assert_sig_snapshot!(calculated_duration, @"2922");
}

#[test]
fn harmonic_from_final_decline_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1).into();

    let calculated_duration = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        initial_decline_rate,
        final_decline_rate,
    )
    .unwrap()
    .incremental_duration()
    .days;

    assert_sig_snapshot!(calculated_duration, @"2922");
}

#[test]
fn harmonic_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let calculated_duration =
        HarmonicParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate)
            .unwrap()
            .incremental_duration()
            .days;

    assert_sig_snapshot!(calculated_duration, @"2922");
}

#[test]
fn harmonic_incremental_volume_at_time() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 2922. };

    let parameters = HarmonicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    // Calculate past the end to check the total.
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 2950. }), @"58784.7197517");

    // Check a point somewhere in the middle.
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 2950. }), @"40359.4050321");
}

#[test]
fn harmonic_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 2922. };

    let parameters = HarmonicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    assert_sig_snapshot!(parameters.final_rate().value(), @"10");
}

#[test]
fn harmonic_incline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.005).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = HarmonicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    assert_sig_snapshot!(parameters.incremental_duration().days, @"3650");
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 4000. }), @"187217.181173");
    assert_sig_snapshot!(parameters.final_rate().value(), @"52.6296829971");
}

#[test]
fn harmonic_decline_rate_wrong_sign() {
    // Incline with a negative decline rate.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(60.);

    let parameters =
        HarmonicParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate);

    insta::assert_snapshot!(parameters.unwrap_err(), @"decline rate has wrong sign");
}

#[test]
fn harmonic_final_decline_rate_impossible() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);

    // A harmonic decline rate only ever falls, so a larger final one is behind us.
    let parameters = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.6).into(),
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // Positive initial decline rate with negative final decline rate.
    let parameters = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // Negative initial decline rate with positive final decline rate.
    let parameters = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn incline_from_final_decline_rate() {
    // The decline rate decreases, so this should succeed.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1).into();
    let final_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    let params = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        initial_decline_rate,
        final_decline_rate,
    )
    .unwrap();
    assert_sig_snapshot!(params.incremental_duration().days, @"1826.25");

    // The decline rate tries to increase, so this should fail.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    let final_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1).into();
    let result = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        initial_decline_rate,
        final_decline_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn incline_with_large_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let large_volume = 50_000.;
    let params =
        HarmonicParameters::from_incremental_volume(initial_rate, decline_rate, large_volume)
            .unwrap();
    assert_sig_snapshot!(params.incremental_duration().days, @"717.866904557");
}

#[test]
fn incline_with_small_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.5).into();
    let volume = 1000.;
    let params =
        HarmonicParameters::from_incremental_volume(initial_rate, decline_rate, volume).unwrap();
    assert_sig_snapshot!(params.incremental_duration().days, @"9.93186499049");
}

#[test]
fn zero_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let zero_time = AverageDaysTime { days: 0. };
    let params =
        HarmonicParameters::from_incremental_duration(initial_rate, decline_rate, zero_time)
            .unwrap();
    assert_sig_snapshot!(params.incremental_duration().days, @"0");
    assert_sig_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn finite_initial_decline_rate() {
    let result = HarmonicParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is infinity, but expected a finite number");

    let result = HarmonicParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::NAN),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is not-a-number, but expected a finite number");

    let result = HarmonicParameters::<AverageDaysTime>::from_final_decline_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        NominalDeclineRate::new(0.1),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is infinity, but expected a finite number");

    let result = HarmonicParameters::<AverageDaysTime>::from_final_decline_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::NAN),
        NominalDeclineRate::new(0.1),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is not-a-number, but expected a finite number");
}

#[test]
fn finite_volume() {
    let result = HarmonicParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");
}

#[test]
fn finite_final_decline_rate() {
    let result = HarmonicParameters::<AverageDaysTime>::from_final_decline_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.5),
        NominalDeclineRate::new(f64::INFINITY),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"final decline rate is infinity, but expected a finite number");
}

#[test]
fn final_rate_roundtrip() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let target_final_rate = ProductionRate::<AverageDaysTime>::new(50.);

    let params =
        HarmonicParameters::from_final_rate(initial_rate, decline_rate, target_final_rate).unwrap();

    let actual_final_rate = params.final_rate().value();
    assert_sig_snapshot!(actual_final_rate, @"50");
}

#[test]
fn duration_range() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let extreme_duration = AverageYearsTime { years: 10000. };
    let result =
        HarmonicParameters::from_incremental_duration(initial_rate, decline_rate, extreme_duration);
    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let reasonable_duration = AverageYearsTime { years: 9.0 };
    let params = HarmonicParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        reasonable_duration,
    )
    .unwrap();
    assert_sig_snapshot!(params.incremental_duration().years, @"9");

    // For harmonic incline with D = -0.1, the singularity is at t_max = 1/|D| = 10 years.
    // Durations at or beyond this point should bne rejected.
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let singularity_duration = AverageYearsTime { years: 10. }; // Exactly at t_max = 1/|D|
    let result = HarmonicParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        singularity_duration,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let beyond_singularity = AverageYearsTime { years: 11. };
    let result = HarmonicParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        beyond_singularity,
    );

    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let just_under_singularity = AverageYearsTime { years: 9.9 }; // Just under t_max = 10
    let params = HarmonicParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        just_under_singularity,
    )
    .unwrap();
    assert_sig_snapshot!(params.final_rate().value(), @"10000");
}

#[test]
fn harmonic_from_incremental_duration_and_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let reference = HarmonicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = HarmonicParameters::from_incremental_duration_and_final_rate(
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
fn harmonic_from_final_rate_and_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let reference = HarmonicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = HarmonicParameters::from_final_rate_and_incremental_volume(
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
fn harmonic_solving_an_incline_keeps_the_sign() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    // A harmonic incline has a singularity at `1 / |d|`.
    let incremental_duration = AverageDaysTime { days: 2. * 365. };

    let reference = HarmonicParameters::from_incremental_duration(
        initial_rate,
        incline_rate,
        incremental_duration,
    )
    .unwrap();

    let solved = HarmonicParameters::from_incremental_duration_and_final_rate(
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
}

#[test]
fn harmonic_solving_rejects_cutoffs_that_imply_no_decline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let result = HarmonicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        initial_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let result = HarmonicParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        ProductionRate::<AverageDaysTime>::new(10.),
        1e300,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // The rate ratio overflows to infinity.
    let result = HarmonicParameters::from_incremental_duration_and_final_rate(
        ProductionRate::<AverageDaysTime>::new(1e300),
        incremental_duration,
        ProductionRate::<AverageDaysTime>::new(1e-11),
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn harmonic_solving_rejects_degenerate_cutoffs() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let result = HarmonicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        AverageDaysTime { days: 0. },
        final_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration is approximately zero, but expected it to be non-zero");

    let result =
        HarmonicParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, 0.);
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
        let incremental_duration = AverageDaysTime { days: duration };
        let result = HarmonicParameters::from_incremental_duration(initial_rate, decline_rate, incremental_duration);

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
        let result = HarmonicParameters::from_incremental_volume(initial_rate, decline_rate, volume);

        if let Ok(params) = result {
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_rate(
        rate in prop::num::f64::ANY,
        initial_decline in prop::num::f64::ANY,
        final_rate in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(initial_decline);
        let final_rate = ProductionRate::<AverageDaysTime>::new(final_rate);
        let result = HarmonicParameters::from_final_rate(
            initial_rate,
            initial_decline_rate,
            final_rate,
        );

        if let Ok(params) = result {
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_decline_rate(
        rate in prop::num::f64::ANY,
        initial_decline in prop::num::f64::ANY,
        final_decline in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageDaysTime>::new(rate);
        let initial_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(initial_decline);
        let final_decline_rate = NominalDeclineRate::<AverageDaysTime>::new(final_decline);
        let result = HarmonicParameters::from_final_decline_rate(
            initial_rate,
            initial_decline_rate,
            final_decline_rate,
        );

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
        let incremental_duration = AverageDaysTime { days: duration };
        let result = HarmonicParameters::from_incremental_duration_and_final_rate(initial_rate, incremental_duration, final_rate);

        if let Ok(params) = result {
            let decline = params.initial_decline_rate().value();
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
        let result = HarmonicParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, volume);

        if let Ok(params) = result {
            let decline = params.initial_decline_rate().value();
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
        singularity_fraction in 0.01f64..0.95,
        decline_span in 0.01f64..10.,
    ) {
        // Cap how far the segment declines, so the final rate stays above the validation floor.
        let duration = (decline_span / decline.abs()).min(duration);

        // Stay short of the singularity at `t_max = 1 / |d|`.
        let duration = if decline < 0. {
            (-singularity_fraction / decline).min(duration)
        } else {
            duration
        };

        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let incremental_duration = AverageYearsTime { years: duration };

        let reference = HarmonicParameters::from_incremental_duration(
            initial_rate,
            decline_rate,
            incremental_duration,
        )
        .unwrap();

        let solved = HarmonicParameters::from_incremental_duration_and_final_rate(
            initial_rate,
            incremental_duration,
            reference.final_rate(),
        )
        .unwrap();
        assert_solved(solved.initial_decline_rate().value(), decline);
        assert_solved(solved.final_rate().value(), reference.final_rate().value());

        let solved = HarmonicParameters::from_final_rate_and_incremental_volume(
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

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

    insta::assert_snapshot!(calculated_duration, @"3650");
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

    insta::assert_snapshot!(calculated_duration, @"2921.9999999999986");
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

    insta::assert_snapshot!(calculated_duration, @"2922");
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

    insta::assert_snapshot!(calculated_duration, @"2922");
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
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 2950. }), @"58784.71975165552");

    // Check a point somewhere in the middle.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 2950. }), @"40359.40503213862");
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

    insta::assert_snapshot!(parameters.final_rate().value(), @"10");
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

    insta::assert_snapshot!(parameters.incremental_duration().days, @"3650");
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 4000. }), @"187217.18117312618");
    insta::assert_snapshot!(parameters.final_rate().value(), @"52.62968299711815");
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

    // Positive decline rate inclining.
    let parameters = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.6).into(),
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"duration is negative, but expected a positive number");

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
    insta::assert_snapshot!(params.incremental_duration().days, @"1826.25");

    // The decline rate tries to increase, so this should fail.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    let final_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1).into();
    let result = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        initial_decline_rate,
        final_decline_rate,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration is negative, but expected a positive number");
}

#[test]
fn incline_with_large_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let large_volume = 50_000.;
    let params =
        HarmonicParameters::from_incremental_volume(initial_rate, decline_rate, large_volume)
            .unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"717.8669045573092");
}

#[test]
fn incline_with_small_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.5).into();
    let volume = 1000.;
    let params =
        HarmonicParameters::from_incremental_volume(initial_rate, decline_rate, volume).unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"9.931864990485755");
}

#[test]
fn zero_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let zero_time = AverageDaysTime { days: 0. };
    let params =
        HarmonicParameters::from_incremental_duration(initial_rate, decline_rate, zero_time)
            .unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
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
    insta::assert_snapshot!(actual_final_rate, @"50");
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
    insta::assert_snapshot!(params.incremental_duration().years, @"9");

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
    insta::assert_snapshot!(params.final_rate().value(), @"10000.00000000009");
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

}

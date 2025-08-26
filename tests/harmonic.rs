use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, HarmonicParameters, NominalDeclineRate, ProductionRate,
};

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

    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::DeclineRateWrongSign)
    ));
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
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));

    // Positive initial decline rate with negative final decline rate.
    let parameters = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
    );
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));

    // Negative initial decline rate with positive final decline rate.
    let parameters = HarmonicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
    );
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));
}

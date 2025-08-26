use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, HyperbolicParameters, NominalDeclineRate, ProductionRate,
};

#[test]
fn hyperbolic_from_incremental_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };
    let exponent = 0.9;

    let calculated_duration = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"3650");
}

#[test]
fn hyperbolic_from_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_volume = 54298.0932992834;
    let exponent = 0.9;

    let calculated_duration = HyperbolicParameters::from_incremental_volume(
        initial_rate,
        initial_decline_rate,
        incremental_volume,
        exponent,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"2643.3545188968474");
}

#[test]
fn hyperbolic_from_final_decline_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.117461894308802).into();
    let exponent = 0.9;

    let calculated_duration = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        initial_decline_rate,
        final_decline_rate,
        exponent,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"2643.3545188968483");
}

#[test]
fn hyperbolic_from_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);
    let exponent = 0.9;

    let calculated_duration = HyperbolicParameters::from_final_rate(
        initial_rate,
        initial_decline_rate,
        final_rate,
        exponent,
    )
    .unwrap()
    .incremental_duration()
    .days;

    insta::assert_snapshot!(calculated_duration, @"2643.354518896851");
}

#[test]
fn hyperbolic_incremental_volume_at_time() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 2643.3552 };
    let exponent = 0.9;

    let parameters = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap();

    // Calculate past the end to check the total.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 2700. }), @"54298.10011031419");

    // Check a point somewhere in the middle.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 2700. }), @"37666.26214690978");
}

#[test]
fn hyperbolic_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 2643.3552 };
    let exponent = 0.9;

    let parameters = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap();

    insta::assert_snapshot!(parameters.final_rate().value(), @"9.999997809619451");
}

#[test]
fn hyperbolic_incline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.005).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        -0.9,
    )
    .unwrap();

    insta::assert_snapshot!(parameters.incremental_duration().days, @"3650");
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 4000. }), @"187066.8962759463");
    insta::assert_snapshot!(parameters.final_rate().value(), @"52.50444884947007");
}

#[test]
fn hyperbolic_decline_rate_wrong_sign() {
    // Incline with a negative decline rate.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(60.);

    let parameters =
        HyperbolicParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate, 0.9);

    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::DeclineRateWrongSign)
    ));
}

#[test]
fn hyperbolic_final_decline_rate_impossible() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);

    // Positive decline rate inclining with positive exponent.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.6).into(),
        0.9,
    );
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));

    // Positive decline rate declining with negative exponent.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.4).into(),
        -0.9,
    );
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));

    // Positive initial decline rate with negative final decline rate.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        0.9,
    );
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));

    // Negative initial decline rate with positive final decline rate.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        0.9,
    );
    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::CannotSolveDecline)
    ));
}

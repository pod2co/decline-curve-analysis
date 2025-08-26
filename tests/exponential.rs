use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, ExponentialParameters, NominalDeclineRate, ProductionRate,
};

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

    insta::assert_snapshot!(calculated_duration, @"3650");
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

    insta::assert_snapshot!(calculated_duration, @"1175.6943950331104");
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

    insta::assert_snapshot!(calculated_duration, @"1175.6943950331104");
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
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 1180. }), @"29219.999049668837");

    // Check a point somewhere in the middle.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 1180. }), @"20238.590688787954");
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

    insta::assert_snapshot!(parameters.final_rate().value(), @"10.000001300932462");
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

    insta::assert_snapshot!(parameters.incremental_duration().days, @"3650");
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 4000. }), @"5365745.699923456");
    insta::assert_snapshot!(parameters.final_rate().value(), @"7395.30554404306");
}

#[test]
fn exponential_decline_rate_wrong_sign() {
    // Incline with a negative decline rate.
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let final_rate = ProductionRate::<AverageDaysTime>::new(60.);

    let parameters =
        ExponentialParameters::from_final_rate(initial_rate, initial_decline_rate, final_rate);

    assert!(matches!(
        parameters,
        Err(decline_curve_analysis::DeclineCurveAnalysisError::DeclineRateWrongSign)
    ));
}

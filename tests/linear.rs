use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, LinearParameters, NominalDeclineRate, ProductionRate,
};

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

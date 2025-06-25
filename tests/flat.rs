use decline_curve_analysis::{AverageDaysTime, FlatParameters, ProductionRate};

#[test]
fn flat_from_incremental_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let calculated_duration =
        FlatParameters::from_incremental_duration(initial_rate, incremental_duration)
            .unwrap()
            .incremental_duration()
            .days;

    insta::assert_snapshot!(calculated_duration, @"3650");
}

#[test]
fn flat_from_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_volume = 50. * 365. * 10.;

    let calculated_duration =
        FlatParameters::from_incremental_volume(initial_rate, incremental_volume)
            .unwrap()
            .incremental_duration()
            .days;

    insta::assert_snapshot!(calculated_duration, @"3650");
}

#[test]
fn flat_incremental_volume_at_time() {
    let rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = FlatParameters::from_incremental_duration(rate, incremental_duration).unwrap();

    // Calculate past the end to check the total.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 3560. }), @"178000");

    // Check a point somewhere in the middle.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 3560. }), @"89000");
}

#[test]
fn flat_final_rate() {
    let rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = FlatParameters::from_incremental_duration(rate, incremental_duration).unwrap();

    insta::assert_snapshot!(parameters.final_rate().value(), @"50");
}

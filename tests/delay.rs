use decline_curve_analysis::{AverageDaysTime, DelayParameters};

#[test]
fn delay_from_incremental_duration() {
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let calculated_duration = DelayParameters::from_incremental_duration(incremental_duration)
        .unwrap()
        .incremental_duration()
        .days;

    insta::assert_snapshot!(calculated_duration, @"3650");
}

#[test]
fn delay_incremental_volume_at_time() {
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = DelayParameters::from_incremental_duration(incremental_duration).unwrap();

    // Calculate past the end to check the total.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 3560. }), @"0");

    // Check a point somewhere in the middle.
    insta::assert_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 3560. }), @"0");
}

#[test]
fn delay_final_rate() {
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let parameters = DelayParameters::from_incremental_duration(incremental_duration).unwrap();

    insta::assert_snapshot!(parameters.final_rate().value(), @"0");
}

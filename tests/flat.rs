use decline_curve_analysis::{AverageDaysTime, FlatParameters, ProductionRate};
use proptest::prelude::*;

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

#[test]
fn flat_zero_rate_from_volume_errors() {
    // Zero rate with positive volume is impossible.
    let rate = ProductionRate::<AverageDaysTime>::new(0.);
    let incremental_volume = 1000.;

    let result = FlatParameters::from_incremental_volume(rate, incremental_volume);
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn zero_duration() {
    let rate = ProductionRate::<AverageDaysTime>::new(100.);
    let zero_time = AverageDaysTime { days: 0. };

    let params = FlatParameters::from_incremental_duration(rate, zero_time).unwrap();

    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn zero_duration_from_zero_volume() {
    let rate = ProductionRate::<AverageDaysTime>::new(0.);
    let incremental_volume = 0.;

    let params = FlatParameters::from_incremental_volume(rate, incremental_volume).unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
}

#[test]
fn flat_rejects_non_finite_parameters() {
    let result = FlatParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(f64::NAN),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"rate is not-a-number, but expected a finite number");

    let result = FlatParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(f64::INFINITY),
        1000.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"rate is infinity, but expected a finite number");

    let result = FlatParameters::<AverageDaysTime>::from_incremental_volume(
        ProductionRate::new(100.),
        f64::INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn flat_from_volume_never_panics_and_results_valid(
        rate in prop::num::f64::ANY,
        volume in prop::num::f64::ANY,
    ) {
        let rate_val = ProductionRate::<AverageDaysTime>::new(rate);
        let result = FlatParameters::from_incremental_volume(rate_val, volume);

        if let Ok(params) = result {
            let duration = params.incremental_duration().days;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);

            let computed_volume = params.incremental_volume();
            prop_assert!(computed_volume >= 0. || computed_volume.is_nan() || computed_volume.is_infinite(),
                "Computed volume should be non-negative, got {}", computed_volume);
        }
    }
}

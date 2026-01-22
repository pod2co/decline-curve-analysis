use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, HyperbolicParameters, NominalDeclineRate, ProductionRate,
};
use proptest::prelude::*;

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
    insta::assert_snapshot!(parameters.unwrap_err(), @"decline rate has wrong sign");
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
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // Positive decline rate declining with negative exponent.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.4).into(),
        -0.9,
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"decline rate has wrong sign");

    // Positive initial decline rate with negative final decline rate.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        0.9,
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // Negative initial decline rate with positive final decline rate.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        0.9,
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"decline rate has wrong sign");
}

#[test]
fn volume_range() {
    // For hyperbolic declines with 0 < b < 1, we calculate max volume as:
    //
    // max volume as time approaches infinity = q_i / (d * (1 - b))
    // = 100 / (0.1 * (1 - 0.5)) = 2000
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let exponent = 0.5;
    let beyond_max = 3000.;
    let result = HyperbolicParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        beyond_max,
        exponent,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let exponent = 0.5;
    let at_max = 100. / (0.1 * 0.5);
    let result =
        HyperbolicParameters::from_incremental_volume(initial_rate, decline_rate, at_max, exponent);
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn exponent_greater_than_one() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let exponent = 1.5;
    let large_volume = 1000.;
    let params = HyperbolicParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        large_volume,
        exponent,
    )
    .unwrap();
    insta::assert_snapshot!(params.incremental_duration().years, @"15.83333333333333");
}

#[test]
fn negative_exponent() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let exponent = -0.5;
    let exceeding_volume = 1000.;
    let result = HyperbolicParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        exceeding_volume,
        exponent,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"decline rate has wrong sign");

    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let exponent = -0.5;
    let large_volume = 10000.;
    let params = HyperbolicParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        large_volume,
        exponent,
    )
    .unwrap();
    insta::assert_snapshot!(params.incremental_duration().years, @"30.396841995794926");
}

#[test]
fn finite_exponent() {
    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        500.,
        f64::NAN,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"exponent is not-a-number, but expected a finite number");

    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        500.,
        f64::INFINITY,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"exponent is infinity, but expected a finite number");
}

#[test]
fn finite_initial_decline_rate() {
    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        1000.,
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is infinity, but expected a finite number");

    let result = HyperbolicParameters::<AverageYearsTime>::from_final_decline_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::INFINITY),
        NominalDeclineRate::new(0.1),
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is infinity, but expected a finite number");

    let result = HyperbolicParameters::<AverageYearsTime>::from_final_decline_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(f64::NAN),
        NominalDeclineRate::new(0.1),
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"initial decline rate is not-a-number, but expected a finite number");
}

#[test]
fn finite_volume() {
    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        f64::INFINITY,
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is infinity, but expected a finite number");
}

#[test]
fn finite_final_decline_rate() {
    let result = HyperbolicParameters::<AverageYearsTime>::from_final_decline_rate(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.5),
        NominalDeclineRate::new(f64::INFINITY),
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"final decline rate is infinity, but expected a finite number");
}

#[test]
fn exponent_range() {
    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        500.,
        0.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"exponent was approximately zero, so an exponential should be used instead");

    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        500.,
        1.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"exponent was approximately one, so a harmonic should be used instead");

    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        500.,
        150.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"exponent too large");

    let result = HyperbolicParameters::<AverageYearsTime>::from_incremental_volume(
        ProductionRate::new(100.),
        NominalDeclineRate::new(0.1),
        500.,
        -150.,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"exponent too large");
}

#[test]
fn zero_duration() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let zero_time = AverageDaysTime { days: 0. };
    let exponent = 0.5;
    let params = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        zero_time,
        exponent,
    )
    .unwrap();
    insta::assert_snapshot!(params.incremental_duration().days, @"0");
    insta::assert_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn zero_volume() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let exponent = 0.5;
    let result =
        HyperbolicParameters::from_incremental_volume(initial_rate, decline_rate, 0., exponent);
    let params = result.unwrap();
    insta::assert_snapshot!(params.incremental_duration().years, @"0");
}

#[test]
fn final_rate_roundtrip() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let target_final_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let exponent = 0.5;

    let params = HyperbolicParameters::from_final_rate(
        initial_rate,
        decline_rate,
        target_final_rate,
        exponent,
    )
    .unwrap();

    let actual_final_rate = params.final_rate().value();
    insta::assert_snapshot!(actual_final_rate, @"49.999999999999986");
}

#[test]
fn duration_range() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.1);
    let exponent = 0.5;
    let extreme_duration = AverageYearsTime { years: 10000. };
    let result = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        decline_rate,
        extreme_duration,
        exponent,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn from_incremental_duration(
        rate in prop::num::f64::ANY,
        decline in prop::num::f64::ANY,
        duration in prop::num::f64::ANY,
        exponent in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let incremental_duration = AverageYearsTime { years: duration };
        let result = HyperbolicParameters::from_incremental_duration(initial_rate, decline_rate, incremental_duration, exponent);

        if let Ok(params) = result {
            let duration = params.incremental_duration().years;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_incremental_volume(
        rate in prop::num::f64::ANY,
        decline in prop::num::f64::ANY,
        volume in prop::num::f64::ANY,
        exponent in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let result = HyperbolicParameters::from_incremental_volume(initial_rate, decline_rate, volume, exponent);

        if let Ok(params) = result {
            let duration = params.incremental_duration().years;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_rate(
        rate in prop::num::f64::ANY,
        decline in prop::num::f64::ANY,
        final_rate_value in prop::num::f64::ANY,
        exponent in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let final_rate = ProductionRate::<AverageYearsTime>::new(final_rate_value);
        let result = HyperbolicParameters::from_final_rate(initial_rate, decline_rate, final_rate, exponent);

        if let Ok(params) = result {
            let duration = params.incremental_duration().years;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_decline_rate(
        rate in prop::num::f64::ANY,
        initial_decline in prop::num::f64::ANY,
        final_decline in prop::num::f64::ANY,
        exponent in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(initial_decline);
        let final_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(final_decline);
        let result = HyperbolicParameters::from_final_decline_rate(initial_rate, initial_decline_rate, final_decline_rate, exponent);

        if let Ok(params) = result {
            let duration = params.incremental_duration().years;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }
}

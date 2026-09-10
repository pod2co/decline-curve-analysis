mod common;
use common::{assert_sig_snapshot, assert_solved};
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

    assert_sig_snapshot!(calculated_duration, @"3650");
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

    assert_sig_snapshot!(calculated_duration, @"2643.3545189");
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

    assert_sig_snapshot!(calculated_duration, @"2643.3545189");
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

    assert_sig_snapshot!(calculated_duration, @"2643.3545189");
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
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 2700. }), @"54298.1001103");

    // Check a point somewhere in the middle.
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 0.5 * 2700. }), @"37666.2621469");
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

    assert_sig_snapshot!(parameters.final_rate().value(), @"9.99999780962");
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

    assert_sig_snapshot!(parameters.incremental_duration().days, @"3650");
    assert_sig_snapshot!(parameters.incremental_volume_at_time(AverageDaysTime { days: 4000. }), @"187066.896276");
    assert_sig_snapshot!(parameters.final_rate().value(), @"52.5044488495");
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

    // A negative exponent makes the decline rate grow, so a smaller final one is never reached.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.4).into(),
        -0.9,
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // The same pair the other way around is reachable.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.4).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.5).into(),
        -0.9,
    )
    .unwrap();
    assert_sig_snapshot!(parameters.incremental_duration().days, @"202.916666667");

    // Positive initial decline rate with negative final decline rate.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        0.9,
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // A segment never crosses from inclining to declining.
    let parameters = HyperbolicParameters::from_final_decline_rate(
        initial_rate,
        NominalDeclineRate::<AverageYearsTime>::new(-0.1).into(),
        NominalDeclineRate::<AverageYearsTime>::new(0.1).into(),
        0.9,
    );
    insta::assert_snapshot!(parameters.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
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
    assert_sig_snapshot!(params.incremental_duration().years, @"15.8333333333");
}

#[test]
fn negative_exponent() {
    // The rate reaches zero at `t_max`, capping the volume at `100 / (1.5 * 0.1)`.
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
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let volume_under_the_cap = 500.;
    let params = HyperbolicParameters::from_incremental_volume(
        initial_rate,
        decline_rate,
        volume_under_the_cap,
        exponent,
    )
    .unwrap();
    // Short of `t_max`, which is 20 years for these parameters.
    assert_sig_snapshot!(params.incremental_duration().years, @"7.40078950105");
    assert_solved(params.incremental_volume(), volume_under_the_cap);

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
    assert_sig_snapshot!(params.incremental_duration().years, @"30.3968419958");
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
    assert_sig_snapshot!(params.incremental_duration().days, @"0");
    assert_sig_snapshot!(params.incremental_volume(), @"0");
}

#[test]
fn zero_volume() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.1);
    let exponent = 0.5;
    let result =
        HyperbolicParameters::from_incremental_volume(initial_rate, decline_rate, 0., exponent);
    let params = result.unwrap();
    assert_sig_snapshot!(params.incremental_duration().years, @"0");
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
    assert_sig_snapshot!(actual_final_rate, @"50");
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

#[test]
fn hyperbolic_from_incremental_duration_and_final_rate() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };
    let exponent = 0.9;

    let reference = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap();

    let solved = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        reference.final_rate(),
        exponent,
    )
    .unwrap();

    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());
}

#[test]
fn hyperbolic_from_final_rate_and_incremental_volume() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 10. * 365. };
    let exponent = 0.9;

    let reference = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap();

    let solved = HyperbolicParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        reference.final_rate(),
        reference.incremental_volume(),
        exponent,
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
fn hyperbolic_solving_rejects_an_exponent_that_belongs_to_another_family() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        final_rate,
        0.,
    );

    insta::assert_snapshot!(
        result.unwrap_err(),
        @"exponent was approximately zero, so an exponential should be used instead"
    );
}

#[test]
fn hyperbolic_holds_precision_for_exponents_near_one() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline_rate = 0.35;
    let duration = AverageYearsTime { years: 10. };

    let final_rate =
        ProductionRate::<AverageYearsTime>::new(100. / (1. + decline_rate * duration.years));
    let volume = 100. * (1. + decline_rate * duration.years).ln() / decline_rate;

    for exponent in [1. + 2e-12, 1. - 2e-12] {
        let solved = HyperbolicParameters::from_final_rate_and_incremental_volume(
            initial_rate,
            final_rate,
            volume,
            exponent,
        )
        .unwrap();

        let relative_error =
            (solved.initial_decline_rate().value() - decline_rate).abs() / decline_rate;
        assert!(
            relative_error < 1e-11,
            "exponent {exponent} solved to a relative error of {relative_error:e}"
        );
    }
}

#[test]
fn hyperbolic_solving_an_incline_keeps_the_sign() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incline_rate = NominalDeclineRate::<AverageYearsTime>::new(-0.2).into();
    let exponent = 0.5;
    // `t_max = -1 / (b * d)` is 10 years here.
    let incremental_duration = AverageDaysTime { days: 2. * 365. };

    let reference = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        incline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap();

    let solved = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        reference.final_rate(),
        exponent,
    )
    .unwrap();

    assert!(solved.initial_decline_rate().value() < 0.);
    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(solved.final_rate().value(), reference.final_rate().value());

    let solved = HyperbolicParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        reference.final_rate(),
        reference.incremental_volume(),
        exponent,
    )
    .unwrap();

    assert!(solved.initial_decline_rate().value() < 0.);
    assert_solved(
        solved.initial_decline_rate().value(),
        reference.initial_decline_rate().value(),
    );
    assert_solved(solved.incremental_volume(), reference.incremental_volume());
}

/// A negative `b * d` product limits the duration rather than ruling the segment out.
#[test]
fn hyperbolic_allows_every_sign_combination() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let decline = NominalDeclineRate::<AverageYearsTime>::new(0.2);
    let incline = NominalDeclineRate::<AverageYearsTime>::new(-0.2);
    let duration = AverageYearsTime { years: 2. };

    // `b * d > 0`: the base only grows, so any duration is reachable.
    let params =
        HyperbolicParameters::from_incremental_duration(initial_rate, decline, duration, 0.5)
            .unwrap();
    assert_sig_snapshot!(params.final_rate().value(), @"69.4444444444");

    let params =
        HyperbolicParameters::from_incremental_duration(initial_rate, incline, duration, -0.5)
            .unwrap();
    assert_sig_snapshot!(params.final_rate().value(), @"144");

    // `b > 0` with an incline: the rate runs away to the singularity.
    let params =
        HyperbolicParameters::from_incremental_duration(initial_rate, incline, duration, 0.5)
            .unwrap();
    assert_sig_snapshot!(params.final_rate().value(), @"156.25");

    // `b < 0` with a decline: the rate runs down to zero at the singularity.
    let params =
        HyperbolicParameters::from_incremental_duration(initial_rate, decline, duration, -0.5)
            .unwrap();
    assert_sig_snapshot!(params.final_rate().value(), @"64");
}

#[test]
fn hyperbolic_rejects_a_duration_past_the_singularity() {
    let initial_rate = ProductionRate::<AverageYearsTime>::new(100.);
    let incline = NominalDeclineRate::<AverageYearsTime>::new(-0.2);
    let exponent = 0.5;
    // `t_max = -1 / (b * d)` is 10 years here.
    let at_singularity = AverageYearsTime { years: 10. };

    let result = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        incline,
        at_singularity,
        exponent,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration too long");

    let just_under = AverageYearsTime { years: 9.9 };
    let params = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        incline,
        just_under,
        exponent,
    )
    .unwrap();
    assert_sig_snapshot!(params.final_rate().value(), @"1000000");
}

#[test]
fn hyperbolic_solving_rejects_cutoffs_that_imply_no_decline() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        initial_rate,
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    let result = HyperbolicParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        ProductionRate::<AverageDaysTime>::new(10.),
        1e300,
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");

    // The rate ratio overflows to infinity.
    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        ProductionRate::<AverageDaysTime>::new(1e300),
        incremental_duration,
        ProductionRate::<AverageDaysTime>::new(1e-11),
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"cannot solve decline: no finite solution exists for the given parameters");
}

#[test]
fn hyperbolic_solving_rejects_degenerate_cutoffs() {
    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        AverageDaysTime { days: 0. },
        final_rate,
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"duration is approximately zero, but expected it to be non-zero");

    let result = HyperbolicParameters::from_final_rate_and_incremental_volume(
        initial_rate,
        final_rate,
        0.,
        0.5,
    );
    insta::assert_snapshot!(result.unwrap_err(), @"incremental volume is approximately zero, but expected it to be non-zero");
}

#[test]
fn hyperbolic_rejects_exponents_too_close_to_zero() {
    const MIN_EXPONENT: f64 = 1e-6;

    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let incremental_duration = AverageDaysTime { days: 10. * 365. };
    let final_rate = ProductionRate::<AverageDaysTime>::new(10.);

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        final_rate,
        1e-12,
    );
    insta::assert_snapshot!(
        result.unwrap_err(),
        @"exponent was approximately zero, so an exponential should be used instead"
    );

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        final_rate,
        1e-9,
    );
    insta::assert_snapshot!(
        result.unwrap_err(),
        @"exponent was approximately zero, so an exponential should be used instead"
    );

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        final_rate,
        MIN_EXPONENT / 10.,
    );
    insta::assert_snapshot!(
        result.unwrap_err(),
        @"exponent was approximately zero, so an exponential should be used instead"
    );

    let result = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        final_rate,
        -MIN_EXPONENT / 10.,
    );
    insta::assert_snapshot!(
        result.unwrap_err(),
        @"exponent was approximately zero, so an exponential should be used instead"
    );

    // Right at the limit the solve still holds about ten significant digits.
    let solved = HyperbolicParameters::from_incremental_duration_and_final_rate(
        initial_rate,
        incremental_duration,
        final_rate,
        MIN_EXPONENT,
    )
    .unwrap();
    let relative_error =
        (solved.final_rate().value() - final_rate.value()).abs() / final_rate.value();
    assert!(
        relative_error < 1e-9,
        "relative error was {relative_error:e}"
    );
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

    /// Nothing the solver returns may carry an infinity or a NaN out with it.
    #[test]
    fn from_incremental_duration_and_final_rate(
        rate in prop::num::f64::ANY,
        duration in prop::num::f64::ANY,
        final_rate_value in prop::num::f64::ANY,
        exponent in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let final_rate = ProductionRate::<AverageYearsTime>::new(final_rate_value);
        let incremental_duration = AverageYearsTime { years: duration };
        let result = HyperbolicParameters::from_incremental_duration_and_final_rate(initial_rate, incremental_duration, final_rate, exponent);

        if let Ok(params) = result {
            let decline = params.initial_decline_rate().value();
            prop_assert!(decline.is_finite(), "Decline rate should be finite, got {}", decline);
            prop_assert!(decline != 0., "Decline rate should be non-zero");
            let duration = params.incremental_duration().years;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn from_final_rate_and_incremental_volume(
        rate in prop::num::f64::ANY,
        final_rate_value in prop::num::f64::ANY,
        volume in prop::num::f64::ANY,
        exponent in prop::num::f64::ANY,
    ) {
        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let final_rate = ProductionRate::<AverageYearsTime>::new(final_rate_value);
        let result = HyperbolicParameters::from_final_rate_and_incremental_volume(initial_rate, final_rate, volume, exponent);

        if let Ok(params) = result {
            let decline = params.initial_decline_rate().value();
            prop_assert!(decline.is_finite(), "Decline rate should be finite, got {}", decline);
            prop_assert!(decline != 0., "Decline rate should be non-zero");
            let duration = params.incremental_duration().years;
            prop_assert!(duration >= 0., "Duration should be non-negative, got {}", duration);
            prop_assert!(duration.is_finite(), "Duration should be finite, got {}", duration);
        }
    }

    #[test]
    fn solving_round_trips_through_both_cutoffs(
        rate in 1.0f64..1e5,
        decline in prop_oneof![-1.0f64..-0.01, 0.01f64..1.0],
        duration in 0.1f64..50.,
        exponent in prop_oneof![-2.0f64..-0.05, 0.05f64..0.95, 1.05f64..2.0],
        singularity_fraction in 0.01f64..0.95,
        decline_span in 0.01f64..10.,
    ) {
        // Cap how far the segment declines, so the final rate stays above the validation floor.
        let duration = (decline_span / decline.abs()).min(duration);

        // Stay short of the singularity at `t_max = -1 / (b * d)`.
        let base_slope = exponent * decline;
        let duration = if base_slope < 0. {
            (-singularity_fraction / base_slope).min(duration)
        } else {
            duration
        };

        let initial_rate = ProductionRate::<AverageYearsTime>::new(rate);
        let decline_rate = NominalDeclineRate::<AverageYearsTime>::new(decline);
        let incremental_duration = AverageYearsTime { years: duration };

        let reference = HyperbolicParameters::from_incremental_duration(
            initial_rate,
            decline_rate,
            incremental_duration,
            exponent,
        )
        .unwrap();

        let solved = HyperbolicParameters::from_incremental_duration_and_final_rate(
            initial_rate,
            incremental_duration,
            reference.final_rate(),
            exponent,
        )
        .unwrap();
        assert_solved(solved.initial_decline_rate().value(), decline);
        assert_solved(solved.final_rate().value(), reference.final_rate().value());

        let solved = HyperbolicParameters::from_final_rate_and_incremental_volume(
            initial_rate,
            reference.final_rate(),
            reference.incremental_volume(),
            exponent,
        )
        .unwrap();
        assert_solved(solved.initial_decline_rate().value(), decline);
        assert_solved(solved.incremental_duration().years, duration);
        assert_solved(solved.incremental_volume(), reference.incremental_volume());
    }
}

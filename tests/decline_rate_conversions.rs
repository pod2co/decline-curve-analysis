use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, NominalDeclineRate, SecantEffectiveDeclineRate,
};

macro_rules! assert_approx_eq {
    ($a:expr, $b:expr, $tolerance:expr) => {
        assert!(
            (($a - $b).abs() < $tolerance),
            "expected {} to be approximately equal to {}",
            $a,
            $b
        );
    };
}

#[test]
fn spee_conversion_examples() {
    let nominal_rates = vec![
        1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.,
        200., 300., 400., 500., 600., 700., 800., 900., 1000., 2000., 3000., 4000., 5000., 6000.,
        7000., 8000., 9000., 10000.,
    ];

    let exponents = vec![0., 0.5, 1., 1.5, 2.];

    // Generate all combinations then verify it with insta. Use `f32` for results so snapshots
    // don't depend on CPU-specific float handling in the least significant bits. We could use
    // insta redactions instead, but this is probably simple enough for this.
    #[derive(serde::Serialize)]
    struct Row {
        nominal_percent: f32,
        tangent_effective: f32,
        secant_effective: Vec<f32>,
    }

    let mut results = vec![];
    for nominal_percent in nominal_rates {
        let nominal = NominalDeclineRate::<AverageYearsTime>::new(nominal_percent / 100.);
        let tangent_effective = nominal.to_tangent_effective().unwrap();

        let mut secant_effective = Vec::new();
        for exponent in exponents.iter().copied() {
            let secant_effective_for_exponent = nominal.to_secant_effective(exponent).unwrap();
            secant_effective
                .push((nominal.to_secant_effective(exponent).unwrap().value() * 100.) as f32);

            // Nominal to secant/tangent effective will be checked by the snapshot, so check
            // the other combinations here:
            // - secant effective to nominal
            // - tangent effective to nominal
            // - secant effective to tangent effective
            // - tangent effective to secant effective
            //
            // Note that we run out of precision once the nominal decline rate is high enough,
            // so change the tolerance depending on the original nominal decline rate:
            // - < 300: 1e-6
            // - < 3000: 1e-3
            // - >= 4000: skip (round-trip will fail because of precision)
            if nominal_percent < 4000. {
                let tolerance = if nominal_percent < 3000. { 1e-6 } else { 1e-3 };

                let secant_to_nominal_result =
                    secant_effective_for_exponent.to_nominal(exponent).unwrap();
                assert_approx_eq!(secant_to_nominal_result.value(), nominal.value(), tolerance);

                let tangent_to_nominal_result = tangent_effective.to_nominal().unwrap();
                assert_approx_eq!(
                    tangent_to_nominal_result.value(),
                    nominal.value(),
                    tolerance
                );

                let secant_to_tangent_result = secant_effective_for_exponent
                    .to_tangent_effective(exponent)
                    .unwrap();
                assert_approx_eq!(
                    secant_to_tangent_result.value(),
                    tangent_effective.value(),
                    tolerance
                );

                let tangent_to_secant_result =
                    tangent_effective.to_secant_effective(exponent).unwrap();
                assert_approx_eq!(
                    tangent_to_secant_result.value(),
                    secant_effective_for_exponent.value(),
                    tolerance
                );
            }
        }

        results.push(Row {
            nominal_percent: nominal_percent as f32,
            tangent_effective: (tangent_effective.value() * 100.) as f32,
            secant_effective,
        });
    }

    // Verify the results with insta.
    insta::assert_ron_snapshot!(results, @r#"
    [
      Row(
        nominal_percent: 1.0,
        tangent_effective: 0.99501663,
        secant_effective: [
          0.99501663,
          0.9925497,
          0.990099,
          0.9876644,
          0.9852457,
        ],
      ),
      Row(
        nominal_percent: 2.0,
        tangent_effective: 1.9801327,
        secant_effective: [
          1.9801327,
          1.9703951,
          1.9607843,
          1.9512976,
          1.9419324,
        ],
      ),
      Row(
        nominal_percent: 3.0,
        tangent_effective: 2.9554467,
        secant_effective: [
          2.9554467,
          2.9338253,
          2.9126213,
          2.8918219,
          2.8714137,
        ],
      ),
      Row(
        nominal_percent: 4.0,
        tangent_effective: 3.921056,
        secant_effective: [
          3.921056,
          3.883122,
          3.8461537,
          3.810111,
          3.774955,
        ],
      ),
      Row(
        nominal_percent: 5.0,
        tangent_effective: 4.8770576,
        secant_effective: [
          4.8770576,
          4.8185606,
          4.7619047,
          4.7069945,
          4.653741,
        ],
      ),
      Row(
        nominal_percent: 6.0,
        tangent_effective: 5.8235464,
        secant_effective: [
          5.8235464,
          5.740409,
          5.6603775,
          5.58326,
          5.5088816,
        ],
      ),
      Row(
        nominal_percent: 7.0,
        tangent_effective: 6.760618,
        secant_effective: [
          6.760618,
          6.64893,
          6.542056,
          6.439655,
          6.3414187,
        ],
      ),
      Row(
        nominal_percent: 8.0,
        tangent_effective: 7.6883655,
        secant_effective: [
          7.6883655,
          7.5443788,
          7.4074073,
          7.276891,
          7.152331,
        ],
      ),
      Row(
        nominal_percent: 9.0,
        tangent_effective: 8.606881,
        secant_effective: [
          8.606881,
          8.427005,
          8.256881,
          8.095645,
          7.9425383,
        ],
      ),
      Row(
        nominal_percent: 10.0,
        tangent_effective: 9.516258,
        secant_effective: [
          9.516258,
          9.297052,
          9.090909,
          8.896561,
          8.712907,
        ],
      ),
      Row(
        nominal_percent: 20.0,
        tangent_effective: 18.126925,
        secant_effective: [
          18.126925,
          17.355371,
          16.666666,
          16.046701,
          15.484574,
        ],
      ),
      Row(
        nominal_percent: 30.0,
        tangent_effective: 25.918179,
        secant_effective: [
          25.918179,
          24.385633,
          23.076923,
          21.941298,
          20.943058,
        ],
      ),
      Row(
        nominal_percent: 40.0,
        tangent_effective: 32.967995,
        secant_effective: [
          32.967995,
          30.555555,
          28.571428,
          26.899557,
          25.464401,
        ],
      ),
      Row(
        nominal_percent: 50.0,
        tangent_effective: 39.346935,
        secant_effective: [
          39.346935,
          36.0,
          33.333332,
          31.138792,
          29.289322,
        ],
      ),
      Row(
        nominal_percent: 60.0,
        tangent_effective: 45.118835,
        secant_effective: [
          45.118835,
          40.828403,
          37.5,
          34.812508,
          32.580013,
        ],
      ),
      Row(
        nominal_percent: 70.0,
        tangent_effective: 50.34147,
        secant_effective: [
          50.34147,
          45.130314,
          41.17647,
          38.032482,
          35.45028,
        ],
      ),
      Row(
        nominal_percent: 80.0,
        tangent_effective: 55.067104,
        secant_effective: [
          55.067104,
          48.97959,
          44.444443,
          40.882206,
          37.98263,
        ],
      ),
      Row(
        nominal_percent: 90.0,
        tangent_effective: 59.343033,
        secant_effective: [
          59.343033,
          52.437572,
          47.36842,
          43.425407,
          40.23857,
        ],
      ),
      Row(
        nominal_percent: 100.0,
        tangent_effective: 63.212055,
        secant_effective: [
          63.212055,
          55.555557,
          50.0,
          45.711647,
          42.264973,
        ],
      ),
      Row(
        nominal_percent: 200.0,
        tangent_effective: 86.46647,
        secant_effective: [
          86.46647,
          75.0,
          66.666664,
          60.314972,
          55.27864,
        ],
      ),
      Row(
        nominal_percent: 300.0,
        tangent_effective: 95.02129,
        secant_effective: [
          95.02129,
          84.0,
          75.0,
          67.90592,
          62.203552,
        ],
      ),
      Row(
        nominal_percent: 400.0,
        tangent_effective: 98.168434,
        secant_effective: [
          98.168434,
          88.888885,
          80.0,
          72.67241,
          66.666664,
        ],
      ),
      Row(
        nominal_percent: 500.0,
        tangent_effective: 99.3262,
        secant_effective: [
          99.3262,
          91.83673,
          83.333336,
          75.990265,
          69.84887,
        ],
      ),
      Row(
        nominal_percent: 600.0,
        tangent_effective: 99.75212,
        secant_effective: [
          99.75212,
          93.75,
          85.71429,
          78.45565,
          72.26499,
        ],
      ),
      Row(
        nominal_percent: 700.0,
        tangent_effective: 99.90881,
        secant_effective: [
          99.90881,
          95.06173,
          87.5,
          80.37236,
          74.180115,
        ],
      ),
      Row(
        nominal_percent: 800.0,
        tangent_effective: 99.96645,
        secant_effective: [
          99.96645,
          96.0,
          88.888885,
          81.91281,
          75.74644,
        ],
      ),
      Row(
        nominal_percent: 900.0,
        tangent_effective: 99.987656,
        secant_effective: [
          99.987656,
          96.694214,
          90.0,
          83.18276,
          77.058426,
        ],
      ),
      Row(
        nominal_percent: 1000.0,
        tangent_effective: 99.99546,
        secant_effective: [
          99.99546,
          97.22222,
          90.90909,
          84.250984,
          78.17821,
        ],
      ),
      Row(
        nominal_percent: 2000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.17355,
          95.2381,
          89.866516,
          84.38262,
        ],
      ),
      Row(
        nominal_percent: 3000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.609375,
          96.77419,
          92.21076,
          87.19631,
        ],
      ),
      Row(
        nominal_percent: 4000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.77324,
          97.560974,
          93.54672,
          88.888885,
        ],
      ),
      Row(
        nominal_percent: 5000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.85207,
          98.039215,
          94.426544,
          90.04963,
        ],
      ),
      Row(
        nominal_percent: 6000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.89594,
          98.36066,
          95.057205,
          90.90909,
        ],
      ),
      Row(
        nominal_percent: 7000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.92284,
          98.59155,
          95.53526,
          91.57848,
        ],
      ),
      Row(
        nominal_percent: 8000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.94051,
          98.765434,
          95.91232,
          92.1189,
        ],
      ),
      Row(
        nominal_percent: 9000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.95274,
          98.9011,
          96.218704,
          92.567055,
        ],
      ),
      Row(
        nominal_percent: 10000.0,
        tangent_effective: 100.0,
        secant_effective: [
          100.0,
          99.961555,
          99.0099,
          96.47346,
          92.94654,
        ],
      ),
    ]
    "#);
}

#[test]
fn secant_to_nominal_daily() {
    let secant = SecantEffectiveDeclineRate::<AverageYearsTime>::new(0.4);
    let exponent = 0.9;

    let nominal_yearly = secant.to_nominal(exponent).unwrap();
    assert_approx_eq!(nominal_yearly.value(), 0.6485188, 1e-6);

    let nominal_daily: NominalDeclineRate<AverageDaysTime> = nominal_yearly.into();
    assert_approx_eq!(nominal_daily.value(), 0.6485188 / 365.25, 1e-6);
}

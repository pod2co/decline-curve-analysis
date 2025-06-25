use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use decline_curve_analysis::{
    AverageDaysTime, AverageYearsTime, HyperbolicParameters, NominalDeclineRate, ProductionRate,
};

fn every_day(p: &HyperbolicParameters<AverageDaysTime>) {
    for d in 0..p.incremental_duration().days as u64 {
        black_box(p.incremental_volume_at_time(AverageDaysTime { days: d as f64 }));
    }
}

fn hyperbolic(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hyperbolic");

    let initial_rate = ProductionRate::<AverageDaysTime>::new(50.);
    let initial_decline_rate = NominalDeclineRate::<AverageYearsTime>::new(0.5).into();
    let incremental_duration = AverageDaysTime { days: 100. * 365. };
    let exponent = 0.7;
    let parameters = HyperbolicParameters::from_incremental_duration(
        initial_rate,
        initial_decline_rate,
        incremental_duration,
        exponent,
    )
    .unwrap();

    group.bench_with_input(
        BenchmarkId::new("Daily", "Incremental Volume"),
        &parameters,
        |b, p| b.iter(|| black_box(every_day(p))),
    );

    group.finish();
}

criterion_group!(benches, hyperbolic);
criterion_main!(benches);

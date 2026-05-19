use criterion::{black_box, criterion_group, criterion_main, Criterion};

const PIPELINE: &str = r#"source "sensor_data"
|> filter(temperature > 30.0)
|> sink "alerts""#;

fn bench_compile(c: &mut Criterion) {
    c.bench_function("aql_compile", |b| {
        b.iter(|| aql_compile::compile(black_box(PIPELINE)).unwrap());
    });
}

criterion_group!(benches, bench_compile);
criterion_main!(benches);

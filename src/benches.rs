#![allow(warnings)]

use crate::square_test::shakespeare_run;
use crate::{gen::gen_tests, square_test::shakespeare_setup};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::{hint::black_box, sync::Arc, time::Duration};
mod gen;
use xactor_benchmarks::{actix_test, shakespeare_test, xactor_test, Spec};
mod square_test;

criterion_group!(shakespeare, bench_shakespeare);
criterion_group!(square, bench_square);
criterion_group!(benches, bench_combined);
criterion_main!(square);

fn bench_combined(c: &mut Criterion) {
    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());

    let tests = gen_tests(Some(2));

    for spec in tests.into_iter() {
        let Spec {
            procs,
            messages,
            parallel,
            ..
        } = spec;
        let mut group = c.benchmark_group(format!(
            "combined: {procs} procs; {messages} msgs; {parallel} threads"
        ));
        //group.sample_size(50);
        //group.measurement_time(Duration::from_secs(20));
        //group.sampling_mode(criterion::SamplingMode::Flat);
        group.throughput(criterion::Throughput::Elements(spec.messages as _));
        group.bench_with_input(BenchmarkId::from_parameter("actix"), &spec, |b, spec| {
            b.iter(|| actix_test::run(black_box(spec)))
        });
        let s_rt = rt.clone();
        group.bench_with_input(
            BenchmarkId::from_parameter("shakespeare"),
            &spec,
            |b, spec| {
                b.to_async(s_rt.as_ref())
                    .iter(|| shakespeare_test::run(black_box(spec)))
            },
        );
        let x_rt = rt.clone();
        group.bench_with_input(BenchmarkId::from_parameter("xactor"), &spec, |b, spec| {
            // See https://github.com/async-rs/async-std/issues/770#issuecomment-633011171
            b.to_async(x_rt.as_ref())
                .iter(|| async { xactor_test::run(black_box(spec)).await })
        });
    }
}

fn bench_actix(c: &mut Criterion) {
    let tests = gen_tests(Some(2));

    let mut group = c.benchmark_group("actix");
    for spec in tests.into_iter() {
        group.bench_with_input(BenchmarkId::from_parameter(&spec), &spec, |b, spec| {
            b.iter(|| actix_test::run(black_box(spec)))
        });
    }
    group.finish();
}

fn bench_shakespeare(c: &mut Criterion) {
    let tests = gen_tests(Some(2));

    let mut group = c.benchmark_group("shakespeare");
    for spec in tests.into_iter() {
        group.bench_with_input(BenchmarkId::from_parameter(&spec), &spec, |b, spec| {
            b.iter(|| shakespeare_test::run(black_box(spec)))
        });
    }
    group.finish();
}

fn bench_xactor(c: &mut Criterion) {
    let tests = gen_tests(Some(2));

    let mut group = c.benchmark_group("xactor");
    for spec in tests.into_iter() {
        group.bench_with_input(BenchmarkId::from_parameter(&spec), &spec, |b, spec| {
            // See https://github.com/async-rs/async-std/issues/770#issuecomment-633011171
            b.iter(|| tokio_test::block_on(async { xactor_test::run(black_box(spec)).await }))
        });
    }
    group.finish();
}

pub fn bench_square(c: &mut Criterion) {
    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let mut g = c.benchmark_group("Square");
    //g.measurement_time(Duration::from_secs(30));
    for n in 1..=10 {
        let n = 2 * n;
        g.throughput(criterion::Throughput::Elements(n * n));
        g.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, n| {
            b.to_async(rt.as_ref()).iter_batched(
                || shakespeare_setup(*n as _),
                shakespeare_run,
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

use cl_format::*;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
//use pprof::criterion::{Output, PProfProfiler};

fn cl_format_reveal_single_a(control_str: &ControlStr, args: Args) -> String {
    control_str.reveal(args).unwrap()
}

fn cl_format_plain_single_a(v: i32) -> String {
    format!("{}", v)
}

fn bench_cl_format_reveal_single_a(c: &mut Criterion) {
    let list0 = vec![tilde!(&1)];
    let control_str = ControlStr::new("~a").unwrap();
    let args = Args::new(list0);
    c.bench_function("bench_cl_format_reveal_single_a", |b| {
        b.iter(|| cl_format_reveal_single_a(&control_str, args.clone()))
    });
}

fn bench_cl_format_plain_single_a(c: &mut Criterion) {
    let a = 1;
    c.bench_function("bench_cl_format_plain_single_a", |b| {
        b.iter(|| cl_format_plain_single_a(a.clone()))
    });
}

criterion_group! {
    name = bench_single;
    config = Criterion::default();
    targets = bench_cl_format_reveal_single_a, //bench_cl_format_plain_single_a
}

criterion_main!(bench_single);

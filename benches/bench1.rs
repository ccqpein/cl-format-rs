#![feature(test)]

extern crate test;

use std::collections::VecDeque;

use cl_format::*;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use test::Bencher;

fn cl_format_macro_make_string(l: &Vec<&dyn TildeAble>) -> String {
    cl_format!(
        "~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}",
        l
    )
    .unwrap()
}

fn cl_format_make_string(control_str: &ControlStr, l: &Vec<&dyn TildeAble>) -> String {
    let args = Args::new(vec![l]);
    control_str.reveal(args).unwrap()
}

fn loop_making_string(l: &Vec<&str>) -> String {
    let mut l: VecDeque<&str> = l.clone().into();
    let mut result = String::new();
    if l.len() == 0 {
        return String::from("empty");
    }
    loop {
        match l.len() {
            0 => {
                //result += "empty";
                return result;
            }
            1 => {
                result += l.pop_front().unwrap();
                return result;
            }
            2 => {
                result += l.pop_front().unwrap();
                result += " and ";
                result += l.pop_front().unwrap();
                return result;
            }
            _ => loop {
                result += l.pop_front().unwrap();
                match l.len() {
                    0 => return result,
                    1 => result += ", and ",
                    _ => result += ", ",
                }
            },
        }
    }
}

#[test]
fn test_result_are_same() {
    let mut list0 = vec![];
    let mut list1 = vec![];
    assert_eq!(
        cl_format_macro_make_string(&list0),
        loop_making_string(&list1)
    );

    list0.push(&1);
    list1.push("1");
    assert_eq!(
        cl_format_macro_make_string(&list0),
        loop_making_string(&list1)
    );

    list0.push(&2);
    list1.push("2");
    assert_eq!(
        cl_format_macro_make_string(&list0),
        loop_making_string(&list1)
    );

    list0.push(&3);
    list1.push("3");
    assert_eq!(
        cl_format_macro_make_string(&list0),
        loop_making_string(&list1)
    );
}

//#[bench]
fn bench_cl_format_macro_making_loop_empty_string(b: &mut Bencher) {
    let list0 = vec![];
    b.iter(move || cl_format_macro_make_string(&list0));
}

//#[bench]
fn bench_cl_format_making_loop_empty_string(b: &mut Bencher) {
    let list0 = vec![];
    let control_str =
        ControlStr::from("~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}").unwrap();
    b.iter(move || cl_format_make_string(&control_str, &list0));
}

//#[bench]
fn bench_normal_making_loop_empty_string(b: &mut Bencher) {
    let list1 = vec![];
    b.iter(move || loop_making_string(&list1));
}

//#[bench]
fn bench_cl_format_macro_making_loop_string(b: &mut Bencher) {
    let list0 = vec![tilde!(&1), &2, &3];
    b.iter(move || cl_format_macro_make_string(&list0));
}

// #[bench]
// fn bench_cl_format_making_loop_string(b: &mut Bencher) {
//     let list0 = vec![tilde!(&1), &2, &3];
//     let control_str =
//         ControlStr::from("~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}").unwrap();
//     b.iter(move || cl_format_make_string(&control_str, &list0));
// }

// #[bench]
// fn bench_normal_making_loop_string(bench: &mut Bencher) {
//     let (a, b, c) = (String::from("1"), String::from("2"), String::from("3"));
//     let list0 = vec![a.as_str(), b.as_str(), c.as_str()];
//     bench.iter(move || loop_making_string(&list0));
// }

fn bench_cl_format_making_loop_string(c: &mut Criterion) {
    let list0 = vec![tilde!(&1), &2, &3];
    let control_str =
        ControlStr::from("~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}").unwrap();
    //b.iter(move || cl_format_make_string(&control_str, &list0));
    c.bench_function("bench_cl_format_making_loop_string", |b| {
        b.iter(|| cl_format_make_string(&control_str, &list0))
    });
}

fn bench_normal_making_loop_string(cr: &mut Criterion) {
    let (a, b, c) = (String::from("1"), String::from("2"), String::from("3"));
    let list0 = vec![a.as_str(), b.as_str(), c.as_str()];
    //bench.iter(move || loop_making_string(&list0));
    cr.bench_function("bench_normal_making_loop_string", |b| {
        b.iter(|| loop_making_string(&list0))
    });
}

criterion_group! {
    name = bench_loop;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_cl_format_making_loop_string, bench_normal_making_loop_string
}
criterion_main!(bench_loop);

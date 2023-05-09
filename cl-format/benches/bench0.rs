#![feature(test)]

extern crate test;

use std::collections::VecDeque;

use cl_format::*;
use test::Bencher;

fn cl_format_reveal_single_a(control_str: &ControlStr, args: Args) -> String {
    //let args = Args::new(vec![l]);
    control_str.reveal(args).unwrap()
}

fn cl_format_plain_single_a(v: i32) -> String {
    format!("{}", v)
}

#[bench]
fn bench_cl_format_reveal_single_a(b: &mut Bencher) {
    let list0 = vec![tilde!(&1)];
    let control_str = ControlStr::from("~a").unwrap();
    let args = Args::new(list0);
    b.iter(move || cl_format_reveal_single_a(&control_str, args.clone()))
}

#[bench]
fn bench_cl_format_plain_single_a(b: &mut Bencher) {
    let a = 1;
    b.iter(move || cl_format_plain_single_a(a.clone()))
}

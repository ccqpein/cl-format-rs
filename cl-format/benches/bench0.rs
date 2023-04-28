#![feature(test)]

extern crate test;

use cl_format::*;
use test::Bencher;

pub fn loop_make_string() {}
pub fn cl_format_make_string() {}

#[bench]
fn bench_comparing_making_loop_string(b: &mut Bencher) {
    b.iter(|| loop_make_string());
    b.iter(|| cl_format_make_string());
}

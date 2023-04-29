#![feature(test)]

extern crate test;

use std::{collections::VecDeque, fmt::Display};

use cl_format::*;
use test::Bencher;

pub fn loop_make_string() {}

pub fn cl_format_macro_make_string(l: &Vec<&dyn TildeAble>) -> String {
    cl_format!(
        "~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}",
        l
    )
    .unwrap()
}

pub fn loop_making_string(l: &Vec<&str>) -> String {
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

#[bench]
fn bench_cl_format_making_loop_string(b: &mut Bencher) {
    let list0 = vec![];
    b.iter(move || cl_format_macro_make_string(&list0));
}

#[bench]
fn bench_normal_making_loop_string(b: &mut Bencher) {
    let list1 = vec![];
    b.iter(move || loop_making_string(&list1));
}

use cl_format::*;

#[test]
fn play_groud_with_macro_0() {
    let a = cl_format!("~a, ~a, ~a", &1_i32, &2, &3);
    assert_eq!(String::from("1, 2, 3"), a.unwrap());

    let s = String::from("abc");
    let a = cl_format!("~a, ~a, ~a, ~S", &1_i32, &2, &3, &s);
    assert_eq!(String::from("1, 2, 3, \"abc\""), a.unwrap());

    let a = cl_format!("start ~a, ~a, ~a, ~a, here", &1_i32, &2, &3, &s);
    assert_eq!(String::from("start 1, 2, 3, abc, here"), a.unwrap());

    let ll: Vec<&dyn TildeAble> = vec![&1, &2, &3];

    let a = cl_format!("~a, ~a, ~a, ~{~a,~}", &1_i32, &2, &3, &ll);
    assert_eq!(String::from("1, 2, 3, 1,2,3,"), a.unwrap());

    let a = cl_format!("~a, ~a, ~a, ~{~a~^,~}", &1_i32, &2, &3, &ll);
    assert_eq!(String::from("1, 2, 3, 1,2,3"), a.unwrap());
}

#[test]
fn play_groud_with_macro_1() {
    let a = cl_format!("The value is: ~a", &1_i32);
    assert_eq!(String::from("The value is: 1"), a.unwrap());

    let s = String::from("foo");
    let a = cl_format!("The value is: ~a", &s);
    assert_eq!(String::from("The value is: foo"), a.unwrap());

    let l = vec![&1 as &dyn TildeAble, &2, &3];
    let a = cl_format!("The value is:\n ~a", &l);
    assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());

    let l = vec![tilde!(&1), &2, &3];
    let a = cl_format!("The value is:\n ~a", &l);
    assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());

    let a = cl_format!("~@{~a~^, ~}", &1, &2, &3);
    assert_eq!(String::from("1, 2, 3"), a.unwrap());

    let l = vec![tilde!(&1), &2, &3];
    let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
    assert_eq!(String::from("1, 2, and 3"), a.unwrap());

    let l = vec![tilde!(&1), &2, &3, &4];
    let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
    assert_eq!(String::from("1, 2, 3, and 4"), a.unwrap());

    let a = cl_format!("~@{~a~#[~;, and ~:;, ~]~}", &1);
    assert_eq!(String::from("1"), a.unwrap());

    let a = cl_format!("~@{~a~#[~;, and ~:;, ~]~}", &1, &2);
    assert_eq!(String::from("1, and 2"), a.unwrap());

    let a = cl_format!("~@{~a~#[~;, and ~:;, ~]~}", &1, &2, &3, &4);
    assert_eq!(String::from("1, 2, 3, and 4"), a.unwrap());

    let l = vec![];
    let a = cl_format!(
        "~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~}",
        &l
    );
    assert_eq!(String::from(""), a.unwrap());

    let l = vec![];
    let a = cl_format!(
        "~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}",
        &l
    );
    assert_eq!(String::from("empty"), a.unwrap());
}

#[test]
fn play_around_with_hands() {
    let cs = ControlStr::new("~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~}").unwrap();

    let mut list = vec![];
    let args = Args::new(vec![&list]);
    assert_eq!(cs.reveal(args).unwrap(), "".to_string());

    list.push(&1);
    let args = Args::new(vec![&list]);
    assert_eq!(cs.reveal(args).unwrap(), "1".to_string());

    list.push(&2);
    let args = Args::new(vec![&list]);
    assert_eq!(cs.reveal(args).unwrap(), "1 and 2".to_string());

    list.push(&3);
    let args = Args::new(vec![&list]);
    assert_eq!(cs.reveal(args).unwrap(), "1, 2, and 3".to_string());

    list.push(&4);
    let args = Args::new(vec![&list]);
    assert_eq!(cs.reveal(args).unwrap(), "1, 2, 3, and 4".to_string());

    let cs = ControlStr::new(
        "print the list: ~{~#[<empty>~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:};",
    )
    .unwrap();
    let args = Args::new(vec![&list]);
    assert_eq!(
        cs.reveal(args).unwrap(),
        "print the list: 1, 2, 3, and 4;".to_string()
    );

    list.clear();
    let args = Args::new(vec![&list]);
    assert_eq!(
        cs.reveal(args).unwrap(),
        "print the list: <empty>;".to_string()
    );
}

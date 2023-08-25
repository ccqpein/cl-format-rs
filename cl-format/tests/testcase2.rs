use cl_format::*;

#[test]
fn test_radix_0() {
    let my_team = String::from("STeam");
    let my_stars = vec![
        String::from("Adam Lambert"),
        String::from("Queen"),
        String::from("snoop dogg"),
    ];

    let stars = my_stars
        .iter()
        .map(|s| tilde!(s))
        .collect::<Vec<&dyn TildeAble>>();

    assert_eq!(
        String::from("my favorite team \"STeam\" will win the superbowl LVIII. And Adam Lambert, Queen, snoop dogg will in half time show"),
        cl_format!(
            "my favorite team ~S will win the superbowl ~@R. And ~{~a~^, ~} will in half time show",
            &my_team,
            &58,
            &stars
        )
        .unwrap()
    );

    assert_eq!(
        String::from("my favorite team \"STeam\" will win the superbowl LVIII. And Adam Lambert, Queen, and snoop dogg will in half time show"),
        cl_format!(
            "my favorite team ~S will win the superbowl ~@R. And ~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~} will in half time show",
            &my_team,
            &58,
            &stars
        )
        .unwrap()
    );

    assert_eq!(
        String::from("my favorite team \"STeam\" will win the superbowl LVIII. And Adam Lambert, Queen, and snoop dogg will in half time show. And the scores should be 38:35"),
        cl_format!(
            "my favorite team ~S will win the superbowl ~@R. And ~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~} will in half time show. And the scores should be ~d:~d",
            &my_team,
            &58,
            &stars,
			&38,
			&35
        )
        .unwrap()
    );
}

#[test]
fn test_radix_1() {
    assert_eq!(
        cl_format!("this is binary: ~2R", &8).unwrap(),
        "this is binary: 1000"
    );

    assert_eq!(
        cl_format!("this is hex: ~16R", &8).unwrap(),
        "this is hex: 8"
    );

    assert_eq!(cl_format!("~7R", &8).unwrap(), "11");

    assert_eq!(
        cl_format!("~2,19,0,,R", &3333).unwrap(),
        "0000000110100000101"
    );

    assert_eq!(cl_format!("~2,7,0,,R", &5).unwrap(), "0000101");

    // assert_eq!(
    //     cl_format!("this is binary: ~2:R", &8).unwrap(),
    //     cl_format!("this is binary: ~2,,,\,,3:R", &8).unwrap(),
    // )

    assert_eq!(cl_format!("~2,,, ,4:R", &20).unwrap(), "1 0100");
    //assert_eq!(cl_format!("~2,,,a,4:R", &20).unwrap(), "1 0100");
}

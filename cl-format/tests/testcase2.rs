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

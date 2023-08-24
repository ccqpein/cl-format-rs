use crate::tildes::*;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::io::{BufRead, Cursor, Seek, SeekFrom};

/// the control string should including:
/// 1. the whole string
/// 2. the parsed tree
#[derive(Debug, Clone, PartialEq)]
pub struct ControlStr<'a> {
    inner: &'a str,
    tildes: Vec<((usize, usize), Tilde)>,
}

impl<'a> ControlStr<'a> {
    fn new(s: &'a str) -> Result<Self, Box<dyn std::error::Error + 'a>> {
        let cc = Cursor::new(s);
        let tildes = Self::scan(cc)?;

        Ok(Self { inner: s, tildes })
    }

    pub fn from<T>(x: T) -> Result<Self, <T as TryInto<ControlStr<'a>>>::Error>
    where
        T: TryInto<ControlStr<'a>> + 'a + ?Sized,
    {
        x.try_into()
    }

    fn scan(
        mut s: Cursor<&'_ str>,
    ) -> Result<Vec<((usize, usize), Tilde)>, Box<dyn std::error::Error + 'a>> {
        let mut buf = vec![];
        let mut has_read_len = 0;
        let mut result = vec![];

        loop {
            //dbg!(s.position());
            s.read_until(b'~', &mut buf)?;
            match buf.last() {
                // find the next '~'
                Some(b'~') => {
                    has_read_len += buf.len() - 1;
                    s.seek(SeekFrom::Current(-1))?;
                }
                _ => return Ok(result),
            }

            let t = Tilde::parse(&mut s)?;
            let end_index = has_read_len + t.len();

            result.push(((has_read_len, end_index), t));
            has_read_len = end_index;
            buf.clear();
        }
    }

    pub fn reveal<'s>(&self, args: Args<'s>) -> Result<String, TildeError> {
        //dbg!(self);
        let mut start = 0;
        let end = self.inner.len();

        let mut result = String::with_capacity(args.left_count());

        for (r, t) in &self.tildes {
            result.push_str(&self.inner[start..r.0]);
            t.reveal(&args, &mut result)?;
            start = r.1;
        }

        result += &self.inner[start..end];

        Ok(result)
    }
}

impl<'a> TryFrom<&'a str> for ControlStr<'a> {
    type Error = Box<dyn std::error::Error + 'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    //use crate::tildes::*;

    use super::*;

    fn parse_test_result<'x, 's>(
        a: impl Iterator<Item = (&'x (usize, usize), Result<String, TildeError>)>,
    ) -> Result<Vec<Option<String>>, String> {
        let mut x = vec![];
        for (_, aa) in a {
            x.push(Some(aa.map_err(|e| e.to_string())?))
        }
        Ok(x)
    }

    fn reveal_tildes<'x, 'a>(
        cs: &'x ControlStr,
        args: &'x Args<'a>,
    ) -> impl Iterator<Item = (&'x (usize, usize), Result<String, TildeError>)> + 'x {
        cs.tildes.iter().map(|(ind, tilde)| {
            let mut b = String::new();
            tilde.reveal(args, &mut b).unwrap();
            (ind, Ok(b))
        })
    }

    #[test]
    fn test_try_from_self() -> Result<(), Box<dyn std::error::Error>> {
        let case = "hello wor~{~a~}";
        let x = ControlStr::new(case)?;
        let y = x.clone();
        //dbg!(&y);

        assert_eq!(ControlStr::try_from(x)?, y);

        Ok(())
    }

    #[test]
    fn test_control_str_scan() -> Result<(), Box<dyn std::error::Error>> {
        let case = "hello wor~{~a~}";
        let c = Cursor::new(case);

        assert_eq!(
            ControlStr::scan(c)?,
            vec![(
                (9, 15),
                Tilde::new(
                    6,
                    TildeKind::Loop((vec![Tilde::new(2, TildeKind::Va)], TildeLoopKind::Nil))
                )
            )]
        );

        let case = "~{~5$~}";
        let c = Cursor::new(case);

        assert_eq!(
            ControlStr::scan(c)?,
            vec![(
                (0, 7),
                Tilde::new(
                    7,
                    TildeKind::Loop((
                        vec![Tilde::new(3, TildeKind::Float(Some("5".to_string())))],
                        TildeLoopKind::Nil
                    ))
                )
            )]
        );

        Ok(())
    }

    #[test]
    fn test_reveal_normal_tildes() -> Result<(), String> {
        let case = "hello wor~a";
        let cs = ControlStr::new(case).map_err(|e| e.to_string())?;
        let arg: &dyn TildeAble = &13_f32;
        dbg!(arg.into_tildekind_va());

        let result: Vec<Option<String>> = vec!["13".to_string()]
            .into_iter()
            .map(|s| Some(s))
            .collect();

        assert_eq!(
            result,
            parse_test_result(reveal_tildes(&cs, &([arg].into()))).map_err(|e| e.to_string())?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_loop_tildes() -> Result<(), Box<dyn std::error::Error>> {
        let case = "hello wor~{~a~}~a";
        let cs = ControlStr::new(case)?;
        //let arg0: &dyn TildeAble = &13_f32;
        //let arg1: &dyn TildeAble = &14_f32;
        //let arg2: &dyn TildeAble = &15_f32;

        let arg0 = 13_f32;
        let arg1 = 14_f32;
        let arg2 = 15_f32;
        let a: Vec<&dyn TildeAble> = vec![&arg0, &arg1];
        //let arg00 = Args::from(vec![&arg0 as &dyn TildeAble, &arg1]);
        //let arg00 = Args::new(a);
        //let arg: Vec<&dyn TildeAble> = vec![&arg00, &arg2];
        let arg: Vec<&dyn TildeAble> = vec![&a, &arg2];

        let result: Vec<Option<String>> = vec!["1314".to_string(), "15".to_string()]
            .into_iter()
            .map(|s| Some(s))
            .collect();

        assert_eq!(
            result,
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let case = "hello, ~@{~a~^, ~}";
        let cs = ControlStr::new(case)?;
        let arg: Vec<&dyn TildeAble> = vec![&1_i64, &2_i64, &3_i64];
        let result: Vec<Option<String>> = vec!["1, 2, 3".to_string()]
            .into_iter()
            .map(|s| Some(s))
            .collect();
        assert_eq!(
            result,
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let case = "hello, ~{~a~^, ~}";
        let cs = ControlStr::new(case)?;
        dbg!(&cs);
        // let a0: Args = Args::new(vec![
        //     &1_i64 as &dyn TildeAble,
        //     &2_i64 as &dyn TildeAble,
        //     &3_i64 as &dyn TildeAble,
        // ]);
        let a0: Vec<&dyn TildeAble> = vec![
            &1_i64 as &dyn TildeAble,
            &2_i64 as &dyn TildeAble,
            &3_i64 as &dyn TildeAble,
        ];
        let arg: Vec<&dyn TildeAble> = vec![&a0];
        let result: Vec<Option<String>> = vec!["1, 2, 3".to_string()]
            .into_iter()
            .map(|s| Some(s))
            .collect();
        assert_eq!(
            result,
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );
        Ok(())
    }

    #[test]
    fn test_reveal_normal_cond_tildes() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~[cero~;uno~;dos~]";
        let cs = ControlStr::new(case)?;

        //dbg!(&cs);

        let arg: Vec<&dyn TildeAble> = vec![&0_usize];
        assert_eq!(
            vec!["cero".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        //
        let arg: Vec<&dyn TildeAble> = vec![&1_usize];
        assert_eq!(
            vec!["uno".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        //
        let case = "~[cero~;uno~:;dos~]";
        let cs = ControlStr::new(case)?;
        //dbg!(&cs);

        let arg: Vec<&dyn TildeAble> = vec![&0_usize];
        assert_eq!(
            vec!["cero".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let arg: Vec<&dyn TildeAble> = vec![&2_usize];
        assert_eq!(
            vec!["dos".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        //dbg!(&cs);
        let arg: Vec<&dyn TildeAble> = vec![&3_usize];
        assert_eq!(
            vec!["dos".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        //dbg!(&cs);
        let arg: Vec<&dyn TildeAble> = vec![&4_usize];
        assert_eq!(
            vec!["dos".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let arg: Vec<&dyn TildeAble> = vec![&100_usize];
        assert_eq!(
            vec!["dos".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let case = "~#[NONE~;first: ~a~;~a and ~a~:;~a, ~a~]";
        let cs = ControlStr::new(case)?;
        let args: Vec<&dyn TildeAble> = vec![&1_i64];
        //dbg!(t.reveal_args(&mut args));
        assert_eq!(
            vec!["first: 1".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(args.into())))?
        );

        let cs = ControlStr::new(case)?;
        let args: Vec<&dyn TildeAble> = vec![&2_i64, &2_i64];
        //dbg!(t.reveal_args(&mut args));
        assert_eq!(
            vec!["2 and 2".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(args.into())))?
        );

        let cs = ControlStr::new(case)?;
        let args: Vec<&dyn TildeAble> = vec![&3_i64, &3_i64, &3_i64];
        //dbg!(t.reveal_args(&mut args));
        assert_eq!(
            vec!["3, 3".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(args.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_sharp_cond_tildes() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~#[NONE~;~a~;~a and ~a~:;~a, ~a~]~#[~; and ~a~:;, ~a, etc~].";
        let cs = ControlStr::new(case)?;
        dbg!(&cs);

        let arg: Vec<&dyn TildeAble> = vec![];
        assert_eq!(
            vec![Some("NONE".to_string()), Some("".to_string())],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_at_cond_tildes() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~@[x = ~a ~]~@[y = ~a~]";
        let cs = ControlStr::new(case)?;
        dbg!(&cs);

        let arg: Vec<&dyn TildeAble> = vec![&Some(&1_i64 as &dyn TildeAble), &None];
        assert_eq!(
            vec![Some("x = 1 ".to_string()), Some("".to_string())],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let case = "~@[x = ~a ~]~@[y = ~a~]";
        let cs = ControlStr::new(case)?;
        let arg: Vec<&dyn TildeAble> = vec![
            &Some(&1_i64 as &dyn TildeAble),
            &Some(&2_usize as &dyn TildeAble),
        ];
        assert_eq!(
            vec!["x = 1 ".to_string(), "y = 2".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_loop_cond_combine() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~{~a~#[~;, and ~:;, ~]~}";
        let cs = ControlStr::new(case)?;
        //dbg!(&cs);

        let a = vec![&1_i64 as &dyn TildeAble, &2_i64 as &dyn TildeAble];
        //let a = Args::from([&1_i64 as &dyn TildeAble, &2_i64 as &dyn TildeAble]);
        let arg: Vec<&dyn TildeAble> = vec![&a];

        assert_eq!(
            vec![Some("1, and 2".to_string())],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        //

        let case = "~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~}";
        let cs = ControlStr::new(case)?;
        //dbg!(&cs);

        //let a = Args::new(vec![]);
        let a: Vec<&dyn TildeAble> = vec![];
        //let aa = vec![&a as &dyn TildeAble];
        let arg: Vec<&dyn TildeAble> = vec![&a];
        assert_eq!(
            vec![Some("".to_string())],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let cs = ControlStr::new(case)?;
        let a = vec![&1_i64 as &dyn TildeAble];
        //let a = Args::from([&1_i64 as &dyn TildeAble]);
        let arg = vec![&a as &dyn TildeAble];
        assert_eq!(
            vec!["1".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let cs = ControlStr::new(case)?;
        let a = vec![&1_i64 as &dyn TildeAble, &2_i64];
        //let a = Args::from([&1_i64 as &dyn TildeAble, &2_i64]);
        let arg: Vec<&dyn TildeAble> = vec![&a as &dyn TildeAble];
        assert_eq!(
            vec!["1 and 2".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let cs = ControlStr::new(case)?;
        let a = vec![&1_i64 as &dyn TildeAble, &2_i64, &3_i64];
        //let a = Args::from([&1_i64 as &dyn TildeAble, &2_i64, &3_i64]);
        let arg: Vec<&dyn TildeAble> = vec![&a as &dyn TildeAble];
        assert_eq!(
            vec!["1, 2, and 3".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let cs = ControlStr::new(case)?;
        let a = vec![&1_i64 as &dyn TildeAble, &2_i64, &3_i64, &4_i64];
        //let a = Args::from([&1_i64 as &dyn TildeAble, &2_i64, &3_i64, &4_i64]);
        let arg: Vec<&dyn TildeAble> = vec![&a as &dyn TildeAble];
        assert_eq!(
            vec!["1, 2, 3, and 4".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let cs = ControlStr::new(case)?;
        let a = vec![&1_i64 as &dyn TildeAble, &2_i64, &3_i64, &4_i64, &5_i64];
        //let a = Args::from([&1_i64 as &dyn TildeAble, &2_i64, &3_i64, &4_i64, &5_i64]);
        let arg: Vec<&dyn TildeAble> = vec![&a as &dyn TildeAble];
        assert_eq!(
            vec!["1, 2, 3, 4, and 5".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let case = "~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~}";
        let cs = ControlStr::new(case)?;
        let a = vec![];
        //let a = Args::new(vec![]);
        let arg: Vec<&dyn TildeAble> = vec![&a];
        assert_eq!(
            vec![Some("".to_string())],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        let case = "~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}";
        let cs = ControlStr::new(case)?;
        let arg: Vec<&dyn TildeAble> = vec![&a];
        assert_eq!(
            vec!["empty".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_star() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~d ~:*(~d)";
        let cs = ControlStr::new(case)?;
        dbg!(&cs);

        let arg = Args::from([&1_i64 as &dyn TildeAble]);
        assert_eq!(
            vec![
                Some("1".to_string()),
                Some("".to_string()),
                Some("1".to_string())
            ],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        //
        let case = "~{~d~*~^ ~}";
        let cs = ControlStr::new(case)?;
        dbg!(&cs);

        //let a = Args::from([&1_i64 as &dyn TildeAble, &2, &3, &4]);
        let a = vec![&1_i64 as &dyn TildeAble, &2, &3, &4];
        let arg: Vec<&dyn TildeAble> = vec![&a as &dyn TildeAble];
        assert_eq!(
            vec!["1 3".to_string()]
                .into_iter()
                .map(|s| Some(s))
                .collect::<Vec<Option<_>>>(),
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_char() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~c ~C ~@c";
        let cs = ControlStr::new(case)?;
        dbg!(&cs);

        let arg = Args::from([
            &'a' as &dyn TildeAble,
            &'b' as &dyn TildeAble,
            &'c' as &dyn TildeAble,
        ]);
        assert_eq!(
            vec![
                Some("a".to_string()),
                Some("b".to_string()),
                Some("'c'".to_string())
            ],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_standard() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~d ~s";
        let cs = ControlStr::new(case)?;

        let s = String::from("hello");
        let arg = Args::from([&1_i64 as &dyn TildeAble, &s]);

        assert_eq!(
            vec![Some("1".to_string()), Some("\"hello\"".to_string())],
            parse_test_result(reveal_tildes(&cs, &(arg.into())))?
        );

        Ok(())
    }

    #[test]
    fn test_reveal_radix() -> Result<(), Box<dyn std::error::Error>> {
        let cs = ControlStr::new("~R")?;
        assert_eq!(
            "one".to_string(),
            cs.reveal([&1_i32 as &dyn TildeAble].into())?
        );

        assert_eq!(
            "sixty-four".to_string(),
            cs.reveal([&64_i128 as &dyn TildeAble].into())?
        );

        let cs = ControlStr::new("~:R")?;
        assert_eq!(
            "first".to_string(),
            cs.reveal([&1_i32 as &dyn TildeAble].into())?
        );

        assert_eq!(
            "sixty-fourth".to_string(),
            cs.reveal([&64_isize as &dyn TildeAble].into())?
        );

        assert_eq!(
            "negative sixty-fourth".to_string(),
            cs.reveal([&-64_isize as &dyn TildeAble].into())?
        );

        //
        let cs = ControlStr::new("~3,,, ,2:R")?;
        assert_eq!(
            "1 22".to_string(),
            cs.reveal([&17_usize as &dyn TildeAble].into())?
        );

        assert_eq!(
            "000001101 0000 0101".to_string(),
            ControlStr::new("~2,19,0, ,4:R")?.reveal([&3333_usize as &dyn TildeAble].into())?
        );

        assert_eq!(
            "1101 0000 0101".to_string(),
            ControlStr::new("~2,8,0, ,4:R")?.reveal([&3333_usize as &dyn TildeAble].into())?
        );

        assert_eq!(
            "6|55|35".to_string(),
            ControlStr::new("~10,,,|,2:R")?.reveal([&65535_usize as &dyn TildeAble].into())?
        );

        //:= Next: more tests here
        Ok(())
    }

    #[test]
    fn test_reveal() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~a, ~a, ~a";
        let cs = ControlStr::from(case)?;
        assert_eq!(
            "1, 2, 3".to_string(),
            cs.reveal([&1_i32 as &dyn TildeAble, &2, &3].into())?
        );

        assert_eq!(
            "4, 5, 6".to_string(),
            cs.reveal([&4_i32 as &dyn TildeAble, &5, &6].into())?
        );
        Ok(())
    }
}

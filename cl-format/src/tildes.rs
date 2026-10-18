use crate::*;

use std::fmt::Debug;
use std::io::Cursor;
use std::ops::Deref;

mod tilde_kinds;
pub use tilde_kinds::*;

mod args;
pub use args::*;

mod tilde_able_impl;

#[doc = r"The tilde struct"]
#[derive(Debug, PartialEq, Clone)]
pub struct Tilde<'a> {
    len: usize,
    value: TildeKind<'a>,
}

impl<'a> Tilde<'a> {
    pub fn new(len: usize, value: TildeKind<'a>) -> Self {
        Self { len, value }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn reveal<C: ArgCursor>(
        &self,
        args: &C,
        buf: &mut String,
    ) -> Result<std::ops::ControlFlow<()>> {
        self.value.match_reveal(args, buf)
    }

    /*
    ===============================
    parse functions below
    ===============================
    */

    /// start from '~' to the key char of tilde kind
    fn scan_for_kind(
        c: &mut Cursor<&'a str>,
    ) -> Result<fn(&mut Cursor<&'a str>) -> Result<Tilde<'a>>> {
        let pos = c.position() as usize;
        let bytes = &c.get_ref().as_bytes()[pos..];
        if bytes.first() != Some(&b'~') {
            return Err(TildeError::new(ErrorKind::ParseError, "should start with ~").into());
        }

        let mut offset = 1;
        let mut escape = false;

        while offset < bytes.len() {
            let b = bytes[offset];
            if b == 0 {
                break;
            }
            offset += 1;
            if !escape {
                match &bytes[..offset] {
                    [.., b'\\'] => {
                        // escape the next one
                        escape = true;
                        continue;
                    }
                    [.., b'a'] | [.., b'A'] => return Ok(Self::parse_value),
                    [.., b'@', b'{'] | [.., b'{'] => return Ok(Self::parse_loop),
                    [.., _, b'$'] | [.., b'$'] | [.., b'f' | b'F'] => return Ok(Self::parse_float),
                    [.., b'd'] | [.., b'D'] => return Ok(Self::parse_digit),
                    [.., b'#' | b':' | b'@', b'['] | [.., b'['] => return Ok(Self::parse_cond),
                    [.., b'^'] => return Ok(Self::parse_loop_end),
                    [.., b':', b'*'] | [.., b'*'] => return Ok(Self::parse_star),
                    [.., _, b'~'] => return Ok(Self::parse_tildes),
                    [.., b'S'] | [.., b's'] => return Ok(Self::parse_standard),
                    [.., b'@', b'c' | b'C'] | [.., b'C' | b'c'] => return Ok(Self::parse_char),
                    [.., b'R' | b'r'] => return Ok(Self::parse_radix),
                    [.., b'%'] => {
                        return Err(TildeError::new(
                            ErrorKind::ParseError,
                            "Unsupport the last symbol of '%', try to use \\n instead",
                        )
                        .into());
                    }
                    _ => {}
                }
            }
            escape = false;
        }
        Err(TildeError::new(ErrorKind::ParseError, "read to end").into())
    }

    /// cursor should located on '~'
    pub fn parse(c: &mut Cursor<&'a str>) -> Result<Self> {
        let parser = Self::scan_for_kind(c)?;
        parser(c)
    }

    /// parse function for '~{~}'
    fn parse_loop(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let s_ref = *c.get_ref();
        let bytes = s_ref.as_bytes();

        let mut loop_kind = TildeLoopKind::Nil;
        let mut total_len = 0;
        match &bytes[pos..] {
            [b'~', b'{', ..] => {
                total_len += 2;
                c.set_position((pos + 2) as u64);
            }
            [b'~', b'@', b'{', ..] => {
                total_len += 3;
                loop_kind = TildeLoopKind::At;
                c.set_position((pos + 3) as u64);
            }
            _ => {
                return Err(TildeError::new(ErrorKind::ParseError, "should start with ~{").into());
            }
        }

        let mut result = vec![];

        loop {
            let pos = c.position() as usize;
            match bytes[pos..].iter().position(|&b| b == b'~') {
                Some(0) => {}
                Some(rel) => {
                    result.push(Tilde::new(rel, TildeKind::Text(&s_ref[pos..pos + rel])));
                    total_len += rel;
                    c.set_position((pos + rel) as u64);
                }
                None => {
                    let rem = bytes.len() - pos;
                    let text_len = rem.saturating_sub(1);
                    result.push(Tilde::new(
                        text_len,
                        TildeKind::Text(&s_ref[pos..pos + text_len]),
                    ));
                    total_len += rem;
                    c.set_position(bytes.len() as u64);
                    return Ok(Tilde::new(total_len, TildeKind::Loop((result, loop_kind))));
                }
            }

            let pos = c.position() as usize;
            match &bytes[pos..] {
                [b'~', b'}', ..] => {
                    c.set_position((pos + 2) as u64);
                    return Ok(Tilde::new(
                        total_len + 2,
                        TildeKind::Loop((result, loop_kind)),
                    ));
                }
                [b'~', b':', b'}', ..] => {
                    c.set_position((pos + 3) as u64);
                    return Ok(Tilde::new(
                        total_len + 3,
                        TildeKind::Loop((
                            result,
                            if loop_kind == TildeLoopKind::Nil {
                                TildeLoopKind::NilColon
                            } else {
                                loop_kind
                            },
                        )),
                    ));
                }
                _ => {}
            }

            // read the tilde
            let next = Tilde::parse(c)?;
            total_len += next.len;
            result.push(next);
        }
    }

    /// parse the ~^ in loop
    fn parse_loop_end(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let bytes = c.get_ref().as_bytes();
        if bytes.get(pos..pos + 2) != Some(b"~^") {
            return Err(TildeError::new(ErrorKind::ParseError, "should start with ~^").into());
        }
        c.set_position((pos + 2) as u64);

        Ok(Tilde {
            len: 2,
            value: TildeKind::LoopEnd,
        })
    }

    /// parse the '~[~]'
    fn parse_cond(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let s_ref = *c.get_ref();
        let bytes = s_ref.as_bytes();
        let mut total_len = 0;
        let mut cond_kind;

        match &bytes[pos..] {
            [b'~', b'[', ..] => {
                total_len += 2;
                cond_kind = TildeCondKind::Nil(false);
                c.set_position((pos + 2) as u64);
            }
            [b'~', b'#', b'[', ..] => {
                total_len += 3;
                cond_kind = TildeCondKind::Sharp;
                c.set_position((pos + 3) as u64);
            }
            [b'~', b'@', b'[', ..] => {
                total_len += 3;
                cond_kind = TildeCondKind::At;
                c.set_position((pos + 3) as u64);
            }
            [b'~', b':', b'[', ..] => {
                total_len += 3;
                cond_kind = TildeCondKind::Colon;
                c.set_position((pos + 3) as u64);
            }
            _ => {
                return Err(TildeError::new(
                    ErrorKind::ParseError,
                    "should start with ~[, ~#[, ~@[",
                )
                .into());
            }
        }

        let mut cache = vec![];
        let mut result: Vec<Tilde<'a>> = vec![];

        loop {
            let pos = c.position() as usize;
            if pos >= bytes.len() {
                break;
            }
            match bytes[pos..].iter().position(|&b| b == b'~') {
                Some(rel) => {
                    if rel > 0 {
                        cache.push(Tilde {
                            len: rel,
                            value: TildeKind::Text(&s_ref[pos..pos + rel]),
                        });
                    }
                    let tilde_pos = pos + rel;
                    match bytes.get(tilde_pos + 1) {
                        Some(&b':') => {
                            if bytes.get(tilde_pos + 2) == Some(&b';') {
                                let cache_len =
                                    cache.iter().map(|t: &Tilde| t.len()).sum::<usize>();
                                result.push(Tilde::new(
                                    cache_len,
                                    TildeKind::VecTilde(std::mem::take(&mut cache)),
                                ));
                                cond_kind.to_true();
                                total_len += 3 + cache_len;
                                c.set_position((tilde_pos + 3) as u64);
                            } else {
                                panic!()
                            }
                        }
                        Some(&b';') => {
                            let cache_len = cache.iter().map(|t: &Tilde| t.len()).sum::<usize>();
                            result.push(Tilde::new(
                                cache_len,
                                TildeKind::VecTilde(std::mem::take(&mut cache)),
                            ));
                            total_len += 2 + cache_len;
                            c.set_position((tilde_pos + 2) as u64);
                        }
                        Some(&b']') => {
                            let cache_len = cache.iter().map(|t: &Tilde| t.len()).sum::<usize>();
                            result.push(Tilde::new(
                                cache_len,
                                TildeKind::VecTilde(std::mem::take(&mut cache)),
                            ));
                            total_len += 2 + cache_len;
                            c.set_position((tilde_pos + 2) as u64);
                            break;
                        }
                        Some(_) => {
                            c.set_position(tilde_pos as u64);
                            let next = Self::parse(c)?;
                            cache.push(next);
                        }
                        None => {
                            return Err(
                                TildeError::new(ErrorKind::ParseError, "read to end").into()
                            );
                        }
                    }
                }
                None => {
                    return Err(TildeError::new(ErrorKind::ParseError, "read to end").into());
                }
            }
        }

        Ok(Tilde::new(total_len, TildeKind::Cond((result, cond_kind))))
    }

    #[cfg(test)]
    fn parse_vec(c: &mut Cursor<&'a str>, end_len: usize) -> Result<Self> {
        let pos = c.position() as usize;
        let s_ref = *c.get_ref();
        if pos + end_len > s_ref.len() {
            return Err(
                TildeError::new(ErrorKind::ParseError, "failed to fill whole buffer").into(),
            );
        }
        let ss = &s_ref[pos..pos + end_len];
        let mut inner_c = Cursor::new(ss);
        let bytes = ss.as_bytes();
        let mut result = vec![];
        let mut total_len = 0;

        loop {
            let ipos = inner_c.position() as usize;
            if ipos >= bytes.len() {
                c.set_position((pos + total_len) as u64);
                return Ok(Tilde::new(total_len, TildeKind::VecTilde(result)));
            }
            match bytes[ipos..].iter().position(|&b| b == b'~') {
                Some(0) => {}
                Some(rel) => {
                    result.push(Tilde::new(rel, TildeKind::Text(&ss[ipos..ipos + rel])));
                    total_len += rel;
                    inner_c.set_position((ipos + rel) as u64);
                }
                None => {
                    let rem = bytes.len() - ipos;
                    result.push(Tilde::new(rem, TildeKind::Text(&ss[ipos..])));
                    total_len += rem;
                    c.set_position((pos + total_len) as u64);
                    return Ok(Tilde::new(total_len, TildeKind::VecTilde(result)));
                }
            }

            let next = Tilde::parse(&mut inner_c)?;
            total_len += next.len;
            result.push(next);
        }
    }

    /// parse function for '~a'
    fn parse_value(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let bytes = &c.get_ref().as_bytes()[pos..];
        if let Some(idx) = bytes.iter().position(|&b| b == b'a' || b == b'A') {
            let len = idx + 1;
            c.set_position((pos + len) as u64);
            return Ok(Tilde::new(len, TildeKind::Va));
        }
        Err(TildeError::new(ErrorKind::ParseError, "should start with ~a").into())
    }

    /// parse the float
    fn parse_float(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let s_ref = *c.get_ref();
        let bytes = &s_ref.as_bytes()[pos..];
        if let Some(idx) = bytes
            .iter()
            .position(|&b| b == b'$' || b == b'f' || b == b'F')
        {
            let len = idx + 1;
            c.set_position((pos + len) as u64);
            let param = if len > 2 {
                &s_ref[pos + 1..pos + idx]
            } else {
                ""
            };
            return Ok(Tilde::new(len, TildeKind::Float(Some(param))));
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the '$' or 'f'").into())
    }

    /// parse the digit
    fn parse_digit(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let s_ref = *c.get_ref();
        let bytes = &s_ref.as_bytes()[pos..];
        if let Some(idx) = bytes.iter().position(|&b| b == b'd' || b == b'D') {
            let len = idx + 1;
            c.set_position((pos + len) as u64);
            let param = if len > 2 {
                Some(&s_ref[pos + 1..pos + idx])
            } else {
                None
            };
            return Ok(Tilde::new(len, TildeKind::Digit(param)));
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the 'd' or 'D'").into())
    }

    /// parse the star
    fn parse_star(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let bytes = c.get_ref().as_bytes();
        match &bytes[pos..] {
            [b'~', b':', b'*', ..] => {
                c.set_position((pos + 3) as u64);
                Ok(Self {
                    len: 3,
                    value: TildeKind::Star(StarKind::Hop),
                })
            }
            [b'~', b'*', ..] => {
                c.set_position((pos + 2) as u64);
                Ok(Self {
                    len: 2,
                    value: TildeKind::Star(StarKind::Skip),
                })
            }
            _ => Err(TildeError::new(ErrorKind::ParseError, "should start with ~* or ~:*").into()),
        }
    }

    fn parse_tildes(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let bytes = c.get_ref().as_bytes();
        match &bytes[pos..] {
            [b'~', n @ b'0'..=b'9', b'~', ..] => {
                c.set_position((pos + 3) as u64);
                Ok(Self {
                    len: 3,
                    value: TildeKind::Tildes((*n - b'0') as usize),
                })
            }
            _ => Err(TildeError::new(ErrorKind::ParseError, "should start with ~n~").into()),
        }
    }

    fn parse_standard(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let bytes = &c.get_ref().as_bytes()[pos..];
        if let Some(idx) = bytes.iter().position(|&b| b == b's' || b == b'S') {
            let len = idx + 1;
            c.set_position((pos + len) as u64);
            return Ok(Tilde::new(len, TildeKind::Standard));
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the 's' or 'S'").into())
    }

    fn parse_char(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let bytes = c.get_ref().as_bytes();
        match &bytes[pos..] {
            [b'~', b'@', b'c' | b'C', ..] => {
                c.set_position((pos + 3) as u64);
                Ok(Self {
                    len: 3,
                    value: TildeKind::Char(CharKind::At),
                })
            }
            [b'~', b'c' | b'C', ..] => {
                c.set_position((pos + 2) as u64);
                Ok(Self {
                    len: 2,
                    value: TildeKind::Char(CharKind::Nil),
                })
            }
            _ => Err(TildeError::new(ErrorKind::ParseError, "should start with ~c or ~@c").into()),
        }
    }

    fn parse_radix(c: &mut Cursor<&'a str>) -> Result<Self> {
        let pos = c.position() as usize;
        let s_ref = *c.get_ref();
        let bytes = &s_ref.as_bytes()[pos..];
        let idx = bytes
            .iter()
            .position(|&b| b == b'R' || b == b'r')
            .ok_or_else(|| TildeError::new(ErrorKind::ParseError, "read to end"))?;
        let total_len = idx + 1;
        c.set_position((pos + total_len) as u64);

        let whole = &s_ref[pos + 1..pos + total_len];

        let parse_usize_helper = |a: &str| -> Result<Option<usize>> {
            if a.is_empty() {
                Ok(None)
            } else {
                Ok(Some(a.parse::<usize>().map_err(|e| {
                    TildeError::new(ErrorKind::ParseError, e.to_string())
                })?))
            }
        };

        let parse_u8_helper = |a: &str| -> Result<Option<u8>> {
            if a.is_empty() {
                Ok(None)
            } else {
                Ok(Some(a.parse::<u8>().map_err(|e| {
                    TildeError::new(ErrorKind::ParseError, e.to_string())
                })?))
            }
        };

        let parse_char_helper = |a: &str| -> Result<Option<char>> {
            if a.is_empty() {
                Ok(None)
            } else {
                Ok(a.chars().last())
            }
        };

        let parse_radix_flag = |a: &str| -> Result<(Option<usize>, Option<RadixFlag>)> {
            match a.split_once(':') {
                Some(("", "R" | "r")) => Ok((None, Some(RadixFlag::Colon))),
                Some(("", "@R" | "@r")) => Ok((None, Some(RadixFlag::AtColon))),
                Some((n, "R" | "r")) => Ok((parse_usize_helper(n)?, Some(RadixFlag::Colon))),
                None if a == "@R" || a == "@r" => Ok((None, Some(RadixFlag::At))),
                None if a == "R" || a == "r" => Ok((None, None)),
                None if a.ends_with('R') || a.ends_with('r') => {
                    Ok((parse_usize_helper(&a[..a.len() - 1])?, None))
                }
                _ => Err(TildeError::new(
                    ErrorKind::ParseError,
                    format!("Parsing the radix flag has problem: {}", a),
                )
                .into()),
            }
        };

        let flag;
        let radix;
        let mut mincol = None;
        let mut padchar = None;
        let mut commachar = None;
        let mut comma_interval = None;

        let mut parts = whole.split(',');
        let p0 = parts.next().unwrap_or("");
        match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (None, None, None, None, None) => {
                let pair = parse_radix_flag(p0)?;
                (radix, flag) = (pair.0.map(|x| x as u8), pair.1);
            }
            (Some(p1), Some(p2), Some(p3), Some(p4), None) => {
                radix = parse_u8_helper(p0)?;
                mincol = parse_usize_helper(p1)?;
                padchar = parse_char_helper(p2)?;
                commachar = parse_char_helper(p3)?;
                (comma_interval, flag) = parse_radix_flag(p4)?;
            }
            _ => {
                return Err(TildeError::new(
                    ErrorKind::ParseError,
                    "The number of radix interval spaces should be 1 or 5",
                )
                .into());
            }
        }

        Ok(Self::new(
            total_len,
            TildeKind::Radix((radix, mincol, padchar, commachar, comma_interval, flag)),
        ))
    }

    // more parsers functions below
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_cursor() {
        let mut testcase0 = Cursor::new("abcd");
        assert_eq!(testcase0.position(), 0);

        let mut buffer: [u8; 1] = [0; 1];
        testcase0.set_position(2);
        let _ = testcase0.read(&mut buffer);
        assert_eq!(buffer[0], b'c');
    }

    #[test]
    fn test_parse_va() -> Result<()> {
        let mut case = Cursor::new("~a");
        assert_eq!(Tilde::parse_value(&mut case)?, Tilde::new(2, TildeKind::Va));

        let mut case = Cursor::new("~A");
        assert_eq!(Tilde::parse_value(&mut case)?, Tilde::new(2, TildeKind::Va));
        Ok(())
    }

    #[test]
    fn test_parse_loop() -> Result<()> {
        let mut case = Cursor::new("~{~}");

        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(4, TildeKind::Loop((Vec::new(), TildeLoopKind::Nil)))
        );

        let mut case = Cursor::new("~{a bc~}");

        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(
                8,
                TildeKind::Loop((
                    vec![Tilde {
                        len: 4,
                        value: TildeKind::Text("a bc")
                    }],
                    TildeLoopKind::Nil
                ))
            ),
        );

        let mut case = Cursor::new("~{a bc~a~}");

        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(
                10,
                TildeKind::Loop((
                    vec![
                        Tilde {
                            len: 4,
                            value: TildeKind::Text("a bc")
                        },
                        Tilde {
                            len: 2,
                            value: TildeKind::Va,
                        }
                    ],
                    TildeLoopKind::Nil
                ))
            )
        );

        let mut case = Cursor::new("~{~aa bc~a~}");

        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(
                12,
                TildeKind::Loop((
                    vec![
                        Tilde {
                            len: 2,
                            value: TildeKind::Va,
                        },
                        Tilde {
                            len: 4,
                            value: TildeKind::Text("a bc")
                        },
                        Tilde {
                            len: 2,
                            value: TildeKind::Va,
                        }
                    ],
                    TildeLoopKind::Nil
                ))
            )
        );

        let mut case = Cursor::new("~@{~a~}");

        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(
                7,
                TildeKind::Loop((
                    vec![Tilde {
                        len: 2,
                        value: TildeKind::Va,
                    },],
                    TildeLoopKind::At
                ))
            )
        );

        let mut case = Cursor::new("~@{~a~^, ~}");

        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(
                11,
                TildeKind::Loop((
                    vec![
                        Tilde {
                            len: 2,
                            value: TildeKind::Va,
                        },
                        Tilde {
                            len: 2,
                            value: TildeKind::LoopEnd,
                        },
                        Tilde {
                            len: 2,
                            value: TildeKind::Text(", "),
                        }
                    ],
                    TildeLoopKind::At
                ))
            )
        );

        let mut case = Cursor::new("~{~a~:}");
        assert_eq!(
            Tilde::parse_loop(&mut case)?,
            Tilde::new(
                7,
                TildeKind::Loop((
                    vec![Tilde {
                        len: 2,
                        value: TildeKind::Va,
                    },],
                    TildeLoopKind::NilColon
                ))
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_float() -> Result<()> {
        let mut case = Cursor::new("~$");
        assert_eq!(
            Tilde::parse_float(&mut case)?,
            Tilde::new(2, TildeKind::Float(Some("")))
        );

        let mut case = Cursor::new("~5$");
        assert_eq!(
            Tilde::parse_float(&mut case)?,
            Tilde::new(3, TildeKind::Float(Some("5")))
        );

        let mut case = Cursor::new("~,5f");
        assert_eq!(
            Tilde::parse_float(&mut case)?,
            Tilde::new(4, TildeKind::Float(Some(",5")))
        );

        Ok(())
    }

    #[test]
    fn test_scan_for_kind() -> Result<()> {
        let case = "~a";
        let mut c = Cursor::new(case);
        let f = Tilde::scan_for_kind(&mut c)?;

        assert_eq!(Tilde::new(2, TildeKind::Va), f(&mut c)?);
        Ok(())
    }

    #[test]
    fn test_parse_vec() -> Result<()> {
        let mut case = Cursor::new("~a and ~a~a~a");
        assert_eq!(
            Tilde::parse_vec(&mut case, 9)?,
            Tilde::new(
                9,
                TildeKind::VecTilde(vec![
                    Tilde {
                        len: 2,
                        value: TildeKind::Va
                    },
                    Tilde {
                        len: 5,
                        value: TildeKind::Text(" and ")
                    },
                    Tilde {
                        len: 2,
                        value: TildeKind::Va
                    }
                ])
            )
        );

        let mut rest = vec![];
        case.read_to_end(&mut rest)?;
        assert_eq!(String::from_utf8(rest)?, "~a~a");

        //
        let mut case = Cursor::new("~a a");
        assert!(Tilde::parse_vec(&mut case, 9).is_err());

        //
        let mut case = Cursor::new("a");
        assert_eq!(
            Tilde::parse_vec(&mut case, 1)?,
            Tilde::new(
                1,
                TildeKind::VecTilde(vec![Tilde {
                    len: 1,
                    value: TildeKind::Text("a")
                }])
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_cond() -> Result<()> {
        let mut case = Cursor::new("~[~]");

        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                4,
                TildeKind::Cond((
                    vec![Tilde {
                        len: 0,
                        value: TildeKind::VecTilde(vec![])
                    }],
                    TildeCondKind::Nil(false)
                ))
            )
        );

        let mut case = Cursor::new("~[cero~]");

        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                8,
                TildeKind::Cond((
                    vec![Tilde {
                        len: 4,
                        value: TildeKind::VecTilde(vec![Tilde::new(4, TildeKind::Text("cero"))]),
                    }],
                    TildeCondKind::Nil(false)
                ))
            ),
        );

        let mut case = Cursor::new("~[cero~;uno~;dos~]");

        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                18,
                TildeKind::Cond((
                    vec![
                        Tilde {
                            len: 4,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                4,
                                TildeKind::Text("cero")
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(3, TildeKind::Text("uno"))])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(3, TildeKind::Text("dos"))])
                        },
                    ],
                    TildeCondKind::Nil(false)
                ))
            )
        );

        let mut case = Cursor::new("~[cero~;uno~;~]");

        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                15,
                TildeKind::Cond((
                    vec![
                        Tilde {
                            len: 4,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                4,
                                TildeKind::Text("cero")
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(3, TildeKind::Text("uno"))])
                        },
                        Tilde {
                            len: 0,
                            value: TildeKind::VecTilde(vec![])
                        },
                    ],
                    TildeCondKind::Nil(false)
                ))
            )
        );

        let mut case = Cursor::new("~:[cero~;uno~]");
        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                14,
                TildeKind::Cond((
                    vec![
                        Tilde {
                            len: 4,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                4,
                                TildeKind::Text("cero")
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(3, TildeKind::Text("uno"))])
                        },
                    ],
                    TildeCondKind::Colon
                ))
            )
        );

        let mut case = Cursor::new("~[cero~;uno~:;dos~]");

        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                19,
                TildeKind::Cond((
                    vec![
                        Tilde {
                            len: 4,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                4,
                                TildeKind::Text("cero")
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(3, TildeKind::Text("uno"))])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(3, TildeKind::Text("dos"))])
                        },
                    ],
                    TildeCondKind::Nil(true)
                ))
            )
        );

        let mut case = Cursor::new("~#[NONE~;~a~;~a and ~a~:;~a, ~a~]");

        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                33,
                TildeKind::Cond((
                    vec![
                        Tilde {
                            len: 4,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                4,
                                TildeKind::Text("NONE")
                            )])
                        },
                        Tilde {
                            len: 2,
                            value: TildeKind::VecTilde(vec![Tilde::new(2, TildeKind::Va)])
                        },
                        Tilde {
                            len: 9,
                            value: TildeKind::VecTilde(vec![
                                Tilde {
                                    len: 2,
                                    value: TildeKind::Va
                                },
                                Tilde {
                                    len: 5,
                                    value: TildeKind::Text(" and ")
                                },
                                Tilde {
                                    len: 2,
                                    value: TildeKind::Va,
                                }
                            ]),
                        },
                        Tilde {
                            len: 6,
                            value: TildeKind::VecTilde(vec![
                                Tilde {
                                    len: 2,
                                    value: TildeKind::Va
                                },
                                Tilde {
                                    len: 2,
                                    value: TildeKind::Text(", ")
                                },
                                Tilde {
                                    len: 2,
                                    value: TildeKind::Va,
                                }
                            ]),
                        },
                    ],
                    TildeCondKind::Sharp
                ))
            )
        );

        let mut case = Cursor::new("~@[x = ~a ~]~@[y = ~a~]");
        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                12,
                TildeKind::Cond((
                    vec![Tilde {
                        len: 7,
                        value: TildeKind::VecTilde(vec![
                            Tilde {
                                len: 4,
                                value: TildeKind::Text("x = ")
                            },
                            Tilde {
                                len: 2,
                                value: TildeKind::Va
                            },
                            Tilde {
                                len: 1,
                                value: TildeKind::Text(" ")
                            }
                        ])
                    }],
                    TildeCondKind::At
                ))
            )
        );

        // parse the second part
        assert_eq!(
            Tilde::parse_cond(&mut case)?,
            Tilde::new(
                11,
                TildeKind::Cond((
                    vec![Tilde {
                        len: 6,
                        value: TildeKind::VecTilde(vec![
                            Tilde {
                                len: 4,
                                value: TildeKind::Text("y = ")
                            },
                            Tilde {
                                len: 2,
                                value: TildeKind::Va
                            },
                        ])
                    }],
                    TildeCondKind::At
                ))
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_star() -> Result<()> {
        let mut case = Cursor::new("~:*");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(3, TildeKind::Star(StarKind::Hop))
        );

        let mut case = Cursor::new("~*");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(2, TildeKind::Star(StarKind::Skip))
        );
        Ok(())
    }

    #[test]
    fn test_parse_tildes() -> Result<()> {
        let mut case = Cursor::new("~9~");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(3, TildeKind::Tildes(9))
        );

        let mut case = Cursor::new("~0~");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(3, TildeKind::Tildes(0))
        );
        Ok(())
    }

    #[test]
    fn test_parse_standard() -> Result<()> {
        let mut case = Cursor::new("~s");
        assert_eq!(Tilde::parse(&mut case)?, Tilde::new(2, TildeKind::Standard));

        let mut case = Cursor::new("~S");
        assert_eq!(Tilde::parse(&mut case)?, Tilde::new(2, TildeKind::Standard));

        let mut case = Cursor::new("~S superbowl");
        assert_eq!(Tilde::parse(&mut case)?, Tilde::new(2, TildeKind::Standard));

        Ok(())
    }

    #[test]
    fn test_parse_char() -> Result<()> {
        let mut case = Cursor::new("~c");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(2, TildeKind::Char(CharKind::Nil))
        );

        let mut case = Cursor::new("~@c");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(3, TildeKind::Char(CharKind::At))
        );

        Ok(())
    }

    #[test]
    fn test_parse_radix() -> Result<()> {
        //let mut case = Cursor::new("~2,8,'0,' ,R");
        let mut case = Cursor::new("~2,8,0, ,R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                10,
                TildeKind::Radix((Some(2), Some(8), Some('0'), Some(' '), None, None))
            )
        );

        let mut case = Cursor::new("~,,,,R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(6, TildeKind::Radix((None, None, None, None, None, None)))
        );

        let mut case = Cursor::new("~2,8,0, ,4:R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                12,
                TildeKind::Radix((
                    Some(2),
                    Some(8),
                    Some('0'),
                    Some(' '),
                    Some(4),
                    Some(RadixFlag::Colon)
                ))
            )
        );

        let mut case = Cursor::new("~2,8,0, ,10:R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                13,
                TildeKind::Radix((
                    Some(2),
                    Some(8),
                    Some('0'),
                    Some(' '),
                    Some(10),
                    Some(RadixFlag::Colon)
                ))
            )
        );

        let mut case = Cursor::new("~3,,, ,2:R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                10,
                TildeKind::Radix((
                    Some(3),
                    None,
                    None,
                    Some(' '),
                    Some(2),
                    Some(RadixFlag::Colon)
                ))
            )
        );

        let mut case = Cursor::new("~2:R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                4,
                TildeKind::Radix((Some(2), None, None, None, None, Some(RadixFlag::Colon)))
            )
        );

        let mut case = Cursor::new("~2R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(3, TildeKind::Radix((Some(2), None, None, None, None, None)))
        );

        let mut case = Cursor::new("~R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(2, TildeKind::Radix((None, None, None, None, None, None)))
        );

        let mut case = Cursor::new("~:R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                3,
                TildeKind::Radix((None, None, None, None, None, Some(RadixFlag::Colon)))
            )
        );

        let mut case = Cursor::new("~@R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                3,
                TildeKind::Radix((None, None, None, None, None, Some(RadixFlag::At)))
            )
        );

        let mut case = Cursor::new("~:@R");
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                4,
                TildeKind::Radix((None, None, None, None, None, Some(RadixFlag::AtColon)))
            )
        );

        let mut case = Cursor::new(r#"~2,,,\a,4:R"#);
        assert_eq!(
            Tilde::parse(&mut case)?,
            Tilde::new(
                11,
                TildeKind::Radix((
                    Some(2),
                    None,
                    None,
                    Some('a'),
                    Some(4),
                    Some(RadixFlag::Colon)
                ))
            )
        );

        Ok(())
    }
}

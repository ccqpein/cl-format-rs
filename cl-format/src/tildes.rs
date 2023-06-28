use crate::*;

use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};
use std::ops::Deref;

mod tilde_kinds;
pub use tilde_kinds::*;

mod reveal_impl;
pub use reveal_impl::*;

mod args;
pub use args::*;

mod tilde_able_impl;
pub use tilde_able_impl::*;

/// The tilde struct
#[derive(Debug, PartialEq, Clone)]
pub struct Tilde {
    len: usize,
    value: TildeKind,
}

impl Tilde {
    pub fn new(len: usize, value: TildeKind) -> Self {
        Self { len, value }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reveal(&self, arg: &dyn TildeAble, mut buf: &mut String) -> Result<(), TildeError> {
        self.value.match_reveal(arg, &mut buf)?;
        Ok(())
    }

    /*
    ===============================
    parse functions below
    ===============================
    */

    /// start from '~' to the key char of tilde kind
    fn scan_for_kind(
        c: &mut Cursor<&'_ str>,
    ) -> Result<
        Box<dyn for<'a, 'b> Fn(&'a mut std::io::Cursor<&'b str>) -> Result<Tilde, TildeError>>,
        TildeError,
    > {
        let mut buf = [0u8; 1];
        c.read(&mut buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
        if buf[0] != b'~' {
            return Err(TildeError::new(ErrorKind::ParseError, "should start with ~").into());
        }

        // read until the tilde key char
        let mut buf = [0_u8; 3];
        let mut buf_offset = 1;
        c.read(&mut buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
        for b in buf {
            if b == 0_u8 {
                break;
            } else {
                buf_offset += 1;
            }
        }

        //dbg!(&c);
        //dbg!(String::from_utf8(buf.to_vec()));

        match buf {
            [b'a', ..] | [b'A', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_value),
                );
            }
            [b'{', ..] | [b'@', b'{', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_loop),
                );
            }
            [b'$', ..] | [b'f' | b'F', ..] | [_, b'$', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_float),
                );
            }
            [b'd', ..] | [b'D', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_digit),
                );
            }
            [b'[', ..] | [b'#' | b':' | b'@', b'[', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_cond),
                );
            }
            [b'^', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_loop_end),
                );
            }
            [b':', b'*', ..] | [b'*', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_star),
                );
            }
            [_, b'~', ..] => {
                //:= can only parse the one digit number
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_tildes),
                );
            }
            [b'S', ..] | [b's', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_standard),
                );
            }
            [b'C', ..] | [b'c', ..] | [b'@', b'c' | b'C', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_char),
                );
            }
            _ => {
                return Err(
                    TildeError::new(ErrorKind::ParseError, "cannot find the key tilde").into(),
                )
            }
        }
    }

    /// cursor should located on '~'
    pub fn parse(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        //dbg!(&c);
        let parser = Self::scan_for_kind(c)?;
        parser(c)
    }

    /// parse function for '~{~}'
    fn parse_loop(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;

        let mut loop_kind = TildeLoopKind::Nil;
        let mut total_len = 0;
        match char_buf {
            [b'~', b'{', ..] => {
                total_len += 2;
                c.seek(SeekFrom::Current(-1))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            }
            [b'~', b'@', b'{'] => {
                total_len += 3;
                loop_kind = TildeLoopKind::At;
            }

            _ => {
                c.seek(SeekFrom::Current(-3))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                return Err(TildeError::new(ErrorKind::ParseError, "should start with ~{").into());
            }
        }

        let mut result = vec![];
        let mut buf = vec![];
        let mut char_buf = [0u8; 3];

        loop {
            // read text until the next '~'
            c.read_until(b'~', &mut buf)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;

            match buf.as_slice() {
                [b'~'] => {
                    c.seek(SeekFrom::Current(-1))
                        .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                }
                [.., b'~'] => {
                    c.seek(SeekFrom::Current(-1))
                        .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                    result.push(Tilde::new(
                        buf.len() - 1,
                        TildeKind::Text(
                            String::from_utf8(buf[..buf.len() - 1].to_vec()).map_err(|e| {
                                TildeError::new(ErrorKind::ParseError, e.to_string())
                            })?,
                        ),
                    ));
                    total_len += buf.len() - 1;
                }
                [..] => {
                    result.push(Tilde::new(
                        buf.len() - 1,
                        TildeKind::Text(
                            String::from_utf8(buf[..buf.len() - 1].to_vec()).map_err(|e| {
                                TildeError::new(ErrorKind::ParseError, e.to_string())
                            })?,
                        ),
                    ));
                    total_len += buf.len();
                    return Ok(Tilde::new(total_len, TildeKind::Loop((result, loop_kind))));
                }
            }

            c.read(&mut char_buf)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;

            match char_buf {
                [b'~', b'}', 0] => {
                    return Ok(Tilde::new(
                        total_len + 2,
                        TildeKind::Loop((result, loop_kind)),
                    ));
                }
                [b'~', b'}', ..] => {
                    c.seek(SeekFrom::Current(-1))
                        .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                    return Ok(Tilde::new(
                        total_len + 2,
                        TildeKind::Loop((result, loop_kind)),
                    ));
                }
                [b'~', b':', b'}'] => {
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
                _ => {
                    let back = 3 - char_buf.iter().filter(|b| **b == 0).count() as i64;
                    c.seek(SeekFrom::Current(-back))
                        .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                }
            }

            // read the tilde
            let next = Tilde::parse(c)?;
            total_len += next.len;
            result.push(next);
            char_buf = [0; 3];
            buf.clear()
        }
    }

    /// parse the ~^ in loop
    fn parse_loop_end(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut char_buf = [0u8; 2]; // three bytes
        c.read(&mut char_buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
        if char_buf != *b"~^" {
            c.seek(SeekFrom::Current(-2))
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            return Err(TildeError::new(ErrorKind::ParseError, "should start with ~^").into());
        }

        Ok(Tilde {
            len: 2,
            value: TildeKind::LoopEnd,
        })
    }

    /// parse the '~[~]'
    fn parse_cond(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut char_buf = [0u8; 3]; // three bytes
        let mut total_len = 0;
        let mut cond_kind;
        c.read(&mut char_buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;

        match char_buf {
            [b'~', b'[', ..] => {
                total_len += 2;
                cond_kind = TildeCondKind::Nil(false);
                c.seek(SeekFrom::Current(-1))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            }
            [b'~', b'#', b'['] => {
                total_len += 3;
                cond_kind = TildeCondKind::Sharp;
            }
            [b'~', b'@', b'['] => {
                total_len += 3;
                cond_kind = TildeCondKind::At;
            }
            [b'~', b':', b'['] => {
                total_len += 3;
                cond_kind = TildeCondKind::Colon;
            }
            _ => {
                c.seek(SeekFrom::Current(-3))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                return Err(TildeError::new(
                    ErrorKind::ParseError,
                    "should start with ~[, ~#[, ~@[",
                )
                .into());
            }
        }

        let mut buffer = vec![];
        let mut one_byte = [0_u8; 1];

        let mut cache = vec![];
        let mut result: Vec<Tilde> = vec![];

        loop {
            c.read_exact(&mut one_byte)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            if one_byte == [0] {
                break;
            }

            if one_byte == [b'~'] {
                if !buffer.is_empty() {
                    cache.push(Tilde {
                        len: buffer.len(),
                        value: TildeKind::Text(
                            String::from_utf8(buffer.clone()).map_err(|e| {
                                TildeError::new(ErrorKind::ParseError, e.to_string())
                            })?,
                        ),
                    });
                    buffer.clear();
                }

                one_byte[0] = 0;
                c.read_exact(&mut one_byte)
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                match one_byte {
                    [b':'] => {
                        one_byte[0] = 0;
                        c.read_exact(&mut one_byte)
                            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                        if one_byte == [b';'] {
                            let cache_len = cache.iter().map(|t: &Tilde| t.len()).sum::<usize>();
                            result.push(Tilde::new(cache_len, TildeKind::VecTilde(cache.clone())));
                            cond_kind.to_true();
                            cache.clear();
                            total_len += 3 + cache_len;
                        } else {
                            panic!()
                        }
                    }
                    [b';'] => {
                        let cache_len = cache.iter().map(|t: &Tilde| t.len()).sum::<usize>();
                        result.push(Tilde::new(cache_len, TildeKind::VecTilde(cache.clone())));
                        cache.clear();
                        total_len += 2 + cache_len;
                    }
                    [b']'] => {
                        let cache_len = cache.iter().map(|t: &Tilde| t.len()).sum::<usize>();
                        result.push(Tilde::new(cache_len, TildeKind::VecTilde(cache.clone())));
                        total_len += 2 + cache_len;
                        break;
                    }
                    _ => {
                        c.seek(SeekFrom::Current(-2))
                            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                        let c = Self::parse(c)?;
                        cache.push(c);
                    }
                }
                one_byte[0] = 0;
            } else {
                buffer.push(one_byte[0]);
            }
        }

        Ok(Tilde::new(total_len, TildeKind::Cond((result, cond_kind))))
    }

    #[cfg(test)]
    fn parse_vec(
        c: &mut Cursor<&'_ str>,
        end_len: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bucket = vec![0; end_len].into_boxed_slice();
        c.read_exact(&mut bucket)?;

        let ss = String::from_utf8(bucket.as_ref().to_vec())?;
        //dbg!(&ss);
        let mut inner_c = Cursor::new(ss.as_str());
        let mut buf = vec![];
        let mut result = vec![];
        let mut total_len = 0;

        loop {
            // read text until the next '~'
            inner_c.read_until(b'~', &mut buf)?;

            match buf.as_slice() {
                [b'~'] => {
                    inner_c.seek(SeekFrom::Current(-1))?;
                }
                [.., b'~'] => {
                    inner_c.seek(SeekFrom::Current(-1))?;
                    result.push(Tilde::new(
                        buf.len() - 1,
                        TildeKind::Text(String::from_utf8(buf[..buf.len() - 1].to_vec())?),
                    ));
                    total_len += buf.len() - 1;
                }
                [] => {
                    c.seek(SeekFrom::Current((end_len - total_len) as i64 * -1))?;
                    return Ok(Tilde::new(total_len, TildeKind::VecTilde(result)));
                }
                [..] => {
                    result.push(Tilde::new(
                        buf.len(),
                        TildeKind::Text(String::from_utf8(buf[..buf.len()].to_vec())?),
                    ));
                    total_len += buf.len();
                    c.seek(SeekFrom::Current((end_len - total_len) as i64 * -1))?;
                    return Ok(Tilde::new(total_len, TildeKind::VecTilde(result)));
                }
            }

            let next = Tilde::parse(&mut inner_c)?;
            total_len += next.len;
            result.push(next);

            buf.clear()
        }
    }

    /// parse function for '~a'
    fn parse_value(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut buf = vec![];
        //:= this for in maybe re-write in helper function
        for t in [b'a', b'A'] {
            c.read_until(t, &mut buf)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            match buf.last() {
                Some(b) if *b == t => return Ok(Tilde::new(buf.len(), TildeKind::Va)),
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "should start with ~a").into())
    }

    /// parse the float
    fn parse_float(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut buf = vec![];

        for t in [b'$', b'f', b'F'] {
            c.read_until(t, &mut buf)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            match buf.last() {
                Some(b) if *b == t => {
                    return Ok(Tilde::new(
                        buf.len(),
                        TildeKind::Float(Some(
                            String::from_utf8(
                                buf.get(1..buf.len() - 1).map_or(Vec::new(), |s| s.to_vec()),
                            )
                            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?,
                        )),
                    ))
                }
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the '$' or 'f'").into())
    }

    /// parse the digit
    fn parse_digit(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut buf = vec![];

        for t in [b'd', b'D'] {
            c.read_until(t, &mut buf)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            match buf.last() {
                Some(b) if *b == t => {
                    let s = String::from_utf8(
                        buf.get(1..buf.len() - 1).map_or(Vec::new(), |s| s.to_vec()),
                    )
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                    return Ok(Tilde::new(
                        buf.len(),
                        if &s == "" {
                            TildeKind::Digit(None)
                        } else {
                            TildeKind::Digit(Some(s))
                        },
                    ));
                }
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the 'd' or 'D'").into())
    }

    /// parse the star
    fn parse_star(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;

        match char_buf {
            [b'~', b':', b'*'] => Ok(Self {
                len: 3,
                value: TildeKind::Star(StarKind::Hop),
            }),
            [b'~', b'*', ..] => {
                c.seek(SeekFrom::Current(-1))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                Ok(Self {
                    len: 2,
                    value: TildeKind::Star(StarKind::Skip),
                })
            }
            _ => {
                c.seek(SeekFrom::Current(-3))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                return Err(
                    TildeError::new(ErrorKind::ParseError, "should start with ~* or ~:*").into(),
                );
            }
        }
    }

    fn parse_tildes(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
        match char_buf {
            [b'~', n @ _, b'~'] => Ok(Self {
                len: 3,
                value: TildeKind::Tildes(
                    String::from_utf8(vec![n])
                        .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?
                        .parse::<usize>()
                        .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?,
                ),
            }),
            _ => {
                c.seek(SeekFrom::Current(-3))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                return Err(TildeError::new(ErrorKind::ParseError, "should start with ~n~").into());
            }
        }
    }

    fn parse_standard(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut buf = vec![];

        for t in [b's', b'S'] {
            c.read_until(t, &mut buf)
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            match buf.last() {
                Some(b) if *b == t => return Ok(Tilde::new(buf.len(), TildeKind::Standard)),
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))
                .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the 's' or 'S'").into())
    }

    fn parse_char(c: &mut Cursor<&'_ str>) -> Result<Self, TildeError> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)
            .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;

        match char_buf {
            [b'~', b'@', b'c' | b'C'] => Ok(Self {
                len: 3,
                value: TildeKind::Char(CharKind::At),
            }),
            [b'~', b'c' | b'C', ..] => {
                c.seek(SeekFrom::Current(-1))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                Ok(Self {
                    len: 2,
                    value: TildeKind::Char(CharKind::Nil),
                })
            }
            _ => {
                c.seek(SeekFrom::Current(-3))
                    .map_err(|e| TildeError::new(ErrorKind::ParseError, e.to_string()))?;
                return Err(
                    TildeError::new(ErrorKind::ParseError, "should start with ~c or ~@c").into(),
                );
            }
        }
    }

    // more parsers functions below
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_va() -> Result<(), Box<dyn std::error::Error>> {
        let mut case = Cursor::new("~a");
        assert_eq!(Tilde::parse_value(&mut case)?, Tilde::new(2, TildeKind::Va));

        let mut case = Cursor::new("~A");
        assert_eq!(Tilde::parse_value(&mut case)?, Tilde::new(2, TildeKind::Va));
        Ok(())
    }

    #[test]
    fn test_parse_loop() -> Result<(), Box<dyn std::error::Error>> {
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
                        value: TildeKind::Text(String::from("a bc"))
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
                            value: TildeKind::Text(String::from("a bc"))
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
                            value: TildeKind::Text(String::from("a bc"))
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
                            value: TildeKind::Text(", ".to_string()),
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
    fn test_parse_float() -> Result<(), Box<dyn std::error::Error>> {
        let mut case = Cursor::new("~$");
        assert_eq!(
            Tilde::parse_float(&mut case)?,
            Tilde::new(2, TildeKind::Float(Some(String::new())))
        );

        let mut case = Cursor::new("~5$");
        assert_eq!(
            Tilde::parse_float(&mut case)?,
            Tilde::new(3, TildeKind::Float(Some("5".to_string())))
        );

        let mut case = Cursor::new("~,5f");
        assert_eq!(
            Tilde::parse_float(&mut case)?,
            Tilde::new(4, TildeKind::Float(Some(",5".to_string())))
        );

        Ok(())
    }

    #[test]
    fn test_scan_for_kind() -> Result<(), Box<dyn std::error::Error>> {
        let case = "~a";
        let mut c = Cursor::new(case);
        let f = Tilde::scan_for_kind(&mut c)?;

        // let mut ss = String::new();
        // c.read_to_string(&mut ss);
        // dbg!(ss);
        // c.seek(SeekFrom::Start(0));
        //dbg!(f(&mut c));

        assert_eq!(Tilde::new(2, TildeKind::Va), f(&mut c)?);
        Ok(())
    }

    #[test]
    fn test_parse_vec() -> Result<(), Box<dyn std::error::Error>> {
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
                        value: TildeKind::Text(String::from(" and "))
                    },
                    Tilde {
                        len: 2,
                        value: TildeKind::Va
                    }
                ])
            )
        );

        //dbg!(&case.position());
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
                    value: TildeKind::Text(String::from("a"))
                }])
            )
        );

        Ok(())
    }

    #[test]
    fn test_parse_cond() -> Result<(), Box<dyn std::error::Error>> {
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
                        value: TildeKind::VecTilde(vec![Tilde::new(
                            4,
                            TildeKind::Text(String::from("cero"))
                        )]),
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
                                TildeKind::Text(String::from("cero"))
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                3,
                                TildeKind::Text(String::from("uno"))
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                3,
                                TildeKind::Text(String::from("dos"))
                            )])
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
                                TildeKind::Text(String::from("cero"))
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                3,
                                TildeKind::Text(String::from("uno"))
                            )])
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
                                TildeKind::Text(String::from("cero"))
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                3,
                                TildeKind::Text(String::from("uno"))
                            )])
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
                                TildeKind::Text(String::from("cero"))
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                3,
                                TildeKind::Text(String::from("uno"))
                            )])
                        },
                        Tilde {
                            len: 3,
                            value: TildeKind::VecTilde(vec![Tilde::new(
                                3,
                                TildeKind::Text(String::from("dos"))
                            )])
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
                                TildeKind::Text(String::from("NONE"))
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
                                    value: TildeKind::Text(String::from(" and "))
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
                                    value: TildeKind::Text(String::from(", "))
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
                                value: TildeKind::Text("x = ".into())
                            },
                            Tilde {
                                len: 2,
                                value: TildeKind::Va
                            },
                            Tilde {
                                len: 1,
                                value: TildeKind::Text(" ".into())
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
                                value: TildeKind::Text("y = ".into())
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
    fn test_parse_star() -> Result<(), Box<dyn std::error::Error>> {
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
    fn test_parse_tildes() -> Result<(), Box<dyn std::error::Error>> {
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
    fn test_parse_standard() -> Result<(), Box<dyn std::error::Error>> {
        let mut case = Cursor::new("~s");
        assert_eq!(Tilde::parse(&mut case)?, Tilde::new(2, TildeKind::Standard));

        let mut case = Cursor::new("~S");
        assert_eq!(Tilde::parse(&mut case)?, Tilde::new(2, TildeKind::Standard));

        Ok(())
    }

    #[test]
    fn test_parse_char() -> Result<(), Box<dyn std::error::Error>> {
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
}

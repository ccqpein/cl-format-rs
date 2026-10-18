use super::tildes::tilde_able_impl::reveal_loop;
use super::Tilde;

use anyhow::Result;
use cl_format_macros::TildeAble;
use std::fmt::Debug;

#[doc = "Error type for tildes parsing"]
#[derive(Debug)]
pub struct TildeError {
    kind: ErrorKind,
    msg: String,
}

impl TildeError {
    pub fn new(kind: ErrorKind, msg: impl AsRef<str>) -> Self {
        Self {
            kind,
            msg: msg.as_ref().to_string(),
        }
    }
}

impl std::error::Error for TildeError {}

impl std::fmt::Display for TildeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TilderError {:?}: {}", self.kind, self.msg)
    }
}

#[doc = "ErrorKind"]
#[derive(Debug)]
pub enum ErrorKind {
    ParseError,
    RevealError,
    EmptyImplenmentError,
    FormatError,
}

#[doc = "TildeCondKind"]
#[derive(Debug, PartialEq, Clone)]
pub enum TildeCondKind {
    Nil(bool), // ~[, bool for the last ~:;
    Sharp,     // ~#[
    At,        // ~@[
    Colon,     // ~:[
}

#[doc = "TildeLoopKind"]
#[derive(Debug, PartialEq, Clone)]
pub enum TildeLoopKind {
    Nil,      // ~{~}
    NilColon, // ~{~:}
    At,       // ~@{~}
}

impl TildeCondKind {
    pub fn to_true(&mut self) {
        match self {
            TildeCondKind::Nil(_) => *self = TildeCondKind::Nil(true),
            _ => (),
        }
    }
}

#[doc = "StarKind"]
#[derive(Debug, PartialEq, Clone)]
pub enum StarKind {
    Hop,
    Skip,
}

#[doc = "CharKind"]
#[derive(Debug, PartialEq, Clone)]
pub enum CharKind {
    Nil,
    At,
}

#[doc = "Radix flag ~@R, ~:R, and ~:@R"]
#[derive(Debug, PartialEq, Clone)]
pub enum RadixFlag {
    At,      // ~@R
    Colon,   // ~:R
    AtColon, // ~:@R
}

#[doc = "TildeKind is the enum that including all potential kind.

Each variant corresponds to a `format_<variant>` method on the `TildeAble` trait.

Check README for custom types.
"]
#[derive(Debug, PartialEq, TildeAble, Clone)]
pub enum TildeKind<'a> {
    /// `~C` and `~:C`
    Char(CharKind),

    /// `~$`, `~5$`, and `~f`
    Float(Option<&'a str>),

    /// Tilde R: Radix, [doc](http://www.lispworks.com/documentation/lw50/CLHS/Body/22_cba.htm)
    Radix(
        (
            Option<u8>,        // radix
            Option<usize>,     // mincol
            Option<char>,      // padchar
            Option<char>,      // commachar
            Option<usize>,     // comma-interval
            Option<RadixFlag>, // flag
        ),
    ),

    /// `~d`, `~:d`, and `~:@d`
    Digit(Option<&'a str>),

    /// `~a`
    Va,

    /// `~*` and `~:*`
    Star(StarKind),

    /// `~s`
    Standard,

    /// for loop expression
    Loop((Vec<Tilde<'a>>, TildeLoopKind)),

    /// for loop stop, `~^`
    LoopEnd,

    /// tilde itself
    Tildes(usize),

    /// `~[` and `~]` condition
    Cond((Vec<Tilde<'a>>, TildeCondKind)),

    /// text inside the tilde
    Text(&'a str),

    /// Vec of tildes
    VecTilde(Vec<Tilde<'a>>),
}

impl<'a> TildeKind<'a> {
    #[inline]
    pub fn match_reveal<C: super::ArgCursor>(
        &self,
        args: &C,
        buf: &mut String,
    ) -> Result<std::ops::ControlFlow<()>> {
        use std::ops::ControlFlow;
        match self {
            TildeKind::Char(_) => {
                let a = args.pop().ok_or_else(|| {
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Char")
                })?;
                a.format_char(self, buf)?;
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Float(_) => {
                let a = args.pop().ok_or_else(|| {
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Float")
                })?;
                a.format_float(self, buf)?;
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Digit(_) => {
                let a = args.pop().ok_or_else(|| {
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Digit")
                })?;
                a.format_digit(self, buf)?;
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Va => {
                let a = args.pop().ok_or_else(|| {
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va")
                })?;
                a.format_va(self, buf)?;
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Standard => {
                let a = args.pop().ok_or_else(|| {
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard")
                })?;
                a.format_standard(self, buf)?;
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Radix(_) => {
                let a = args.pop().ok_or_else(|| {
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix")
                })?;
                a.format_radix(self, buf)?;
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Star(kind) => {
                match kind {
                    StarKind::Hop => {
                        args.back();
                    }
                    StarKind::Skip => {
                        args.pop();
                    }
                }
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Tildes(n) => {
                buf.extend(std::iter::repeat('~').take(*n));
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Text(s) => {
                buf.push_str(s);
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::LoopEnd => {
                if args.left_count() == 0 {
                    Ok(ControlFlow::Break(()))
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            }
            TildeKind::VecTilde(vv) => {
                for v in vv {
                    if v.reveal(args, buf)?.is_break() {
                        return Ok(ControlFlow::Break(()));
                    }
                }
                Ok(ControlFlow::Continue(()))
            }
            TildeKind::Loop((vv, kind)) => match kind {
                TildeLoopKind::Nil | TildeLoopKind::NilColon => {
                    let a = args
                        .pop()
                        .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                    a.format_loop(self, buf)?;
                    Ok(ControlFlow::Continue(()))
                }
                TildeLoopKind::At => {
                    reveal_loop(vv, args, buf)?;
                    Ok(ControlFlow::Continue(()))
                }
            },
            TildeKind::Cond((vv, kind)) => match kind {
                TildeCondKind::Sharp => {
                    let l = args.left_count();
                    if l >= vv.len() {
                        vv[vv.len() - 1].reveal(args, buf)
                    } else {
                        vv[l].reveal(args, buf)
                    }
                }
                TildeCondKind::Nil(has_default) => {
                    let a = args
                        .pop()
                        .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                    let idx = a.as_cond_index().ok_or_else(|| {
                        TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond")
                    })?;
                    match vv.get(idx) {
                        Some(tt) => tt.reveal(args, buf),
                        None if *has_default => match vv.last() {
                            Some(tt) => tt.reveal(args, buf),
                            None => Ok(ControlFlow::Continue(())),
                        },
                        None => Ok(ControlFlow::Continue(())),
                    }
                }
                TildeCondKind::Colon => {
                    let a = args
                        .pop()
                        .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                    let b = a.as_cond_bool().ok_or_else(|| {
                        TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond")
                    })?;
                    let idx = usize::from(b);
                    let tt = vv.get(idx).ok_or_else(|| {
                        TildeError::new(ErrorKind::FormatError, "cannot get tilde")
                    })?;
                    tt.reveal(args, buf)
                }
                TildeCondKind::At => {
                    let a = args
                        .pop()
                        .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                    a.format_cond(self, buf)?;
                    Ok(ControlFlow::Continue(()))
                }
            },
        }
    }
}

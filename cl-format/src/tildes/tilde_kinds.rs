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

#[derive(Debug)]
pub(super) struct TildeNil;

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
pub enum TildeKind {
    /// `~C` and `~:C`
    Char(CharKind),

    /// `~$`, `~5$`, and `~f`
    Float(Option<String>),

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
    Digit(Option<String>),

    /// `~a`
    Va,

    /// `~*` and `~:*`
    Star(StarKind),

    /// `~s`
    Standard,

    /// for loop expression
    Loop((Vec<Tilde>, TildeLoopKind)),

    /// for loop stop, `~^`
    LoopEnd,

    /// tilde itself
    Tildes(usize),

    /// `~[` and `~]` condition
    Cond((Vec<Tilde>, TildeCondKind)),

    /// text inside the tilde
    Text(String),

    /// Vec of tildes
    VecTilde(Vec<Tilde>),
}

impl TildeKind {
    pub fn match_reveal(&self, arg: &dyn TildeAble, buf: &mut String) -> Result<()> {
        match self {
            TildeKind::Char(_) => arg.format_char(self, buf),
            TildeKind::Float(_) => arg.format_float(self, buf),
            TildeKind::Digit(_) => arg.format_digit(self, buf),
            TildeKind::Va => arg.format_va(self, buf),
            TildeKind::Loop(_) => arg.format_loop(self, buf),
            TildeKind::LoopEnd => {
                Err(TildeError::new(ErrorKind::RevealError, "loop end cannot reveal").into())
            }
            TildeKind::Tildes(n) => {
                buf.extend(std::iter::repeat('~').take(*n));
                Ok(())
            }
            TildeKind::Text(s) => {
                buf.push_str(s);
                Ok(())
            }
            TildeKind::VecTilde(vv) => {
                for v in vv {
                    v.reveal(arg, buf)?;
                }
                Ok(())
            }
            TildeKind::Cond((_, _)) => arg.format_cond(self, buf),
            TildeKind::Star(_) => arg.format_star(self, buf),
            TildeKind::Standard => arg.format_standard(self, buf),
            TildeKind::Radix(_) => arg.format_radix(self, buf),
        }
    }
}

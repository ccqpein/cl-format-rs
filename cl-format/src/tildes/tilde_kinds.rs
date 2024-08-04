use super::Tilde;

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

The most of variants inside has its implement trait. Like the `~d` is `TildeKind::Digit` and the type that can be revealed as `~a` should implement `TildeKindDigit` trait.

Check README for custom types.
"]
#[derive(Debug, PartialEq, TildeAble, Clone)]
pub enum TildeKind {
    #[implTo(char)]
    /// `~C` and `~:C`
    Char(CharKind),

    /// `~$`, `~5$`, and `~f`
    Float(Option<String>),

    #[implTo(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize)]
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

    #[implTo(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize)]
    /// `~d`, `~:d`, and `~:@d`
    Digit(Option<String>),

    #[implTo(
        f32, f64, char, i8, i16, i32, i64, i128, isize, bool, u8, u16, u32, u64, u128, usize,
        String, TildeNil
    )]
    /// `~a`
    Va,

    /// `~*` and `~:*`
    Star(StarKind),

    #[implTo(f32, f64, char, i32, i64, usize, bool, u32, u64, String)]
    /// `~s`
    Standard,

    /// for loop expression
    Loop((Vec<Tilde>, TildeLoopKind)),

    /// for loop stop, `~^`
    LoopEnd,

    /// tilde itself
    Tildes(usize),

    #[implTo(usize, bool)]
    /// `~[` and `~]` condition
    Cond((Vec<Tilde>, TildeCondKind)),

    /// text inside the tilde
    Text(String),

    #[implTo(TildeNil)]
    /// Vec of tildes
    VecTilde(Vec<Tilde>),
}

impl TildeKind {
    pub fn match_reveal(&self, arg: &dyn TildeAble, buf: &mut String) -> Result<(), TildeError> {
        //dbg!(arg);
        //dbg!(self);
        match self {
            TildeKind::Char(_) => {
                let a = arg.into_tildekind_char().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self, buf);
            }
            TildeKind::Float(_) => {
                let a = arg.into_tildekind_float().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self, buf);
            }
            TildeKind::Digit(_) => {
                let a = arg.into_tildekind_digit().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self, buf);
            }
            TildeKind::Va => {
                //dbg!(&arg);
                let a = arg.into_tildekind_va().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self, buf);
            }
            TildeKind::Loop(_) => {
                let a = arg.into_tildekind_loop().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Loop").into(),
                )?;

                return a.format(self, buf);
            }
            TildeKind::LoopEnd => {
                Err(TildeError::new(ErrorKind::RevealError, "loop end cannot reveal").into())
            }
            TildeKind::Tildes(n) => {
                buf.push_str(
                    String::from_utf8(vec![b'~'; *n])
                        .map_err(|e| TildeError::new(ErrorKind::RevealError, e.to_string()))?
                        .as_str(),
                );
                Ok(())
            }
            TildeKind::Text(s) => {
                buf.push_str(s);
                Ok(())
            }
            TildeKind::VecTilde(_) => {
                let a = arg.into_tildekind_vectilde().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to VecTilde").into(),
                )?;

                return a.format(self, buf);
            }
            TildeKind::Cond((_, _)) => {
                let a = arg.into_tildekind_cond().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond").into(),
                )?;
                return a.format(self, buf);
            }
            TildeKind::Star(_) => {
                let a = arg.into_tildekind_star().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Star").into(),
                )?;
                return a.format(self, buf);
            }
            TildeKind::Standard => {
                let a = arg.into_tildekind_standard().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard").into(),
                )?;
                return a.format(self, buf);
            }
            TildeKind::Radix(_) => {
                let a = arg.into_tildekind_radix().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix").into(),
                )?;
                return a.format(self, buf);
            } //_ => unimplemented!(),
        }
    }
}

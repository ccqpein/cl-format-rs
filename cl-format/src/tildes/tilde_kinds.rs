use super::Tilde;

use cl_format_macros::TildeAble;
use std::fmt::Debug;

#[derive(Debug)]
pub struct TildeError {
    kind: ErrorKind,
    msg: String,
}

impl TildeError {
    pub(super) fn new(kind: ErrorKind, msg: impl AsRef<str>) -> Self {
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

#[derive(Debug)]
pub(super) enum ErrorKind {
    ParseError,
    RevealError,
    EmptyImplenmentError,
    FormatError,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TildeCondKind {
    Nil(bool), // ~[, bool for the last ~:;
    Sharp,     // ~#[
    At,        // ~@[
    Colon,     // ~:[
}

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

#[derive(Debug, PartialEq, Clone)]
pub enum StarKind {
    Hop,
    Skip,
}

#[derive(Debug)]
pub(super) struct TildeNil;

#[derive(Debug, PartialEq, Clone)]
pub enum CharKind {
    Nil,
    At,
}

#[derive(Debug, PartialEq, TildeAble, Clone)]
pub enum TildeKind {
    /// ~C ~:C
    #[implTo(char)]
    Char(CharKind),

    /// ~$ ~5$ ~f
    //:= next
    Float(Option<String>),

    /// ~d ~:d ~:@d
    #[implTo(i32, i64, u32, u64, usize)]
    Digit(Option<String>),

    #[implTo(f32, f64, char, i32, i64, usize, bool, u32, u64, String, TildeNil)]
    /// ~a
    Va,

    /// ~* and ~:*
    Star(StarKind),

    #[implTo(f32, f64, char, i32, i64, usize, bool, u32, u64, String)]
    /// ~s
    Standard,

    /// loop
    Loop((Vec<Tilde>, TildeLoopKind)),

    /// loop stop, ~^
    LoopEnd,

    /// tilde itself
    Tildes(usize),

    #[implTo(usize, bool)]
    /// ~[ ~] condition
    Cond((Vec<Tilde>, TildeCondKind)),

    /// text inside the tilde
    Text(String),

    /// vec
    #[implTo(TildeNil)]
    VecTilde(Vec<Tilde>),
}

impl TildeKind {
    pub fn match_reveal(&self, arg: &dyn TildeAble) -> Result<Option<String>, TildeError> {
        //dbg!(arg);
        //dbg!(&self);
        match self {
            TildeKind::Char(_) => {
                let a = arg.into_tildekind_char().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self);
            }
            TildeKind::Float(_) => {
                let a = arg.into_tildekind_float().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self);
            }
            TildeKind::Digit(_) => {
                let a = arg.into_tildekind_digit().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self);
            }
            TildeKind::Va => {
                let a = arg.into_tildekind_va().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into(),
                )?;

                return a.format(self);
            }
            TildeKind::Loop(_) => {
                let a = arg.into_tildekind_loop().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Loop").into(),
                )?;

                return a.format(self);
            }
            TildeKind::LoopEnd => {
                Err(TildeError::new(ErrorKind::RevealError, "loop end cannot reveal").into())
            }
            TildeKind::Tildes(n) => {
                Ok(Some(String::from_utf8(vec![b'~'; *n]).map_err(|e| {
                    TildeError::new(ErrorKind::RevealError, e.to_string())
                })?))
            }
            TildeKind::Text(s) => Ok(Some(s.to_string())),
            TildeKind::VecTilde(_) => {
                let a = arg.into_tildekind_vectilde().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to VecTilde").into(),
                )?;

                return a.format(self);
            }
            TildeKind::Cond((_, _)) => {
                let a = arg.into_tildekind_cond().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond").into(),
                )?;
                return a.format(self);
            }
            TildeKind::Star(_) => {
                let a = arg.into_tildekind_star().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Star").into(),
                )?;
                return a.format(self);
            }
            TildeKind::Standard => {
                let a = arg.into_tildekind_standard().ok_or::<TildeError>(
                    TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard").into(),
                )?;
                return a.format(self);
            }
        }
    }
}

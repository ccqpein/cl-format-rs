use crate::*;
use cl_format_macros::*;
use std::any::Any;
use std::cell::RefCell;
use std::convert::TryInto;
use std::fmt::Debug;
use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};
use std::ops::Deref;

/// The args for control string to use. Can pop and go back for some
/// condition reveal.
#[derive(Debug)]
pub struct Args<'a> {
    len: usize,
    inner: Vec<&'a dyn TildeAble>,
    ind: RefCell<usize>,
}

impl<'a> Args<'a> {
    pub fn new(i: Vec<&'a dyn TildeAble>) -> Self {
        Self {
            len: i.len(),
            inner: i,
            ind: RefCell::new(0),
        }
    }

    pub fn pop(&self) -> Option<&dyn TildeAble> {
        let r = self.inner.get(*self.ind.borrow())?;
        *self.ind.borrow_mut() += 1;
        Some(*r)
    }

    pub fn back(&self) -> Option<&dyn TildeAble> {
        let i = match *self.ind.borrow() {
            0 => return None,
            n @ _ => n - 1,
        };

        let r = self.inner.get(i)?;
        *self.ind.borrow_mut() -= 1;
        Some(*r)
    }

    pub fn left_count(&self) -> usize {
        self.len - *self.ind.borrow()
    }

    pub fn reset(&self) {
        *self.ind.borrow_mut() = 0;
    }
}

impl<'a, const N: usize> From<[&'a dyn TildeAble; N]> for Args<'a> {
    fn from(value: [&'a dyn TildeAble; N]) -> Self {
        Self::new(value.to_vec())
    }
}

impl<'a> From<&'_ [&'a dyn TildeAble]> for Args<'a> {
    fn from(value: &'_ [&'a dyn TildeAble]) -> Self {
        Self::new(value.to_vec())
    }
}

impl<'a, 's: 'a> From<Vec<&'s dyn TildeAble>> for Args<'a> {
    fn from(value: Vec<&'s dyn TildeAble>) -> Self {
        Self::new(value)
    }
}

impl<'a, T> From<&T> for Args<'a>
where
    T: Deref<Target = [&'a dyn TildeAble]>,
{
    fn from(value: &T) -> Self {
        Self::from(value.deref())
    }
}

impl<'a> IntoIterator for Args<'a> {
    type Item = &'a dyn TildeAble;

    type IntoIter = std::vec::IntoIter<&'a dyn TildeAble>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[derive(Debug)]
struct TildeError {
    kind: ErrorKind,
    msg: String,
}

impl TildeError {
    fn new(kind: ErrorKind, msg: impl AsRef<str>) -> Self {
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
enum ErrorKind {
    ParseError,
    RevealError,
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
    fn to_true(&mut self) {
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
struct TildeNil;

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
    pub fn match_reveal(
        &self,
        arg: &dyn TildeAble,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
            TildeKind::Tildes(n) => Ok(Some(String::from_utf8(vec![b'~'; *n])?)),
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

/// impl mamually
impl TildeAble for Option<&dyn TildeAble> {
    fn len(&self) -> usize {
        match self {
            Some(_) => 1,
            None => 0,
        }
    }

    fn into_tildekind_cond(&self) -> Option<&dyn TildeKindCond> {
        Some(self)
    }
}

impl<'a> TildeAble for Args<'a> {
    fn len(&self) -> usize {
        self.left_count()
    }

    fn into_tildekind_char(&self) -> Option<&dyn TildeKindChar> {
        match self.pop() {
            Some(a) => a.into_tildekind_char(),
            None => None,
        }
    }

    fn into_tildekind_float(&self) -> Option<&dyn TildeKindFloat> {
        match self.pop() {
            Some(a) => a.into_tildekind_float(),
            None => None,
        }
    }

    fn into_tildekind_digit(&self) -> Option<&dyn TildeKindDigit> {
        match self.pop() {
            Some(a) => a.into_tildekind_digit(),
            None => None,
        }
    }

    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        match self.pop() {
            Some(a) => a.into_tildekind_va(),
            None => None,
        }
    }

    fn into_tildekind_star(&self) -> Option<&dyn TildeKindStar> {
        Some(self)
    }

    fn into_tildekind_loop(&self) -> Option<&dyn TildeKindLoop> {
        Some(self)
    }

    fn into_tildekind_loopend(&self) -> Option<&dyn TildeKindLoopEnd> {
        None
    }

    fn into_tildekind_cond(&self) -> Option<&dyn TildeKindCond> {
        Some(self)
    }

    fn into_tildekind_text(&self) -> Option<&dyn TildeKindText> {
        None
    }

    fn into_tildekind_vectilde(&self) -> Option<&dyn TildeKindVecTilde> {
        Some(self)
    }

    fn into_tildekind_standard(&self) -> Option<&dyn TildeKindStandard> {
        match self.pop() {
            Some(a) => a.into_tildekind_standard(),
            None => None,
        }
    }

    fn into_tildekind_tildes(&self) -> Option<&dyn TildeKindTildes> {
        None
    }
}

impl TildeAble for Vec<&dyn TildeAble> {
    fn len(&self) -> usize {
        self.len()
    }

    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        Some(self)
    }

    fn into_tildekind_loop(&self) -> Option<&dyn TildeKindLoop> {
        Some(self)
    }
}

//========================================
// TildeKindDigit
//========================================
multi_tilde_impl!(TildeKindDigit, [i32, i64, u32, u64, usize], self, {
    Ok(Some(format!("{}", self)))
});

//========================================
// TildeKindChar
//========================================
/// impl, re-define the format method for over writing the default method
impl TildeKindChar for char {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::Char(CharKind::At) => Ok(Some(format!("'{}'", self))),
            TildeKind::Char(CharKind::Nil) => Ok(Some(format!("{}", self))),
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Char").into()),
        }
    }
}

//========================================
// TildeKindVa
//========================================
multi_tilde_impl!(
    TildeKindVa,
    [f32, f64, char, i32, i64, usize, u32, u64, String],
    self,
    { Ok(Some(format!("{}", self))) }
);

impl TildeKindVa for bool {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if *self {
            Ok(Some("true".into()))
        } else {
            Ok(Some("false".into()))
        }
    }
}

impl TildeKindVa for TildeNil {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some("nil".into()))
    }
}

impl TildeKindVa for Vec<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("{:?}", self)))
    }
}

//========================================
// TildeKindLoop
//========================================
impl<'a> TildeKindLoop for Args<'a> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            // self[0] is the Vec<&dyn TildeAble> of loop
            TildeKind::Loop((_, TildeLoopKind::Nil | TildeLoopKind::NilColon)) => {
                let a = self.pop().ok_or::<String>("run out args".into())?;
                tkind.match_reveal(a)
            }
            TildeKind::Loop((vv, TildeLoopKind::At)) => {
                //let mut new_args = self.clone();
                let mut result = vec![];

                'a: loop {
                    for t in vv {
                        if let TildeKind::LoopEnd = t.value {
                            if self.left_count() != 0 {
                                continue;
                            } else {
                                break 'a;
                            }
                        }

                        result.push(t.reveal(self)?);
                    }
                    //dbg!(self);
                    if self.left_count() == 0 {
                        break;
                    }
                }

                Ok(Some(
                    result
                        .into_iter()
                        .filter_map(|a| a)
                        .collect::<Vec<_>>()
                        .as_slice()
                        .join(""),
                ))
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Arg to Loop").into()),
        }
    }
}

impl<'a> TildeKindLoop for Vec<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::Loop((_, TildeLoopKind::Nil)) => {
                let mut new_kind = tkind.clone();

                match &mut new_kind {
                    TildeKind::Loop((_, k @ TildeLoopKind::Nil)) => {
                        if self.len() != 0 {
                            *k = TildeLoopKind::At
                        } else {
                            return Ok(None);
                        }
                    }
                    _ => unreachable!(),
                };
                new_kind.match_reveal(&Args::from(self))
            }
            TildeKind::Loop((_, TildeLoopKind::NilColon)) => {
                let mut new_kind = tkind.clone();
                match &mut new_kind {
                    TildeKind::Loop((_, k @ TildeLoopKind::NilColon)) => *k = TildeLoopKind::At,
                    _ => unreachable!(),
                };
                new_kind.match_reveal(&Args::from(self))
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Vec to Loop").into()),
        }
    }
}

//========================================
// TildeKindCond
//========================================
impl TildeKindCond for usize {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        //dbg!(self);
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Nil(true))) => match vv.get(*self) {
                Some(tt) => tt.reveal(&TildeNil),
                None => {
                    let last = vv.len() - 1;
                    match vv.get(last) {
                        Some(tt) => tt.reveal(&TildeNil),
                        None => Ok(None),
                    }
                }
            },
            TildeKind::Cond((vv, TildeCondKind::Nil(false))) => match vv.get(*self) {
                Some(tt) => tt.reveal(&TildeNil),
                None => Ok(None),
            },
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl TildeKindCond for bool {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Colon)) => {
                if *self {
                    vv.get(1)
                        .ok_or::<String>("cannot get tilde".into())?
                        .reveal(&TildeNil)
                } else {
                    vv.get(0)
                        .ok_or::<String>("cannot get tilde".into())?
                        .reveal(&TildeNil)
                }
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl TildeKindCond for Option<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::At)) => match self {
                Some(a) => {
                    //println!("here: {:?}", a);
                    let k = TildeKind::VecTilde(vv.clone());
                    // VecTilde need the vec
                    // TildeCondKind::At only accept one arg

                    k.match_reveal(&Args::from([*a]))
                }
                None => Ok(None),
            },
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl<'a> TildeKindCond for Args<'a> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Sharp)) => {
                let l = self.left_count();
                if l >= vv.len() {
                    vv[vv.len() - 1].reveal(self)
                } else {
                    vv[l].reveal(self)
                }
            }
            TildeKind::Cond((_, _)) => {
                let a = self.pop().ok_or::<String>("run out args".into())?;
                tkind.match_reveal(a)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

//========================================
// TildeKindVecTilde
//========================================
impl TildeKindVecTilde for TildeNil {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::VecTilde(vv) => {
                let mut result = vec![];
                for t in vv {
                    result.push(t.reveal(self)?);
                }
                Ok(Some(
                    result
                        .into_iter()
                        .filter_map(|a| a)
                        .collect::<Vec<_>>()
                        .as_slice()
                        .join(""),
                ))
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to VecTilde").into()),
        }
    }
}

impl<'a> TildeKindVecTilde for Args<'a> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::VecTilde(vv) => {
                let mut result = vec![];
                for t in vv {
                    result.push(t.reveal(self)?);
                }
                Ok(Some(
                    result
                        .into_iter()
                        .filter_map(|a| a)
                        .collect::<Vec<_>>()
                        .as_slice()
                        .join(""),
                ))
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to VecTilde").into()),
        }
    }
}

//========================================
// TildeKindStar
//========================================
impl<'a> TildeKindStar for Args<'a> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match tkind {
            TildeKind::Star(StarKind::Hop) => {
                self.back(); // back to last one, make it hop

                Ok(None)
            }
            TildeKind::Star(StarKind::Skip) => {
                self.pop();
                Ok(None)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Star").into()),
        }
    }
}

//========================================
// TildeKindStandard
//========================================
impl TildeKindStandard for String {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("\"{}\"", self)))
    }
}

impl TildeKindStandard for char {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("'{}'", self)))
    }
}

multi_tilde_impl!(
    TildeKindStandard,
    [f32, f64, i32, i64, usize, bool, u32, u64],
    self,
    { Ok(Some(format!("{}", self))) }
);

/*=========================================================*/

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

    pub fn reveal(
        &self,
        arg: &dyn TildeAble,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.value.match_reveal(arg)
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
        Box<
            dyn for<'a, 'b> Fn(
                &'a mut std::io::Cursor<&'b str>,
            ) -> Result<Tilde, Box<dyn std::error::Error>>,
        >,
        Box<dyn std::error::Error>,
    > {
        let mut buf = [0u8; 1];
        c.read(&mut buf)?;
        if buf[0] != b'~' {
            return Err(TildeError::new(ErrorKind::ParseError, "should start with ~").into());
        }

        // read until the tilde key char
        let mut buf = [0_u8; 3];
        let mut buf_offset = 1;
        c.read(&mut buf)?;
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
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_value),
                );
            }
            [b'{', ..] | [b'@', b'{', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_loop),
                );
            }
            [b'$', ..] | [b'f', ..] | [b'F', ..] | [_, b'$', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_float),
                );
            }
            [b'd', ..] | [b'D', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_digit),
                );
            }
            [b'[', ..] | [b'#', b'[', ..] | [b':', b'[', ..] | [b'@', b'[', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_cond),
                );
            }
            [b'^', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_loop_end),
                );
            }
            [b':', b'*', ..] | [b'*', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_star),
                );
            }
            [_, b'~', ..] => {
                //:= can only parse the one digit number
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_tildes),
                );
            }
            [b'S', ..] | [b's', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
                return Ok(
                    #[rustc_box]
                    Box::new(Self::parse_standard),
                );
            }
            [b'C', ..] | [b'c', ..] | [b'@', b'c' | b'C', ..] => {
                c.seek(SeekFrom::Current(-buf_offset))?; // back to start
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
    pub fn parse(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        //dbg!(&c);
        let parser = Self::scan_for_kind(c)?;
        parser(c)
    }

    /// parse function for '~{~}'
    fn parse_loop(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)?;

        let mut loop_kind = TildeLoopKind::Nil;
        let mut total_len = 0;
        match char_buf {
            [b'~', b'{', ..] => {
                total_len += 2;
                c.seek(SeekFrom::Current(-1))?;
            }
            [b'~', b'@', b'{'] => {
                total_len += 3;
                loop_kind = TildeLoopKind::At;
            }

            _ => {
                c.seek(SeekFrom::Current(-3))?;
                return Err(TildeError::new(ErrorKind::ParseError, "should start with ~{").into());
            }
        }

        let mut result = vec![];
        let mut buf = vec![];
        let mut char_buf = [0u8; 3];

        loop {
            // read text until the next '~'
            c.read_until(b'~', &mut buf)?;

            match buf.as_slice() {
                [b'~'] => {
                    c.seek(SeekFrom::Current(-1))?;
                }
                [.., b'~'] => {
                    c.seek(SeekFrom::Current(-1))?;
                    result.push(Tilde::new(
                        buf.len() - 1,
                        TildeKind::Text(String::from_utf8(buf[..buf.len() - 1].to_vec())?),
                    ));
                    total_len += buf.len() - 1;
                }
                [..] => {
                    result.push(Tilde::new(
                        buf.len() - 1,
                        TildeKind::Text(String::from_utf8(buf[..buf.len() - 1].to_vec())?),
                    ));
                    total_len += buf.len();
                    return Ok(Tilde::new(total_len, TildeKind::Loop((result, loop_kind))));
                }
            }

            c.read(&mut char_buf)?;

            match char_buf {
                [b'~', b'}', 0] => {
                    return Ok(Tilde::new(
                        total_len + 2,
                        TildeKind::Loop((result, loop_kind)),
                    ));
                }
                [b'~', b'}', ..] => {
                    c.seek(SeekFrom::Current(-1))?;
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
                    c.seek(SeekFrom::Current(-back))?;
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
    fn parse_loop_end(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut char_buf = [0u8; 2]; // three bytes
        c.read(&mut char_buf)?;
        if char_buf != *b"~^" {
            c.seek(SeekFrom::Current(-2))?;
            return Err(TildeError::new(ErrorKind::ParseError, "should start with ~^").into());
        }

        Ok(Tilde {
            len: 2,
            value: TildeKind::LoopEnd,
        })
    }

    /// parse the '~[~]'
    fn parse_cond(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut char_buf = [0u8; 3]; // three bytes
        let mut total_len = 0;
        let mut cond_kind;
        c.read(&mut char_buf)?;

        match char_buf {
            [b'~', b'[', ..] => {
                total_len += 2;
                cond_kind = TildeCondKind::Nil(false);
                c.seek(SeekFrom::Current(-1))?;
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
                c.seek(SeekFrom::Current(-3))?;
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
            c.read_exact(&mut one_byte)?;
            if one_byte == [0] {
                break;
            }

            if one_byte == [b'~'] {
                if !buffer.is_empty() {
                    cache.push(Tilde {
                        len: buffer.len(),
                        value: TildeKind::Text(String::from_utf8(buffer.clone())?),
                    });
                    buffer.clear();
                }

                one_byte[0] = 0;
                c.read_exact(&mut one_byte)?;
                match one_byte {
                    [b':'] => {
                        one_byte[0] = 0;
                        c.read_exact(&mut one_byte)?;
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
                        c.seek(SeekFrom::Current(-2))?;
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
    fn parse_value(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = vec![];
        //:= this for in maybe re-write in helper function
        for t in [b'a', b'A'] {
            c.read_until(t, &mut buf)?;
            match buf.last() {
                Some(b) if *b == t => return Ok(Tilde::new(buf.len(), TildeKind::Va)),
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "should start with ~a").into())
    }

    /// parse the float
    fn parse_float(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = vec![];

        for t in [b'$', b'f', b'F'] {
            c.read_until(t, &mut buf)?;
            match buf.last() {
                Some(b) if *b == t => {
                    return Ok(Tilde::new(
                        buf.len(),
                        TildeKind::Float(Some(String::from_utf8(
                            buf.get(1..buf.len() - 1).map_or(Vec::new(), |s| s.to_vec()),
                        )?)),
                    ))
                }
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the '$' or 'f'").into())
    }

    /// parse the digit
    fn parse_digit(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = vec![];

        for t in [b'd', b'D'] {
            c.read_until(t, &mut buf)?;
            match buf.last() {
                Some(b) if *b == t => {
                    let s = String::from_utf8(
                        buf.get(1..buf.len() - 1).map_or(Vec::new(), |s| s.to_vec()),
                    )?;
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
            c.seek(SeekFrom::Current(-(buf.len() as i64)))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the 'd' or 'D'").into())
    }

    /// parse the star
    fn parse_star(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)?;

        match char_buf {
            [b'~', b':', b'*'] => Ok(Self {
                len: 3,
                value: TildeKind::Star(StarKind::Hop),
            }),
            [b'~', b'*', ..] => {
                c.seek(SeekFrom::Current(-1))?;
                Ok(Self {
                    len: 2,
                    value: TildeKind::Star(StarKind::Skip),
                })
            }
            _ => {
                c.seek(SeekFrom::Current(-3))?;
                return Err(
                    TildeError::new(ErrorKind::ParseError, "should start with ~* or ~:*").into(),
                );
            }
        }
    }

    fn parse_tildes(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)?;
        match char_buf {
            [b'~', n @ _, b'~'] => Ok(Self {
                len: 3,
                value: TildeKind::Tildes(String::from_utf8(vec![n])?.parse::<usize>()?),
            }),
            _ => {
                c.seek(SeekFrom::Current(-3))?;
                return Err(TildeError::new(ErrorKind::ParseError, "should start with ~n~").into());
            }
        }
    }

    fn parse_standard(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut buf = vec![];

        for t in [b's', b'S'] {
            c.read_until(t, &mut buf)?;
            match buf.last() {
                Some(b) if *b == t => return Ok(Tilde::new(buf.len(), TildeKind::Standard)),
                _ => (),
            }
            c.seek(SeekFrom::Current(-(buf.len() as i64)))?;
            buf.clear();
        }
        Err(TildeError::new(ErrorKind::ParseError, "cannot find the 's' or 'S'").into())
    }

    fn parse_char(c: &mut Cursor<&'_ str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut char_buf = [0u8; 3]; // three bytes
        c.read(&mut char_buf)?;

        match char_buf {
            [b'~', b'@', b'c' | b'C'] => Ok(Self {
                len: 3,
                value: TildeKind::Char(CharKind::At),
            }),
            [b'~', b'c' | b'C', ..] => {
                c.seek(SeekFrom::Current(-1))?;
                Ok(Self {
                    len: 2,
                    value: TildeKind::Char(CharKind::Nil),
                })
            }
            _ => {
                c.seek(SeekFrom::Current(-3))?;
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

use super::*;

//========================================
// TildeKindDigit
//========================================
multi_tilde_impl!(TildeKindDigit, [i32, i64, u32, u64, usize], self, {
    Ok(Some(self.to_string()))
});

//========================================
// TildeKindChar
//========================================
/// impl, re-define the format method for over writing the default method
impl TildeKindChar for char {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        match tkind {
            TildeKind::Char(CharKind::At) => Ok(Some(format!("'{}'", self))),
            TildeKind::Char(CharKind::Nil) => Ok(Some(self.to_string())),
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
    { Ok(Some(self.to_string())) }
);

impl TildeKindVa for bool {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        if *self {
            Ok(Some("true".into()))
        } else {
            Ok(Some("false".into()))
        }
    }
}

impl TildeKindVa for TildeNil {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        Ok(Some("nil".into()))
    }
}

impl TildeKindVa for Vec<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        Ok(Some(format!("{:?}", self)))
    }
}

//========================================
// TildeKindLoop
//========================================
impl<'a> TildeKindLoop for Args<'a> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        match tkind {
            // self[0] is the Vec<&dyn TildeAble> of loop
            TildeKind::Loop((_, TildeLoopKind::Nil | TildeLoopKind::NilColon)) => {
                let a = self
                    .pop()
                    .ok_or::<TildeError>(TildeError::new(ErrorKind::FormatError, "run out args"))?;
                tkind.match_reveal(a)
            }
            TildeKind::Loop((vv, TildeLoopKind::At)) => {
                let mut result = Vec::with_capacity(self.len());

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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Colon)) => {
                if *self {
                    vv.get(1)
                        .ok_or::<TildeError>(TildeError::new(
                            ErrorKind::FormatError,
                            "cannot get tilde",
                        ))?
                        .reveal(&TildeNil)
                } else {
                    vv.get(0)
                        .ok_or::<TildeError>(TildeError::new(
                            ErrorKind::FormatError,
                            "cannot get tilde",
                        ))?
                        .reveal(&TildeNil)
                }
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl TildeKindCond for Option<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
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
                let a = self
                    .pop()
                    .ok_or::<TildeError>(TildeError::new(ErrorKind::FormatError, "run out args"))?;
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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        match tkind {
            TildeKind::VecTilde(vv) => {
                let result = vv.iter().map(|t| t.reveal(self)).try_fold(
                    Vec::with_capacity(vv.len()),
                    |mut acc, ele| {
                        acc.push(ele?);
                        Ok(acc)
                    },
                )?;

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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        match tkind {
            TildeKind::VecTilde(vv) => {
                let result = vv.iter().map(|t| t.reveal(self)).try_fold(
                    Vec::with_capacity(vv.len()),
                    |mut acc, ele| {
                        acc.push(ele?);
                        Ok(acc)
                    },
                )?;

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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
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
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        Ok(Some(format!("\"{}\"", self)))
    }
}

impl TildeKindStandard for char {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
        Ok(Some(format!("'{}'", self)))
    }
}

multi_tilde_impl!(
    TildeKindStandard,
    [f32, f64, i32, i64, usize, bool, u32, u64],
    self,
    { Ok(Some(self.to_string())) }
);

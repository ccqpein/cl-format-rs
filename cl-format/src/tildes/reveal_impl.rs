use super::*;

//========================================
// TildeKindDigit
//========================================
multi_tilde_impl2!(
    TildeKindDigit,
    [i32, i64, u32, u64, usize],
    self,
    buf,
    format_to_digit,
    {
        buf.push_str(self.to_string().as_str());
        Ok(())
    }
);

//========================================
// TildeKindChar
//========================================
/// impl, re-define the format method for over writing the default method
impl TildeKindChar for char {
    fn format_to_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Char(CharKind::At) => {
                buf.push_str(format!("'{}'", self).as_str());
                Ok(())
            }
            TildeKind::Char(CharKind::Nil) => {
                buf.push_str(self.to_string().as_str());
                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Char").into()),
        }
    }
}

//========================================
// TildeKindVa
//========================================
multi_tilde_impl2!(
    TildeKindVa,
    [f32, f64, char, i32, i64, usize, u32, u64, String],
    self,
    buf,
    format_to_va,
    {
        buf.push_str(self.to_string().as_str());
        Ok(())
    }
);

impl TildeKindVa for bool {
    fn format_to_va(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        if *self {
            buf.push_str("true");
        } else {
            buf.push_str("false");
        }
        Ok(())
    }
}

impl TildeKindVa for TildeNil {
    fn format_to_va(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str("nil");
        Ok(())
    }
}

impl TildeKindVa for Vec<&dyn TildeAble2> {
    fn format_to_va(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(format!("{:?}", self).as_str());
        Ok(())
    }
}

//========================================
// TildeKindLoop
//========================================
impl<'a> TildeKindLoop for Args<'a> {
    fn format_to_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            // self[0] is the Vec<&dyn TildeAble> of loop
            TildeKind::Loop((_, TildeLoopKind::Nil | TildeLoopKind::NilColon)) => {
                let a = self
                    .pop()
                    .ok_or::<TildeError>(TildeError::new(ErrorKind::FormatError, "run out args"))?;
                tkind.match_reveal(a, buf)
            }
            TildeKind::Loop((vv, TildeLoopKind::At)) => {
                'a: loop {
                    for t in vv {
                        if let TildeKind::LoopEnd = t.value {
                            if self.left_count() != 0 {
                                continue;
                            } else {
                                break 'a;
                            }
                        }
                        t.reveal(self, buf)?;
                    }
                    //dbg!(self);
                    if self.left_count() == 0 {
                        break;
                    }
                }

                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Arg to Loop").into()),
        }
    }
}

impl<'a> TildeKindLoop for Vec<&dyn TildeAble2> {
    fn format_to_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Loop((_, TildeLoopKind::Nil)) => {
                let mut new_kind = tkind.clone();

                match &mut new_kind {
                    TildeKind::Loop((_, k @ TildeLoopKind::Nil)) => {
                        if self.len() != 0 {
                            *k = TildeLoopKind::At
                        } else {
                            return Ok(());
                        }
                    }
                    _ => unreachable!(),
                };
                new_kind.match_reveal(&Args::from(self), buf)
            }
            TildeKind::Loop((_, TildeLoopKind::NilColon)) => {
                let mut new_kind = tkind.clone();
                match &mut new_kind {
                    TildeKind::Loop((_, k @ TildeLoopKind::NilColon)) => *k = TildeLoopKind::At,
                    _ => unreachable!(),
                };
                new_kind.match_reveal(&Args::from(self), buf)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Vec to Loop").into()),
        }
    }
}

//========================================
// TildeKindCond
//========================================
impl TildeKindCond for usize {
    fn format_to_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        //dbg!(self);
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Nil(true))) => match vv.get(*self) {
                Some(tt) => {
                    tt.reveal(&TildeNil, buf)?;
                    Ok(())
                }
                None => {
                    let last = vv.len() - 1;
                    match vv.get(last) {
                        Some(tt) => {
                            tt.reveal(&TildeNil, buf)?;
                            Ok(())
                        }
                        None => Ok(()),
                    }
                }
            },
            TildeKind::Cond((vv, TildeCondKind::Nil(false))) => match vv.get(*self) {
                Some(tt) => {
                    tt.reveal(&TildeNil, buf)?;
                    Ok(())
                }
                None => Ok(()),
            },
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl TildeKindCond for bool {
    fn format_to_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Colon)) => {
                if *self {
                    vv.get(1)
                        .ok_or::<TildeError>(TildeError::new(
                            ErrorKind::FormatError,
                            "cannot get tilde",
                        ))?
                        .reveal(&TildeNil, buf)?;

                    Ok(())
                } else {
                    vv.get(0)
                        .ok_or::<TildeError>(TildeError::new(
                            ErrorKind::FormatError,
                            "cannot get tilde",
                        ))?
                        .reveal(&TildeNil, buf)?;

                    Ok(())
                }
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl TildeKindCond for Option<&dyn TildeAble2> {
    fn format_to_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::At)) => match self {
                Some(a) => {
                    //println!("here: {:?}", a);
                    let k = TildeKind::VecTilde(vv.clone());
                    // VecTilde need the vec
                    // TildeCondKind::At only accept one arg

                    k.match_reveal(&Args::from([*a]), buf)
                }
                None => Ok(()),
            },
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

impl<'a> TildeKindCond for Args<'a> {
    fn format_to_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::Sharp)) => {
                let l = self.left_count();
                if l >= vv.len() {
                    vv[vv.len() - 1].reveal(self, buf)?;
                    Ok(())
                } else {
                    vv[l].reveal(self, buf)?;
                    Ok(())
                }
            }
            TildeKind::Cond((_, _)) => {
                let a = self
                    .pop()
                    .ok_or::<TildeError>(TildeError::new(ErrorKind::FormatError, "run out args"))?;
                tkind.match_reveal(a, buf)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

//========================================
// TildeKindVecTilde
//========================================
impl TildeKindVecTilde for TildeNil {
    fn format_to_vectilde(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::VecTilde(vv) => {
                for v in vv {
                    v.reveal(self, buf)?;
                }

                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to VecTilde").into()),
        }
    }
}

impl<'a> TildeKindVecTilde for Args<'a> {
    fn format_to_vectilde(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::VecTilde(vv) => {
                for v in vv {
                    v.reveal(self, buf)?;
                }
                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to VecTilde").into()),
        }
    }
}

//========================================
// TildeKindStar
//========================================
impl<'a> TildeKindStar for Args<'a> {
    fn format_to_star(&self, tkind: &TildeKind, _buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Star(StarKind::Hop) => {
                self.back(); // back to last one, make it hop

                Ok(())
            }
            TildeKind::Star(StarKind::Skip) => {
                self.pop();
                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Star").into()),
        }
    }
}

//========================================
// TildeKindStandard
//========================================
impl TildeKindStandard for String {
    fn format_to_standard(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(format!("\"{}\"", self).as_str());
        Ok(())
    }
}

impl TildeKindStandard for char {
    fn format_to_standard(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(format!("'{}'", self).as_str());
        Ok(())
    }
}

multi_tilde_impl2!(
    TildeKindStandard,
    [f32, f64, i32, i64, usize, bool, u32, u64],
    self,
    buf,
    format_to_standard,
    {
        buf.push_str(self.to_string().as_str());
        Ok(())
    }
);

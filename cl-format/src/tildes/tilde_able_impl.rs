use std::cell::Cell;
use std::fmt::Write;

use super::*;

impl<'a, 'arg> TildeAble for Args<'a, 'arg> {
    fn len(&self) -> usize {
        self.left_count()
    }

    fn format_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Char"))?;
        a.format_char(tkind, buf)
    }

    fn format_float(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Float"))?;
        a.format_float(tkind, buf)
    }

    fn format_digit(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Digit"))?;
        a.format_digit(tkind, buf)
    }

    fn format_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Va"))?;
        a.format_va(tkind, buf)
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard"))?;
        a.format_standard(tkind, buf)
    }

    fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix"))?;
        a.format_radix(tkind, buf)
    }

    fn format_star(&self, tkind: &TildeKind, _buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Star(StarKind::Hop) => {
                self.back();
                Ok(())
            }
            TildeKind::Star(StarKind::Skip) => {
                self.pop();
                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Star").into()),
        }
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Loop((_, TildeLoopKind::Nil | TildeLoopKind::NilColon)) => {
                let a = self
                    .pop()
                    .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                a.format_loop(tkind, buf)
            }
            TildeKind::Loop((vv, TildeLoopKind::At)) => reveal_loop(vv, self, buf),
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Arg to Loop").into()),
        }
    }

    fn format_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
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
                    .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                a.format_cond(tkind, buf)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }

    fn format_vectilde(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
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

/// Zero-allocation cursor over a homogeneous slice `&[T]` where `T: TildeAble`.
#[derive(Debug)]
pub(super) struct SliceArgs<'a, T: TildeAble> {
    slice: &'a [T],
    ind: Cell<usize>,
}

impl<'a, T: TildeAble> SliceArgs<'a, T> {
    #[inline]
    pub(super) fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            ind: Cell::new(0),
        }
    }

    #[inline]
    fn pop(&self) -> Option<&'a dyn TildeAble> {
        let ind = self.ind.get();
        let r = self.slice.get(ind)?;
        self.ind.set(ind + 1);
        Some(r as &dyn TildeAble)
    }

    #[inline]
    fn back(&self) -> Option<&'a dyn TildeAble> {
        let i = match self.ind.get() {
            0 => return None,
            n => n - 1,
        };
        let r = self.slice.get(i)?;
        self.ind.set(i);
        Some(r as &dyn TildeAble)
    }

    #[inline]
    fn left_count(&self) -> usize {
        self.slice.len() - self.ind.get()
    }
}

impl<'a, T: TildeAble> TildeAble for SliceArgs<'a, T> {
    fn len(&self) -> usize {
        self.left_count()
    }

    fn format_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Char"))?;
        a.format_char(tkind, buf)
    }

    fn format_float(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Float"))?;
        a.format_float(tkind, buf)
    }

    fn format_digit(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Digit"))?;
        a.format_digit(tkind, buf)
    }

    fn format_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Va"))?;
        a.format_va(tkind, buf)
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard"))?;
        a.format_standard(tkind, buf)
    }

    fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        let a = self
            .pop()
            .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix"))?;
        a.format_radix(tkind, buf)
    }

    fn format_star(&self, tkind: &TildeKind, _buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Star(StarKind::Hop) => {
                self.back();
                Ok(())
            }
            TildeKind::Star(StarKind::Skip) => {
                self.pop();
                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Star").into()),
        }
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Loop((_, TildeLoopKind::Nil | TildeLoopKind::NilColon)) => {
                let a = self
                    .pop()
                    .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                a.format_loop(tkind, buf)
            }
            TildeKind::Loop((vv, TildeLoopKind::At)) => reveal_loop(vv, self, buf),
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Arg to Loop").into()),
        }
    }

    fn format_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
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
                    .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
                a.format_cond(tkind, buf)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }

    fn format_vectilde(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
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

/// Blanket implementation for references (`&str`, `&T`, `&[T]`, `&dyn TildeAble`, etc.)
impl<T: TildeAble + ?Sized> TildeAble for &T {
    fn len(&self) -> usize {
        (**self).len()
    }

    fn format_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_char(tkind, buf)
    }

    fn format_float(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_float(tkind, buf)
    }

    fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_radix(tkind, buf)
    }

    fn format_digit(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_digit(tkind, buf)
    }

    fn format_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_va(tkind, buf)
    }

    fn format_star(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_star(tkind, buf)
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_standard(tkind, buf)
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_loop(tkind, buf)
    }

    fn format_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_cond(tkind, buf)
    }

    fn format_vectilde(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_vectilde(tkind, buf)
    }
}

/// Generic implementation for `Option<T>` (works for `Option<i32>`, `Option<String>`, `Option<&dyn TildeAble>`, etc.)
impl<T: TildeAble> TildeAble for Option<T> {
    fn len(&self) -> usize {
        match self {
            Some(_) => 1,
            None => 0,
        }
    }

    fn format_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Cond((vv, TildeCondKind::At)) => match self {
                Some(a) => {
                    let arg_ref: &dyn TildeAble = a;
                    let args = Args::new_cow(std::slice::from_ref(&arg_ref));
                    for v in vv {
                        v.reveal(&args, buf)?;
                    }
                    Ok(())
                }
                None => Ok(()),
            },
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Cond").into()),
        }
    }
}

/// Generic implementation for slices `[T]` (and `&[T]` via `&T` blanket impl)
impl<T: TildeAble> TildeAble for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn format_va(&self, _tkind: &TildeKind, buf: &mut String) -> Result<()> {
        write!(buf, "{:?}", self)
            .map_err(|e| TildeError::new(ErrorKind::FormatError, e.to_string()).into())
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Loop((vv, TildeLoopKind::Nil)) => {
                if self.is_empty() {
                    return Ok(());
                }
                reveal_loop(vv, &SliceArgs::new(self), buf)
            }
            TildeKind::Loop((vv, TildeLoopKind::NilColon)) => {
                reveal_loop(vv, &SliceArgs::new(self), buf)
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format Vec to Loop").into()),
        }
    }
}

/// Generic implementation for `Vec<T>` (works for `Vec<i32>`, `Vec<String>`, `Vec<&dyn TildeAble>`, etc.)
impl<T: TildeAble> TildeAble for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn format_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.as_slice().format_va(tkind, buf)
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.as_slice().format_loop(tkind, buf)
    }
}

/// Generic implementation for fixed-size arrays `[T; N]`
impl<T: TildeAble, const N: usize> TildeAble for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn format_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.as_slice().format_va(tkind, buf)
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.as_slice().format_loop(tkind, buf)
    }
}

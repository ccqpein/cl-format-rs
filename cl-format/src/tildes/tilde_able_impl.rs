use std::fmt::Write;

use super::*;

/// Blanket implementation for references (`&str`, `&T`, `&[T]`, `&dyn TildeAble`, etc.)
impl<T: TildeAble + ?Sized> TildeAble for &T {
    fn len(&self) -> usize {
        (**self).len()
    }

    fn as_cond_index(&self) -> Option<usize> {
        (**self).as_cond_index()
    }

    fn as_cond_bool(&self) -> Option<bool> {
        (**self).as_cond_bool()
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

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_standard(tkind, buf)
    }

    fn format_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_loop(tkind, buf)
    }

    fn format_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        (**self).format_cond(tkind, buf)
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
                    let args = SliceArgs::new(std::slice::from_ref(a));
                    for v in vv {
                        if v.reveal(&args, buf)?.is_break() {
                            break;
                        }
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

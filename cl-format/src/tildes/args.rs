use std::borrow::Cow;
use std::cell::Cell;

use super::*;

/// The args for control string to use.
#[derive(Debug, Clone)]
pub struct Args<'a, 'arg> {
    len: usize,
    inner: Cow<'arg, [&'a dyn TildeAble]>,

    /// need mutate borrow in immutate Args
    ind: Cell<usize>,
}

impl<'a, 'arg> Args<'a, 'arg> {
    pub fn new(i: Vec<&'a dyn TildeAble>) -> Self {
        Self {
            len: i.len(),
            inner: Cow::Owned(i),
            ind: Cell::new(0),
        }
    }

    pub fn new_cow(i: &'arg [&'a dyn TildeAble]) -> Self {
        Self {
            len: i.len(),
            inner: Cow::Borrowed(i),
            ind: Cell::new(0),
        }
    }

    #[inline]
    pub fn pop(&self) -> Option<&'a dyn TildeAble> {
        let ind = self.ind.get();
        let r = self.inner.get(ind)?;
        self.ind.set(ind + 1);
        Some(*r)
    }

    #[inline]
    pub fn back(&self) -> Option<&'a dyn TildeAble> {
        let i = match self.ind.get() {
            0 => return None,
            n => n - 1,
        };

        let r = self.inner.get(i)?;
        self.ind.set(i);
        Some(*r)
    }

    #[inline]
    pub fn left_count(&self) -> usize {
        self.len - self.ind.get()
    }

    #[inline]
    pub fn reset(&self) {
        self.ind.set(0);
    }
}

/// Trait for argument cursors used during format string revealing.
///
/// Supports both homogeneous cursors (`Args`, `SliceArgs<T>`) and heterogeneous
/// compile-time tuple cursors (`TupleArgs<(T0, T1, ...)>`) without requiring `dyn` fat pointers.
pub trait ArgCursor {
    fn skip(&self) -> bool;
    fn back(&self) -> bool;
    fn left_count(&self) -> usize;

    fn pop_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_float(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_digit(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()>;
    fn pop_cond_index(&self) -> Result<usize>;
    fn pop_cond_bool(&self) -> Result<bool>;
}

macro_rules! impl_cursor_methods_from_pop {
    () => {
        #[inline]
        fn skip(&self) -> bool {
            self.pop().is_some()
        }

        #[inline]
        fn back(&self) -> bool {
            self.back_item().is_some()
        }

        #[inline]
        fn left_count(&self) -> usize {
            self.remaining()
        }

        #[inline]
        fn pop_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Char"))?;
            a.format_char(tkind, buf)
        }

        #[inline]
        fn pop_float(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Float"))?;
            a.format_float(tkind, buf)
        }

        #[inline]
        fn pop_digit(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Digit"))?;
            a.format_digit(tkind, buf)
        }

        #[inline]
        fn pop_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Va"))?;
            a.format_va(tkind, buf)
        }

        #[inline]
        fn pop_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self.pop().ok_or_else(|| {
                TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard")
            })?;
            a.format_standard(tkind, buf)
        }

        #[inline]
        fn pop_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix"))?;
            a.format_radix(tkind, buf)
        }

        #[inline]
        fn pop_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
            a.format_loop(tkind, buf)
        }

        #[inline]
        fn pop_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
            a.format_cond(tkind, buf)
        }

        #[inline]
        fn pop_cond_index(&self) -> Result<usize> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
            a.as_cond_index().ok_or_else(|| {
                TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond").into()
            })
        }

        #[inline]
        fn pop_cond_bool(&self) -> Result<bool> {
            let a = self
                .pop()
                .ok_or_else(|| TildeError::new(ErrorKind::FormatError, "run out args"))?;
            a.as_cond_bool().ok_or_else(|| {
                TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond").into()
            })
        }
    };
}

impl<'a, 'arg> Args<'a, 'arg> {
    #[inline]
    fn back_item(&self) -> Option<&'a dyn TildeAble> {
        Args::back(self)
    }

    #[inline]
    fn remaining(&self) -> usize {
        Args::left_count(self)
    }
}

impl<'a, 'arg> ArgCursor for Args<'a, 'arg> {
    impl_cursor_methods_from_pop!();
}

/// Zero-allocation cursor over a homogeneous slice `&[T]` where `T: TildeAble`.
#[derive(Debug)]
pub struct SliceArgs<'a, T: TildeAble> {
    slice: &'a [T],
    ind: Cell<usize>,
}

impl<'a, T: TildeAble> SliceArgs<'a, T> {
    #[inline]
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            ind: Cell::new(0),
        }
    }

    #[inline]
    fn pop(&self) -> Option<&'a T> {
        let ind = self.ind.get();
        let r = self.slice.get(ind)?;
        self.ind.set(ind + 1);
        Some(r)
    }

    #[inline]
    fn back_item(&self) -> Option<&'a T> {
        let i = match self.ind.get() {
            0 => return None,
            n => n - 1,
        };
        let r = self.slice.get(i)?;
        self.ind.set(i);
        Some(r)
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.slice.len() - self.ind.get()
    }
}

impl<'a, T: TildeAble> ArgCursor for SliceArgs<'a, T> {
    impl_cursor_methods_from_pop!();
}

/// Zero-`dyn` cursor over a heterogeneous tuple `(T0, T1, ...)`.
#[derive(Debug, Clone)]
pub struct TupleArgs<T> {
    tuple: T,
    ind: Cell<usize>,
}

impl<T> TupleArgs<T> {
    #[inline]
    pub fn new(tuple: T) -> Self {
        Self {
            tuple,
            ind: Cell::new(0),
        }
    }
}

impl ArgCursor for TupleArgs<()> {
    #[inline]
    fn skip(&self) -> bool {
        false
    }
    #[inline]
    fn back(&self) -> bool {
        false
    }
    #[inline]
    fn left_count(&self) -> usize {
        0
    }
    #[inline]
    fn pop_char(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Char").into())
    }
    #[inline]
    fn pop_float(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Float").into())
    }
    #[inline]
    fn pop_digit(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Digit").into())
    }
    #[inline]
    fn pop_va(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into())
    }
    #[inline]
    fn pop_standard(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard").into())
    }
    #[inline]
    fn pop_radix(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix").into())
    }
    #[inline]
    fn pop_loop(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::FormatError, "run out args").into())
    }
    #[inline]
    fn pop_cond(&self, _: &TildeKind, _: &mut String) -> Result<()> {
        Err(TildeError::new(ErrorKind::FormatError, "run out args").into())
    }
    #[inline]
    fn pop_cond_index(&self) -> Result<usize> {
        Err(TildeError::new(ErrorKind::FormatError, "run out args").into())
    }
    #[inline]
    fn pop_cond_bool(&self) -> Result<bool> {
        Err(TildeError::new(ErrorKind::FormatError, "run out args").into())
    }
}

macro_rules! impl_tuple_arg_cursor {
    ($len:expr; $(($idx:tt, $T:ident)),+) => {
        impl<$($T: TildeAble),+> ArgCursor for TupleArgs<($($T,)+)> {
            #[inline]
            fn skip(&self) -> bool {
                let ind = self.ind.get();
                if ind < $len {
                    self.ind.set(ind + 1);
                    true
                } else {
                    false
                }
            }

            #[inline]
            fn back(&self) -> bool {
                match self.ind.get() {
                    0 => false,
                    n => {
                        self.ind.set(n - 1);
                        true
                    }
                }
            }

            #[inline]
            fn left_count(&self) -> usize {
                $len - self.ind.get()
            }

            #[inline]
            fn pop_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_char(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Char").into()),
                }
            }

            #[inline]
            fn pop_float(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_float(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Float").into()),
                }
            }

            #[inline]
            fn pop_digit(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_digit(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Digit").into()),
                }
            }

            #[inline]
            fn pop_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_va(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Va").into()),
                }
            }

            #[inline]
            fn pop_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_standard(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Standard").into()),
                }
            }

            #[inline]
            fn pop_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_radix(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::RevealError, "cannot reveal to Radix").into()),
                }
            }

            #[inline]
            fn pop_loop(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_loop(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::FormatError, "run out args").into()),
                }
            }

            #[inline]
            fn pop_cond(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.format_cond(tkind, buf)
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::FormatError, "run out args").into()),
                }
            }

            #[inline]
            fn pop_cond_index(&self) -> Result<usize> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.as_cond_index().ok_or_else(|| {
                                TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond").into()
                            })
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::FormatError, "run out args").into()),
                }
            }

            #[inline]
            fn pop_cond_bool(&self) -> Result<bool> {
                let ind = self.ind.get();
                match ind {
                    $(
                        $idx => {
                            self.ind.set(ind + 1);
                            self.tuple.$idx.as_cond_bool().ok_or_else(|| {
                                TildeError::new(ErrorKind::RevealError, "cannot reveal to Cond").into()
                            })
                        }
                    )+
                    _ => Err(TildeError::new(ErrorKind::FormatError, "run out args").into()),
                }
            }
        }
    };
}

impl_tuple_arg_cursor!(1; (0, T0));
impl_tuple_arg_cursor!(2; (0, T0), (1, T1));
impl_tuple_arg_cursor!(3; (0, T0), (1, T1), (2, T2));
impl_tuple_arg_cursor!(4; (0, T0), (1, T1), (2, T2), (3, T3));
impl_tuple_arg_cursor!(5; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4));
impl_tuple_arg_cursor!(6; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5));
impl_tuple_arg_cursor!(7; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6));
impl_tuple_arg_cursor!(8; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7));
impl_tuple_arg_cursor!(9; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8));
impl_tuple_arg_cursor!(10; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9));
impl_tuple_arg_cursor!(11; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9), (10, T10));
impl_tuple_arg_cursor!(12; (0, T0), (1, T1), (2, T2), (3, T3), (4, T4), (5, T5), (6, T6), (7, T7), (8, T8), (9, T9), (10, T10), (11, T11));

impl<'a, const N: usize> From<[&'a dyn TildeAble; N]> for Args<'a, '_> {
    fn from(value: [&'a dyn TildeAble; N]) -> Self {
        Self::new(value.to_vec())
    }
}

impl<'a, 'arg> From<&'arg [&'a dyn TildeAble]> for Args<'a, 'arg> {
    fn from(value: &'arg [&'a dyn TildeAble]) -> Self {
        Self::new_cow(value)
    }
}

impl<'a> From<Vec<&'a dyn TildeAble>> for Args<'a, '_> {
    fn from(value: Vec<&'a dyn TildeAble>) -> Self {
        Self::new(value)
    }
}

impl<'a, 'arg, T> From<&'arg T> for Args<'a, 'arg>
where
    T: Deref<Target = [&'a dyn TildeAble]>,
{
    fn from(value: &'arg T) -> Self {
        Self::new_cow(value.deref())
    }
}

impl<'a, 'arg> IntoIterator for Args<'a, 'arg> {
    type Item = &'a dyn TildeAble;

    type IntoIter = std::vec::IntoIter<&'a dyn TildeAble>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_owned().into_iter()
    }
}

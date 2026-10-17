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
    pub fn pop(&self) -> Option<&dyn TildeAble> {
        let ind = self.ind.get();
        let r = self.inner.get(ind)?;
        self.ind.set(ind + 1);
        Some(*r)
    }

    #[inline]
    pub fn back(&self) -> Option<&dyn TildeAble> {
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

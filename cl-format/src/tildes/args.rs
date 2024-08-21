use std::borrow::Cow;

use super::*;

/// The args for control string to use.
#[derive(Debug, Clone)]
pub struct Args<'a, 'arg> {
    len: usize,
    inner: Cow<'arg, [&'a dyn TildeAble]>,

    /// need mutate borrow in immutate Args
    ind: RefCell<usize>,
}

impl<'a, 'arg> Args<'a, 'arg> {
    pub fn new(i: Vec<&'a dyn TildeAble>) -> Self {
        Self {
            len: i.len(),
            inner: Cow::Owned(i),
            ind: RefCell::new(0),
        }
    }

    pub fn new_cow(i: &'arg [&'a dyn TildeAble]) -> Self {
        Self {
            len: i.len(),
            inner: Cow::Borrowed(i),
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

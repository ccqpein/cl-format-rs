use super::*;

/// The args for control string to use.
#[derive(Debug, Clone)]
pub struct Args<'a> {
    len: usize,
    inner: Vec<&'a dyn TildeAble>,

    /// need mutate borrow in immutate Args
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

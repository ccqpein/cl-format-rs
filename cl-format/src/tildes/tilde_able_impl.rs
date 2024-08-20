use super::*;

impl<'a, 'arg> TildeAble for Args<'a, 'arg> {
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

    fn into_tildekind_radix(&self) -> Option<&dyn TildeKindRadix> {
        match self.pop() {
            Some(a) => a.into_tildekind_radix(),
            None => None,
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

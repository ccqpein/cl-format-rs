use radix_fmt::{radix, Radix};
use std::fmt::Write;
use std::iter::{self, successors};

use super::*;

//========================================
// TildeKindDigit
//========================================
multi_tilde_impl!(TildeKindDigit, [i32, i64, u32, u64, usize], self, buf, {
    buf.push_str(self.to_string().as_str());
    Ok(())
});

//========================================
// TildeKindChar
//========================================
/// impl, re-define the format method for over writing the default method
impl TildeKindChar for char {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
multi_tilde_impl!(
    TildeKindVa,
    [f32, f64, char, i32, i64, usize, u32, u64, String],
    self,
    buf,
    {
        buf.push_str(self.to_string().as_str());
        Ok(())
    }
);

impl TildeKindVa for bool {
    fn format(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        if *self {
            buf.push_str("true");
        } else {
            buf.push_str("false");
        }
        Ok(())
    }
}

impl TildeKindVa for TildeNil {
    fn format(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str("nil");
        Ok(())
    }
}

impl TildeKindVa for Vec<&dyn TildeAble> {
    fn format(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(format!("{:?}", self).as_str());
        Ok(())
    }
}

//========================================
// TildeKindLoop
//========================================
impl<'a> TildeKindLoop for Args<'a> {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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

impl<'a> TildeKindLoop for Vec<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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

impl TildeKindCond for Option<&dyn TildeAble> {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, tkind: &TildeKind, _buf: &mut String) -> Result<(), TildeError> {
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
    fn format(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(format!("\"{}\"", self).as_str());
        Ok(())
    }
}

impl TildeKindStandard for char {
    fn format(&self, _: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(format!("'{}'", self).as_str());
        Ok(())
    }
}

multi_tilde_impl!(
    TildeKindStandard,
    [f32, f64, i32, i64, usize, bool, u32, u64],
    self,
    buf,
    {
        buf.push_str(self.to_string().as_str());
        Ok(())
    }
);

//========================================
// TildeKindRadix
//========================================
const NUMERALS: [(usize, [&'static str; 10]); 4] = [
    (
        1000,
        ["", "M", "MM", "MMM", "--", "-", "--", "---", "----", "--"],
    ),
    (
        100,
        ["", "C", "CC", "CCC", "CD", "D", "DC", "DCC", "DCCC", "CM"],
    ),
    (
        10,
        ["", "X", "XX", "XXX", "XL", "L", "LX", "LXX", "LXXX", "XC"],
    ),
    (
        1,
        ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX"],
    ),
];

/// make roman
fn into_roman(n: usize) -> Result<String, TildeError> {
    if n > 3999 {
        return Err(TildeError::new(
            ErrorKind::FormatError,
            "number is too big to reveal as roman numerals",
        ));
    }
    Ok(NUMERALS
        .iter()
        .map(|&(base, nums)| nums[(n / base) % 10])
        .collect())
}

/// make number
const ONES: [&str; 20] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];

const ORDINAL_ONES: [&str; 20] = [
    "zeroth",
    "first",
    "second",
    "third",
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
    "fourteenth",
    "fifteenth",
    "sixteenth",
    "seventeenth",
    "eighteenth",
    "nineteenth",
];

const TENS: [&str; 10] = [
    "zero", "ten", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

const ORDINAL_TENS: [&str; 10] = [
    "zeroth",
    "tenth",
    "twentieth",
    "thirtieth",
    "fortieth",
    "fiftieth",
    "sixtieth",
    "seventieth",
    "eightieth",
    "ninetieth",
];

const ORDERS: [&str; 7] = [
    "zero",
    "thousand",
    "million",
    "billion",
    "trillion",
    "quadrillion",
    "quintillion", // enough for u64::MAX
];

const ORDINAL_ORDERS: [&str; 7] = [
    "zeroth",
    "thousandth",
    "millionth",
    "billionth",
    "trillionth",
    "quadrillionth",
    "quintillionth", // enough for u64::MAX
];

pub fn into_english(num: usize, buf: &mut String) {
    match num {
        0..=19 => {
            buf.push_str(ONES[num as usize]);
        }
        20..=99 => {
            let upper = (num / 10) as usize;
            match num % 10 {
                0 => buf.push_str(TENS[upper]),
                lower => {
                    buf.push_str(TENS[upper]);
                    buf.push_str("-");
                    into_english(lower, buf);
                }
            }
        }
        100..=999 => format_num(num, 100, "hundred", buf),
        _ => {
            let (div, order) = successors(Some(1_usize), |v| v.checked_mul(1000_usize))
                .zip(ORDERS.iter())
                .find(|&(e, _)| e > num / 1000)
                .unwrap();

            format_num(num, div, order, buf)
        }
    }
}

fn format_num(num: usize, div: usize, order: &str, buf: &mut String) {
    match (num / div, num % div) {
        (upper, 0) => {
            into_english(upper, buf);
            buf.push_str(" ");
            buf.push_str(order)
        }
        (upper, lower) => {
            into_english(upper, buf);
            buf.push_str(" ");
            buf.push_str(order);
            buf.push_str(" ");
            into_english(lower, buf);
        }
    }
}

pub fn into_ordinal_english(num: usize, buf: &mut String) {
    match num {
        0..=19 => {
            buf.push_str(ORDINAL_ONES[num as usize]);
        }
        20..=99 => {
            let upper = (num / 10) as usize;
            match num % 10 {
                0 => buf.push_str(ORDINAL_TENS[upper]),
                lower => {
                    buf.push_str(TENS[upper]);
                    buf.push_str("-");
                    into_ordinal_english(lower, buf);
                }
            }
        }
        100..=999 => format_ordinal_num(num, 100, "hundred", buf),
        _ => {
            let (div, order) = successors(Some(1_usize), |v| v.checked_mul(1000_usize))
                .zip(ORDERS.iter())
                .find(|&(e, _)| e > num / 1000)
                .unwrap();
            //dbg!(&div);
            //dbg!(&order);
            format_ordinal_num(num, div, order, buf)
        }
    }
}

fn orders_to_ordinal(order: &str) -> &str {
    ORDINAL_ORDERS[ORDERS.iter().position(|s| *s == order).unwrap()]
}

fn format_ordinal_num(num: usize, div: usize, order: &str, buf: &mut String) {
    //dbg!(&num);
    //dbg!(&div);
    match (num / div, num % div) {
        (upper, 0) => {
            into_english(upper, buf);
            buf.push_str(" ");
            buf.push_str(orders_to_ordinal(order))
        }
        (upper, lower) => {
            into_english(upper, buf);
            buf.push_str(" ");
            buf.push_str(order);
            buf.push_str(" ");
            into_ordinal_english(lower, buf);
        }
    }
}

fn format_helper(
    buf: &mut String,
    orginal: String,
    mincol: &Option<usize>,
    padchar: &Option<char>,
    commachar: &Option<char>,
    comma_interval: &Option<usize>,
    flag: &Option<RadixFlag>,
) -> Result<(), TildeError> {
    let mut inner_buf = String::with_capacity(orginal.len());
    if let Some(RadixFlag::Colon) = flag {
        match comma_interval {
            Some(n) => {
                let commachar = match commachar {
                    Some(c) => *c,
                    None => ',',
                };

                if orginal.len() > *n {
                    let mut edge = orginal.len() % n;
                    if edge == 0 {
                        edge = *n;
                    };
                    inner_buf.push_str(&orginal[0..edge]);
                    loop {
                        if edge >= orginal.len() {
                            break;
                        }
                        inner_buf.push(commachar);
                        inner_buf.push_str(&orginal[edge..edge + n]);
                        edge += n;
                    }
                } else {
                    inner_buf.push_str(&orginal);
                }
            }
            None => (),
        }
    } else {
        inner_buf = orginal
    }

    match mincol {
        Some(n) => {
            if *n > inner_buf.len() {
                buf.push_str(
                    &iter::repeat(padchar.unwrap_or(' '))
                        .take(*n - inner_buf.len())
                        .collect::<String>(),
                );
                buf.push_str(&inner_buf);
            }
        }
        None => buf.push_str(&inner_buf),
    }

    Ok(())
}

impl TildeKindRadix for i32 {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        match tkind {
            TildeKind::Radix((ra, mincol, padchar, commachar, comma_interval, flag)) => {
                match (ra, mincol, padchar, commachar, comma_interval, flag) {
                    (ra, None, None, None, None, None) => {
                        if ra.is_none() {
                            // ~R
                            if *self < 0 {
                                buf.push_str("negative");
                                into_english(self.abs() as usize, buf)
                            } else {
                                into_english(self.abs() as usize, buf)
                            }
                        } else {
                            // ~xR
                            if *self < 0 {
                                write!(buf, "-{}", Radix::new(-*self, ra.unwrap())).map_err(
                                    |e| TildeError::new(ErrorKind::FormatError, e.to_string()),
                                )?
                            } else {
                                write!(buf, "{}", Radix::new(*self, ra.unwrap())).map_err(|e| {
                                    TildeError::new(ErrorKind::FormatError, e.to_string())
                                })?
                            }
                        }
                    }
                    (ra, None, None, None, None, Some(RadixFlag::Colon)) => {
                        if ra.is_none() {
                            // ~:R
                            if *self < 0 {
                                buf.push_str("negative");
                                into_ordinal_english(self.abs() as usize, buf)
                            } else {
                                into_ordinal_english(self.abs() as usize, buf)
                            }
                        } else {
                            // ~x:R
                            // ~x:R == ~x,,,',,3:R
                            if *self < 0 {
                                buf.push('-');
                                let s = Radix::new(-*self, ra.unwrap()).to_string();
                                format_helper(buf, s, mincol, padchar, &Some(','), &Some(3), flag)?;
                            } else {
                                let s = Radix::new(-*self, ra.unwrap()).to_string();
                                format_helper(buf, s, mincol, padchar, &Some(','), &Some(3), flag)?;
                            }
                        }
                    }

                    (None, None, None, None, None, Some(RadixFlag::At)) => {
                        // ~@R
                        if *self <= 0 {
                            return Err(TildeError::new(
                                ErrorKind::FormatError,
                                "negative cannot be roman numerals",
                            ));
                        } else {
                            buf.push_str(&into_roman(*self as usize)?);
                        }
                    }

                    (None, None, None, None, None, Some(RadixFlag::AtColon)) => {
                        return Err(TildeError::new(
                            ErrorKind::FormatError,
                            "old Roman numeral haven't supported yet",
                        ));
                    }

                    (ra, _, _, _, _, _) => {
                        if *self < 0 {
                            buf.push('-');
                            let s = Radix::new(-*self, ra.unwrap()).to_string();
                            format_helper(
                                buf,
                                s,
                                mincol,
                                padchar,
                                commachar,
                                comma_interval,
                                flag,
                            )?;
                        } else {
                            let s = Radix::new(-*self, ra.unwrap()).to_string();
                            format_helper(
                                buf,
                                s,
                                mincol,
                                padchar,
                                commachar,
                                comma_interval,
                                flag,
                            )?;
                        }
                    }
                }
            }
            _ => {
                return Err(
                    TildeError::new(ErrorKind::RevealError, "cannot format to Radix").into(),
                )
            }
        }
        Ok(())
    }
}

//:= Next

impl TildeKindRadix for i64 {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        Err(TildeError::new(ErrorKind::EmptyImplenmentError, "haven't implenmented yet").into())
    }
}

impl TildeKindRadix for u32 {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        Err(TildeError::new(ErrorKind::EmptyImplenmentError, "haven't implenmented yet").into())
    }
}

impl TildeKindRadix for u64 {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        Err(TildeError::new(ErrorKind::EmptyImplenmentError, "haven't implenmented yet").into())
    }
}

impl TildeKindRadix for usize {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        Err(TildeError::new(ErrorKind::EmptyImplenmentError, "haven't implenmented yet").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_english() {
        let mut buf = String::new();
        into_english(12345, &mut buf);
        assert_eq!(
            buf,
            String::from("twelve thousand three hundred forty-five")
        );

        let mut buf = String::new();
        into_english(0, &mut buf);
        assert_eq!(buf, String::from("zero"));

        let mut buf = String::new();
        into_english(1000000000000000001, &mut buf);
        assert_eq!(buf, String::from("one quintillion one"));

        let mut buf = String::new();
        into_english(2132314453234, &mut buf);
        assert_eq!(buf, String::from("two trillion one hundred thirty-two billion three hundred fourteen million four hundred fifty-three thousand two hundred thirty-four"));
    }

    #[test]
    fn test_into_ordinal_english() {
        let mut buf = String::new();
        into_ordinal_english(55, &mut buf);

        assert_eq!(buf, String::from("fifty-fifth"));

        let mut buf = String::new();
        into_ordinal_english(345, &mut buf);

        assert_eq!(buf, String::from("three hundred forty-fifth"));

        let mut buf = String::new();
        into_ordinal_english(12345, &mut buf);

        assert_eq!(
            buf,
            String::from("twelve thousand three hundred forty-fifth")
        );

        let mut buf = String::new();
        into_ordinal_english(1000000000000000, &mut buf);
        assert_eq!(buf, String::from("one quadrillionth"));

        let mut buf = String::new();
        into_ordinal_english(1000000000000000001, &mut buf);
        assert_eq!(buf, String::from("one quintillion first"));

        let mut buf = String::new();
        into_ordinal_english(2132314453234, &mut buf);
        assert_eq!(buf, String::from("two trillion one hundred thirty-two billion three hundred fourteen million four hundred fifty-three thousand two hundred thirty-fourth"));
    }

    #[test]
    fn test_radix_format() {
        let mut buf = String::new();
        assert!(write!(buf, "{}", Radix::new(17, 3)).is_ok());
        assert_eq!(buf, String::from("122"));

        let mut buf = String::new();
        assert!(write!(buf, "{}", Radix::new(345, 2)).is_ok());
        assert_eq!(buf, String::from("101011001"));
    }

    #[test]
    fn test_format_helper() {
        let mut buf = String::new();
        assert!(format_helper(
            &mut buf,
            Radix::new(345, 2).to_string(),
            &None,
            &None,
            &Some(','),
            &Some(3),
            &Some(RadixFlag::Colon),
        )
        .is_ok());
        assert_eq!(buf, String::from("101,011,001"));

        let mut buf = String::new();
        assert!(format_helper(
            &mut buf,
            Radix::new(345, 2).to_string(),
            &None,
            &None,
            &Some(','),
            &Some(3),
            &None,
        )
        .is_ok());
        assert_eq!(buf, String::from("101011001"));

        let mut buf = String::new();
        assert!(format_helper(
            &mut buf,
            Radix::new(345, 3).to_string(),
            &None,
            &None,
            &Some(','),
            &Some(3),
            &Some(RadixFlag::Colon),
        )
        .is_ok());
        assert_eq!(buf, String::from("110,210"));

        let mut buf = String::new();
        assert!(format_helper(
            &mut buf,
            Radix::new(17, 3).to_string(),
            &None,
            &None,
            &Some(','),
            &Some(2),
            &Some(RadixFlag::Colon),
        )
        .is_ok());
        assert_eq!(buf, String::from("1,22"));

        let mut buf = String::new();
        assert!(format_helper(
            &mut buf,
            Radix::new(17, 10).to_string(),
            &None,
            &None,
            &Some(','),
            &Some(2),
            &Some(RadixFlag::Colon),
        )
        .is_ok());
        assert_eq!(buf, String::from("17"));

        let mut buf = String::new();
        assert!(format_helper(
            &mut buf,
            Radix::new(17, 10).to_string(),
            &Some(5),
            &Some('a'),
            &Some(','),
            &Some(2),
            &Some(RadixFlag::Colon),
        )
        .is_ok());
        assert_eq!(buf, String::from("aaa17"));
    }
}

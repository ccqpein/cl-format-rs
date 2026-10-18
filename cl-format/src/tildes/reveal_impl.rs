use radix_fmt::Radix;
use std::fmt::Write;
use std::iter::{self, successors};

use super::*;

#[inline]
fn push_u64(mut n: u64, buf: &mut String) {
    if n < 10 {
        buf.push((b'0' + n as u8) as char);
        return;
    }
    let mut tmp = [0u8; 20];
    let mut i = 20;
    while n > 0 {
        i -= 1;
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.push_str(unsafe { std::str::from_utf8_unchecked(&tmp[i..]) });
}

#[inline]
fn push_i64(n: i64, buf: &mut String) {
    if n < 0 {
        buf.push('-');
        push_u64(n.unsigned_abs(), buf);
    } else {
        push_u64(n as u64, buf);
    }
}

#[inline]
fn push_u128(mut n: u128, buf: &mut String) {
    if n <= u64::MAX as u128 {
        push_u64(n as u64, buf);
        return;
    }
    let mut tmp = [0u8; 39];
    let mut i = 39;
    while n > 0 {
        i -= 1;
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.push_str(unsafe { std::str::from_utf8_unchecked(&tmp[i..]) });
}

#[inline]
fn push_i128(n: i128, buf: &mut String) {
    if n < 0 {
        buf.push('-');
        push_u128(n.unsigned_abs(), buf);
    } else {
        push_u128(n as u128, buf);
    }
}

//========================================
// Loop helper
//========================================
pub(super) fn reveal_loop<C: ArgCursor>(vv: &[Tilde], args: &C, buf: &mut String) -> Result<()> {
    loop {
        for t in vv {
            if t.reveal(args, buf)?.is_break() {
                return Ok(());
            }
        }
        if args.left_count() == 0 {
            break;
        }
    }

    Ok(())
}

//========================================
// TildeAble for char, String, str, bool, floats, ()
//========================================
impl TildeAble for char {
    fn format_char(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        match tkind {
            TildeKind::Char(CharKind::At) => {
                buf.push('\'');
                buf.push(*self);
                buf.push('\'');
                Ok(())
            }
            TildeKind::Char(CharKind::Nil) => {
                buf.push(*self);
                Ok(())
            }
            _ => Err(TildeError::new(ErrorKind::RevealError, "cannot format to Char").into()),
        }
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push(*self);
        Ok(())
    }

    fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push('\'');
        buf.push(*self);
        buf.push('\'');
        Ok(())
    }
}

impl TildeAble for str {
    fn len(&self) -> usize {
        str::len(self)
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push_str(self);
        Ok(())
    }

    fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push('"');
        buf.push_str(self);
        buf.push('"');
        Ok(())
    }
}

impl TildeAble for String {
    fn len(&self) -> usize {
        String::len(self)
    }

    fn format_va(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.as_str().format_va(tkind, buf)
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.as_str().format_standard(tkind, buf)
    }
}

impl TildeAble for bool {
    fn as_cond_bool(&self) -> Option<bool> {
        Some(*self)
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        if *self {
            buf.push_str("true");
        } else {
            buf.push_str("false");
        }
        Ok(())
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.format_va(tkind, buf)
    }
}

impl TildeAble for f32 {
    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        write!(buf, "{}", self)
            .map_err(|e| TildeError::new(ErrorKind::FormatError, e.to_string()).into())
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.format_va(tkind, buf)
    }
}

impl TildeAble for f64 {
    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        write!(buf, "{}", self)
            .map_err(|e| TildeError::new(ErrorKind::FormatError, e.to_string()).into())
    }

    fn format_standard(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        self.format_va(tkind, buf)
    }
}

impl TildeAble for () {
    fn len(&self) -> usize {
        0
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push_str("nil");
        Ok(())
    }
}

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
fn into_roman(n: usize) -> Result<String> {
    if n > 3999 {
        return Err(TildeError::new(
            ErrorKind::FormatError,
            "number is too big to reveal as roman numerals",
        )
        .into());
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
) -> Result<()> {
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
            } else {
                buf.push_str(&inner_buf);
            }
        }
        None => buf.push_str(&inner_buf),
    }

    Ok(())
}

fn format_radix_isize(val: isize, tkind: &TildeKind, buf: &mut String) -> Result<()> {
    match tkind {
        TildeKind::Radix((ra, mincol, padchar, commachar, comma_interval, flag)) => {
            match (ra, mincol, padchar, commachar, comma_interval, flag) {
                (ra, None, None, None, None, None) => {
                    if ra.is_none() {
                        // ~R
                        if val < 0 {
                            buf.push_str("negative ");
                            into_english(val.abs() as usize, buf)
                        } else {
                            into_english(val.abs() as usize, buf)
                        }
                    } else {
                        // ~xR
                        if val < 0 {
                            write!(buf, "-{}", Radix::new(-val, ra.unwrap()))
                                .map_err(|e| TildeError::new(ErrorKind::FormatError, e.to_string()))?
                        } else {
                            write!(buf, "{}", Radix::new(val, ra.unwrap()))
                                .map_err(|e| TildeError::new(ErrorKind::FormatError, e.to_string()))?
                        }
                    }
                }
                (ra, None, None, None, None, Some(RadixFlag::Colon)) => {
                    if ra.is_none() {
                        // ~:R
                        if val < 0 {
                            buf.push_str("negative ");
                            into_ordinal_english(val.abs() as usize, buf)
                        } else {
                            into_ordinal_english(val.abs() as usize, buf)
                        }
                    } else {
                        // ~x:R
                        // ~x:R == ~x,,,',,3:R
                        if val < 0 {
                            buf.push('-');
                            let s = Radix::new(-val, ra.unwrap()).to_string();
                            format_helper(buf, s, mincol, padchar, &Some(','), &Some(3), flag)?;
                        } else {
                            let s = Radix::new(val, ra.unwrap()).to_string();
                            format_helper(buf, s, mincol, padchar, &Some(','), &Some(3), flag)?;
                        }
                    }
                }

                (None, None, None, None, None, Some(RadixFlag::At)) => {
                    // ~@R
                    if val <= 0 {
                        return Err(TildeError::new(
                            ErrorKind::FormatError,
                            "negative cannot be roman numerals",
                        )
                        .into());
                    } else {
                        buf.push_str(&into_roman(val as usize)?);
                    }
                }

                (None, None, None, None, None, Some(RadixFlag::AtColon)) => {
                    return Err(TildeError::new(
                        ErrorKind::FormatError,
                        "old Roman numeral haven't supported yet",
                    )
                    .into());
                }

                (ra, _, _, _, _, _) => {
                    if val < 0 {
                        buf.push('-');
                        let s = Radix::new(-val, ra.unwrap()).to_string();
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
                        let s = Radix::new(val, ra.unwrap()).to_string();
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
            return Err(TildeError::new(ErrorKind::RevealError, "cannot format to Radix").into())
        }
    }
    Ok(())
}

fn format_radix_usize(val: usize, tkind: &TildeKind, buf: &mut String) -> Result<()> {
    match tkind {
        TildeKind::Radix((ra, mincol, padchar, commachar, comma_interval, flag)) => {
            match (ra, mincol, padchar, commachar, comma_interval, flag) {
                (ra, None, None, None, None, None) => {
                    if ra.is_none() {
                        // ~R
                        into_english(val, buf)
                    } else {
                        // ~xR
                        write!(buf, "{}", Radix::new(val, ra.unwrap()))
                            .map_err(|e| TildeError::new(ErrorKind::FormatError, e.to_string()))?
                    }
                }
                (ra, None, None, None, None, Some(RadixFlag::Colon)) => {
                    if ra.is_none() {
                        // ~:R
                        into_ordinal_english(val, buf)
                    } else {
                        // ~x:R
                        // ~x:R == ~x,,,',,3:R
                        let s = Radix::new(val, ra.unwrap()).to_string();
                        format_helper(buf, s, mincol, padchar, &Some(','), &Some(3), flag)?;
                    }
                }

                (None, None, None, None, None, Some(RadixFlag::At)) => {
                    // ~@R
                    buf.push_str(&into_roman(val)?);
                }

                (None, None, None, None, None, Some(RadixFlag::AtColon)) => {
                    return Err(TildeError::new(
                        ErrorKind::FormatError,
                        "old Roman numeral haven't supported yet",
                    )
                    .into());
                }

                (ra, _, _, _, _, _) => {
                    let s = Radix::new(val, ra.unwrap()).to_string();
                    format_helper(buf, s, mincol, padchar, commachar, comma_interval, flag)?;
                }
            }
        }
        _ => {
            return Err(TildeError::new(ErrorKind::RevealError, "cannot format to Radix").into())
        }
    }
    Ok(())
}

macro_rules! impl_tilde_able_signed {
    ($($t:ty),+) => {
        $(
            impl TildeAble for $t {
                fn format_digit(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
                    push_i64(*self as i64, buf);
                    Ok(())
                }

                fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
                    push_i64(*self as i64, buf);
                    Ok(())
                }

                fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
                    push_i64(*self as i64, buf);
                    Ok(())
                }

                fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                    format_radix_isize(*self as isize, tkind, buf)
                }
            }
        )+
    };
}

impl_tilde_able_signed!(i8, i16, i32, i64, isize);

impl TildeAble for i128 {
    fn format_digit(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_i128(*self, buf);
        Ok(())
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_i128(*self, buf);
        Ok(())
    }

    fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_i128(*self, buf);
        Ok(())
    }

    fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        format_radix_isize(*self as isize, tkind, buf)
    }
}

macro_rules! impl_tilde_able_unsigned {
    ($($t:ty),+) => {
        $(
            impl TildeAble for $t {
                fn format_digit(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
                    push_u64(*self as u64, buf);
                    Ok(())
                }

                fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
                    push_u64(*self as u64, buf);
                    Ok(())
                }

                fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
                    push_u64(*self as u64, buf);
                    Ok(())
                }

                fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
                    format_radix_usize(*self as usize, tkind, buf)
                }
            }
        )+
    };
}

impl_tilde_able_unsigned!(u8, u16, u32, u64);

impl TildeAble for usize {
    fn as_cond_index(&self) -> Option<usize> {
        Some(*self)
    }

    fn format_digit(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_u64(*self as u64, buf);
        Ok(())
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_u64(*self as u64, buf);
        Ok(())
    }

    fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_u64(*self as u64, buf);
        Ok(())
    }

    fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        format_radix_usize(*self, tkind, buf)
    }
}

impl TildeAble for u128 {
    fn format_digit(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_u128(*self, buf);
        Ok(())
    }

    fn format_va(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_u128(*self, buf);
        Ok(())
    }

    fn format_standard(&self, _: &TildeKind, buf: &mut String) -> Result<()> {
        push_u128(*self, buf);
        Ok(())
    }

    fn format_radix(&self, tkind: &TildeKind, buf: &mut String) -> Result<()> {
        format_radix_usize(*self as usize, tkind, buf)
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

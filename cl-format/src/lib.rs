#![feature(let_chains)]
#![feature(pattern)]
#![feature(rustc_attrs)]

mod control_str;
mod tildes;

pub use control_str::*;
pub use tildes::*;

/// multi_tilde_impl!(TildeKindVa, [float, char, String], self, {Err("un-implenmented yet".into())})
///
/// will expand
///
/// impl TildeKindVa for float{
///     fn format(&self, tkind: &TildeKind) -> Result<String, Box<dyn std::error::Error>> {
///         Err("un-implenmented yet".into())
///     }
/// }
/// ...
#[macro_export]
macro_rules! multi_tilde_impl {
    ($implName:ident, [$($y:ident),+], $s:ident,$body:block) => {
		$(
			impl $implName for $y {
				fn format(&$s, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>>
					$body

			}
		)+
    };
}

/// add the &dyn TildeAble to the expr
#[macro_export]
macro_rules! tilde {
    ($arg:expr) => {
        $arg as &dyn crate::TildeAble
    };
}

/// cl_format! should like vec! macro
///
/// cl_format!(control_str, &a, &b, &c) => {
///      let c = control_str::ControlStr::from("~a, ~a, ~a")?;
///      let a = Into::<
///             tildes::Args<'_>,
///         >::into([
///             &1 as &dyn tildes::TildeAble,
///             &2 as &dyn tildes::TildeAble,
///             &3 as &dyn tildes::TildeAble,
///         ]);
///         c.reveal(a)
/// }
#[macro_export]
macro_rules! cl_format {
	($control_str:expr) =>	{
		{
			let c = crate::ControlStr::from($control_str).expect("making control string has issue");
			let a = crate::Args::new(vec![]);
			c.reveal(a)
		}
	};
    ($control_str:expr, $($ele:expr),*) =>	{
		{
			let c = crate::ControlStr::from($control_str).expect("making control string has issue");
			let a = Into::<crate::Args<'_>>::into([$(tilde!($ele)),*]);
			c.reveal(a)
		}
	}

}

#[cfg(test)]
mod tests {}

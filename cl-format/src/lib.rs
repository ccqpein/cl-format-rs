#![doc = r#"`cl-format` s the Rust implementation of the Common Lisp [format](http://www.lispworks.com/documentation/lw50/CLHS/Body/f_format.htm) function.

## Usage ##

There are two ways to use this library. You can use the `cl_format!` macro, or generate the control string and format your arguments by yourself for more flexibility.

First, add `cl-format = "0.2"` in your `Cargo.toml`.

### Use macro ###

`~a` is the most common directive I like to use, so let's start from normal `~a`:

```rust
use cl_format::*;

let a = cl_format!("~a, ~a, ~a", &1_i32, &2, &3);
assert_eq!(String::from("1, 2, 3"), a.unwrap());
```

All arguments used for formatting have to be borrowed, and they must implement the `TildeAble` trait. Check the [Implement for custom type](#implement-for-custom-type) section for more details.

Here is more usage of the macro. Escaping the double quote symbol for strings:

```rust
use cl_format::*;
let s = String::from("abc");
let a = cl_format!("~a, ~a, ~a, ~S", &1_i32, &2, &3, &s);
assert_eq!(String::from("1, 2, 3, \"abc\""), a.unwrap());
```

Or not:

```rust
use cl_format::*;
let a = cl_format!("start ~a, ~a, ~a, ~a, here", &1_i32, &2, &3, &s);
assert_eq!(String::from("start 1, 2, 3, abc, here"), a.unwrap());
```

Let's make some loops inside the control string like Lispers do:

```rust
use cl_format::*;
let ll: Vec<&dyn TildeAble> = vec![&1, &2, &3];
let a = cl_format!("~a, ~a, ~a, ~{~a,~}", &1_i32, &2, &3, &ll);
assert_eq!(String::from("1, 2, 3, 1,2,3,"), a.unwrap());
```

Wait, we have an unnecessary comma at the end of the result, let's clean it up:

```rust
use cl_format::*;
let a = cl_format!("~a, ~a, ~a, ~{~a~^,~}", &1_i32, &2, &3, &ll);
assert_eq!(String::from("1, 2, 3, 1,2,3"), a.unwrap());
```

I suddenly don't want to loop the Vec anymore:

```rust
use cl_format::*;
let l = vec![&1 as &dyn TildeAble, &2, &3];
let a = cl_format!("The value is:\n ~a", &l);
assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());
```

Now, we have some inconsistency between Common Lisp and Rust. In Common Lisp, `~%` in the control string is the new line, but we are in Rust now, so `\n` is going to work.

I think I am a bit tired of showing the type as `&dyn TildeAble` to elements inside Vec. But I haven't found a way to avoid it yet. If you know, let me know. So I added some macros:


```rust
use cl_format::*;
let l = vec![tilde!(&1), &2, &3];
let a = cl_format!("The value is:\n ~a", &l);
assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());
```

As in Common Lisp, we can loop through all arguments instead of putting them inside a Vec:

```rust
use cl_format::*;
let a = cl_format!("~@{~a~^, ~}", &1, &2, &3);
assert_eq!(String::from("1, 2, 3"), a.unwrap());
```

Now, let's try some condition control (you can get the meaning of the condition control string in the `Conditional Formatting` chapter of [A Few FORMAT Recipes](https://gigamonkeys.com/book/a-few-format-recipes.html)):

```rust
use cl_format::*;
let l = vec![tilde!(&1), &2, &3];
let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
assert_eq!(String::from("1, 2, and 3"), a.unwrap());

let l = vec![tilde!(&1), &2, &3, &4];
let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
assert_eq!(String::from("1, 2, 3, and 4"), a.unwrap());
```

### Manually ###

Using macros will generate the control string instance every time. It might be wasteful if you are trying to use a control string everywhere because it is flexible enough for multiple uses.

We can generate it by ourselves:

```rust
let cs = cl_format::ControlStr::from("~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~}").unwrap();
```

Then we can generate the Args for the control string to reveal:

```rust
use cl_format::Args;
let mut list = vec![];
let args = Args::new(vec![&list]);
```

Let's use it several times by giving different lengths of arguments:

```rust
use cl_format::*;
use cl_format::cl_format;

// this equal cl_format!(cs, &list)
assert_eq!(cs.reveal(args).unwrap(), "".to_string());

list.push(&1);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1".to_string());

list.push(&2);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1 and 2".to_string());

list.push(&3);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1, 2, and 3".to_string());

list.push(&4);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1, 2, 3, and 4".to_string());
```

Let's try a mixed example: 

```rust
use cl_format::*;

let my_team = String::from("STeam");
let my_stars = vec![
    String::from("Adam Lambert"),
    String::from("Queen"),
    String::from("snoop dogg"),
];

let stars = my_stars
    .iter()
    .map(|s| tilde!(s))
    .collect::<Vec<&dyn TildeAble>>();
	
assert_eq!(
    String::from("my favorite team \"STeam\" will win the superbowl LVIII. And Adam Lambert, Queen, and snoop dogg will in half time show. And the scores should be 38:35"),
    cl_format!(
        "my favorite team ~S will win the superbowl ~@R. And ~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~} will in half time show. And the scores should be ~d:~d",
        &my_team,
        &58,
        &stars,
        &38,
        &35
    )
    .unwrap()
);

```

### Implement for custom type ###

So far, we have only shown the basic types. It would be better if we could make our type be revealed as well.

Here is a demo on how to implement:

```rust
use cl_format::*;

// has to derive to Debug
#[derive(Debug)]
struct MyStruct {
    a: usize,
    b: String,
}

impl TildeAble for MyStruct {
	// there are a lot methods inside, but not every of them
	// we are need.
	
	// ~a is good enough
	fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        Some(self)
    }
	
	// ~d just for show case
	fn into_tildekind_digit(&self) -> Option<&dyn TildeKindDigit> {
        Some(self)
    }
	
	// how many elements you want cl_format treat this type
	// 1 is enough. And this one has to implement
	fn len(&self) -> usize {
        1
    }
}

/// By now, your IDE should give you some errors, letting you implement `TildeKindVa` and `TildeKindDigit`. 

impl TildeKindVa for MyStruct {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(&format!("a: {}, b: {}", self.a, self.b));
        Ok(())
    }
}

impl TildeKindDigit for MyStruct {
    fn format(&self, tkind: &TildeKind, buf: &mut String) -> Result<(), TildeError> {
        buf.push_str(&format!("{}", self.a));
        Ok(())
    }
}
```

Now `MyStruct` can be used by `cl_format`, but as you guessed, only for `~a` and `~d`

```rust
use cl_format::*;

let s = MyStruct {
    a: 1,
    b: "b".to_string(),
};

assert_eq!("a: 1, b: b".to_string(), cl_format!("~a", &s).unwrap());
assert_eq!(
    "a: 1, b: b lalalal a: 1, b: b".to_string(),
    cl_format!("~a lalalal ~a", &s, &s).unwrap()
);

assert_eq!("1".to_string(), cl_format!("~d", &s).unwrap());
assert_eq!(
    "First: a: 1, b: b; Second: 1".to_string(),
    cl_format!("First: ~a; Second: ~d", &s, &s).unwrap()
);
```

## Format directives ##

This is the table of which directives have been implemented:

| tilde                     | rust type                                                                                    |
|:-------------------------:|:--------------------------------------------------------------------------------------------:|
| `~a`                      | f32, f64, char, i8, i16, i32, i64, i128, isize, bool, u8, u16, u32, u64, u128, usize, String |
| `~s`                      | f32, f64, char, i32, i64, usize, bool, u32, u64, String                                      |
| `~d`                      | i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize                               |
| `~C`                      | char                                                                                         |
| `~[~]` (normal condition) | bool, usize                                                                                  |
| `~R`                      | i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize                               |

"#]
#![feature(let_chains)]
#![feature(pattern)]

mod control_str;
mod tildes;

pub use control_str::*;
pub use tildes::*;

#[doc = r#"Helper macro for implementing type with specific Tilde traits

For example `multi_tilde_impl!(TildeKindVa, [float, char, String], self, {Err("un-implenmented yet".into())})`

will expand

```rust
impl TildeKindVa for float {
    fn format(&self, tkind: &TildeKind) -> Result<String, Box<dyn std::error::Error>> {
        Err("un-implenmented yet".into())
    }
}
```"#]
#[macro_export]
macro_rules! multi_tilde_impl {
    ($implName:ident, [$($y:ident),+], $s:ident, $buf:ident, $body:block) => {
		$(
			impl $implName for $y {
				fn format(&$s, _: &TildeKind, $buf: &mut String) -> Result<(), TildeError>
					$body

			}
		)+
    };
}

#[doc = r"Macro for adding `as &dyn crate::TildeAble` to the expr"]
#[macro_export]
macro_rules! tilde {
    ($arg:expr) => {
        $arg as &dyn cl_format::TildeAble
    };
}

#[doc = r#"`cl_format!` is the macro for quick using cl-format

```rust
cl_format!(control_str, &a, &b, &c) => {
     let c = control_str::ControlStr::new("~a, ~a, ~a").expect("making control string has issue");
     let a = Into::<
            tildes::Args<'_>,
        >::into([
            &1 as &dyn tildes::TildeAble,
            &2 as &dyn tildes::TildeAble,
            &3 as &dyn tildes::TildeAble,
        ]);
        c.reveal(a)
}
```
For example:

```rust
let l = vec![tilde!(&1), &2, &3, &4];
let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
assert_eq!(String::from("1, 2, 3, and 4"), a.unwrap());
```"#]
#[macro_export]
macro_rules! cl_format {
	($control_str:expr) =>	{
		{
			let c = cl_format::ControlStr::new($control_str).expect("making control string has issue");
			let a = cl_format::Args::new(vec![]);
			c.reveal(a)
		}
	};
    ($control_str:expr, $($ele:expr),*) =>	{
		{
			let c = cl_format::ControlStr::new($control_str).expect("making control string has issue");
			let a = Into::<Args<'_,'_>>::into([$(tilde!($ele)),*]);
			c.reveal(a)
		}
	}

}

#[cfg(test)]
mod tests {}

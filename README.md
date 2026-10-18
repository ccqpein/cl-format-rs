# cl-format #

[![Crates.io](https://img.shields.io/crates/v/cl-format.svg)](https://crates.io/crates/cl-format)

`cl-format` s the Rust implementation of the Common Lisp [format](http://www.lispworks.com/documentation/lw50/CLHS/Body/f_format.htm) function.

TL;DR: Use several directives like `~a ~{~}` and flexible condition and loop control strings to format the string from arguments.

Here are several pages for you to have a general idea if you are not familiar with Common Lisp:

+ [A Few FORMAT Recipes](https://gigamonkeys.com/book/a-few-format-recipes.html)
+ [Wiki: Format (Common Lisp)](https://en.wikipedia.org/wiki/Format_(Common_Lisp))

*CAUTION: I haven't implemented all format directives yet. I am working on it. See below to find those that have been done.*

*BTW: I am trying to copy the behaviors of the format function of Common Lisp, but there might be some compromises.*

- [Usage](#usage)
  - [Use macro](#use-macro)
  - [Manually](#manually)
  - [Implement for custom type](#implement-for-custom-type)
- [Format directives](#format-directives)

## Usage ##

There are two ways to use this library. You can use the `cl_format!` macro, or generate the control string and format your arguments by yourself for more flexibility.

First, add `cl-format = "0.2"` in your `Cargo.toml`.
 
### Use macro ###

`~a` is the most common directive I like to use, so let's start from normal `~a`:

```rust
let a = cl_format!("~a, ~a, ~a", &1_i32, &2, &3);
assert_eq!(String::from("1, 2, 3"), a.unwrap());
```

All arguments used for formatting have to be borrowed, and they must implement the `TildeAble` trait. Check the [Implement for custom type](#implement-for-custom-type) section for more details.

Here is more usage of the macro. Escaping the double quote symbol for strings:

```rust
let s = String::from("abc");
let a = cl_format!("~a, ~a, ~a, ~S", &1_i32, &2, &3, &s);
assert_eq!(String::from("1, 2, 3, \"abc\""), a.unwrap());
```

Or not:

```rust
let a = cl_format!("start ~a, ~a, ~a, ~a, here", &1_i32, &2, &3, &s);
assert_eq!(String::from("start 1, 2, 3, abc, here"), a.unwrap());
```

Let's make some loops inside the control string like Lispers do:

```rust
let ll = vec![1, 2, 3];
let a = cl_format!("~a, ~a, ~a, ~{~a,~}", &1_i32, &2, &3, &ll);
assert_eq!(String::from("1, 2, 3, 1,2,3,"), a.unwrap());
```

Wait, we have an unnecessary comma at the end of the result, let's clean it up:

```rust
let a = cl_format!("~a, ~a, ~a, ~{~a~^,~}", &1_i32, &2, &3, &ll);
assert_eq!(String::from("1, 2, 3, 1,2,3"), a.unwrap());
```

I suddenly don't want to loop the Vec anymore:

```rust
let l = vec![1, 2, 3];
let a = cl_format!("The value is:\n ~a", &l);
assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());
```

Now, we have some inconsistency between Common Lisp and Rust. In Common Lisp, `~%` in the control string is the new line, but we are in Rust now, so `\n` is going to work.

For homogeneous collections (`Vec<T>`, `&[T]`, `[T; N]`) or fixed-size heterogeneous tuples (`(T0, T1, ...)` up to 12 elements), you can pass them directly without `&dyn TildeAble` or `tilde!`:

```rust
let words = vec!["hello", "world"];
let a = cl_format!("~{~a~^, ~}", &words);
assert_eq!(String::from("hello, world"), a.unwrap());

// Heterogeneous tuple in ~{...~} without &dyn TildeAble:
let mixed = (1, "two", 3.5, true);
let a = cl_format!("~{~a~^, ~}", &mixed);
assert_eq!(String::from("1, two, 3.5, true"), a.unwrap());
```

If you need a dynamically sized heterogeneous `Vec<&dyn TildeAble>`, you can use the `tilde!` macro:

```rust
let l = vec![tilde!(&1), &"two", &3];
let a = cl_format!("The value is:\n ~a", &l);
assert_eq!(String::from("The value is:\n [1, \"two\", 3]"), a.unwrap());
```

As in Common Lisp, we can loop through all arguments instead of putting them inside a Vec:

```rust
let a = cl_format!("~@{~a~^, ~}", &1, &2, &3);
assert_eq!(String::from("1, 2, 3"), a.unwrap());
```

Now, let's try some condition control (you can get the meaning of the condition control string in the `Conditional Formatting` chapter of [A Few FORMAT Recipes](https://gigamonkeys.com/book/a-few-format-recipes.html)):

```rust
let l = vec![1, 2, 3];
let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
assert_eq!(String::from("1, 2, and 3"), a.unwrap());

let l = vec![1, 2, 3, 4];
let a = cl_format!("~{~a~#[~;, and ~:;, ~]~}", &l);
assert_eq!(String::from("1, 2, 3, and 4"), a.unwrap());
```

When you pass a string literal to `cl_format!`, the format string syntax and argument count are validated at **compile time**. Flat format strings (without `~{` or `~[`) expand directly into inline buffer writes without runtime control-string parsing, while loop/conditional format strings cache their parsed `ControlStr` in a `OnceLock` and dispatch via `TupleArgs`.

### Manually ###

If you have a runtime-constructed control string or want to manage the `ControlStr` instance yourself, you can construct `ControlStr` directly:

```rust
let cs = cl_format::ControlStr::new("~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~}").unwrap();
```

Then we can generate `Args` (or `TupleArgs` for zero-`dyn` static dispatch) for the control string to reveal:

```rust
let mut list: Vec<i32> = vec![];
let args = Args::new(vec![&list]);
```

Let's use it several times by giving different lengths of arguments:

```rust
// this is equal to cl_format!(cs, &list)
assert_eq!(cs.reveal(args).unwrap(), "".to_string());

list.push(1);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1".to_string());

list.push(2);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1 and 2".to_string());

list.push(3);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1, 2, and 3".to_string());

list.push(4);
let args = Args::new(vec![&list]);
assert_eq!(cs.reveal(args).unwrap(), "1, 2, 3, and 4".to_string());

// Or reveal directly using TupleArgs without &dyn TildeAble:
assert_eq!(
    cs.reveal_cursor(&TupleArgs::new((&list,))).unwrap(),
    "1, 2, 3, and 4".to_string()
);
```

Let's try a mixed example: 

```rust
let my_team = "STeam";
let my_stars = vec![
    "Adam Lambert",
    "Queen",
    "snoop dogg",
];
	
assert_eq!(
    String::from("my favorite team \"STeam\" will win the superbowl LVIII. And Adam Lambert, Queen, and snoop dogg will in half time show. And the scores should be 38:35"),
    cl_format!(
        "my favorite team ~S will win the superbowl ~@R. And ~{~#[~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~} will in half time show. And the scores should be ~d:~d",
        &my_team,
        &58,
        &my_stars,
        &38,
        &35
    )
    .unwrap()
);
```

### Implement for custom type ###

So far, we have only shown the basic types. It would be better if we could make our type be revealed as well.

Here is a demo on how to implement `TildeAble`:

```rust
use cl_format::*;

// has to derive Debug
#[derive(Debug)]
struct MyStruct {
    a: usize,
    b: String,
}

impl TildeAble for MyStruct {
    // ~a
    fn format_va(&self, _tkind: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push_str(&format!("a: {}, b: {}", self.a, self.b));
        Ok(())
    }

    // ~d
    fn format_digit(&self, _tkind: &TildeKind, buf: &mut String) -> Result<()> {
        buf.push_str(&format!("{}", self.a));
        Ok(())
    }
}
```

Now `MyStruct` can be used by `cl_format`, but as you guessed, only for `~a` and `~d`

```rust
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

| tilde                     | rust type                                                                                                            |
|:-------------------------:|:--------------------------------------------------------------------------------------------------------------------:|
| `~a`                      | f32, f64, char, i8, i16, i32, i64, i128, isize, bool, u8, u16, u32, u64, u128, usize, String, &str, Vec<T>, &[T], [T; N], (T0, ..., Tn) |
| `~s`                      | f32, f64, char, i8, i16, i32, i64, i128, isize, bool, u8, u16, u32, u64, u128, usize, String, &str                   |
| `~d`                      | i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize                                                       |
| `~C`                      | char                                                                                                                 |
| `~f`, `~$`                | f32, f64                                                                                                             |
| `~{~}` (loop)             | Vec<T>, &[T], [T; N], (T0, ..., Tn) (up to 12 elements)                                                              |
| `~[~]` (condition)        | bool (`~:[`), usize (`~[`), Option<T> (`~@[`)                                                                        |
| `~R`                      | i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize                                                       |


# cl-format #

`cl-format` s the Rust implementation of the Common Lisp [format](http://www.lispworks.com/documentation/lw50/CLHS/Body/f_format.htm) function.

TL;DR: Use several directives like ~a ~{~} and flexible condition and loop control strings to format the string from arguments.

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

First, add `cl-format = "0.1"` in your `Cargo.toml`.
 
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
let ll: Vec<&dyn TildeAble> = vec![&1, &2, &3];
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
let l = vec![&1 as &dyn TildeAble, &2, &3];
let a = cl_format!("The value is:\n ~a", &l);
assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());
```

Now, we have some inconsistency between Common Lisp and Rust. In Common Lisp, `~%` in the control string is the new line, but we are in Rust now, so `\n` is going to work.

I think I am a bit tired of showing the type as `&dyn TildeAble` to elements inside Vec. But I haven't found a way to avoid it yet. If you know, let me know. So I added some macros:


```rust
let l = vec![tilde!(&1), &2, &3];
let a = cl_format!("The value is:\n ~a", &l);
assert_eq!(String::from("The value is:\n [1, 2, 3]"), a.unwrap());
```

As in Common Lisp, we can loop through all arguments instead of putting them inside a Vec:

```rust
let a = cl_format!("~@{~a~^, ~}", &1, &2, &3);
assert_eq!(String::from("1, 2, 3"), a.unwrap());
```

Now, let's try some condition control (you can get the meaning of the condition control string in the `Conditional Formatting` chapter of [A Few FORMAT Recipes](https://gigamonkeys.com/book/a-few-format-recipes.html)):

```rust
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
let mut list = vec![];
let args = Args::new(vec![&list]);
```

Let's use it several times by giving different lengths of arguments:

```rust
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
```

By now, your IDE should give you some errors, letting you implement `TildeKindVa` and `TildeKindDigit`. 

```rust
impl TildeKindVa for MyStruct {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("a: {}, b: {}", self.a, self.b)))
    }
}

impl TildeKindDigit for MyStruct {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("{}", self.a)))
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

| tilde                     | rust type                                               |
|:-------------------------:|:-------------------------------------------------------:|
| `~a`                      | f32, f64, char, i32, i64, usize, bool, u32, u64, String |
| `~s`                      | f32, f64, char, i32, i64, usize, bool, u32, u64, String |
| `~d`                      | i32, i64, u32, u64, usize                               |
| `~C`                      | char                                                    |
| `~[~]` (normal condition) | bool, usize                                             |


## TODO ##

+ [ ] doc

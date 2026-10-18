# Architecture & Design Optimization Report (2026-10-17)

## Overview

Following the initial allocation and hot-path optimizations (documented in `doc/optimization_2026-10-17.md`), we redesigned the core dispatch, AST representation, argument cursor abstraction, and macro expansion architecture of `cl-format`:

1. **#1 — Generic Container & Reference Support (`Vec<T>`, `&[T]`, `[T; N]`, `Option<T>`, `str`, `&T`)**: Users can pass `&Vec<i32>`, `&Vec<String>`, `&str`, and `&Option<T>` directly without manual `Vec<&dyn TildeAble>` / `tilde!()` boxing.
2. **#2 — Decoupling the Argument Cursor (`ArgCursor`) from Value Formatting (`TildeAble`)**: Separated cursor state traversal (`Args`, `SliceArgs<T>`) from value formatting (`TildeAble`) and removed `TildeNil`.
3. **#3 — Collapsing Two-Stage Trait-Object Dispatch into `TildeAble`**: Replaced the 12 generated `TildeKind*` traits and `into_tildekind_*(&self) -> Option<&dyn TildeKind*>` upcast methods with direct `format_*` default methods on `TildeAble`.
4. **#4 — Zero-Copy AST (`ControlStr<'a>`, `Tilde<'a>`, `TildeKind<'a>`) & Direct Byte-Slice Parser**: Replaced heap-allocated `String` payloads inside `TildeKind` (`Text`, `Float`, `Digit`) with borrowed `&'a str` slices into the original control string and removed remaining `Cursor::read`/`seek` and `parse_radix` allocations.
5. **#5 — Per-Call-Site `OnceLock<ControlStr<'static>>` Caching in `cl_format!`**: Cached parsed `ControlStr<'static>` instances at each `cl_format!("literal", ...)` call site so repeated calls skip runtime parsing entirely.
6. **#6 — `ControlFlow<()>` Propagation for `~^` (`LoopEnd`)**: Propagated `std::ops::ControlFlow<()>` through `Tilde::reveal` and `TildeKind::match_reveal` so `~^` cleanly terminates enclosing loops even when nested inside `~[...~]`, `~#[...~]`, `~:[...~]`, or `~@[...~]`.
7. **#7 — Stable Rust Compilation**: Removed unused `#![feature(let_chains)]` and `#![feature(pattern)]` nightly gates.

---

## Detailed Breakdown: What Changed and Why It Is Faster

### 1. Collapsing Two-Stage Trait-Object Dispatch (#3) & Generic Containers (#1)

* **Files Modified**: `cl-format-macros/src/lib.rs`, `cl-format/src/tildes/tilde_kinds.rs`, `cl-format/src/tildes/reveal_impl.rs`, `cl-format/src/tildes/tilde_able_impl.rs`
* **What Changed**:
  * Updated `#[derive(TildeAble)]` in `cl-format-macros` to generate `fn format_<variant>(&self, _tkind: &TildeKind, _buf: &mut String) -> Result<()>` directly on `TildeAble` (with default implementations returning `RevealError`), deleting all 12 `TildeKind*` sub-traits (`TildeKindVa`, `TildeKindDigit`, `TildeKindLoop`, etc.) and the `#[implTo(...)]` attribute lists.
  * Implemented `TildeAble` generically for `str`, `&T` (where `T: TildeAble + ?Sized`), `[T]`, `Vec<T>`, `[T; N]`, and `Option<T>`.
* **Why It Makes Things Faster**:
  * **Eliminates Vtable Hop #1**: Previously, formatting any value required **two** indirect vtable calls per directive:
    1. Call `arg.into_tildekind_va()` via the `TildeAble` vtable to construct a fat pointer `Option<&dyn TildeKindVa>`.
    2. Branch on the `Option`, then call `.format(self, buf)` via the second `TildeKindVa` vtable.
    By placing `format_va`, `format_digit`, etc. directly on `TildeAble`, formatting a `&dyn TildeAble` jumps straight to the target function in a single vtable call with no intermediate `Option<&dyn TildeKind*>` construction.

---

### 2. Separating `ArgCursor` from `TildeAble` (#2) & `ControlFlow<()>` for `~^` (#6)

* **Files Modified**: `cl-format/src/tildes/args.rs`, `cl-format/src/tildes/tilde_kinds.rs`, `cl-format/src/tildes/reveal_impl.rs`, `cl-format/src/tildes/tilde_able_impl.rs`, `cl-format/src/tildes.rs`, `cl-format/src/control_str.rs`
* **What Changed**:
  * Introduced the `ArgCursor` trait with an associated item type:
    ```rust
    pub trait ArgCursor {
        type Item: TildeAble + ?Sized;

        fn pop(&self) -> Option<&Self::Item>;
        fn back(&self) -> Option<&Self::Item>;
        fn left_count(&self) -> usize;
    }
    ```
  * Implemented `ArgCursor` for `Args<'a, 'arg>` (`type Item = dyn TildeAble + 'a`) and `SliceArgs<'a, T>` (`type Item = T`), and removed their `TildeAble` implementations as well as `TildeNil`.
  * Changed `Tilde::reveal<C: ArgCursor>(&self, args: &C, buf: &mut String) -> Result<ControlFlow<()>>` and `TildeKind::match_reveal<C: ArgCursor>` to operate directly on the cursor `C`.
  * Added `as_cond_index(&self) -> Option<usize>` (for `usize`) and `as_cond_bool(&self) -> Option<bool>` (for `bool`) on `TildeAble` so `~[...~]` and `~:[...~]` clauses execute with the live `args: &C` cursor instead of a dummy `TildeNil`.
* **Why It Makes Things Faster**:
  * **Eliminates Vtable Indirection on the Argument Cursor**:
    Previously, `Args` implemented `TildeAble`. That meant every directive in a control string or loop first dispatched dynamically through `<Args as TildeAble>::format_*`, which called `self.pop()` and then dispatched dynamically a *second* time through `<ConcreteType as TildeAble>::format_*`. Worse, cursor-only directives like `~*` (`Star`), `~#[` (`Cond(Sharp)`), and `VecTilde` paid dynamic dispatch overhead just to read or advance the cursor index!
  * **Static Monomorphization over `C: ArgCursor`**:
    Because `match_reveal<C: ArgCursor>` and `reveal_loop<C: ArgCursor>` are generic over `C`, the compiler monomorphizes them for `Args` and `SliceArgs<T>`:
    * `args.pop()`, `args.back()`, and `args.left_count()` are **inlined direct `Cell<usize>` loads/stores** with zero vtable dispatch.
    * When iterating over a homogeneous slice like `Vec<i32>` or `Vec<String>` via `SliceArgs<'a, T>`, `ArgCursor::Item` is the concrete type `T` (`i32`, `String`, etc.) rather than `dyn TildeAble`. Thus, `a.format_va(self, buf)` inside `SliceArgs<i32>` is **100% statically dispatched and inlined** with zero dynamic dispatch!
    * Even for heterogeneous `Vec<&dyn TildeAble>`, removing the outer `Args`/`SliceArgs` `TildeAble` vtable layer cuts the number of dynamic dispatches in half (from 2 vtable calls per directive to 1).
* **Benchmark Impact**:
  * `bench_cl_format_making_loop_string`: `101.68 ns` → **`81.07 ns`** (-20.3%)
  * `bench_cl_format_making_loop_empty_string`: `45.67 ns` → **`40.89 ns`** (-10.5%)

---

### 3. Zero-Copy AST (`Tilde<'a>`, `TildeKind<'a>`) & Direct Byte-Slice Parser (#4)

* **Files Modified**: `cl-format/src/tildes/tilde_kinds.rs`, `cl-format/src/tildes.rs`, `cl-format/src/control_str.rs`
* **What Changed**:
  * Parameterized `Tilde<'a>` and `TildeKind<'a>` over the lifetime `'a` of the input control string `&'a str`:
    * `TildeKind::Text(&'a str)` instead of `TildeKind::Text(String)`
    * `TildeKind::Float(Option<&'a str>)` instead of `TildeKind::Float(Option<String>)`
    * `TildeKind::Digit(Option<&'a str>)` instead of `TildeKind::Digit(Option<String>)`
  * Replaced `Cursor::read(&mut char_buf)` and `Cursor::seek(...)` in `parse_loop`, `parse_loop_end`, `parse_cond`, `parse_star`, `parse_tildes`, and `parse_char` with direct slice pattern matching on `&bytes[pos..]`.
  * Rewrote `Tilde::parse_radix` to slice `&s_ref[pos + 1..pos + total_len]` and iterate with `.split(',')` and `.split_once(':')` on stack references instead of allocating `bucket: Vec<u8>`, `String::from_utf8`, `VecDeque`, and `Vec<&str>`.
* **Why It Makes Things Faster**:
  * `ControlStr<'a>` already borrows `inner: &'a str`. Previously, every literal text segment inside `~{...~}` or `~[...~]` (such as `"empty"`, `" and "`, `", and "`, `", "` in `"~{~#[empty~;~a~;~a and ~a~:;~@{~a~#[~;, and ~:;, ~]~}~]~:}"`) called `.to_string()`, performing 4 separate heap `malloc` + `memcpy` + `free` cycles every time the control string was parsed and dropped.
  * With `TildeKind::Text(&'a str)`, parsing a text segment is just a pointer + length slice (`&s_ref[pos..pos + rel]`), requiring **zero heap allocations** for text literals or directive parameters.
* **Benchmark Impact**:
  * `bench_cl_format_macro_making_loop_empty_string`: `460.28 ns` → **`322.39 ns`** (-30.0%)
  * `bench_cl_format_macro_making_loop_string`: `505.59 ns` → **`365.28 ns`** (-27.7%)

---

### 4. Per-Call-Site `OnceLock<ControlStr<'static>>` Caching in `cl_format!` (#5)

* **Files Modified**: `cl-format/src/lib.rs`
* **What Changed**:
  Added `:literal` matcher arms to `cl_format!`:
  ```rust
  ($control_str:literal, $($ele:expr),*) => {
      {
          static __CL_FORMAT_CS: std::sync::OnceLock<cl_format::ControlStr<'static>> =
              std::sync::OnceLock::new();
          let c = __CL_FORMAT_CS.get_or_init(|| {
              cl_format::ControlStr::new($control_str).expect("making control string has issue")
          });
          let arr = [$(tilde!($ele)),*];
          let a = cl_format::Args::new_cow(&arr);
          c.reveal(a)
      }
  };
  ```
  (with fallback `:expr` arms for dynamic runtime format strings).
* **Why It Makes Things Faster**:
  * Because `ControlStr<'static>` is immutable and `Send + Sync` (containing only `&'static str` slices and `Vec`s of AST nodes), each `cl_format!("...", ...)` call site gets its own dedicated `static OnceLock<ControlStr<'static>>`.
  * The control string is parsed **only once** on the first execution of that call site. Every subsequent execution performs a single atomic-acquire load (`~0.5 ns`) to borrow `&'static ControlStr<'static>`—with **no hash table lookup, no mutex lock, no string hashing, and zero AST allocations**—and then immediately runs `c.reveal(a)` with a stack-allocated `Args::new_cow(&arr)`.
  * In fact, `cl_format!("...", l)` is now slightly **faster** than manual `ControlStr::reveal(Args::new(vec![l]))` (`72.32 ns` vs `81.35 ns`, and `30.73 ns` vs `41.19 ns` for empty) because `cl_format!` borrows a stack array `&arr` via `Args::new_cow` whereas `Args::new(vec![l])` allocates a `Vec` on the heap!
* **Benchmark Impact**:
  * `bench_cl_format_macro_making_loop_empty_string`: `322.39 ns` → **`30.73 ns`** (**-90.5%**, 10.5x faster than uncached)
  * `bench_cl_format_macro_making_loop_string`: `365.28 ns` → **`72.32 ns`** (**-80.2%**, 5.1x faster than uncached)

---

## Complete Benchmark Comparison

| Benchmark                                             | Original Baseline | After Step 8 (Alloc Opts) | After Arch #1,#3,#7 | After Arch #2,#6 (`ArgCursor`) | After Arch #4 (Zero-Copy AST) | After Arch #5 (`OnceLock` Macro) | Overall Speedup vs Baseline |
|-------------------------------------------------------|------------------:|--------------------------:|--------------------:|-------------------------------:|------------------------------:|---------------------------------:|----------------------------:|
| `bench_cl_format_reveal_single_a`                     |          58.52 ns |                  30.68 ns |            30.69 ns |                       30.64 ns |                      30.81 ns |                     **30.93 ns** |                    **1.9x** |
| `bench_cl_format_making_loop_string`                  |         681.61 ns |                 103.76 ns |           101.68 ns |                       81.07 ns |                      82.48 ns |                     **81.35 ns** |                    **8.4x** |
| `bench_cl_format_making_loop_empty_string`            |         418.20 ns |                  49.52 ns |            45.67 ns |                       40.89 ns |                      41.54 ns |                     **41.19 ns** |                   **10.2x** |
| `bench_cl_format_macro_making_loop_empty_string`      |         1577.9 ns |                 407.45 ns |           408.09 ns |                      460.28 ns |                     322.39 ns |                     **30.73 ns** |                   **51.3x** |
| `bench_cl_format_macro_making_loop_string`            |         1842.7 ns |                 468.87 ns |           467.96 ns |                      505.59 ns |                     365.28 ns |                     **72.32 ns** |                   **25.5x** |
| `bench_normal_making_loop_string` *(reference)*       |          58.17 ns |                  58.42 ns |            59.35 ns |                       58.47 ns |                      58.97 ns |                     **59.44 ns** |                           — |
| `bench_normal_making_loop_empty_string` *(reference)* |          12.37 ns |                  12.18 ns |            12.71 ns |                       12.56 ns |                      12.56 ns |                     **12.44 ns** |                           — |

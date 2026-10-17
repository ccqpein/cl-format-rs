# Performance Optimization Report (2026-10-17)

## Overview & Root-Cause Analysis

Profiling `cl-format` with Criterion (`benches/bench0.rs` and `benches/bench1.rs`) and inspecting `benches/bench0_flamegraph.svg` showed that the majority of execution time was spent inside allocator routines (`malloc`, `realloc`, `free`), `Vec::clone`, `String::clone`, and `drop_in_place<TildeKind>`.

Although it initially appeared that building the output `String` was the bottleneck, tracing the execution of both `ControlStr::reveal` (runtime formatting) and `ControlStr::new` (control string parsing) revealed that almost all heap allocations came from **hidden intermediate allocations**:

1. **Deep-cloning the parsed `TildeKind` AST** on every loop (`~{...~}`) and conditional (`~@[...~]`) reveal.
2. **Eagerly allocating error message `String`s** on the happy path via `.ok_or(TildeError::new(...))`.
3. **Allocating temporary `String`s** via `.to_string()` and `format!()` for every formatted primitive/string value before appending to the output buffer.
4. **Under-allocating the output `String` capacity** (`String::with_capacity(args.left_count())`, which was `1` byte), forcing repeated `realloc` and `memmove` calls.
5. **Formatting integers via `core::fmt::Formatter`** instead of writing ASCII digits directly into the output buffer.
6. **Allocating temporary `Vec<u8>` buffers and cloning AST vectors during parsing** (`ControlStr::scan`, `Tilde::scan_for_kind`, `Tilde::parse_cond`, `Tilde::parse_loop`, etc.).

---

## Step-by-Step Optimizations: What Changed and Why

### Step 1: Avoid Cloning the `TildeKind` AST in `TildeKindLoop` and `TildeKindCond`

* **Files Modified**: `cl-format/src/tildes/reveal_impl.rs`
* **What Changed**:
  Extracted a `reveal_loop(vv: &[Tilde], args: &Args, buf: &mut String)` helper function and called it directly from `<Args as TildeKindLoop>::format` and `<Vec<&dyn TildeAble> as TildeKindLoop>::format`. Also updated `<Option<&dyn TildeAble> as TildeKindCond>::format` to iterate over `vv` directly using `Args::new_cow(std::slice::from_ref(a))`.
* **Why**:
  Previously, `<Vec<&dyn TildeAble> as TildeKindLoop>::format` did:
  ```rust
  let mut new_kind = tkind.clone();
  // mutated TildeLoopKind::Nil / NilColon -> TildeLoopKind::At
  new_kind.match_reveal(&Args::from(self.as_slice()), buf)
  ```
  Just to change the enum tag from `Nil` / `NilColon` to `At`, `tkind.clone()` **deep-cloned the entire loop AST subtree**—allocating 8 nested `Vec<Tilde>`s and 4 heap `String`s (`"empty"`, `" and "`, `", and "`, `", "`) on every call to `reveal`, and immediately dropping them when `format` returned. Even when the input `Vec` was empty, `tkind.clone()` executed first. Iterating over the borrowed slice `vv: &[Tilde]` eliminates all AST cloning and dropping during `reveal`.
* **Benchmark Impact**:
  * `bench_cl_format_making_loop_string`: `681.61 ns` → **`369.32 ns`** (-45.8%)
  * `bench_cl_format_making_loop_empty_string`: `418.20 ns` → **`115.09 ns`** (-72.5%)

---

### Step 2: Eliminate Temporary `String` Allocations (`to_string()` / `format!()`) During Formatting

* **Files Modified**: `cl-format/src/tildes/reveal_impl.rs`, `cl-format/src/tildes/tilde_kinds.rs`
* **What Changed**:
  * Replaced `buf.push_str(self.to_string().as_str())` with direct writes into `buf`:
    * `String` (`~a`): `buf.push_str(self)` (and `buf.push('"'); buf.push_str(self); buf.push('"');` for `~S`).
    * `char` (`~a`, `~c`, `~@c`, `~S`): `buf.push(*self)` (with surrounding `'\''` for `~@c` and `~S`).
    * Numeric types: `write!(buf, "{}", self)`.
  * Replaced `String::from_utf8(vec![b'~'; *n])` in `TildeKind::Tildes(n)` with `buf.extend(std::iter::repeat('~').take(*n))`.
* **Why**:
  Calling `self.to_string()` or `format!(...)` allocates a temporary `String` on the heap, formats into it, copies its bytes into `buf`, and immediately frees the temporary `String`. Worse still, for `String` in `TildeKindVa`, `self.to_string()` cloned the heap `String` unnecessarily before appending it to `buf`. Writing directly into `buf: &mut String` eliminates all intermediate heap allocations.
* **Benchmark Impact**:
  * `bench_cl_format_reveal_single_a`: `57.142 ns` → **`51.700 ns`** (-9.5%)
  * `bench_cl_format_making_loop_string`: `369.32 ns` → **`356.86 ns`** (-3.4%)

---

### Step 3: Pre-allocate `String` Capacity in `ControlStr::reveal` Using Control String Length

* **Files Modified**: `cl-format/src/control_str.rs`
* **What Changed**:
  Changed `let mut result = String::with_capacity(args.left_count());` to `let mut result = String::with_capacity(end);` (where `end = self.inner.len()`).
* **Why**:
  `args.left_count()` is the number of top-level arguments passed to `reveal`. When formatting a single list (`vec![&list]`) or a single value (`vec![&1]`), `args.left_count()` is `1`. Calling `String::with_capacity(1)` allocated a 1-byte heap buffer (`malloc(1)`), which then had to `realloc` and copy bytes multiple times (1 → 8 → 16 bytes) as characters were appended. Using the control string length (`self.inner.len()`) allocates sufficient capacity upfront in a single `malloc` call with zero `realloc`s for typical outputs.
* **Benchmark Impact**:
  * `bench_cl_format_making_loop_string`: `356.86 ns` → **`325.07 ns`** (-8.9%)
  * `bench_cl_format_making_loop_empty_string`: `124.90 ns` → **`109.31 ns`** (-12.5%)

---

### Step 4: Use `Cell<usize>` in `Args`, `Args::new_cow` in `cl_format!`, and Inline `VecTilde` in `match_reveal`

* **Files Modified**: `cl-format/src/tildes/args.rs`, `cl-format/src/lib.rs`, `cl-format/src/tildes/tilde_kinds.rs`
* **What Changed**:
  1. Replaced `ind: RefCell<usize>` with `ind: Cell<usize>` in `Args`.
  2. Updated the `cl_format!` macro to bind a stack array `let arr = [$(tilde!($ele)),*];` and pass `Args::new_cow(&arr)` (`Cow::Borrowed`) instead of converting the array into a heap-allocated `Vec` via `From<[&dyn TildeAble; N]>`.
  3. Handled `TildeKind::VecTilde(vv)` directly in `TildeKind::match_reveal` (`for v in vv { v.reveal(arg, buf)?; }`) instead of dispatching through `arg.into_tildekind_vectilde()`.
* **Why**:
  * `usize` is `Copy`, so `Cell<usize>` provides interior mutability with plain register reads/writes, avoiding `RefCell`'s runtime borrow-flag checks on every `pop()`, `back()`, and `left_count()`.
  * `Args` already supports `Cow::Borrowed`, so `cl_format!` does not need to heap-allocate a `Vec` for its argument list.
  * `TildeKindVecTilde` had identical implementations for `Args` and `TildeNil` (simply iterating over `vv`), so handling `VecTilde` directly in `match_reveal` removes two vtable dispatches per conditional branch.
* **Benchmark Impact**:
  * `bench_cl_format_making_loop_string`: `325.07 ns` → **`264.60 ns`** (-18.6%)
  * `bench_cl_format_making_loop_empty_string`: `109.31 ns` → **`96.144 ns`** (-12.0%)

---

### Step 5: Replace Eager `.ok_or(TildeError::new(...))` with Lazy `.ok_or_else(|| ...)`

* **Files Modified**: `cl-format/src/tildes/tilde_kinds.rs`, `cl-format/src/tildes/reveal_impl.rs`
* **What Changed**:
  Replaced every `.ok_or::<TildeError>(TildeError::new(ErrorKind::..., "...").into())` in `TildeKind::match_reveal` and `reveal_impl.rs` with `.ok_or_else(|| TildeError::new(ErrorKind::..., "..."))`.
* **Why**:
  In Rust, `Option::ok_or(err)` evaluates its argument **eagerly** before checking whether the `Option` is `Some` or `None`. Because `TildeError::new` calls `msg.as_ref().to_string()`, **every single directive dispatch in `match_reveal` and every `self.pop()` in loops/conditionals was heap-allocating a `String` for the error message and immediately freeing it**, even when no error occurred! Switching to `.ok_or_else(|| ...)` makes error construction lazy so zero error `String`s are allocated on the happy path.
* **Benchmark Impact**:
  * `bench_cl_format_making_loop_string`: `264.60 ns` → **`130.82 ns`** (-50.5%, beating the `< 200ns` target)
  * `bench_cl_format_making_loop_empty_string`: `96.144 ns` → **`48.723 ns`** (-49.3%)
  * `bench_cl_format_reveal_single_a`: `51.711 ns` → **`39.455 ns`** (-23.7%)

---

### Step 6: Fast Stack-Buffer Integer Formatting in `reveal_impl.rs`

* **Files Modified**: `cl-format/src/tildes/reveal_impl.rs`
* **What Changed**:
  Added inline `push_u64`, `push_i64`, `push_u128`, and `push_i128` helpers that write ASCII digits into a small stack buffer (`[u8; 20]` / `[u8; 39]`, with a single-character fast path for `n < 10`) and append directly to `buf: &mut String`. Used these in `TildeKindVa` and `TildeKindDigit`.
* **Why**:
  `write!(buf, "{}", self)` constructs a `core::fmt::Formatter`, goes through `Display::fmt` → `Formatter::pad_integral`, and dispatches through the `&mut dyn core::fmt::Write` vtable. Since `~a` and `~d` without padding parameters just need base-10 ASCII digits, formatting directly into a stack buffer avoids `core::fmt` overhead completely.
* **Benchmark Impact**:
  * `bench_cl_format_reveal_single_a`: `39.455 ns` → **`30.388 ns`** (-23.0%)
  * `bench_cl_format_making_loop_string`: `130.82 ns` → **`105.43 ns`** (-19.4%)

---

### Step 7: Eliminate `bucket` Allocation in `scan_for_kind` and `cache.clone()` in `parse_cond`

* **Files Modified**: `cl-format/src/tildes.rs`
* **What Changed**:
  1. Changed `Tilde::scan_for_kind` to inspect the underlying byte slice `&c.get_ref().as_bytes()[pos..]` in place and return a plain function pointer `fn(&mut Cursor<&'_ str>) -> Result<Tilde, TildeError>` instead of allocating `let mut bucket = vec![b'~']` and returning `Box<dyn Fn(...)>`.
  2. Replaced `cache.clone()` + `cache.clear()` and `buffer.clone()` + `buffer.clear()` in `Tilde::parse_cond` with `std::mem::take(&mut cache)` and `std::mem::take(&mut buffer)`.
* **Why**:
  In the `cl_format!` macro benchmarks, over `1,150 ns` was spent inside `ControlStr::new` parsing the control string:
  * `scan_for_kind` was allocating a heap `Vec<u8>` (`bucket`) for every `~` directive in the control string (10 directives per call).
  * `parse_cond` was calling `cache.clone()` on every `~;`, `~:;`, and `~]` branch, deep-cloning each branch's `Vec<Tilde>` (and all nested `Tilde`s and `String`s) just before clearing `cache`. `std::mem::take` moves the `Vec` in $O(1)$ with zero allocations.
* **Benchmark Impact**:
  * `bench_cl_format_macro_making_loop_empty_string`: `1204.3 ns` → **`653.66 ns`** (-45.7%)
  * `bench_cl_format_macro_making_loop_string`: `1279.7 ns` → **`718.85 ns`** (-43.8%)

---

### Step 8: Slice `&str` / `&[u8]` Directly in `ControlStr::scan` and `Tilde` Parsers Instead of `read_until` Buffers

* **Files Modified**: `cl-format/src/control_str.rs`, `cl-format/src/tildes.rs`
* **What Changed**:
  Replaced `Cursor::read_until` / `read_exact` temporary `Vec<u8>` buffers with direct byte-slice scanning (`bytes[pos..].iter().position(...)`) and `&str` slicing (`s_ref[start..end].to_string()`) in `ControlStr::scan`, `Tilde::parse_loop`, `Tilde::parse_cond`, `Tilde::parse_value`, `Tilde::parse_float`, `Tilde::parse_digit`, `Tilde::parse_tildes`, and `Tilde::parse_standard`.
* **Why**:
  Because `Cursor<&str>` wraps an in-memory UTF-8 `&str`, allocating a temporary `Vec<u8>` (`let mut buf = vec![]`), copying bytes into it via `read_until`, and then calling `String::from_utf8(buf.to_vec())` performed 2–3 unnecessary heap allocations per directive (including allocating a `Vec<u8>` just to read the 2 bytes of every `~a`). Scanning the byte slice directly and slicing `&str` eliminates all temporary parser buffers and redundant UTF-8 validation checks.
* **Benchmark Impact**:
  * `bench_cl_format_macro_making_loop_empty_string`: `653.66 ns` → **`407.45 ns`** (-37.7%)
  * `bench_cl_format_macro_making_loop_string`: `718.85 ns` → **`468.87 ns`** (-34.8%)

---

## Summary Benchmark Progression

| Benchmark | Baseline | Step 1 | Step 2 | Step 3 | Step 4 | Step 5 | Step 6 | Step 7 | Step 8 (Final) | Total Speedup |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `bench_cl_format_reveal_single_a` | 58.52 ns | 57.14 ns | 51.70 ns | 50.93 ns | 51.71 ns | 39.46 ns | 30.39 ns | 30.83 ns | **30.68 ns** | **1.9x** |
| `bench_cl_format_making_loop_string` | 681.61 ns | 369.32 ns | 356.86 ns | 325.07 ns | 264.60 ns | 130.82 ns | 105.43 ns | 105.40 ns | **103.76 ns** | **6.6x** |
| `bench_cl_format_making_loop_empty_string` | 418.20 ns | 115.09 ns | 124.90 ns | 109.31 ns | 96.14 ns | 48.72 ns | 49.14 ns | 49.03 ns | **49.52 ns** | **8.4x** |
| `bench_cl_format_macro_making_loop_empty_string` | 1577.9 ns | 1262.7 ns | 1254.1 ns | 1250.9 ns | 1479.2 ns | 1185.6 ns | 1204.3 ns | 653.66 ns | **407.45 ns** | **3.9x** |
| `bench_cl_format_macro_making_loop_string` | 1842.7 ns | 1537.8 ns | 1499.3 ns | 1498.4 ns | 1698.0 ns | 1267.1 ns | 1279.7 ns | 718.85 ns | **468.87 ns** | **3.9x** |
| `bench_normal_making_loop_string` *(reference)* | 58.17 ns | 57.77 ns | 57.62 ns | 57.09 ns | 56.80 ns | 56.18 ns | 57.93 ns | 56.39 ns | **58.42 ns** | — |
| `bench_normal_making_loop_empty_string` *(reference)* | 12.37 ns | 12.27 ns | 12.13 ns | 12.15 ns | 12.15 ns | 12.04 ns | 12.26 ns | 12.34 ns | **12.18 ns** | — |

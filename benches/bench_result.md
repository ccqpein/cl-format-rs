# Benchmark results #

**4/29/2023**

| benchmark names                                | results                |
|------------------------------------------------|------------------------|
| bench_cl_format_macro_making_loop_empty_string | 1,546 ns/iter (+/- 24) |
| bench_cl_format_macro_making_loop_string       | 2,053 ns/iter (+/- 34) |
| bench_cl_format_making_loop_empty_string       | 545 ns/iter (+/- 8)    |
| bench_cl_format_making_loop_string             | 1,034 ns/iter (+/- 21) |
| bench_normal_making_loop_empty_string          | 13 ns/iter (+/- 0)     |
| bench_normal_making_loop_string                | 60 ns/iter (+/- 1)     |
| bench_cl_format_reveal_single_a                | 84.663 ns              |
| bench_cl_format_plain_single_a                 | 25.153 ns              |


`bench_cl_format_making_loop_string` target is < 200ns


**6/6/2023**

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_macro_making_loop_empty_string | 1683 ns   |
| bench_cl_format_macro_making_loop_string       | 2279 ns   |
| bench_cl_format_making_loop_empty_string       | 582.97 ns |
| bench_cl_format_making_loop_string             | 1020 ns   |
| bench_normal_making_loop_empty_string          | 13.512 ns |
| bench_normal_making_loop_string                | 60.577 ns |
| bench_cl_format_reveal_single_a                | 79.576 ns |
| bench_cl_format_plain_single_a                 | 25.153 ns |

**6/7/2023**

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 74.855 ns |
| bench_cl_format_making_loop_string             | 983.83 ns |
| bench_normal_making_loop_string                | 58.859 ns |
| bench_cl_format_macro_making_loop_empty_string | 1631 ns   |
| bench_normal_making_loop_empty_string          | 13.260 ns |
| bench_cl_format_making_loop_empty_string       | 547.69 ns |
| bench_cl_format_macro_making_loop_string       | 2138 ns   |


**6/27/2023**

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 68.717 ns |
| bench_cl_format_making_loop_string             | 729.71 ns |
| bench_normal_making_loop_string                | 57.941 ns |
| bench_cl_format_macro_making_loop_empty_string | 1496 ns   |
| bench_normal_making_loop_empty_string          | 12.983 ns |
| bench_cl_format_making_loop_empty_string       | 417.72 ns |
| bench_cl_format_macro_making_loop_string       | 1825 ns   |

**10/17/2026 (Baseline before optimization)**

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 58.522 ns |
| bench_cl_format_making_loop_string             | 681.61 ns |
| bench_normal_making_loop_string                | 58.165 ns |
| bench_cl_format_macro_making_loop_empty_string | 1577.9 ns |
| bench_normal_making_loop_empty_string          | 12.371 ns |
| bench_cl_format_making_loop_empty_string       | 418.20 ns |
| bench_cl_format_macro_making_loop_string       | 1842.7 ns |

**10/17/2026 - Step 1: Avoid cloning `TildeKind` AST in `TildeKindLoop` and `TildeKindCond`**
> Extracted `reveal_loop` helper in `reveal_impl.rs` so `<Vec<&dyn TildeAble> as TildeKindLoop>::format` and `<Option<&dyn TildeAble> as TildeKindCond>::format` iterate over `vv: &[Tilde]` directly instead of calling `tkind.clone()` / `vv.clone()`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 57.142 ns |
| bench_cl_format_making_loop_string             | 369.32 ns |
| bench_normal_making_loop_string                | 57.772 ns |
| bench_cl_format_macro_making_loop_empty_string | 1262.7 ns |
| bench_normal_making_loop_empty_string          | 12.267 ns |
| bench_cl_format_making_loop_empty_string       | 115.09 ns |
| bench_cl_format_macro_making_loop_string       | 1537.8 ns |

**10/17/2026 - Step 2: Eliminate temporary `String` allocations (`to_string()` / `format!()`) during formatting**
> Replaced `buf.push_str(self.to_string().as_str())` and `format!()` in `TildeKindVa`, `TildeKindDigit`, `TildeKindChar`, `TildeKindStandard`, and `TildeKind::Tildes` with direct `write!(buf, "{self}")`, `buf.push(...)`, and `buf.push_str(self)`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 51.700 ns |
| bench_cl_format_making_loop_string             | 356.86 ns |
| bench_normal_making_loop_string                | 57.621 ns |
| bench_cl_format_macro_making_loop_empty_string | 1254.1 ns |
| bench_normal_making_loop_empty_string          | 12.127 ns |
| bench_cl_format_making_loop_empty_string       | 124.90 ns |
| bench_cl_format_macro_making_loop_string       | 1499.3 ns |

**10/17/2026 - Step 3: Pre-allocate `String` capacity in `ControlStr::reveal` using control string length**
> Changed `String::with_capacity(args.left_count())` (which allocated a 1-byte buffer and forced multiple `realloc`s) to `String::with_capacity(self.inner.len())` in `ControlStr::reveal`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 50.927 ns |
| bench_cl_format_making_loop_string             | 325.07 ns |
| bench_normal_making_loop_string                | 57.086 ns |
| bench_cl_format_macro_making_loop_empty_string | 1250.9 ns |
| bench_normal_making_loop_empty_string          | 12.153 ns |
| bench_cl_format_making_loop_empty_string       | 109.31 ns |
| bench_cl_format_macro_making_loop_string       | 1498.4 ns |

**10/17/2026 - Step 4: Use `Cell<usize>` in `Args`, `Args::new_cow` in `cl_format!`, and inline `VecTilde` in `match_reveal`**
> Replaced `RefCell<usize>` with `Cell<usize>` in `Args`, borrowed a stack array via `Args::new_cow` in `cl_format!`, and handled `TildeKind::VecTilde` directly in `TildeKind::match_reveal`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 51.711 ns |
| bench_cl_format_making_loop_string             | 264.60 ns |
| bench_normal_making_loop_string                | 56.799 ns |
| bench_cl_format_macro_making_loop_empty_string | 1479.2 ns |
| bench_normal_making_loop_empty_string          | 12.145 ns |
| bench_cl_format_making_loop_empty_string       | 96.144 ns |
| bench_cl_format_macro_making_loop_string       | 1698.0 ns |

**10/17/2026 - Step 5: Replace eager `.ok_or(TildeError::new(...))` with lazy `.ok_or_else(|| ...)`**
> `Option::ok_or(TildeError::new(...))` eagerly evaluated `TildeError::new` (which heap-allocated a `String` for the error message and immediately freed it) on every single directive dispatch and `Args::pop()`, even when no error occurred. Switched to lazy `.ok_or_else(|| TildeError::new(...))`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 39.455 ns |
| bench_cl_format_making_loop_string             | 130.82 ns |
| bench_normal_making_loop_string                | 56.182 ns |
| bench_cl_format_macro_making_loop_empty_string | 1185.6 ns |
| bench_normal_making_loop_empty_string          | 12.037 ns |
| bench_cl_format_making_loop_empty_string       | 48.723 ns |
| bench_cl_format_macro_making_loop_string       | 1267.1 ns |

**10/17/2026 - Step 6: Fast stack-buffer integer formatting in `reveal_impl.rs`**
> Added inline `push_u64`/`push_i64`/`push_u128`/`push_i128` helpers for `TildeKindVa` and `TildeKindDigit`, bypassing `core::fmt::Formatter` and `&mut dyn fmt::Write` vtable overhead.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 30.388 ns |
| bench_cl_format_making_loop_string             | 105.43 ns |
| bench_normal_making_loop_string                | 57.930 ns |
| bench_cl_format_macro_making_loop_empty_string | 1204.3 ns |
| bench_normal_making_loop_empty_string          | 12.257 ns |
| bench_cl_format_making_loop_empty_string       | 49.136 ns |
| bench_cl_format_macro_making_loop_string       | 1279.7 ns |

**10/17/2026 - Step 7: Eliminate `bucket` allocation in `scan_for_kind` and `cache.clone()` in `parse_cond`**
> Sliced cursor bytes directly in `Tilde::scan_for_kind` (removing `let mut bucket = vec![b'~']` and `Box<dyn Fn>`) and replaced `cache.clone()` / `buffer.clone()` with `std::mem::take` in `Tilde::parse_cond`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 30.825 ns |
| bench_cl_format_making_loop_string             | 105.40 ns |
| bench_normal_making_loop_string                | 56.386 ns |
| bench_cl_format_macro_making_loop_empty_string | 653.66 ns |
| bench_normal_making_loop_empty_string          | 12.341 ns |
| bench_cl_format_making_loop_empty_string       | 49.032 ns |
| bench_cl_format_macro_making_loop_string       | 718.85 ns |

**10/17/2026 - Step 8: Slice `&str` / `&[u8]` directly in `ControlStr::scan` and `Tilde` parsers instead of `read_until` buffers**
> Replaced `Cursor::read_until` / `read_exact` temporary `Vec<u8>` buffers with direct byte-slice scanning (`iter().position(...)`) and `&str` slicing in `ControlStr::scan`, `parse_loop`, `parse_cond`, `parse_value`, `parse_float`, `parse_digit`, `parse_tildes`, and `parse_standard`.

| benchmark names                                | results   |
|------------------------------------------------|-----------|
| bench_cl_format_reveal_single_a                | 30.682 ns |
| bench_cl_format_making_loop_string             | 103.76 ns |
| bench_normal_making_loop_string                | 58.416 ns |
| bench_cl_format_macro_making_loop_empty_string | 407.45 ns |
| bench_normal_making_loop_empty_string          | 12.184 ns |
| bench_cl_format_making_loop_empty_string       | 49.516 ns |
| bench_cl_format_macro_making_loop_string       | 468.87 ns |

## flamegraph ##

`sudo cargo flamegraph -o bench0_flamegraph.svg --bench bench0 -- --bench`

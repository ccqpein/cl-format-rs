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


## flamegraph ##

`sudo cargo flamegraph -o bench0_flamegraph.svg --bench bench0 -- --bench`

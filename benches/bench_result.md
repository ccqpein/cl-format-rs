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


bench_cl_format_making_loop_string target is <200ns

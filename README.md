# pars

A simple Rust CLI tool to extract Python function definitions from a `.py` file.

### Vision:
Build a tool that statically parses an entire codebase, giving relevant information like function definitions and heirarchy, the caller-calle relationship etc. A better developed version of the sidebar visible on right-side when you open a file on the github website is maybe a decent description. 

## Tasks

- [ ] How does `Arc` and `Mutex` actually work?
- [ ] Benchmark with different file sizes.
- [ ] The code for parsing is extremely inefficient, from a simplicity of understanding perspective if not speed; find a better way.
- [ ] Define a file which lists function signature syntax across different languages to extend usability.
- [ ] Extend parsing to constants, macros, structs, classes and other common patterns across languages.


## Install

```cargo install pars```

## Usage
```pars path/to/file.py```

## Example 
```pars example.py ```

```

Path: /Users/simarmalhotra/cv/cv.py
Total file length: 10311 bytes
[Thread] Read offset 0, size 10311
Done reading in 725.25µs

Function Call Hierarchy:
---------------------------
└── chan_vese (line 210)
    ├── supported_float_type (line 10)
    ├── _cv_init_level_set (line 194)
    │   ├── _cv_checkerboard (line 154)
    │   ├── _cv_large_disk (line 168)
    │   └── _cv_small_disk (line 181)
    ├── format_time (line 6)
    ├── _cv_energy (line 126)
    │   ├── _cv_heavyside (line 78)
    │   ├── _cv_difference_from_average_term (line 107)
    │   │   └── _cv_calculate_averages (line 92)
    │   └── _cv_edge_length_term (line 116)
    │       └── _cv_delta (line 85)
    ├── _cv_calculate_variation (line 32)
    └── _cv_reset_level_set (line 147)

```


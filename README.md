# pars

A simple Rust CLI tool to extract Python function definitions from a `.py` file.

### Vision:
Build a tool that parses an entire codebase, giving relevant information like function definitions and heirarchy, and more stuff that I haven't wrapped my brain around yet. (and maybe) augment w/ llms. 

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


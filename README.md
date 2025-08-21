# pars

A simple Rust CLI tool to extract Python function definitions from a `.py` file.

### Vision:
Build a tool that statically parses an entire codebase, giving relevant information like function definitions and heirarchy, the caller-calle relationship etc. A better developed version of the sidebar visible on right-side when you open a file on the github website is maybe a decent description. 



## Install

```cargo install pars```

## Usage
```pars path/to/file.py```

## Example 
```pars example.py ```

```
Analyzing file: /Users/***/cv/cv.py
Configuration: threads=8, block_size=16KB, cache=true, parallel_read=false
File size: 10311 bytes

Using cached parse results // or Cached parse results to: /Users/***/cv/cv.cache
Parsing completed in 351.917µs
Found 15 functions

Function Call Hierarchy:
========================================
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

Summary:
  Total functions: 15
  Root functions: 1
  Orphan functions: 0
  Total function calls: 14 

```


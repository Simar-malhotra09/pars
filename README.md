# pars

A simple Rust CLI tool to extract Python function definitions from a `.py` file.

### Vision:
Build a tool that parses an entire codebase, giving relevant information like function definitions and heirarchy, and more stuff that I haven't wrapped my brain around yet. (and maybe) augment w/ llms. 

## Install

```cargo install pars```

## Usage
```pars path/to/file.py```

## Example 
pars example.py
expect:

```
Function Call Hierarchy:
---------------------------
└── save_combined_image (line 243)
    └── run_and_display (line 233)
        └── process_irregular_polygons (line 94)
            ├── advanced_shape_filtering (line 10)
            ├── smooth_contour_spline (line 42)
            └── calculate_polygon_area_robust (line 70)
```


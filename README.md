# pars

A simple Rust CLI tool to extract Python function definitions from a `.py` file.

## Install

```cargo install pars```

## Usage
```pars path/to/file.py```

## Example 

example.py
```
def greet(name):
    return "Hello " + name

def add(
    x,
    y
):
    return x + y
```
run

```
pars example.py
```

expect
```
[FUNC]: def greet(name):
[FUNC]: def add( x, y ):
```

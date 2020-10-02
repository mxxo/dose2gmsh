# dose2gmsh
[![Crates](https://img.shields.io/crates/v/dose2gmsh.svg)](https://crates.io/crates/dose2gmsh)
[![Docs](https://docs.rs/dose2gmsh/badge.svg)](https://docs.rs/dose2gmsh/)

Convert `EGSnrc 3ddose` files for visualization in Gmsh. 

Check out the [API documentation](https://docs.rs/dose2gmsh/latest).

# Usage
```shell
# convert water_block.3ddose to msh
$ dose2gmsh --input-file=water_block.3ddose

# specify the output file name
$ dose2gmsh --input-file=water_block.3ddose  --output-file=output.msh

$ dose2gmsh --help 

dose2gmsh 0.1.0
Max Orok <maxwellorok@gmail.com>
Convert dosxyznrc 3ddose files to Gmsh msh files

USAGE:
    dose2gmsh.exe [OPTIONS] --input-file <input-file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input-file <input-file>      The input 3ddose file
    -o, --output-file <output-file>    The output file name, defaults to <input_file>.msh
```

## Supported formats 
* Gmsh mesh version 2 `[.msh]` 

## Installation

You'll need `cargo`, the Rust package manager to get started. 

Install using `cargo install`. 

```shell
$ cargo install dose2gmsh
```

You can update your installation with `cargo install --force`. 

## Adding output formats 

`dose2gmsh` parses `3ddose` files into a intermediate `DoseBlock` structure. After parsing, it writes the `DoseBlock` data to an output file. 

You can add an output format by implementing a new `DoseBlock::write*` method. Use `DoseBlock::write_gmsh` as a starting point. 

If you'd like help implementing a new output format, open an issue.  

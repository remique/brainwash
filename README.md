# Brainwash

Toy Brainfuck Compiler written with Rust & LLVM as a learning project (before diving into the rabbit hole of implementing a fully-fledged compiler).

## Usage 

To compile and run this program, you need to have LLVM installed on your machine. Help page:

```
$ cargo run -- -h

Brainwash
Emil Jaszczuk <emj1054@gmail.com>

USAGE:
    bf-compiler [FLAGS] [OPTIONS] <INPUT>

FLAGS:
    -h, --help        Prints help information
    -p, --profiler    Shows how long each step takes (unimplemented)

OPTIONS:
    -o, --output <FILE>    Sets the output file [default: main]

ARGS:
    <INPUT>    Sets the input file to compile
```

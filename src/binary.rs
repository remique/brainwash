use std::error::Error;
use std::fmt::Display;
use std::io;
use std::path::Path;
use std::process::{Command, Output};

pub struct Optimizer;

impl Optimizer {
    pub fn new() {}
}

pub struct BinaryGenerator<T: AsRef<Path> + Display> {
    input: T,
}

impl<T: AsRef<Path> + Display> BinaryGenerator<T> {
    pub fn new(filename: T) -> Self {
        BinaryGenerator { input: filename }
    }

    /// Function that uses all the utility functions that
    /// generate an executable binary
    pub fn compile(&self) -> Result<(), Box<dyn Error>> {
        self.generate_bitcode()?;
        self.generate_assembly()?;
        self.generate_executable()?;
        self.remove_intermediate()?;

        Ok(())
    }

    /// Generate LLVM bitcode file
    fn generate_bitcode(&self) -> io::Result<Output> {
        Command::new("llvm-as")
            .args(&[
                format!("{}.ll", self.input).as_str(),
                "-o",
                format!("{}.bc", self.input).as_str(),
            ])
            .output()
    }

    /// Compile LLVM bitcode into architecture specific assembly
    fn generate_assembly(&self) -> io::Result<Output> {
        Command::new("llc")
            .args(&[
                "-filetype=obj",
                format!("{}.bc", self.input).as_str(),
                "-o",
                format!("{}.o", self.input).as_str(),
            ])
            .output()
    }

    /// Compile assembly into an executable file with clang
    fn generate_executable(&self) -> io::Result<Output> {
        Command::new("clang")
            .args(&[
                format!("{}.o", self.input).as_str(),
                "-o",
                format!("{}", self.input).as_str(),
            ])
            .output()
    }

    /// Utility function that removes all intermediate files manually
    fn remove_intermediate(&self) -> io::Result<Output> {
        Command::new("rm")
            .args(&[
                "-rf",
                format!("{}.ll", self.input).as_str(),
                format!("{}.bc", self.input).as_str(),
                format!("{}.o", self.input).as_str(),
            ])
            .output()
    }
}

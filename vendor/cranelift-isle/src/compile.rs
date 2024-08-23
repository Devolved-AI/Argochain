//! Compilation process, from AST to Sema to Sequences of Insts.

use std::path::Path;

use crate::error::Errors;
use crate::{ast, codegen, sema};

/// Compile the given AST definitions into Rust source code.
pub fn compile(defs: &ast::Defs, options: &codegen::CodegenOptions) -> Result<String, Errors> {
    let mut typeenv = sema::TypeEnv::from_ast(defs)?;
    let termenv = sema::TermEnv::from_ast(&mut typeenv, defs)?;
    let terms = crate::overlap::check(&typeenv, &termenv)?;
    Ok(codegen::codegen(&typeenv, &termenv, &terms, options))
}

/// Compile the given files into Rust source code.
pub fn from_files<P: AsRef<Path>>(
    inputs: impl IntoIterator<Item = P>,
    options: &codegen::CodegenOptions,
) -> Result<String, Errors> {
    let lexer = crate::lexer::Lexer::from_files(inputs)?;
    let defs = crate::parser::parse(lexer)?;
    compile(&defs, options)
}

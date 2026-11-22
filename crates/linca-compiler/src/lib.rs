//! The Linca compiler as a library (Compiler RFC §10).

pub mod diag;
pub mod source;

pub use diag::Diagnostic;
pub use source::{SourceFile, SourceSet};

/// Compiles a schema root and returns its diagnostics.
///
/// No phase exists yet, so the result is always empty.
pub fn compile(sources: &SourceSet) -> Vec<Diagnostic> {
    let _ = sources;
    Vec::new()
}

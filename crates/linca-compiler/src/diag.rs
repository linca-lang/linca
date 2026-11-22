//! Diagnostic codes and diagnostics.

/// Every diagnostic code the compiler can report, sorted. Empty until a check is implemented.
pub const KNOWN_CODES: &[&str] = &[];

/// A diagnostic with the position of its primary span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    /// Path of the file relative to the schema root, with `/` as separator.
    pub file: String,
    /// 1-based line where the primary span starts.
    pub line: u32,
}

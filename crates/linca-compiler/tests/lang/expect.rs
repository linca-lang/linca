//! Stage 1 rules of tests/data/lang/README.md: annotations and the comparison.

use std::collections::BTreeSet;

const MARKER: &str = "//~";
const CODE_LEN: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Annotation {
    pub file: String,
    pub line: u32,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reported {
    pub file: String,
    pub line: u32,
    pub code: String,
}

pub fn is_code(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == CODE_LEN && matches!(b[0], b'E' | b'W') && b[1..].iter().all(u8::is_ascii_digit)
}

/// Scans raw lines for `//~ <CODE>`, also inside block comments and strings.
/// A `//~` that is not followed by whitespace and a code is an error of the case file.
pub fn parse_annotations(file: &str, text: &str) -> Result<Vec<Annotation>, String> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let number = index as u32 + 1;
        let malformed = || format!("{file}:{number}: malformed annotation, expected `//~ <CODE>`");
        let mut rest = line;
        while let Some(at) = rest.find(MARKER) {
            let after = &rest[at + MARKER.len()..];
            let tail = after.trim_start_matches([' ', '\t']);
            let spaced = tail.len() < after.len();
            let code = tail
                .get(..CODE_LEN)
                .filter(|code| is_code(code))
                .ok_or_else(malformed)?;
            let tail = &tail[CODE_LEN..];
            if !spaced || tail.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                return Err(malformed());
            }
            found.push(Annotation {
                file: file.to_owned(),
                line: number,
                code: code.to_owned(),
            });
            rest = tail;
        }
    }
    Ok(found)
}

/// Returns one message per violated rule; empty means the case passes.
///
/// 1. every annotation has a reported diagnostic with its code, file and line;
/// 2. the set of reported codes equals the set of annotated codes;
/// 3. a file without annotations has no diagnostics.
///
/// More diagnostics with an annotated code on other lines are allowed.
pub fn compare(annotations: &[Annotation], reported: &[Reported]) -> Vec<String> {
    let mut problems = Vec::new();

    for a in annotations {
        let hit = reported
            .iter()
            .any(|r| r.file == a.file && r.line == a.line && r.code == a.code);
        if !hit {
            problems.push(format!(
                "{}:{}: expected {}, not reported there",
                a.file, a.line, a.code
            ));
        }
    }

    let annotated: BTreeSet<&str> = annotations.iter().map(|a| a.code.as_str()).collect();
    let got: BTreeSet<&str> = reported.iter().map(|r| r.code.as_str()).collect();
    for code in got.difference(&annotated) {
        problems.push(format!(
            "code {code} is reported but not annotated anywhere"
        ));
    }
    for code in annotated.difference(&got) {
        problems.push(format!("code {code} is annotated but never reported"));
    }

    let annotated_files: BTreeSet<&str> = annotations.iter().map(|a| a.file.as_str()).collect();
    for r in reported {
        if !annotated_files.contains(r.file.as_str()) {
            problems.push(format!(
                "{}:{}: {} reported in a file without annotations",
                r.file, r.line, r.code
            ));
        }
    }

    problems.sort();
    problems.dedup();
    problems
}

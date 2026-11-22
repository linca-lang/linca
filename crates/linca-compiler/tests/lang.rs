//! The language test case runner (Compiler RFC §11, tests/data/lang/README.md).

#[path = "lang/expect.rs"]
mod expect;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use expect::{Annotation, Reported, compare, is_code, parse_annotations};
use libtest_mimic::{Arguments, Failed, Trial};
use linca_compiler::{SourceSet, compile, diag::KNOWN_CODES};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn data_dir() -> PathBuf {
    repo_root().join("tests/data/lang")
}

/// One case: a single `.linca` file or a schema root directory.
struct Case {
    /// Path below tests/data/lang, `/` separated, without the `.linca` extension for a file.
    id: String,
    root: bool,
    pending: bool,
}

/// Names of the two cases of a diagnostic directory.
const TWINS: [&str; 2] = ["bad", "ok"];

fn sorted_entries(dir: &Path) -> Vec<(String, bool)> {
    let mut entries: Vec<(String, bool)> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| {
            let entry = entry.expect("read directory entry");
            let name = entry.file_name().into_string().expect("UTF-8 file name");
            (name, entry.path().is_dir())
        })
        .collect();
    entries.sort();
    entries
}

/// Finds the cases by the layout of tests/data/lang/README.md:
/// `diagnostics/<CODE>_<name>/{bad,ok}[.linca]`, `valid/<NN_section>/<file>.linca` or
/// `valid/<NN_section>/<dir>/`, `projects/<name>/`, `pending/<file>.linca`.
fn discover(data: &Path) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut add = |id: String, root: bool, pending: bool| cases.push(Case { id, root, pending });
    let file_stem = |name: &str| name.strip_suffix(".linca").map(str::to_owned);

    for (group, _) in sorted_entries(&data.join("diagnostics"))
        .into_iter()
        .filter(|e| e.1)
    {
        for (name, is_dir) in sorted_entries(&data.join("diagnostics").join(&group)) {
            let id = |stem: &str| format!("diagnostics/{group}/{stem}");
            match (is_dir, file_stem(&name)) {
                (true, _) if TWINS.contains(&name.as_str()) => add(id(&name), true, false),
                (false, Some(stem)) if TWINS.contains(&stem.as_str()) => {
                    add(id(&stem), false, false)
                }
                _ => {}
            }
        }
    }
    for (section, _) in sorted_entries(&data.join("valid"))
        .into_iter()
        .filter(|e| e.1)
    {
        for (name, is_dir) in sorted_entries(&data.join("valid").join(&section)) {
            let id = |stem: &str| format!("valid/{section}/{stem}");
            match (is_dir, file_stem(&name)) {
                (true, _) => add(id(&name), true, false),
                (false, Some(stem)) => add(id(&stem), false, false),
                _ => {}
            }
        }
    }
    for (name, _) in sorted_entries(&data.join("projects"))
        .into_iter()
        .filter(|e| e.1)
    {
        add(format!("projects/{name}"), true, false);
    }
    for (name, is_dir) in sorted_entries(&data.join("pending")) {
        if let (false, Some(stem)) = (is_dir, file_stem(&name)) {
            add(format!("pending/{stem}"), false, true);
        }
    }
    cases
}

fn fail(problems: Vec<String>) -> Result<(), Failed> {
    if problems.is_empty() {
        Ok(())
    } else {
        Err(Failed::from(problems.join("\n")))
    }
}

/// Compiles one case and applies the stage 1 rules.
fn run_case(case: &Case) -> Result<(), Failed> {
    let data = data_dir();
    let (sources, prefix) = if case.root {
        (
            SourceSet::load_root(&data.join(&case.id)),
            format!("{}/", case.id),
        )
    } else {
        let path = data.join(format!("{}.linca", case.id));
        let dir = case.id.rsplit_once('/').map_or("", |(dir, _)| dir);
        (SourceSet::load_file(&path), format!("{dir}/"))
    };
    let sources = sources.map_err(|e| Failed::from(format!("cannot load case: {e}")))?;

    let mut annotations: Vec<Annotation> = Vec::new();
    for file in sources.files() {
        let name = format!("{prefix}{}", file.path);
        annotations.extend(parse_annotations(&name, &file.text).map_err(Failed::from)?);
    }
    let reported: Vec<Reported> = compile(&sources)
        .into_iter()
        .map(|d| Reported {
            file: format!("{prefix}{}", d.file),
            line: d.line,
            code: d.code.to_owned(),
        })
        .collect();

    fail(compare(&annotations, &reported))
}

fn case_trials() -> Vec<Trial> {
    let cases = discover(&data_dir());
    let found = !cases.is_empty();
    let discovery = Trial::test("cases::discovery_finds_cases", move || {
        if found {
            Ok(())
        } else {
            Err(Failed::from("no cases found in tests/data/lang"))
        }
    });
    std::iter::once(discovery)
        .chain(cases.into_iter().map(|case| {
            // Pending cases need plugin configuration and are not runnable yet (README).
            let pending = case.pending;
            Trial::test(format!("cases::{}", case.id), move || run_case(&case))
                .with_ignored_flag(pending)
        }))
        .collect()
}

// ---- code table -----------------------------------------------------------

/// Codes in the table of Specification §12.
fn spec_codes() -> BTreeSet<String> {
    let text =
        std::fs::read_to_string(repo_root().join("docs/Specification.md")).expect("read spec");
    let mut in_section = false;
    let mut codes = BTreeSet::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            in_section = line.starts_with("## 12.");
        } else if in_section
            && let Some(cell) = line.strip_prefix('|').and_then(|l| l.split('|').next())
            && is_code(cell.trim())
        {
            codes.insert(cell.trim().to_owned());
        }
    }
    codes
}

/// Codes annotated in every `.linca` file of the cases, pending ones included.
fn annotated_codes() -> Result<BTreeSet<String>, String> {
    let mut codes = BTreeSet::new();
    let mut stack = vec![data_dir()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "linca") {
                let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                for a in parse_annotations(&path.display().to_string(), &text)? {
                    codes.insert(a.code);
                }
            }
        }
    }
    Ok(codes)
}

fn diff(name_a: &str, a: &BTreeSet<String>, name_b: &str, b: &BTreeSet<String>) -> Vec<String> {
    let only_a = a
        .difference(b)
        .map(|c| format!("{c} is in {name_a} but not in {name_b}"));
    let only_b = b
        .difference(a)
        .map(|c| format!("{c} is in {name_b} but not in {name_a}"));
    only_a.chain(only_b).collect()
}

fn check_spec_vs_cases() -> Result<(), Failed> {
    let cases = annotated_codes().map_err(Failed::from)?;
    fail(diff(
        "Specification §12",
        &spec_codes(),
        "the cases",
        &cases,
    ))
}

fn check_spec_vs_compiler() -> Result<(), Failed> {
    let known: BTreeSet<String> = KNOWN_CODES.iter().map(|c| (*c).to_owned()).collect();
    fail(diff(
        "Specification §12",
        &spec_codes(),
        "the compiler",
        &known,
    ))
}

fn code_table_trials() -> Vec<Trial> {
    vec![
        Trial::test("code_table::spec_and_cases_agree", check_spec_vs_cases),
        // Inverted on purpose: it passes while the check fails and starts failing the moment the
        // compiler knows every code, which is the signal to remove this wrapper.
        Trial::test(
            "code_table::compiler_knows_spec_codes (expected to fail: compiler has no codes yet)",
            || match check_spec_vs_compiler() {
                Err(_) => Ok(()),
                Ok(()) => Err(Failed::from(
                    "the compiler knows every code: drop this wrapper",
                )),
            },
        ),
    ]
}

// ---- tests of the runner itself --------------------------------------------

fn ann(file: &str, line: u32, code: &str) -> Annotation {
    Annotation {
        file: file.into(),
        line,
        code: code.into(),
    }
}

fn rep(file: &str, line: u32, code: &str) -> Reported {
    Reported {
        file: file.into(),
        line,
        code: code.into(),
    }
}

fn must_differ(annotations: &[Annotation], reported: &[Reported]) -> Result<(), Failed> {
    if compare(annotations, reported).is_empty() {
        Err(Failed::from("comparison passed, it must fail"))
    } else {
        Ok(())
    }
}

fn must_match(annotations: &[Annotation], reported: &[Reported]) -> Result<(), Failed> {
    fail(compare(annotations, reported))
}

fn expect_eq<T: PartialEq + std::fmt::Debug>(got: T, want: T) -> Result<(), Failed> {
    if got == want {
        Ok(())
    } else {
        Err(Failed::from(format!("got {got:?}, want {want:?}")))
    }
}

fn runner_trials() -> Vec<Trial> {
    let t = |name: &str, f: fn() -> Result<(), Failed>| Trial::test(format!("runner::{name}"), f);
    vec![
        t("parse_annotation_at_line_end", || {
            let got = parse_annotations("a", "x: Int,    //~ E0106\n").map_err(Failed::from)?;
            expect_eq(got, vec![ann("a", 1, "E0106")])
        }),
        t("parse_annotation_inside_block_comment", || {
            let got = parse_annotations("a", "/* open\n  //~ W0001\n*/\n").map_err(Failed::from)?;
            expect_eq(got, vec![ann("a", 2, "W0001")])
        }),
        t("parse_two_annotations_on_one_line", || {
            let got = parse_annotations("a", "x //~ E0101 //~ E0102").map_err(Failed::from)?;
            expect_eq(got, vec![ann("a", 1, "E0101"), ann("a", 1, "E0102")])
        }),
        t("parse_no_annotation", || {
            expect_eq(
                parse_annotations("a", "// plain comment\nx: Int\n"),
                Ok(vec![]),
            )
        }),
        t("parse_malformed_annotation_is_an_error", || {
            for bad in [
                "x //~ e0101",
                "x //~E0101",
                "x //~ E01",
                "x //~",
                "x //~ E0101x",
            ] {
                if parse_annotations("a", bad).is_ok() {
                    return Err(Failed::from(format!("accepted {bad:?}")));
                }
            }
            Ok(())
        }),
        t("compare_exact_match_passes", || {
            must_match(&[ann("a", 3, "E0101")], &[rep("a", 3, "E0101")])
        }),
        t("compare_same_code_on_further_line_passes", || {
            must_match(
                &[ann("a", 3, "E0502")],
                &[rep("a", 3, "E0502"), rep("a", 9, "E0502")],
            )
        }),
        t("compare_clean_file_without_diagnostics_passes", || {
            must_match(&[], &[])
        }),
        t("compare_missing_code_fails", || {
            must_differ(&[ann("a", 3, "E0101")], &[])
        }),
        t("compare_extra_code_fails", || {
            must_differ(
                &[ann("a", 3, "E0101")],
                &[rep("a", 3, "E0101"), rep("a", 5, "E0102")],
            )
        }),
        t("compare_code_on_wrong_line_fails", || {
            must_differ(&[ann("a", 3, "E0101")], &[rep("a", 4, "E0101")])
        }),
        t("compare_code_in_wrong_file_fails", || {
            must_differ(&[ann("a", 3, "E0101")], &[rep("b", 3, "E0101")])
        }),
        t("compare_two_codes_on_one_line_fails", || {
            must_differ(
                &[ann("a", 3, "E0101")],
                &[rep("a", 3, "E0101"), rep("a", 3, "E0102")],
            )
        }),
        t(
            "compare_file_without_annotations_with_diagnostic_fails",
            || must_differ(&[], &[rep("a", 1, "E0101")]),
        ),
        t(
            "compare_unannotated_file_next_to_annotated_one_fails",
            || {
                must_differ(
                    &[ann("a", 3, "E0101")],
                    &[rep("a", 3, "E0101"), rep("b", 1, "E0101")],
                )
            },
        ),
    ]
}

fn main() {
    let args = Arguments::from_args();
    let mut trials = case_trials();
    trials.extend(code_table_trials());
    trials.extend(runner_trials());
    libtest_mimic::run(&args, trials).exit();
}

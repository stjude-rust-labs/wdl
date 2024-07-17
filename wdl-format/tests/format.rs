//! The format file tests.
//!
//! This test looks for directories in `tests/format`.
//!
//! Each directory is expected to contain:
//!
//! * `source.wdl` - the test input source to parse.
//! * `source.formatted` - the expected formatted output.
//!
//! The `source.formatted` file may be automatically generated or updated by
//! setting the `BLESS` environment variable when running this test.

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Buffer;
use codespan_reporting::term::Config;
use colored::Colorize;
use pretty_assertions::StrComparison;
use rayon::prelude::*;
use wdl_ast::Diagnostic;
use wdl_format::format::format_document;

fn find_tests() -> Vec<PathBuf> {
    // Check for filter arguments consisting of test names
    let mut filter = HashSet::new();
    for arg in std::env::args().skip_while(|a| a != "--").skip(1) {
        if !arg.starts_with('-') {
            filter.insert(arg);
        }
    }

    let mut tests: Vec<PathBuf> = Vec::new();
    for entry in Path::new("tests/format").read_dir().unwrap() {
        let entry = entry.expect("failed to read directory");
        let path = entry.path();
        if !path.is_dir()
            || (!filter.is_empty()
                && !filter.contains(entry.file_name().to_str().expect("name should be UTF-8")))
        {
            continue;
        }

        tests.push(path);
    }

    tests.sort();
    tests
}

fn format_diagnostics(diagnostics: &[Diagnostic], path: &Path, source: &str) -> String {
    let file = SimpleFile::new(path.as_os_str().to_str().unwrap(), source);
    let mut buffer = Buffer::no_color();
    for diagnostic in diagnostics {
        term::emit(
            &mut buffer,
            &Config::default(),
            &file,
            &diagnostic.to_codespan(),
        )
        .expect("should emit");
    }

    String::from_utf8(buffer.into_inner()).expect("should be UTF-8")
}

fn compare_result(path: &Path, result: &str) -> Result<(), String> {
    if env::var_os("BLESS").is_some() {
        fs::write(path, &result).map_err(|e| {
            format!(
                "failed to write result file `{path}`: {e}",
                path = path.display()
            )
        })?;
        return Ok(());
    }

    let expected = fs::read_to_string(path)
        .map_err(|e| {
            format!(
                "failed to read result file `{path}`: {e}",
                path = path.display()
            )
        })?
        .replace("\r\n", "\n");

    if expected != result {
        return Err(format!(
            "result is not as expected:\n{}",
            StrComparison::new(&expected, &result),
        ));
    }

    Ok(())
}

fn run_test(test: &Path, ntests: &AtomicUsize) -> Result<(), String> {
    let path = test.join("source.wdl");
    let source = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "failed to read source file `{path}`: {e}",
            path = path.display()
        )
    })?;

    let formatted = format_document(&source).map_err(|e| {
        format!(
            "failed to format `{path}`: {e}",
            path = path.display(),
            e = format_diagnostics(&e, path.as_path(), &source)
        )
    })?;
    compare_result(path.with_extension("formatted").as_path(), &formatted)?;

    ntests.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn main() {
    let tests = find_tests();
    println!("\nrunning {} tests\n", tests.len());

    let ntests = AtomicUsize::new(0);
    let errors = tests
        .par_iter()
        .filter_map(|test| {
            let test_name = test.file_stem().and_then(OsStr::to_str).unwrap();
            match std::panic::catch_unwind(|| {
                match run_test(test, &ntests)
                    .map_err(|e| format!("failed to run test `{path}`: {e}", path = test.display()))
                    .err()
                {
                    Some(e) => {
                        println!("test {test_name} ... {failed}", failed = "failed".red());
                        Some((test_name, e))
                    }
                    None => {
                        println!("test {test_name} ... {ok}", ok = "ok".green());
                        None
                    }
                }
            }) {
                Ok(result) => result,
                Err(e) => {
                    println!(
                        "test {test_name} ... {panicked}",
                        panicked = "panicked".red()
                    );
                    Some((
                        test_name,
                        format!(
                            "test panicked: {e:?}",
                            e = e
                                .downcast_ref::<String>()
                                .map(|s| s.as_str())
                                .or_else(|| e.downcast_ref::<&str>().copied())
                                .unwrap_or("no panic message")
                        ),
                    ))
                }
            }
        })
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        eprintln!(
            "\n{count} test(s) {failed}:",
            count = errors.len(),
            failed = "failed".red()
        );

        for (name, msg) in errors.iter() {
            eprintln!("{name}: {msg}", msg = msg.red());
        }

        exit(1);
    }

    println!(
        "\ntest result: ok. {} passed\n",
        ntests.load(Ordering::SeqCst)
    );
}
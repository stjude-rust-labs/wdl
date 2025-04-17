//! The wdl-doc tests.
//!
//! This test documents the contents of the `tests/codebase` directory.
//!
//! The built docs are expected to be in the `tests/output_docs` directory.
//!
//! The docs may be automatically generated or updated by
//! setting the `BLESS` environment variable when running this test.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

use pretty_assertions::StrComparison;
use wdl_doc::document_workspace;

/// Copied from https://stackoverflow.com/a/65192210
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Recursively read every file in a directory
fn read_dir_recursively(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(read_dir_recursively(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

#[tokio::main]
async fn main() {
    let test_dir = Path::new("tests").join("codebase");
    let docs_dir = Path::new("tests").join("output_docs");

    // If `tests/codebase/docs` exists, delete it
    if test_dir.join("docs").exists() {
        fs::remove_dir_all(test_dir.join("docs")).unwrap();
    }

    match document_workspace(test_dir.to_path_buf(), None::<&str>, true).await {
        Ok(_) => {
            println!("Successfully generated docs");
        }
        Err(e) => {
            eprintln!("Failed to generate docs: {}", e);
            exit(1);
        }
    }

    // If the `BLESS` environment variable is set, update the expected output
    // by deleting the contents of the `tests/output_docs` directory and
    // repopulating it with the generated docs (at `tests/codebase/docs/`).
    if env::var("BLESS").is_ok() {
        if docs_dir.exists() {
            fs::remove_dir_all(&docs_dir).unwrap();
        }
        fs::create_dir_all(&docs_dir).unwrap();
        copy_dir_all(test_dir.join("docs"), &docs_dir).unwrap();

        println!("Blessed docs");
        exit(0);
    }

    // Compare the generated docs with the expected output.
    // Recursively read the contents of the `tests/codebase/docs` directory
    // and compare them with the contents of the `tests/output_docs` directory.
    // If the contents are different, print the differences and exit with a
    // non-zero exit code.
    let mut success = true;
    for file_name in read_dir_recursively(&test_dir.join("docs")).unwrap() {
        let expected_file = docs_dir.join(file_name.strip_prefix(test_dir.join("docs")).unwrap());
        if !expected_file.exists() {
            println!("Missing file: {}", expected_file.display());
            success = false;
            continue;
        }

        if expected_file.extension().and_then(|e| e.to_str()) == Some("png") {
            // Ignore image files
            continue;
        }

        let expected_contents = fs::read_to_string(&expected_file)
            .unwrap()
            .replace("\\", "/");
        let generated_contents = fs::read_to_string(&file_name)
            .unwrap()
            .replace("\r\n", "\n")
            .replace("\\", "/");

        if expected_contents != generated_contents {
            println!("File contents differ: {}", expected_file.display());
            println!(
                "Diff:\n{}",
                StrComparison::new(&expected_contents, &generated_contents)
            );
            success = false;
        }
    }

    if success {
        println!("Docs are as expected");
        exit(0);
    } else {
        exit(1);
    }
}

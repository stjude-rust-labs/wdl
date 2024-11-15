//! Module for utility functions to support the standard library.

use std::borrow::Cow;
use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use itertools::Either;
use wdl_grammar::lexer::Lexer;
use wdl_grammar::lexer::v1::Token;

use crate::Array;
use crate::CompoundValue;
use crate::PrimitiveValue;
use crate::StorageUnit;
use crate::Value;

/// Used to calculate the disk size of a value.
///
/// The value may be a file or a directory or a compound type containing files
/// or directories.
///
/// The size of a directory is based on the sum of the files contained in the
/// directory.
pub fn calculate_disk_size(value: &Value, unit: StorageUnit, cwd: &Path) -> Result<f64> {
    match value {
        Value::None => Ok(0.0),
        Value::Primitive(v) => primitive_disk_size(v, unit, cwd),
        Value::Compound(v) => compound_disk_size(v, unit, cwd),
    }
}

/// Calculates the disk size of the given primitive value in the given unit.
fn primitive_disk_size(value: &PrimitiveValue, unit: StorageUnit, cwd: &Path) -> Result<f64> {
    match value {
        PrimitiveValue::File(path) => {
            let path = cwd.join(path.as_str());
            let metadata = path.metadata().with_context(|| {
                format!(
                    "failed to read metadata for file `{path}`",
                    path = path.display()
                )
            })?;

            if !metadata.is_file() {
                bail!("path `{path}` is not a file", path = path.display());
            }

            Ok(unit.convert(metadata.len()))
        }
        PrimitiveValue::Directory(path) => calculate_directory_size(&cwd.join(path.as_str()), unit),
        _ => Ok(0.0),
    }
}

/// Calculates the disk size for a compound value in the given unit.
fn compound_disk_size(value: &CompoundValue, unit: StorageUnit, cwd: &Path) -> Result<f64> {
    match value {
        CompoundValue::Pair(pair) => Ok(calculate_disk_size(pair.left(), unit, cwd)?
            + calculate_disk_size(pair.right(), unit, cwd)?),
        CompoundValue::Array(array) => Ok(array.elements().iter().try_fold(0.0, |t, e| {
            anyhow::Ok(t + calculate_disk_size(e, unit, cwd)?)
        })?),
        CompoundValue::Map(map) => Ok(map.elements().iter().try_fold(0.0, |t, (k, v)| {
            anyhow::Ok(t + primitive_disk_size(k, unit, cwd)? + calculate_disk_size(v, unit, cwd)?)
        })?),
        CompoundValue::Object(object) => {
            Ok(object.members().iter().try_fold(0.0, |t, (_, v)| {
                anyhow::Ok(t + calculate_disk_size(v, unit, cwd)?)
            })?)
        }
        CompoundValue::Struct(s) => Ok(s.members().iter().try_fold(0.0, |t, (_, v)| {
            anyhow::Ok(t + calculate_disk_size(v, unit, cwd)?)
        })?),
    }
}

/// Calculates the size of the given directory in the given unit.
fn calculate_directory_size(path: &Path, unit: StorageUnit) -> Result<f64> {
    // Don't follow symlinks as a security measure
    let metadata = path.symlink_metadata().with_context(|| {
        format!(
            "failed to read metadata for directory `{path}`",
            path = path.display()
        )
    })?;

    if !metadata.is_dir() {
        bail!("path `{path}` is not a directory", path = path.display());
    }

    // Create a queue for processing directories
    let mut queue: Vec<Cow<'_, Path>> = Vec::new();
    queue.push(path.into());

    // Process each directory in the queue, adding the sizes of its files
    let mut size = 0.0;
    while let Some(path) = queue.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry.with_context(|| {
                format!(
                    "failed to read entry of directory `{path}`",
                    path = path.display()
                )
            })?;

            // Note: `DirEntry::metadata` doesn't follow symlinks
            let metadata = entry.metadata().with_context(|| {
                format!(
                    "failed to read metadata for file `{path}`",
                    path = entry.path().display()
                )
            })?;
            if metadata.is_dir() {
                queue.push(entry.path().into());
            } else {
                size += unit.convert(metadata.len());
            }
        }
    }

    Ok(size)
}

/// Determines if the given string is a valid WDL identifier.
pub fn is_ident(s: &str) -> bool {
    let mut lexer = Lexer::new(s);
    if !lexer
        .next()
        .map(|r| matches!(r, (Ok(Token::Ident), _)))
        .unwrap_or(false)
    {
        return false;
    }

    lexer.next().is_none()
}

/// Represents a header in a TSV (tab-separated value) file.
pub enum TsvHeader {
    /// The header was explicitly specified as an `Array[String]`.
    Specified(Array),
    /// The header was read from the file.
    File(String),
}

impl TsvHeader {
    /// Gets the column names in the header.
    ///
    /// # Panics
    ///
    /// Panics if a specified header contains a value that is not a string.
    pub fn columns(&self) -> impl Iterator<Item = &str> {
        match self {
            Self::Specified(array) => Either::Left(array.elements().iter().map(|v| {
                v.as_string()
                    .expect("header value must be a string")
                    .as_str()
            })),
            Self::File(s) => Either::Right(s.split('\t')),
        }
    }
}

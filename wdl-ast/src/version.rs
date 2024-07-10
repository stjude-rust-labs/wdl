//! Representation for version definitions.

use std::str::FromStr;

/// Represents a supported V1 WDL version.
// NOTE: it is expected that this enumeration is in increasing order of 1.x versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum V1 {
    /// The document version is 1.0.
    Zero,
    /// The document version is 1.1.
    One,
    /// The document version is 1.2.
    Two,
}

impl std::fmt::Display for V1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            V1::Zero => write!(f, "WDL v1.0"),
            V1::One => write!(f, "WDL v1.1"),
            V1::Two => write!(f, "WDL v1.2"),
        }
    }
}

/// Represents a supported WDL version.
// NOTE: it is expected that this enumeration is in increasing order of WDL versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SupportedVersion {
    /// The document version is 1.x.
    V1(V1),
}

impl std::fmt::Display for SupportedVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedVersion::V1(version) => write!(f, "{version}"),
        }
    }
}

impl FromStr for SupportedVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Self::V1(V1::Zero)),
            "1.1" => Ok(Self::V1(V1::One)),
            "1.2" => Ok(Self::V1(V1::Two)),
            _ => Err(()),
        }
    }
}

use crate::SupportedVersion;

/// Configuration for parsers in this crate.
///
/// The `serde` traits are implemented for the purpose of customizing
/// configurations of particular test cases. The representation is not
/// particularly user-friendly, but it could be made so in the future if there
/// is demand.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ParserConfig {
    fallback_version: Option<SupportedVersion>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            fallback_version: None,
        }
    }
}

impl ParserConfig {
    /// Sets a supported version to be used as a fallback if the `version`
    /// statement found in a document is syntactically valid but not
    /// recognized by the parser.
    ///
    /// This is useful for providing a best-effort parse of an unrecognized
    /// version. Since the syntax of WDL documents may change across
    /// different versions, a warning will be emitted if the fallback is
    /// used, and the behavior of the subsequent parse is not specified.
    pub fn with_fallback_version(mut self, fallback_version: SupportedVersion) -> Self {
        self.fallback_version = Some(fallback_version);
        self
    }

    /// Gets the configured fallback version if one is set.
    pub fn fallback_version(&self) -> Option<SupportedVersion> {
        self.fallback_version
    }
}

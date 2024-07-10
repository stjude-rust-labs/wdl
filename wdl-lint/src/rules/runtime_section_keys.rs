//! A lint rule for the `runtime` section keys.
//!
//! Note that this lint rule will only emit diagnostics for WDL documents that
//! have a major version of 1 but a minor version of less than 2, as the
//! `runtime` section was deprecated in WDL v1.2.

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use lazy_static::lazy_static;
use wdl_ast::span_of;
use wdl_ast::v1::RuntimeItem;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::version::V1;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the runtime section rule.
const ID: &str = "RuntimeSectionKeys";

/// A deprecated key.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeprecatedKey {
    /// The key name.
    name: &'static str,
    /// The equivalent key that should be used instead.
    replacement: &'static str,
}

/// A runtime key and the corresponding status of that key.
///
/// These are intended to be assigned at a per-version level of granularity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Key {
    /// A key that is deprecated in favor of another key.
    Deprecated(DeprecatedKey),
    /// A runtime key that is recommended to be included.
    Recommended(&'static str),
    /// A runtime key that has a reserved meaning in the specification but which
    /// execution engines are _not_ required to support. These are also called
    /// "hints" in WDL parlance.
    Hint(&'static str),
    /// A runtime key that has a reserved meaning in the specification and which
    /// execution engines are required to support (but don't necessarily have to
    /// be present in WDL documents).
    ReservedMandatory(&'static str),
}

impl Key {
    /// Returns the name of the key.
    pub fn name(&self) -> &str {
        match self {
            Key::Deprecated(key) => key.name,
            Key::Hint(key) => key,
            Key::Recommended(key) => key,
            Key::ReservedMandatory(key) => key,
        }
    }

    /// Returns whether a key is recommended to be included.
    pub fn is_recommended(&self) -> bool {
        match self {
            Key::Deprecated(_) | Key::Hint(_) => false,
            Key::Recommended(_) | Key::ReservedMandatory(_) => true,
        }
    }
}

lazy_static! {
    /// The mapping between `runtime` keys and their kind for WDL v1.0.
    ///
    /// Link: https://github.com/openwdl/wdl/blob/main/versions/1.0/SPEC.md#runtime-section
    static ref V1_0_KEYS: HashSet<Key> = {
        let mut keys = HashSet::new();
        keys.insert(Key::Recommended("docker"));
        keys.insert(Key::Recommended("memory"));
        keys
    };

    /// The mapping between `runtime` keys and their kind for WDL v1.1.
    ///
    /// Link: https://github.com/openwdl/wdl/blob/wdl-1.1/SPEC.md#runtime-section
    static ref V1_1_KEYS: HashSet<Key> = {
        let mut keys = HashSet::new();
        keys.insert(Key::ReservedMandatory("container"));
        keys.insert(Key::Deprecated(DeprecatedKey{ name: "docker", replacement: "container" }));
        keys.insert(Key::ReservedMandatory("cpu"));
        keys.insert(Key::ReservedMandatory("memory"));
        keys.insert(Key::ReservedMandatory("gpu"));
        keys.insert(Key::ReservedMandatory("disks"));
        keys.insert(Key::ReservedMandatory("maxRetries"));
        keys.insert(Key::ReservedMandatory("returnCodes"));
        keys.insert(Key::Hint("maxCpu"));
        keys.insert(Key::Hint("maxMemory"));
        keys.insert(Key::Hint("shortTask"));
        keys.insert(Key::Hint("localizationOptional"));
        keys.insert(Key::Hint("inputs"));
        keys.insert(Key::Hint("outputs"));
        keys
    };
}

/// Creates a "deprecated runtime key" diagnostic.
fn deprecated_runtime_key(key: &DeprecatedKey, span: Span) -> Diagnostic {
    Diagnostic::warning(format!(
        "the `{}` runtime key has been deprecated in favor of `{}`",
        key.name, key.replacement
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(format!(
        "change the name of the `{}` key to `{}`",
        key.name, key.replacement
    ))
}

/// Creates an "non-reserved runtime key" diagnostic.
fn non_reserved_runtime_key(key: &str, span: Span, specification: &str) -> Diagnostic {
    Diagnostic::warning(format!(
        "the `{key}` runtime key is not reserved in {specification}; therefore, its inclusion in \
         the `runtime` section is deprecated"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(
        "if a reserved key name was intended, correct the spelling; otherwise, remove the key",
    )
}

/// Creates a "missing recommended runtime key" diagnostic.
fn missing_recommended_key(key: &str, span: Span, specification: &str) -> Diagnostic {
    Diagnostic::note(format!(
        "the `{key}` runtime key is recommended by {specification}"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(format!(
        "include an entry for the `{key}` key in the `runtime` section"
    ))
}

/// Inspects a runtime key for issues within v1.1 WDL documents.
fn inspect_v1_1_key(state: &mut Diagnostics, key: &str, span: Span, specification: &str) {
    let entries = V1_1_KEYS
        .iter()
        .filter(|entry| entry.name() == key)
        .collect::<Vec<_>>();

    let entry = match entries.len() {
        0..=1 => entries.into_iter().next(),
        // SAFETY: if more than one key is matched, a key is likely duplicated
        // in the relevant static key map.
        _ => unreachable!(),
    };

    match entry {
        Some(kind) => {
            // If the key was found in the map, the only potential problem that
            // can be encountered is if the key is deprecated.
            if let Key::Deprecated(key) = kind {
                state.add(deprecated_runtime_key(key, span));
            }
        }
        None => {
            // If the key was _not_ found in the map, that means the key was not
            // one of the permitted values for WDL v1.1.
            state.add(non_reserved_runtime_key(key, span, specification));
        }
    }
}

/// Detects the use of deprecated, unknown, or missing runtime keys.
#[derive(Debug, Default, Clone)]
pub struct RuntimeSectionKeysRule {
    /// The detected version of the current document.
    version: Option<SupportedVersion>,
    /// The span of the `runtime` section for the current task.
    current_runtime_span: Option<Span>,
    /// The keys that were seen for the current `runtime` section.
    keys_seen: HashMap<String, Span>,
}

impl Rule for RuntimeSectionKeysRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that `runtime` sections have the appropriate keys."
    }

    fn explanation(&self) -> &'static str {
        "The behavior of this rule is different depending on the WDL version:
        
        For WDL v1.0 documents, the `docker` and `memory` keys are recommended, but the inclusion \
         of any number of other keys is encouraged.
        
        For WDL v1.1 documents, 
        
        - A list of mandatory, reserved keywords will be recommended for inclusion if they are not \
         present. Here, 'mandatory' refers to the requirement that all execution engines support \
         this key—not that the key must be present in the `runtime` section.
        - Optional, reserved \"hint\" keys are also permitted but not flagged when they are \
         missing.
        - Further, the WDL v1.1 specification deprecates the inclusion of non-reserved keys in a \
         `runtime` section. As such, any non-reserved keys will be flagged for removal.
         
         For WDL v1.2 documents and later, this rule does not evaluate because `runtime` sections \
         are deprecated."
    }

    fn tags(&self) -> crate::TagSet {
        TagSet::new(&[Tag::Completeness, Tag::Deprecated])
    }
}

/// A utility method to parse the recommended keys from a static set of runtime
/// keys from either WDL v1.0 or WDL v1.1.
fn recommended_keys<T: Deref<Target = HashSet<Key>>>(keys: T) -> HashSet<Key> {
    keys.clone()
        .into_iter()
        .filter(|key| key.is_recommended())
        .collect::<HashSet<Key>>()
}

impl Visitor for RuntimeSectionKeysRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: wdl_ast::VisitReason,
        document: &wdl_ast::Document,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry.
        *self = Default::default();

        // NOTE: this rule is dependent on the version of the WDL document.
        self.version = document
            .version_statement()
            .and_then(|s| s.version().as_str().parse().ok());
    }

    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        _: &TaskDefinition,
    ) {
        match reason {
            VisitReason::Enter => {
                self.current_runtime_span = None;
                self.keys_seen = Default::default();
            }
            VisitReason::Exit => {
                // If a runtime section span has not been encountered, then
                // presumably there won't be any place to put missing
                // recommended runtime keys.
                let runtime_span = match self.current_runtime_span {
                    Some(span) => span,
                    None => return,
                };

                if let Some(SupportedVersion::V1(minor_version)) = self.version {
                    let recommended_keys = match minor_version {
                        V1::Zero => recommended_keys(&V1_0_KEYS.clone()),
                        V1::One => recommended_keys(&V1_1_KEYS.clone()),
                        _ => Default::default(),
                    };

                    for key in recommended_keys {
                        if !self.keys_seen.contains_key(key.name()) {
                            state.add(missing_recommended_key(
                                key.name(),
                                runtime_span,
                                &format!("the {minor_version} specification"),
                            ));
                        }
                    }
                }
            }
        }
    }

    fn runtime_section(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        section: &RuntimeSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        match self.current_runtime_span {
            // SAFETY: if this occurs, somehow two runtime sections were allowed
            // to be parsed successfully within a WDL document. This should be
            // disallowed by the validation prior to lints running.
            //
            // If that is not the case, then check to ensure whether this lint
            // rule is appropriately resetting itself across documents.
            Some(_) => unreachable!(),
            None => self.current_runtime_span = Some(span_of(section)),
        }
    }

    fn runtime_item(&mut self, state: &mut Self::State, reason: VisitReason, item: &RuntimeItem) {
        if reason == VisitReason::Exit {
            return;
        }

        let name = item.name();

        // If the version is none, either the document has no version (will be
        // reported by another validation rule) or the visitor has not run the
        // `document()` callback (not possible, as that callback always executes
        // first). As such, if the version is `None`, this rule does not apply.
        if let Some(SupportedVersion::V1(minor_version)) = self.version {
            // NOTE: the only keys that actually need to be individually
            // inspected are WDL v1.1 keys because
            //
            // - WDL v1.0 contains no deprecated keys: the only issue that can
            // occur is when one of the two recommendeds key is omitted, and
            // that is handled at the end of the `document()` callback.
            // - WDL v1.2 deprecates the `runtime` section, so any WDL document with a
            //   version of 1.2 or later should ignore the keys and report the section as
            //   deprecated (in another rule).
            if minor_version == V1::One {
                inspect_v1_1_key(
                    state,
                    name.as_str(),
                    name.span(),
                    &format!("the {} specification", minor_version),
                );
            }
        }

        self.keys_seen.insert(name.as_str().to_owned(), name.span());
    }
}

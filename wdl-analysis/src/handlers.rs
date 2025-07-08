//! Language server protocol handlers.

mod completions;
mod find_all_references;
mod goto_definition;

pub use completions::*;
pub use find_all_references::*;
pub use goto_definition::*;
use wdl_ast::Span;

use crate::DiagnosticsConfig;
use crate::Document;
use crate::diagnostics;
use crate::document::ScopeRef;
use crate::types::v1::EvaluationContext;

/// Context for evaluating expression types for lsp handlers.
#[derive(Debug)]
pub struct TypeEvalContext<'a> {
    /// The scope reference containing the variable and name bindings at the
    /// current position.
    scope: ScopeRef<'a>,
    /// The document being analyzed.
    document: &'a Document,
}

impl EvaluationContext for TypeEvalContext<'_> {
    fn version(&self) -> wdl_ast::SupportedVersion {
        self.document
            .version()
            .expect("document should have a version")
    }

    fn resolve_name(&self, name: &str, _span: Span) -> Option<crate::types::Type> {
        self.scope.lookup(name).map(|n| n.ty().clone())
    }

    fn resolve_type_name(
        &mut self,
        name: &str,
        span: Span,
    ) -> std::result::Result<crate::types::Type, wdl_ast::Diagnostic> {
        if let Some(s) = self.document.struct_by_name(name) {
            if let Some(ty) = s.ty() {
                return Ok(ty.clone());
            }
        }
        Err(diagnostics::unknown_type(name, span))
    }

    fn task(&self) -> Option<&crate::document::Task> {
        None
    }

    fn diagnostics_config(&self) -> DiagnosticsConfig {
        DiagnosticsConfig::default()
    }

    fn add_diagnostic(&mut self, _: wdl_ast::Diagnostic) {}
}

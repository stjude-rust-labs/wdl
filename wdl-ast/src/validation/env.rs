//! Validation of `env` declarations.

use crate::AstNodeExt;
use crate::AstToken;
use crate::Diagnostic;
use crate::Diagnostics;
use crate::Document;
use crate::Span;
use crate::SupportedVersion;
use crate::VisitReason;
use crate::Visitor;
use crate::v1;
use crate::version::V1;

/// Creates an "env type not primitive" diagnostic.
fn env_type_not_primitive(env_span: Span, ty: &v1::Type, ty_span: Span) -> Diagnostic {
    Diagnostic::error("environment variable modifier can only be used on primitive types")
        .with_label(
            format!("type `{ty}` cannot be used as an environment variable"),
            ty_span,
        )
        .with_label(
            "declaration is an environment variable due to this modifier",
            env_span,
        )
}

/// Checks the type to see if it is legal as an environment variable.
///
/// Returns `None` if the type is legal otherwise it returns the span of the
/// type.
fn check_type(ty: &v1::Type) -> Option<Span> {
    match ty {
        v1::Type::Map(ty) => Some(ty.span()),
        v1::Type::Array(ty) => Some(ty.span()),
        v1::Type::Pair(ty) => Some(ty.span()),
        v1::Type::Object(ty) => Some(ty.span()),
        v1::Type::Ref(ty) => Some(ty.span()),
        v1::Type::Primitive(_) => None,
    }
}

/// An AST visitor that ensures that environment variable modifiers only exist
/// on primitive type declarations.
#[derive(Debug, Default)]
pub struct EnvVisitor {
    /// The version of the document we're currently visiting.
    version: Option<SupportedVersion>,
}

impl Visitor for EnvVisitor {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        version: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        *self = Default::default();
        self.version = Some(version);
    }

    fn bound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &v1::BoundDecl) {
        // Only visit decls for WDL >=1.2
        if self.version.expect("should have a version") < SupportedVersion::V1(V1::Two) {
            return;
        }

        if reason == VisitReason::Exit {
            return;
        }

        if let Some(env_span) = decl.env().map(|t| t.span()) {
            let ty = decl.ty();
            if let Some(span) = check_type(&ty) {
                state.add(env_type_not_primitive(env_span, &ty, span));
            }
        }
    }

    fn unbound_decl(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        decl: &v1::UnboundDecl,
    ) {
        // Only visit decls for WDL >=1.2
        if self.version.expect("should have a version") < SupportedVersion::V1(V1::Two) {
            return;
        }

        if reason == VisitReason::Exit {
            return;
        }

        if let Some(env_span) = decl.env().map(|t| t.span()) {
            let ty = decl.ty();
            if let Some(span) = check_type(&ty) {
                state.add(env_type_not_primitive(env_span, &ty, span));
            }
        }
    }
}

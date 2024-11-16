//! Module for the WDL standard library implementation.

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use wdl_analysis::stdlib::Binding;
use wdl_analysis::stdlib::STDLIB as ANALYSIS_STDLIB;
use wdl_analysis::types::Type;
use wdl_analysis::types::TypeEq;
use wdl_analysis::types::Types;
use wdl_ast::Diagnostic;
use wdl_ast::Span;

use crate::Coercible;
use crate::EvaluationContext;
use crate::Value;

mod basename;
mod ceil;
mod find;
mod floor;
mod glob;
mod join_paths;
mod matches;
mod max;
mod min;
mod read_boolean;
mod read_float;
mod read_int;
mod read_lines;
mod read_map;
mod read_string;
mod read_tsv;
mod round;
mod size;
mod stderr;
mod stdout;
mod sub;
mod write_lines;
mod write_map;
mod write_tsv;

/// Represents a function call argument.
pub struct CallArgument {
    /// The value of the argument.
    value: Value,
    /// The span of the expression of the argument.
    span: Span,
}

impl CallArgument {
    /// Constructs a new call argument given its value and span.
    pub const fn new(value: Value, span: Span) -> Self {
        Self { value, span }
    }

    /// Constructs a `None` call argument.
    pub const fn none() -> Self {
        Self {
            value: Value::None,
            span: Span::new(0, 0),
        }
    }
}

/// Represents function call context.
pub struct CallContext<'a> {
    /// The evaluation context for the call.
    context: &'a dyn EvaluationContext,
    /// The call site span.
    call_site: Span,
    /// The arguments to the call.
    arguments: &'a [CallArgument],
    /// The return type.
    return_type: Type,
}

impl<'a> CallContext<'a> {
    /// Constructs a new call context given the call arguments.
    pub fn new(
        context: &'a dyn EvaluationContext,
        call_site: Span,
        arguments: &'a [CallArgument],
        return_type: Type,
    ) -> Self {
        Self {
            context,
            call_site,
            arguments,
            return_type,
        }
    }

    /// Gets the types collection associated with the call.
    pub fn types(&self) -> &Types {
        self.context.types()
    }

    /// Gets the current working directory for the call.
    pub fn cwd(&self) -> &Path {
        self.context.cwd()
    }

    /// Gets the temp directory for the call.
    pub fn tmp(&self) -> &Path {
        self.context.tmp()
    }

    /// Gets the stdout value for the call.
    pub fn stdout(&self) -> Option<Value> {
        self.context.stdout()
    }

    /// Gets the stderr value for the call.
    pub fn stderr(&self) -> Option<Value> {
        self.context.stderr()
    }

    /// Coerces an argument to the given type.
    ///
    /// # Panics
    ///
    /// Panics if the given index is out of range or if the value fails to
    /// coerce to the given type.
    #[inline]
    fn coerce_argument(&self, index: usize, ty: impl Into<Type>) -> Value {
        self.arguments[index]
            .value
            .coerce(self.context.types(), ty.into())
            .expect("value should coerce")
    }

    /// Checks to see if the calculated return type equals the given type.
    ///
    /// This is only used in assertions made by the function implementations.
    #[cfg(debug_assertions)]
    fn return_type_eq(&self, ty: impl Into<Type>) -> bool {
        self.return_type.type_eq(self.context.types(), &ty.into())
    }
}

/// Represents a WDL function implementation callback.
type Callback = fn(context: CallContext<'_>) -> Result<Value, Diagnostic>;

/// Represents an implementation signature for a WDL standard library function.
#[derive(Debug, Clone, Copy)]
pub struct Signature {
    /// The display string of the signature.
    ///
    /// This is only used for unit tests.
    #[allow(unused)]
    display: &'static str,
    /// The implementation callback of the signature.
    callback: Callback,
}

impl Signature {
    /// Constructs a new signature given its display and callback.
    const fn new(display: &'static str, callback: Callback) -> Self {
        Self { display, callback }
    }
}

/// Represents a standard library function.
#[derive(Debug, Clone, Copy)]
pub struct Function {
    /// The signatures of the function.
    signatures: &'static [Signature],
}

impl Function {
    /// Constructs a new function given its signatures.
    const fn new(signatures: &'static [Signature]) -> Self {
        Self { signatures }
    }

    /// Calls the function given the binding and call context.
    #[inline]
    pub fn call(
        &self,
        binding: Binding<'_>,
        context: CallContext<'_>,
    ) -> Result<Value, Diagnostic> {
        (self.signatures[binding.index()].callback)(context)
    }
}

/// Represents the WDL standard library.
#[derive(Debug)]
pub struct StandardLibrary {
    /// The implementation functions for the standard library.
    functions: HashMap<&'static str, Function>,
}

impl StandardLibrary {
    /// Gets a function from the standard library.
    ///
    /// Returns `None` if the function isn't in the WDL standard library.
    #[inline]
    pub fn get(&self, name: &str) -> Option<Function> {
        self.functions.get(name).copied()
    }
}

/// Represents the mapping between function name and overload index to the
/// implementation callback.
pub static STDLIB: LazyLock<StandardLibrary> = LazyLock::new(|| {
    let mut functions = HashMap::with_capacity(ANALYSIS_STDLIB.functions().len());
    assert!(functions.insert("floor", floor::descriptor()).is_none());
    assert!(functions.insert("ceil", ceil::descriptor()).is_none());
    assert!(functions.insert("round", round::descriptor()).is_none());
    assert!(functions.insert("min", min::descriptor()).is_none());
    assert!(functions.insert("max", max::descriptor()).is_none());
    assert!(functions.insert("find", find::descriptor()).is_none());
    assert!(functions.insert("matches", matches::descriptor()).is_none());
    assert!(functions.insert("sub", sub::descriptor()).is_none());
    assert!(
        functions
            .insert("basename", basename::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("join_paths", join_paths::descriptor())
            .is_none()
    );
    assert!(functions.insert("glob", glob::descriptor()).is_none());
    assert!(functions.insert("size", size::descriptor()).is_none());
    assert!(functions.insert("stdout", stdout::descriptor()).is_none());
    assert!(functions.insert("stderr", stderr::descriptor()).is_none());
    assert!(
        functions
            .insert("read_string", read_string::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("read_int", read_int::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("read_float", read_float::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("read_boolean", read_boolean::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("read_lines", read_lines::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("write_lines", write_lines::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("read_tsv", read_tsv::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("write_tsv", write_tsv::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("read_map", read_map::descriptor())
            .is_none()
    );
    assert!(
        functions
            .insert("write_map", write_map::descriptor())
            .is_none()
    );

    StandardLibrary { functions }
});

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_analysis::stdlib::TypeParameters;

    use super::*;

    /// A test to verify that the STDLIB function types from `wdl-analysis`
    /// aligns with the STDLIB implementation from `wdl-engine`.
    #[test]
    fn verify_stdlib() {
        for (name, func) in ANALYSIS_STDLIB.functions() {
            match STDLIB.functions.get(name) {
                Some(imp) => match func {
                    wdl_analysis::stdlib::Function::Monomorphic(f) => {
                        assert_eq!(
                            imp.signatures.len(),
                            1,
                            "signature count mismatch for function `{name}`"
                        );
                        assert_eq!(
                            f.signature()
                                .display(
                                    ANALYSIS_STDLIB.types(),
                                    &TypeParameters::new(f.signature().type_parameters())
                                )
                                .to_string(),
                            imp.signatures[0].display,
                            "signature mismatch for function `{name}`"
                        );
                    }
                    wdl_analysis::stdlib::Function::Polymorphic(f) => {
                        assert_eq!(
                            imp.signatures.len(),
                            f.signatures().len(),
                            "signature count mismatch for function `{name}`"
                        );
                        for (i, sig) in f.signatures().iter().enumerate() {
                            assert_eq!(
                                sig.display(
                                    ANALYSIS_STDLIB.types(),
                                    &TypeParameters::new(sig.type_parameters())
                                )
                                .to_string(),
                                imp.signatures[i].display,
                                "signature mismatch for function `{name}` (index {i})"
                            );
                        }
                    }
                },
                None => {
                    // TODO: make this a failure in the future once the entire STDLIB is implemented
                    continue;
                }
            }
        }
    }
}

//! Generates a parse tree and abstract syntax tree and prints the warnings from
//! both trees.

use wdl::ast;
use wdl::grammar;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = std::env::args().nth(1).expect("missing src");
    let contents = std::fs::read_to_string(src)?;

    let mut warnings = Vec::new();

    let pt = grammar::v1::parse(&contents)?;
    if let Some(lints) = pt.warnings().cloned() {
        warnings.extend(lints);
    }

    let ast = ast::v1::parse(pt)?;
    if let Some(lints) = ast.warnings().cloned() {
        warnings.extend(lints);
    }

    for warning in warnings {
        eprintln!("{}", warning);
    }

    Ok(())
}

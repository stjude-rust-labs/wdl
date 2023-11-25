/// Parses a [`Document`](crate::v1::Document) from the provided input (and
/// assumes that the grammar parsing will `unwrap()`). This method is helpful
/// for quickly bootstrapping up a [`Document`](crate::v1::Document) in your
/// tests.
///
/// # Arguments
///
/// * `$document` - a [`&str`](str) defining the
///   [`Document`](crate::v1::Document).
pub macro parse_document($document:literal) {{
    let parse_tree =
        wdl_grammar::v1::parse_rule(wdl_grammar::v1::Rule::document, $document).unwrap();
    crate::v1::parse(parse_tree)
}}

/// Scaffolds a test to ensure that a targeted entity is able to be constructed
/// from a valid [parsed node](wdl_grammar::v1::Rule) using `try_from()`.
///
/// # Arguments
///
/// * `$input` - a [`&str`](str) that is parsed into the defined `$type_`.
/// * `$type_` - the name of the [rule](wdl_grammar::v1::Rule) to parse the
///   `$input` as.
/// * `$target` - the name of the entity to attempt to construct from the
///   `$type_`.
pub macro valid_node($input:literal, $type_:ident, $target:ident) {{
    let parse_node = wdl_grammar::v1::parse_rule(wdl_grammar::v1::Rule::$type_, $input)
        .unwrap()
        .into_inner();

    $target::try_from(parse_node).unwrap()
}}

/// Scaffolds a test to ensure that a targeted entity fails to be constructed
/// from an invalid [parsed node](wdl_grammar::v1::Rule) using `try_from()`.
///
/// # Arguments
///
/// * `$input` - a [`&str`](str) that is parsed into the defined `$type_`.
/// * `$type_` - the name of the [rule](wdl_grammar::v1::Rule) to parse the
///   `$input` as.
/// * `$name` - the name of the target entity included in the error message.
/// * `$target` - the name of the target entity to attempt to construct from the
///   `$type_` as a Rust identifier.
pub macro invalid_node($input:literal, $type_:ident, $name:ident, $target:ident) {
    let parse_node = wdl_grammar::v1::parse_rule(wdl_grammar::v1::Rule::$type_, $input)
        .unwrap()
        .into_inner();

    let err = $target::try_from(parse_node).unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "invalid node: {} cannot be parsed from node type {:?}",
            stringify!($name),
            wdl_grammar::v1::Rule::$type_
        )
    );
}

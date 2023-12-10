//! Macros.

#[cfg(test)]
pub(crate) mod test;

/// Checks to ensure that a node is of a certain [`Rule`](wdl_grammar::v1::Rule)
/// type. If the node does not match the specified `$type_`, an
/// [`Error::InvalidNode`](crate::v1::Error::InvalidNode) is returned.
///
/// # Arguments
///
/// * `$node` - the [`Pair`](pest::iterators::Pair) (or "node") to examine.
/// * `$type_` - the rule type that the node must match. Note that only the rule
///   name is provided: `wdl_grammar::v1::Rule::` is prepended to the
///   expression.
pub macro check_node($node:expr, $type_:ident) {
    if $node.as_rule() != wdl_grammar::v1::Rule::$type_ {
        return Err(Self::Error::Common(crate::v1::Error::InvalidNode(format!(
            "{} cannot be parsed from node type {:?}",
            stringify!($type_),
            $node.as_rule(),
        ))));
    }
}

/// Unwraps exactly one node from a [`Pair`](pest::iterators::Pair), regardless
/// of node type. This macro is intended for situations where a node contains
/// one and only one child node and you'd like to unwrap to the inner node. If
/// either zero nodes or more than one nodes are found, the respective
/// [`Error`](crate::Error) is thrown.
///
/// **Note:** the type of the node is not considered in this macro. If you'd
/// like to extract a particular node type, see [`extract_one!()`].
///
/// # Arguments
///
/// * `$node` - the [`Pair`](pest::iterators::Pair) (or "node") to extract a
///   node from.
/// * `$within` - the name of the rule we're extracting from (needed for
///   constructing the error message if zero or more than one nodes are found).
pub macro unwrap_one($node:expr, $within:ident) {{
    let mut nodes = $node.into_inner();

    match nodes.len() {
        0 => Err(Self::Error::Common(crate::v1::Error::MissingNode(format!(
            "expected one node within a {} node",
            stringify!($within)
        )))),
        1 => Ok(nodes.next().unwrap()),
        _ => Err(Self::Error::Common(crate::v1::Error::MultipleNodes(
            format!("expected one node within a {} node", stringify!($within)),
        ))),
    }
}}

/// Extracts exactly one node of a particular type from a
/// [`Pair`](pest::iterators::Pair). This macro is intended for situations where
/// a node contains one and only one child node of a particular type and you'd
/// like to extract that node. For the immediate children of the node being
/// examined, if either zero nodes match the desired node type or multiple nodes
/// match the desired node type, the respective [`Error`](crate::Error) is
/// thrown.
///
/// **Note:** if the node only has one child, [`unwrap_one!()`] is recommended.
///
/// **Note:** if you'd like to do a depth-first search of the entire tree rather
/// than simply examining the node's immediate children, [`dive_one!()`] is
/// recommended.
///
/// # Arguments
///
/// * `$node` - the [`Pair`](pest::iterators::Pair) (or "node") to dive within.
/// * `$type_` - the rule type to dive for. Note that only the rule name is
///   provided: `wdl_grammar::v1::Rule::` is prepended to the expression.
/// * `$within` - the name of the rule we're diving into (needed for
///   constructing the error message if zero or more than one nodes matching
///   `$type_` are found).
pub macro extract_one($node:expr, $type_:ident, $within:ident, $err:path) {{
    let mut nodes = $node
        .into_inner()
        .filter(|x| matches!(x.as_rule(), wdl_grammar::v1::Rule::$type_))
        .collect::<Vec<_>>();

    match nodes.len() {
        0 => Err($err(crate::v1::Error::MissingNode(format!(
            "expected one {} node within a {} node",
            stringify!($type_),
            stringify!($within)
        )))),
        1 => Ok(nodes.pop().unwrap()),
        _ => Err($err(crate::v1::Error::MissingNode(format!(
            "expected one {} node within a {} node",
            stringify!($type_),
            stringify!($within)
        )))),
    }
}}

/// Dives into a [`Pair`](pest::iterators::Pair) to find exactly one node
/// matching the provided `$type_`. Notably, this method does a depth-first
/// search of the entire parse tree underneath the provided node—not just the
/// immediate level below.
///
/// **Note:** if the node only has one child, [`unwrap_one!()`] is recommended.
///
/// **Note:** if you'd like to examining only the node's immediate children,
/// [`extract_one!()`] is recommended.
///
/// # Arguments
///
/// * `$node` - the [`Pair`](pest::iterators::Pair) (or "node") to dive within.
/// * `$type_` - the rule type to dive for. Note that only the rule name is
///   provided: `wdl_grammar::v1::Rule::` is prepended to the expression.
/// * `$within` - the name of the rule we're diving into (needed for
///   constructing the error message if zero or more than one nodes matching
///   `$type_` are found).
pub macro dive_one($node:expr, $type_:ident, $within:ident, $err:path) {{
    let mut nodes = $node
        .into_inner()
        .flatten()
        .filter(|x| matches!(x.as_rule(), wdl_grammar::v1::Rule::$type_))
        .collect::<Vec<_>>();

    match nodes.len() {
        0 => Err($err(crate::v1::Error::MissingNode(format!(
            "expected one {} node within a {} node",
            stringify!($type_),
            stringify!($within)
        )))),
        1 => Ok(nodes.pop().unwrap()),
        _ => Err($err(crate::v1::Error::MissingNode(format!(
            "expected one {} node within a {} node",
            stringify!($type_),
            stringify!($within)
        )))),
    }
}}

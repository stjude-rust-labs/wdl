//! Implements find all references functionality

use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use line_index::LineIndex;
use lsp_types::Location;
use url::Url;
use wdl_ast::AstNode;
use wdl_ast::SyntaxKind;
use wdl_ast::TreeToken;

use super::goto_definition::position_to_offset;
use crate::Document;
use crate::SourcePosition;
use crate::SourcePositionEncoding;
use crate::graph::DocumentGraph;
use crate::graph::DocumentGraphExt;
use crate::handlers;
use crate::handlers::location_from_span;
use crate::handlers::position;

/// Represents a target definition for which references are being searched.
#[derive(Debug)]
struct TargetDefinition {
    /// The identifier text of the target symbol.
    name: String,
    /// The syntax kind of the target's parent node, used to determine reference
    /// context.
    kind: SyntaxKind,
    /// The location where the target is defined.
    location: Location,
}

/// Finds all references to the identifier at the given position.
///
/// It first resolves the definition of the identifier at the specified
/// position, then searches through the appropriate scope of
/// documents to find all references to that definition.
pub fn find_all_references(
    graph: &DocumentGraph,
    document_uri: Url,
    position: SourcePosition,
    encoding: SourcePositionEncoding,
) -> Result<Vec<Location>> {
    let definition_location = handlers::goto_definition(graph, document_uri, position, encoding)
        .context("failed to resolve symbol definition")?
        .ok_or_else(|| {
            anyhow!(
                "no definition location found for symbol at position: {}:{}",
                position.line,
                position.character
            )
        })?;

    let target = resolve_target_definition(graph, &definition_location, encoding)
        .context("failed to resolve target definition")?;

    let search_scope = determine_search_scope(&target, graph);

    let mut locations = Vec::new();
    for doc_index in search_scope {
        collect_references_from_document(graph, doc_index, &target, encoding, &mut locations)
            .with_context(|| {
                format!("failed to collect references from document at index {doc_index:?}")
            })?;
    }

    Ok(locations)
}

/// Resolves the target definition from a definition location.
fn resolve_target_definition(
    graph: &DocumentGraph,
    definition_location: &Location,
    encoding: SourcePositionEncoding,
) -> Result<TargetDefinition> {
    let doc_index = graph
        .get_index(&definition_location.uri)
        .ok_or_else(|| anyhow!("definition document not in graph"))?;

    let node = graph.get(doc_index);
    let document = node
        .document()
        .ok_or_else(|| anyhow!("definition document not analyzed"))?;

    let lines = node
        .parse_state()
        .lines()
        .ok_or_else(|| anyhow!("missing line index for target"))?;

    let offset = position_to_offset(
        lines,
        SourcePosition::new(
            definition_location.range.start.line,
            definition_location.range.start.character,
        ),
        encoding,
    )
    .context("failed to convert position to offset")?;

    let token = document
        .root()
        .inner()
        .token_at_offset(offset)
        .find(|t| t.kind() == SyntaxKind::Ident)
        .ok_or_else(|| anyhow!("could not find target token at definition site"))?;

    let parent = token
        .parent()
        .ok_or_else(|| anyhow!("token has no parent"))?;

    Ok(TargetDefinition {
        name: token.text().to_string(),
        kind: parent.kind(),
        location: definition_location.clone(),
    })
}

/// Determines the search scope for finding references based on the target
/// definition type.
///
/// # Returns
///
/// A vector of document indices that should be searched for references:
/// - For local symbols: only the target document
/// - For imported symbols: the target document + all dependent documents
/// - For unknown symbol types: defaults to target document only
fn determine_search_scope(
    target: &TargetDefinition,
    graph: &DocumentGraph,
) -> Vec<petgraph::graph::NodeIndex> {
    let doc_index = graph
        .get_index(&target.location.uri)
        .expect("target document should exist in graph");

    match target.kind {
        // variables are local to the file.
        SyntaxKind::BoundDeclNode
        | SyntaxKind::UnboundDeclNode
        | SyntaxKind::ScatterStatementNode => {
            vec![doc_index]
        }
        // Things which can be imported from other files
        SyntaxKind::StructDefinitionNode
        | SyntaxKind::TaskDefinitionNode
        | SyntaxKind::WorkflowDefinitionNode => {
            let mut scope = vec![doc_index];
            let mut dependents = graph.dependents(doc_index).collect::<Vec<_>>();
            scope.append(&mut dependents);
            scope
        }

        _ => vec![doc_index],
    }
}

/// Collects references to the target symbol form a single document.
fn collect_references_from_document(
    graph: &DocumentGraph,
    doc_index: petgraph::graph::NodeIndex,
    target: &TargetDefinition,
    encoding: SourcePositionEncoding,
    locations: &mut Vec<Location>,
) -> Result<()> {
    let node = graph.get(doc_index);
    let document = match node.document() {
        Some(doc) => doc,
        None => return Ok(()),
    };

    let lines = match node.parse_state().lines() {
        Some(lines) => lines,
        None => return Ok(()),
    };

    find_references_in_doc(graph, document, target, lines, encoding, locations)
}

/// Searches for references to the target symbol within a single document.
///
/// 1. Traverse all tokens in the document's CST
/// 2. Filter for identifier tokens matching the target name
/// 3. For each match, resolve its definition using goto definition
/// 4. If the resolved definition matches the target, add the reference location
fn find_references_in_doc(
    graph: &DocumentGraph,
    document: &Document,
    target: &TargetDefinition,
    lines: &Arc<LineIndex>,
    encoding: SourcePositionEncoding,
    locations: &mut Vec<Location>,
) -> Result<()> {
    let root = document.root().inner().clone();

    for token in root
        .descendants_with_tokens()
        .filter_map(|el| el.into_token())
    {
        if token.kind() == SyntaxKind::Ident && token.text() == target.name {
            let token_pos = position(lines, token.text_range().start())
                .context("failed to convert token position")?;
            let source_pos = SourcePosition::new(token_pos.line, token_pos.character);

            let resolved_location = handlers::goto_definition(
                graph,
                document.uri().as_ref().clone(),
                source_pos,
                encoding,
            )
            .context("failed to resolve token definition")?;

            if let Some(location) = resolved_location {
                if location == target.location {
                    let reference_location =
                        location_from_span(document.uri(), token.span(), lines)
                            .context("failed to create reference location")?;

                    locations.push(reference_location);
                }
            }
        }
    }
    Ok(())
}

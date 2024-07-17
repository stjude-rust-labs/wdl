//! Elements used during formatting.

use std::collections::HashMap;
use std::iter::Peekable;

use nonempty::NonEmpty;
use wdl_ast::AstToken as _;
use wdl_ast::Element;
use wdl_ast::Node;
use wdl_ast::SyntaxKind;

use crate::NEWLINE;
use crate::PreToken;
use crate::TokenStream;
use crate::Writable;

pub mod node;

/// Trivia associated with some more formidable element.
///
/// Trivia would be things like comments and whitespace.
#[derive(Clone, Debug, Default)]
pub struct Trivia {
    /// Any preceeding trivia.
    preceeding: Option<NonEmpty<Box<FormatElement>>>,

    /// Any inline trivia.
    inline: Option<NonEmpty<Box<FormatElement>>>,
}

impl Trivia {
    /// Any preceeding trivia that are not whitespaces.
    pub fn preceeding(&self) -> Option<impl Iterator<Item = &FormatElement>> {
        self.preceeding.as_ref().map(|trivia| {
            trivia
                .into_iter()
                .filter(|t| !matches!(t.element().kind(), SyntaxKind::Whitespace))
                .map(|t| &**t)
        })
    }

    /// Any inline trivia that are not whitespaces.
    pub fn inline(&self) -> Option<impl Iterator<Item = &FormatElement>> {
        self.inline.as_ref().map(|trivia| {
            trivia
                .into_iter()
                .filter(|t| !matches!(t.element().kind(), SyntaxKind::Whitespace))
                .map(|t| &**t)
        })
    }
}

/// A formattable element.
#[derive(Clone, Debug)]
pub struct FormatElement {
    /// The inner element.
    element: Element,

    /// Trivia associated with the element.
    trivia: Trivia,

    /// Children as format elements.
    children: Option<NonEmpty<Box<FormatElement>>>,
}

impl FormatElement {
    /// Creates a new [`FormatElement`].
    pub fn new(
        element: Element,
        trivia: Trivia,
        children: Option<NonEmpty<Box<FormatElement>>>,
    ) -> Self {
        Self {
            element,
            trivia,
            children,
        }
    }

    /// Gets the inner element.
    pub fn element(&self) -> &Element {
        &self.element
    }

    /// Gets the trivia.
    pub fn trivia(&self) -> &Trivia {
        &self.trivia
    }

    /// Gets the children for this node.
    pub fn children(&self) -> Option<impl Iterator<Item = &FormatElement>> {
        self.children
            .as_ref()
            .map(|children| children.into_iter().map(|child| &**child))
    }

    /// Collects all of the children into a hashmap based on their
    /// [`SyntaxKind`]. This is often useful when formatting if you want to,
    /// say, iterate through all children of a certain kind.
    ///
    /// # Notes
    ///
    /// * This clones the underlying children. It's meant to be a cheap clone,
    ///   but you should be aware of the (relatively small) performance hit.
    pub fn children_by_kind(&self) -> HashMap<SyntaxKind, Vec<FormatElement>> {
        let mut results = HashMap::new();

        if let Some(children) = self.children() {
            for child in children {
                results
                    .entry(child.element().kind())
                    .or_insert(Vec::new())
                    // NOTE: this clone is very cheap, as the underlying
                    // elements are mostly reference counts.
                    .push(child.to_owned())
            }
        }

        results
    }

    /// Writes any preceeding trivia to the stream.
    pub fn write_preceeding_trivia(&self, stream: &mut TokenStream<PreToken>) {
        if let Some(trivia) = self.trivia().preceeding() {
            for t in trivia.filter(|t| !matches!(t.element().kind(), SyntaxKind::Whitespace)) {
                t.write(stream);
            }
        }
    }

    /// Writes any inline trivia to the stream.
    pub fn write_inline_trivia(&self, stream: &mut TokenStream<PreToken>) {
        if let Some(trivia) = self.trivia().inline() {
            for t in trivia.filter(|t| !matches!(t.element().kind(), SyntaxKind::Whitespace)) {
                t.write(stream);
            }
        }
    }
}

/// An extension trait for formatting [`Element`]s.
pub trait AstElementFormatExt {
    /// Consumes `self` and returns the [`Element`] as a [`FormatElement`].
    fn into_format_element(self) -> FormatElement;
}

impl AstElementFormatExt for Element {
    fn into_format_element(self) -> FormatElement
    where
        Self: Sized,
    {
        let children = match &self {
            Element::Node(node) => collate(node),
            Element::Token(_) => None,
        };

        FormatElement::new(self, Default::default(), children)
    }
}

/// Collects a list of iterables into an [`Option<NonEmpty>`].
fn collect_optional<T>(mut iter: impl Iterator<Item = T>) -> Option<NonEmpty<T>> {
    if let Some(first) = iter.next() {
        let mut vec = NonEmpty::new(first);
        vec.extend(iter);
        Some(vec)
    } else {
        None
    }
}

/// Takes elements while a particular predicate is true _without_ consuming the
/// element that breaks the chain.
fn take_while_peek<'a, I, P>(
    iter: &'a mut Peekable<I>,
    predicate: P,
) -> impl Iterator<Item = I::Item> + 'a
where
    I: Iterator,
    P: Fn(&I::Item) -> bool + 'a,
{
    std::iter::from_fn(move || {
        if let Some(next_item) = iter.peek() {
            if predicate(next_item) {
                iter.next()
            } else {
                None
            }
        } else {
            None
        }
    })
}

/// Collates the children of a particular node.
fn collate(node: &Node) -> Option<NonEmpty<Box<FormatElement>>> {
    let mut results = Vec::new();
    let mut stream = node
        .syntax()
        .children_with_tokens()
        .map(Element::cast)
        .peekable();

    while stream.peek().is_some() {
        let preceeding = collect_optional(
            take_while_peek(stream.by_ref(), |node| node.is_trivia())
                .map(|item| Box::new(item.into_format_element())),
        );

        let element = match stream.next() {
            Some(node) => node,
            None => break,
        };

        let inline = collect_optional(
            take_while_peek(stream.by_ref(), |element| {
                if element.is_trivia() {
                    // If the element is trivia, we need to check if it contains a
                    // newline.
                    match element {
                        Element::Node(_) => {
                            // SAFETY: if this is reached, then the code needs to be
                            // altered. The fact that nodes should not be trivia is
                            // not baked into the code per se, but it's not expected
                            // to ever occur. If this ends up happening and it makes
                            // sense to change this, feel free to do so.
                            unreachable!("nodes should not be trivia")
                        }
                        Element::Token(token) => {
                            // NOTE: if the token _is_ whitespace, then return false
                            // only if the token contains a newline. Else, this
                            // should continue consuming the whitespace.
                            token
                                .as_whitespace()
                                .map(|whitespace| !whitespace.syntax().text().contains(NEWLINE))
                                .unwrap_or(true)
                        }
                    }
                } else {
                    // If the element isn't trivia, we don't consume it.
                    false
                }
            })
            .map(|item| Box::new(item.into_format_element())),
        );

        let children = match element {
            Element::Node(ref node) => collate(node),
            Element::Token(_) => None,
        };

        results.push(Box::new(FormatElement {
            element,
            trivia: Trivia { preceeding, inline },
            children,
        }));
    }

    if !results.is_empty() {
        let mut results = results.into_iter();
        // SAFETY: we just checked to ensure that `results` wasn't empty, so
        // this will always unwrap.
        let mut children = NonEmpty::new(results.next().unwrap());
        children.extend(results);
        Some(children)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use wdl_ast::Document;
    use wdl_ast::Node;
    use wdl_ast::SyntaxKind;

    use crate::element::node::AstNodeFormatExt;

    #[test]
    fn smoke() {
        let (document, diagnostics) = Document::parse(
            "version 1.2

# This is a comment attached to the task.
task foo # This is an inline comment on the task ident.
{

} # This is an inline comment on the task.

# This is a comment attached to the workflow.
workflow bar # This is an inline comment on the workflow ident.
{
  # This is attached to the call.
  call foo {}
} # This is an inline comment on the workflow.",
        );

        assert!(diagnostics.is_empty());
        let document = document.ast().into_v1().unwrap();

        let format_element = Node::Ast(document).into_format_element();
        let mut children = format_element.children().unwrap();

        ////////////////////////////////////////////////////////////////////////////////
        // Version statement
        ////////////////////////////////////////////////////////////////////////////////

        let version = children.next().expect("version statement element");
        assert_eq!(
            version.element().syntax().kind(),
            SyntaxKind::VersionStatementNode
        );

        assert!(version.trivia().preceeding().is_none());
        assert!(version.trivia().inline().is_none());

        let mut version_children = version.children().unwrap();
        assert_eq!(
            version_children.next().unwrap().element().kind(),
            SyntaxKind::VersionKeyword
        );
        assert_eq!(
            version_children.next().unwrap().element().kind(),
            SyntaxKind::Version
        );

        ////////////////////////////////////////////////////////////////////////////////
        // Task Definition
        ////////////////////////////////////////////////////////////////////////////////

        let task = children.next().expect("task element");
        assert_eq!(
            task.element().syntax().kind(),
            SyntaxKind::TaskDefinitionNode
        );

        // Preceeding.

        let mut preceeding = task.trivia().preceeding().unwrap();

        let comment = preceeding
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(comment.kind(), SyntaxKind::Comment);
        assert_eq!(comment.text(), "# This is a comment attached to the task.");

        // Inline.

        let mut inline = task.trivia().inline().unwrap();

        let comment = inline
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(comment.kind(), SyntaxKind::Comment);
        assert_eq!(comment.text(), "# This is an inline comment on the task.");

        assert!(inline.next().is_none());

        // Children.

        let mut task_children = task.children().unwrap();
        assert_eq!(
            task_children.next().unwrap().element().kind(),
            SyntaxKind::TaskKeyword
        );

        let ident = task_children.next().unwrap();
        assert_eq!(ident.element().kind(), SyntaxKind::Ident);

        let mut ident_inline = ident.trivia().inline().unwrap();

        let inline_comment = ident_inline
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(inline_comment.kind(), SyntaxKind::Comment);
        assert_eq!(
            inline_comment.text(),
            "# This is an inline comment on the task ident."
        );

        assert_eq!(
            task_children.next().unwrap().element().kind(),
            SyntaxKind::OpenBrace
        );
        assert_eq!(
            task_children.next().unwrap().element().kind(),
            SyntaxKind::CloseBrace
        );

        assert!(task_children.next().is_none());

        ////////////////////////////////////////////////////////////////////////////////
        // Workflow Definition
        ////////////////////////////////////////////////////////////////////////////////

        let workflow = children.next().expect("workflow element");
        assert_eq!(
            workflow.element().syntax().kind(),
            SyntaxKind::WorkflowDefinitionNode
        );

        // Preceeding.

        let mut preceeding = workflow.trivia().preceeding().unwrap();

        let comment = preceeding
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(comment.kind(), SyntaxKind::Comment);
        assert_eq!(
            comment.text(),
            "# This is a comment attached to the workflow."
        );

        // Inline.

        let mut inline = workflow.trivia().inline().unwrap();

        let comment = inline
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(comment.kind(), SyntaxKind::Comment);
        assert_eq!(
            comment.text(),
            "# This is an inline comment on the workflow."
        );

        assert!(inline.next().is_none());

        // Children.

        let mut workflow_children = workflow.children().unwrap();

        assert_eq!(
            workflow_children.next().unwrap().element().kind(),
            SyntaxKind::WorkflowKeyword
        );

        let ident = workflow_children.next().unwrap();
        assert_eq!(ident.element().kind(), SyntaxKind::Ident);

        let mut ident_inline = ident.trivia().inline().unwrap();

        let inline_comment = ident_inline
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(inline_comment.kind(), SyntaxKind::Comment);
        assert_eq!(
            inline_comment.text(),
            "# This is an inline comment on the workflow ident."
        );

        assert!(ident_inline.next().is_none());

        assert_eq!(
            workflow_children.next().unwrap().element().kind(),
            SyntaxKind::OpenBrace
        );

        let call = workflow_children.next().unwrap();
        assert_eq!(call.element().kind(), SyntaxKind::CallStatementNode);

        let mut call_preceeding = call.trivia().preceeding().unwrap();

        let comment = call_preceeding
            .next()
            .unwrap()
            .element()
            .syntax()
            .into_token()
            .unwrap();
        assert_eq!(comment.kind(), SyntaxKind::Comment);
        assert_eq!(comment.text(), "# This is attached to the call.");

        assert!(call_preceeding.next().is_none());

        assert_eq!(
            workflow_children.next().unwrap().element().kind(),
            SyntaxKind::CloseBrace
        );

        assert!(workflow_children.next().is_none());
    }
}

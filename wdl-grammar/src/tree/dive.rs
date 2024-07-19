//! Utilities for traversing a syntax tree while collecting elements of interest
//! (i.e., "diving" for elements).

use std::collections::VecDeque;
use std::iter::FusedIterator;
use std::marker::PhantomData;

use rowan::Language;

use crate::SyntaxElement;
use crate::SyntaxNode;
use crate::WorkflowDescriptionLanguage;

/// An iterator that explores every node within a
/// [`SyntaxNode`](rowan::SyntaxNode) yielding all of the elements match the
/// conditions laid out in `match_predicate`.
///
/// There are also facilities to ignore certain subtrees within the tree via the
/// `ignore_predicate` callback. If the entire tree is desired, one can simply
/// have the `ignore_predicate` always return `false`.
#[derive(Debug)]
pub struct DiveIterator<L, M, I>
where
    L: Language,
    M: Fn(&rowan::SyntaxElement<L>) -> bool,
    I: Fn(&rowan::SyntaxNode<L>) -> bool,
{
    /// The queue of nodes to be visited.
    nodes: VecDeque<rowan::SyntaxElement<L>>,
    /// The function that evaluates when checking if an element matches.
    match_predicate: M,
    /// The function that evaluates when checking if the children underneath a
    /// node in the tree should be ignored.
    ignore_predicate: I,
    /// A pointer to the [`Language`].
    _phantom: PhantomData<L>,
}

impl<L, M, I> DiveIterator<L, M, I>
where
    L: Language,
    M: Fn(&rowan::SyntaxElement<L>) -> bool,
    I: Fn(&rowan::SyntaxNode<L>) -> bool,
{
    /// Creates a new [`DiveIterator`].
    ///
    /// Note that this iterator traverses elements as a breadth-first traversal
    /// (otherwise known as level-order).
    pub fn new(root: rowan::SyntaxElement<L>, match_predicate: M, ignore_predicate: I) -> Self {
        let mut nodes = VecDeque::new();
        nodes.push_back(root);

        Self {
            nodes,
            match_predicate,
            ignore_predicate,
            _phantom: PhantomData,
        }
    }
}

impl<L, M, I> Iterator for DiveIterator<L, M, I>
where
    L: Language,
    M: Fn(&rowan::SyntaxElement<L>) -> bool,
    I: Fn(&rowan::SyntaxNode<L>) -> bool,
{
    type Item = rowan::SyntaxElement<L>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(element) = self.nodes.pop_front() {
            match &element {
                rowan::SyntaxElement::Node(node) => {
                    if !(self.ignore_predicate)(node) {
                        self.nodes.extend(node.children_with_tokens());
                    }
                }
                // NOTE: tokens have no children to explore.
                rowan::SyntaxElement::Token(_) => {}
            }

            if (self.match_predicate)(&element) {
                // NOTE: this is an inexpensive clone of a red node.
                return Some(element.clone());
            }
        }

        None
    }
}

impl<L, M, I> FusedIterator for DiveIterator<L, M, I>
where
    L: Language,
    M: Fn(&rowan::SyntaxElement<L>) -> bool,
    I: Fn(&rowan::SyntaxNode<L>) -> bool,
{
}

/// Elements of a syntax tree upon which a dive can be performed.
pub trait Divable<L>
where
    L: Language,
{
    /// Visits every element in a syntax tree and accumulates all
    /// [`SyntaxElement`](rowan::SyntaxElement)s for which the `match_predicate`
    /// evaluates to `true`.
    ///
    /// No guarantees are made about the order in which elements will be
    /// traversed or collected.
    fn dive<M>(&self, match_predicate: M) -> impl Iterator<Item = rowan::SyntaxElement<L>>
    where
        M: Fn(&rowan::SyntaxElement<L>) -> bool,
    {
        self.dive_with_ignore(match_predicate, |_| false)
    }

    /// Visits every element in a syntax tree and accumulates all
    /// [`SyntaxElement`](rowan::SyntaxElement)s for which the `match_predicate`
    /// evaluates to `true`. No children nodes will be searched for
    /// [`SyntaxNode`](rowan::SyntaxNode)s that are visited and for which the
    /// `ignore_predicate` evaluates to `true`. In this way, you can ignore
    /// entire sections of the tree.
    ///
    /// No guarantees are made about the order in which elements will be
    /// traversed or collected.
    fn dive_with_ignore<M, I>(
        &self,
        match_predicate: M,
        ignore_predicate: I,
    ) -> impl Iterator<Item = rowan::SyntaxElement<L>>
    where
        M: Fn(&rowan::SyntaxElement<L>) -> bool,
        I: Fn(&rowan::SyntaxNode<L>) -> bool;
}

impl<D, L> Divable<L> for &D
where
    D: Divable<L>,
    L: Language,
{
    fn dive_with_ignore<M, I>(
        &self,
        match_predicate: M,
        ignore_predicate: I,
    ) -> impl Iterator<Item = rowan::SyntaxElement<L>>
    where
        M: Fn(&rowan::SyntaxElement<L>) -> bool,
        I: Fn(&rowan::SyntaxNode<L>) -> bool,
    {
        D::dive_with_ignore(self, match_predicate, ignore_predicate)
    }
}

impl Divable<WorkflowDescriptionLanguage> for SyntaxNode {
    fn dive_with_ignore<M, I>(
        &self,
        match_predicate: M,
        ignore_predicate: I,
    ) -> impl Iterator<Item = SyntaxElement>
    where
        M: Fn(&SyntaxElement) -> bool,
        I: Fn(&SyntaxNode) -> bool,
    {
        DiveIterator::new(
            // NOTE: this is an inexpensive clone of a red node.
            SyntaxElement::Node(self.clone()),
            match_predicate,
            ignore_predicate,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use rowan::GreenNode;

    use crate::dive::Divable;
    use crate::SyntaxKind;
    use crate::SyntaxNode;
    use crate::SyntaxTree;

    fn get_syntax_node() -> SyntaxNode {
        static GREEN_NODE: OnceLock<GreenNode> = OnceLock::new();

        let green_node = GREEN_NODE
            .get_or_init(|| {
                let (tree, diagnostics) = SyntaxTree::parse(
                    r#"version 1.2

task hello {
    String a_private_declaration = false
}

workflow world {
    String another_private_declaration = true
}"#,
                );

                assert!(diagnostics.is_empty());
                tree.green()
            })
            .clone();

        SyntaxNode::new_root(green_node)
    }

    #[test]
    fn it_dives_correctly() {
        let tree = get_syntax_node();

        let mut idents = tree.dive(|element| element.kind() == SyntaxKind::Ident);

        // NOTE: unlike the visitor, which carries out a preorder traversal, the
        // current implementation of dive in this create uses a level-order
        // traversal. Thus, the names of the workflow and task will be seen
        // before their inner members.
        assert_eq!(idents.next().unwrap().as_token().unwrap().text(), "hello");

        assert_eq!(idents.next().unwrap().as_token().unwrap().text(), "world");

        assert_eq!(
            idents.next().unwrap().as_token().unwrap().text(),
            "a_private_declaration"
        );

        assert_eq!(
            idents.next().unwrap().as_token().unwrap().text(),
            "another_private_declaration"
        );

        assert!(idents.next().is_none());
    }

    #[test]
    fn it_dives_with_ignores_correctly() {
        let tree = get_syntax_node();

        let mut ignored_idents = tree.dive_with_ignore(
            |element| element.kind() == SyntaxKind::Ident,
            |node| node.kind() == SyntaxKind::WorkflowDefinitionNode,
        );

        assert_eq!(
            ignored_idents.next().unwrap().as_token().unwrap().text(),
            "hello"
        );
        assert_eq!(
            ignored_idents.next().unwrap().as_token().unwrap().text(),
            "a_private_declaration"
        );

        // The idents contained in the workflow are not included in the results,
        // as we explicitly ignored any workflow definition nodes.
        assert!(ignored_idents.next().is_none());
    }
}

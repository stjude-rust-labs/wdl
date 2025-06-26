//! The `container` item within the `runtime` block.

use crate::AstNode;
use crate::AstToken;
use crate::SyntaxKind;
use crate::SyntaxNode;
use crate::TreeNode;
use crate::v1::RuntimeItem;
use crate::v1::TASK_REQUIREMENT_CONTAINER;
use crate::v1::TASK_REQUIREMENT_CONTAINER_ALIAS;
use crate::v1::common::container::value;
use crate::v1::common::container::value::Value;

/// The `container` item within a `runtime` block.
#[derive(Debug)]
pub struct Container<N: TreeNode = SyntaxNode>(RuntimeItem<N>);

impl<N: TreeNode> Container<N> {
    /// Gets the [`Value`] from a [`Container`] (if it can be parsed).
    pub fn value(&self) -> value::Result<Value<N>> {
        Value::try_from(self.0.expr())
    }
}

impl<N: TreeNode> AstNode<N> for Container<N> {
    fn can_cast(kind: SyntaxKind) -> bool {
        RuntimeItem::<N>::can_cast(kind)
    }

    fn cast(inner: N) -> Option<Self> {
        RuntimeItem::cast(inner).and_then(|item| Container::try_from(item).ok())
    }

    fn inner(&self) -> &N {
        self.0.inner()
    }
}

impl<N: TreeNode> TryFrom<RuntimeItem<N>> for Container<N> {
    type Error = ();

    fn try_from(value: RuntimeItem<N>) -> Result<Self, Self::Error> {
        if [TASK_REQUIREMENT_CONTAINER, TASK_REQUIREMENT_CONTAINER_ALIAS]
            .iter()
            .any(|key| value.name().text() == *key)
        {
            return Ok(Self(value));
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::ParseResult;

    #[test]
    fn simple_example() {
        let ParseResult {
            document,
            diagnostics,
        } = Document::parse(
            r#"version 1.2

task hello {
    runtime {
        container: "ubuntu"
    }
}"#,
        );

        assert!(diagnostics.is_empty());

        let section = document
            .ast()
            .as_v1()
            .expect("v1 ast")
            .tasks()
            .next()
            .expect("the 'hello' task to exist")
            .runtime()
            .expect("the 'runtime' block to exist");

        let container = section.items().filter_map(|p| p.into_container());

        assert!(container.count() == 1);
    }

    #[test]
    fn missing_container_item() {
        let ParseResult {
            document,
            diagnostics,
        } = Document::parse(
            r#"version 1.2

task hello {
    runtime {
        foo: "ubuntu"
    }
}"#,
        );

        assert!(diagnostics.is_empty());

        let section = document
            .ast()
            .as_v1()
            .expect("v1 ast")
            .tasks()
            .next()
            .expect("the 'hello' task to exist")
            .runtime()
            .expect("the 'runtime' block to exist");

        let container = section.items().filter_map(|p| p.into_container());

        assert!(container.count() == 0);
    }

    #[test]
    fn docker_alias() {
        let ParseResult {
            document,
            diagnostics,
        } = Document::parse(
            r#"version 1.2

task hello {
    runtime {
        docker: "ubuntu"
    }
}"#,
        );

        assert!(diagnostics.is_empty());

        let section = document
            .ast()
            .as_v1()
            .expect("v1 ast")
            .tasks()
            .next()
            .expect("the 'hello' task to exist")
            .runtime()
            .expect("the 'runtime' block to exist");

        let container = section.items().filter_map(|p| p.into_container());

        assert!(container.count() == 1);
    }
}

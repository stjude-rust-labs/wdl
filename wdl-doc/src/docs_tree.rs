//! Implementations for a [`DocsTree`] which represents the DOCS directory.

use std::collections::HashMap;
use std::path::PathBuf;

use maud::Markup;

/// The type of a page.
///
/// Tasks and Workflows have an HTML description associated with them.
/// Index pages do not have a type.
#[derive(Debug)]
pub enum PageType {
    /// A struct page.
    Struct,
    /// A task page.
    Task(Markup),
    /// A workflow page.
    Workflow(Markup),
}

/// An HTML page in the DOCS directory.
#[derive(Debug)]
pub struct HTMLPage {
    /// The display name of the page.
    name: String,
    /// The absoluute path to the entry.
    abs_path: PathBuf,
    /// The type of the page. Index pages do not have a type.
    page_type: Option<PageType>,
}

impl HTMLPage {
    /// Create a new Table of Contents entry.
    pub fn new(name: String, abs_path: PathBuf, page_type: Option<PageType>) -> Self {
        Self {
            name,
            abs_path,
            page_type,
        }
    }

    /// Get the name of the entry.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the type of the entry.
    pub fn page_type(&self) -> &Option<PageType> {
        &self.page_type
    }

    /// Get the path to the entry.
    pub fn path(&self) -> &PathBuf {
        &self.abs_path
    }
}

/// A node in the DOCS directory tree.
#[derive(Debug)]
pub struct Node {
    name: String,
    path: PathBuf,
    page: Option<HTMLPage>,
    children: HashMap<String, Node>,
}

impl Node {
    /// Create a new node.
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            page: None,
            children: HashMap::new(),
        }
    }

    /// Get the name of the node.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the path of the node.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Check if the node has a page associated with it.
    fn has_page(&self) -> bool {
        self.page.is_some()
    }

    /// Check if the node is an index page.
    fn is_index(&self) -> bool {
        if self.has_page() {
            return self.page.as_ref().unwrap().page_type().is_none();
        }
        false
    }

    /// Get the page associated with the node.
    pub fn page(&self) -> &Option<HTMLPage> {
        &self.page
    }

    /// Get the children of the node.
    fn children(&self) -> &HashMap<String, Node> {
        &self.children
    }

    /// Get index pages in the node.
    fn get_index_pages(&self) -> Vec<&HTMLPage> {
        let mut index_pages = Vec::new();

        if self.is_index() {
            index_pages.push(self.page.as_ref().unwrap());
        }

        for child in self.children.values() {
            index_pages.extend(child.get_index_pages());
        }

        index_pages
    }

    /// Get non-index pages in the node.
    pub fn get_non_index_pages(&self) -> Vec<&HTMLPage> {
        let mut non_index_pages = Vec::new();

        if self.has_page() && !self.is_index() {
            non_index_pages.push(self.page.as_ref().unwrap());
        }

        for child in self.children.values() {
            non_index_pages.extend(child.get_non_index_pages());
        }

        non_index_pages
    }

    /// Depth first traversal of the tree.
    pub fn depth_first_traversal(&self) -> Vec<&Node> {
        let mut nodes = Vec::new();

        nodes.push(self);

        for child in self.children.values() {
            nodes.extend(child.depth_first_traversal());
        }

        nodes
    }
}

/// A tree representing the DOCS directory.
#[derive(Debug)]
pub struct DocsTree {
    root: Node,
}

impl DocsTree {
    /// Create a new DOCS tree.
    pub fn new(root: Node) -> Self {
        Self { root }
    }

    /// Get the root of the tree.
    pub fn root(&self) -> &Node {
        &self.root
    }

    /// Get the root of the tree as mutable.
    pub fn root_mut(&mut self) -> &mut Node {
        &mut self.root
    }

    /// Add a path to the tree.
    pub fn add_path(&mut self, abs_path: PathBuf, page: Option<HTMLPage>) {
        let root = self.root_mut();
        let path = abs_path.strip_prefix(&root.path).unwrap();
        let components = path.components().collect::<Vec<_>>();
        let cur_path = root.path.clone();

        let mut current_node = root;

        for component in components {
            let component = component.as_os_str().to_str().unwrap().to_string();
            if current_node.children.contains_key(&component) {
                current_node = current_node.children.get_mut(&component).unwrap();
            } else {
                let new_node = Node::new(component.clone(), cur_path.join(&component));
                current_node.children.insert(component.clone(), new_node);
                current_node = current_node.children.get_mut(&component).unwrap();
            }
        }

        current_node.page = page;
    }

    /// Get the Node associated with a path.
    pub fn get_node(&self, abs_path: &PathBuf) -> Option<&Node> {
        let root = self.root();
        let path = abs_path.strip_prefix(&root.path).unwrap();
        let components = path.components().collect::<Vec<_>>();

        let mut current_node = root;

        for component in components {
            let component = component.as_os_str().to_str().unwrap().to_string();
            if current_node.children.contains_key(&component) {
                current_node = current_node.children.get(&component).unwrap();
            } else {
                return None;
            }
        }

        Some(current_node)
    }

    /// Get the page associated with a path.
    pub fn get_page(&self, abs_path: &PathBuf) -> Option<&HTMLPage> {
        let root = self.root();
        let path = abs_path.strip_prefix(&root.path).unwrap();
        let components = path.components().collect::<Vec<_>>();

        let mut current_node = root;

        for component in components {
            let component = component.as_os_str().to_str().unwrap().to_string();
            if current_node.children.contains_key(&component) {
                current_node = current_node.children.get(&component).unwrap();
            } else {
                return None;
            }
        }

        current_node.page().as_ref()
    }

    /// Get all index pages in the tree.
    pub fn get_index_pages(&self) -> Vec<&HTMLPage> {
        let mut index_pages = Vec::new();
        let root = self.root();

        for child in root.children().values() {
            index_pages.extend(child.get_index_pages());
        }

        index_pages
    }

    /// Depth first traversal of the tree. Iterates over all non-root nodes.
    pub fn depth_first_traversal(&self) -> Vec<&Node> {
        let mut nodes = Vec::new();
        let root = self.root();

        for child in root.children().values() {
            nodes.extend(child.depth_first_traversal());
        }

        nodes
    }
}

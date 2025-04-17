//! Implementations for a [`DocsTree`] which represents the docs directory.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::path::absolute;
use std::rc::Rc;

use maud::Markup;
use maud::html;
use pathdiff::diff_paths;

use crate::Document;
use crate::full_page;
use crate::r#struct::Struct;
use crate::task::Task;
use crate::workflow::Workflow;
use crate::write_assets;

/// The type of a page.
#[derive(Debug)]
pub enum PageType {
    /// An index page.
    Index(Document),
    /// A struct page.
    Struct(Struct),
    /// A task page.
    Task(Task),
    /// A workflow page.
    Workflow(Workflow),
}

/// An HTML page in the docs directory.
#[derive(Debug)]
pub struct HTMLPage {
    /// The display name of the page.
    name: String,
    /// The type of the page.
    page_type: PageType,
}

impl HTMLPage {
    /// Create a new HTML page.
    pub fn new(name: String, page_type: PageType) -> Self {
        Self { name, page_type }
    }

    /// Get the name of the page.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the type of the page.
    pub fn page_type(&self) -> &PageType {
        &self.page_type
    }
}

/// A node in the docs directory tree.
#[derive(Debug)]
struct Node {
    /// The name of the node.
    name: String,
    /// The absolute path to the node.
    path: PathBuf,
    /// The page associated with the node.
    page: Option<Rc<HTMLPage>>,
    /// The children of the node.
    children: BTreeMap<String, Node>,
}

impl Node {
    /// Create a new node.
    pub fn new<P: Into<PathBuf>>(name: String, path: P) -> Self {
        Self {
            name,
            path: path.into(),
            page: None,
            children: BTreeMap::new(),
        }
    }

    /// Get the name of the node.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the absolute path of the node.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the page associated with the node.
    pub fn page(&self) -> Option<&Rc<HTMLPage>> {
        self.page.as_ref()
    }

    /// Get the children of the node.
    pub fn children(&self) -> &BTreeMap<String, Node> {
        &self.children
    }

    /// Gather the node and its children in a Depth First Traversal order.
    pub fn depth_first_traversal(&self) -> Vec<&Node> {
        fn recurse_depth_first<'a>(node: &'a Node, nodes: &mut Vec<&'a Node>) {
            nodes.push(node);

            for child in node.children.values() {
                recurse_depth_first(child, nodes);
            }
        }

        let mut nodes = Vec::new();
        recurse_depth_first(self, &mut nodes);

        nodes
    }
}

/// A tree representing the docs directory.
#[derive(Debug)]
pub struct DocsTree {
    /// The root of the tree.
    ///
    /// `root.path` is the path to the docs directory and is absolute.
    root: Node,
    /// The absolute path to the stylesheet.
    stylesheet: PathBuf,
    /// The absolute path to the assets directory.
    assets: PathBuf,
}

impl DocsTree {
    /// Create a new docs tree.
    pub fn new(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let abs_path = absolute(root.as_ref()).unwrap();
        write_assets(&abs_path)?;
        let node = Node::new(
            abs_path.file_name().unwrap().to_str().unwrap().to_string(),
            abs_path.clone(),
        );

        let stylesheet = abs_path.join("style.css");

        Ok(Self {
            root: node,
            stylesheet,
            assets: abs_path.join("assets"),
        })
    }

    /// Create a new docs tree with a stylesheet.
    pub fn new_with_stylesheet(
        root: impl AsRef<Path>,
        stylesheet: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let abs_path = absolute(root.as_ref()).unwrap();
        write_assets(&abs_path)?;
        let in_stylesheet = absolute(stylesheet.as_ref())?;
        let new_stylesheet = abs_path.join("style.css");
        std::fs::copy(in_stylesheet, &new_stylesheet)?;

        let node = Node::new(
            abs_path.file_name().unwrap().to_str().unwrap().to_string(),
            abs_path.clone(),
        );

        Ok(Self {
            root: node,
            stylesheet: new_stylesheet,
            assets: abs_path.join("assets"),
        })
    }

    /// Get the root of the tree.
    fn root(&self) -> &Node {
        &self.root
    }

    /// Get the root of the tree as mutable.
    fn root_mut(&mut self) -> &mut Node {
        &mut self.root
    }

    /// Get the absolute path to the stylesheet.
    pub fn stylesheet(&self) -> &PathBuf {
        &self.stylesheet
    }

    /// Get the absolute path to the assets directory.
    pub fn assets(&self) -> &PathBuf {
        &self.assets
    }

    /// Get a relative path to the stylesheet.
    pub fn stylesheet_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(&self.stylesheet, path).unwrap()
    }

    /// Get a relative path to the assets directory.
    pub fn assets_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(&self.assets, path).unwrap()
    }

    /// Get a relative path to the root index page.
    pub fn root_index_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(self.root.path().join("index.html"), path).unwrap()
    }

    /// Add a page to the tree.
    pub fn add_page<P: Into<PathBuf>>(&mut self, abs_path: P, page: Rc<HTMLPage>) {
        let root = self.root_mut();
        let path = abs_path.into();
        let rel_path = path
            .strip_prefix(&root.path)
            .expect("path should be in the docs directory");

        let mut current_node = root;

        let mut components = rel_path.components().peekable();
        while let Some(component) = components.next() {
            let cur_name = component.as_os_str().to_str().unwrap();
            if current_node.children.contains_key(cur_name) {
                current_node = current_node.children.get_mut(cur_name).unwrap();
            } else {
                let new_node = Node::new(cur_name.to_string(), current_node.path().join(component));
                current_node.children.insert(cur_name.to_string(), new_node);
                current_node = current_node.children.get_mut(cur_name).unwrap();
            }
            if let Some(next_component) = components.peek() {
                if next_component.as_os_str().to_str().unwrap() == "index.html" {
                    break;
                }
            }
        }

        current_node.page = Some(page);
    }

    /// Get the Node associated with a path.
    fn get_node<P: AsRef<Path>>(&self, abs_path: P) -> Option<&Node> {
        let root = self.root();
        let path = abs_path.as_ref();
        let rel_path = path.strip_prefix(&root.path).unwrap();

        let mut current_node = root;

        for component in rel_path
            .components()
            .map(|c| c.as_os_str().to_str().unwrap())
        {
            if current_node.children.contains_key(component) {
                current_node = current_node.children.get(component).unwrap();
            } else {
                return None;
            }
        }

        Some(current_node)
    }

    /// Get the page associated with a path.
    pub fn get_page<P: AsRef<Path>>(&self, abs_path: P) -> Option<&Rc<HTMLPage>> {
        self.get_node(abs_path).and_then(|node| node.page())
    }

    /// Helps render a sidebar component given a node and a base path.
    fn sidebar_recurse(&self, node: &Node, base: &Path) -> Markup {
        html! {
            @if let Some(page) = node.page() {
                @match page.page_type() {
                    PageType::Index(_) => {
                        @if base.starts_with(node.path()) {
                            div class="flex items-center gap-x-1 dark:text-slate-50" {
                                img src=(self.assets_relative_to(base).join("selected-dir.png").to_string_lossy()) class="w-4 h-4" alt="Directory icon";
                                p class="" { a href=(diff_paths(node.path().join("index.html"), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        } @else {
                            div class="flex items-center gap-x-1 hover:text-slate-300" {
                                img src=(self.assets_relative_to(base).join("unselected-dir.png").to_string_lossy()) class="w-4 h-4" alt="Directory icon";
                                p class="" { a href=(diff_paths(node.path().join("index.html"), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        }
                    },
                    PageType::Struct(_) => {
                        @if base.starts_with(node.path().parent().unwrap()) {
                            div class="flex items-center gap-x-1 dark:text-slate-50" {
                                img src=(self.assets_relative_to(base).join("selected-struct.png").to_string_lossy()) class="w-4 h-4" alt="Struct icon";
                                p class="" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        } @else {
                            div class="flex items-center gap-x-1 hover:text-slate-300" {
                                img src=(self.assets_relative_to(base).join("unselected-struct.png").to_string_lossy()) class="w-4 h-4" alt="Struct icon";
                                p class="" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        }
                    },
                    PageType::Task(_) => {
                        @if base.starts_with(node.path().parent().unwrap()) {
                            div class="flex items-center gap-x-1 dark:text-slate-50" {
                                img src=(self.assets_relative_to(base).join("selected-task.png").to_string_lossy()) class="w-4 h-4" alt="Task icon";
                                p class="" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        } @else {
                            div class="flex items-center gap-x-1 hover:text-slate-300" {
                                img src=(self.assets_relative_to(base).join("unselected-task.png").to_string_lossy()) class="w-4 h-4" alt="Task icon";
                                p class="" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        }
                    },
                    PageType::Workflow(_) => {
                        @if base.starts_with(node.path().parent().unwrap()) {
                            div class="flex items-center gap-x-1 dark:text-slate-50" {
                                img src=(self.assets_relative_to(base).join("selected-workflow.png").to_string_lossy()) class="w-4 h-4" alt="Workflow icon";
                                p class="" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        } @else {
                            div class="flex items-center gap-x-1 hover:text-slate-300" {
                                img src=(self.assets_relative_to(base).join("unselected-workflow.png").to_string_lossy()) class="w-4 h-4" alt="Workflow icon";
                                p class="" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (page.name()) } }
                            }
                        }
                    }
                }
            } @else {
                @if base.starts_with(node.path()) {
                    div class="flex items-center gap-x-1 dark:text-slate-50" {
                        img src=(self.assets_relative_to(base).join("selected-dir.png").to_string_lossy()) class="w-4 h-4" alt="Directory icon";
                        p class="" { (node.name()) }
                    }
                } @else {
                    div class="flex items-center gap-x-1" {
                        img src=(self.assets_relative_to(base).join("unselected-dir.png").to_string_lossy()) class="w-4 h-4" alt="Directory icon";
                        p class="" { (node.name()) }
                    }
                }
            }
            ul class="" {
                @for child in node.children().values() {
                    li class="px-2 border-l border-gray-500 ml-2" { (self.sidebar_recurse(child, base)) }
                }
            }
        }
    }

    /// Render a left sidebar component given a path.
    ///
    /// The sidebar will contain a table of contents for the docs directory.
    /// Every node in the tree will be visited in a Depth First Traversal order.
    /// If the node has a page associated with it, a link to the page will be
    /// rendered. If the node does not have a page associated with it, the
    /// name of the node will be rendered. All links will be relative to the
    /// given path.
    pub fn render_left_sidebar<P: AsRef<Path>>(&self, path: P) -> Markup {
        let root = self.root();
        let base = path.as_ref().parent().unwrap();

        html! {
            div class="flex flex-col gap-y-3 top-0 left-0 h-screen min-w-[269px] text-ellipsis text-nowrap p-4 dark:bg-slate-900 dark:text-slate-400 overflow-y-auto overflow-x-clip" {
                img src=(self.assets_relative_to(base).join("sprocket-logo.png").to_string_lossy()) class="w-1/2 h-1/2 mb-4" alt="Sprocket logo";
                form class="flex items-center gap-x-2 w-full h-full rounded-md border border-slate-700 px-2 mb-4" {
                    img src=(self.assets_relative_to(base).join("search.png").to_string_lossy()) class="w-4 h-4" alt="Search icon";
                    input type="text" placeholder="Search" class="w-full h-full text-slate-300";
                }
                div class="w-full h-full rounded-md flex items-center gap-x-2 px-2" {
                    div class="flex grow items-center gap-x-1" {
                        div class="flex grow items-center gap-x-1 border-b dark:text-slate-400 hover:text-slate-300" {
                            img src=(self.assets_relative_to(base).join("list-bullet.png").to_string_lossy()) class="w-4 h-4" alt="List icon";
                            p { "Workflows" }
                        }
                        div class="flex grow items-center gap-x-1 border-b dark:text-slate-50" {
                            img src=(self.assets_relative_to(base).join("folder.png").to_string_lossy()) class="w-4 h-4" alt="List icon";
                            p { "Full Directory" }
                        }
                    }
                }
                ul class="" {
                    div class="flex flex-row items-center gap-x-1 dark:text-slate-50" {
                        img src=(self.assets_relative_to(base).join("selected-dir.png").to_string_lossy()) class="w-4 h-4" alt="Directory icon";
                        p class="" { a href=(self.root_index_relative_to(base).to_string_lossy()) { (root.name()) } }
                    }
                    @for node in root.children().values() {
                        @if node.name() != "external" {
                            li class="px-2 border-l border-gray-500 ml-2" { (self.sidebar_recurse(node, base)) }
                        }
                    }
                    @if let Some(external) = root.children().get("external") {
                        li class="px-2 border-l border-gray-500 ml-2" { (self.sidebar_recurse(external, base)) }
                    }
                }
            }
        }
    }

    /// Render a right sidebar component.
    pub fn render_right_sidebar(&self) -> Markup {
        html! {
            div class="top-0 right-0 h-screen min-w-[240px] w-[240px] p-4 dark:bg-red-900 dark:text-white" {
                h1 class="text-2xl text-center" { "Sidebar" }
                p class="" { "Right Sidebar" }
            }
        }
    }

    /// Render every page in the tree.
    pub fn render_all(&self) -> anyhow::Result<()> {
        let root = self.root();

        for node in root.depth_first_traversal() {
            if let Some(page) = node.page() {
                self.write_page(page.as_ref(), node.path())?;
            }
        }

        self.write_homepage()?;
        Ok(())
    }

    /// Write the homepage to disk.
    fn write_homepage(&self) -> anyhow::Result<()> {
        let root = self.root();
        let index_path = root.path().join("index.html");

        let left_sidebar = self.render_left_sidebar(&index_path);
        let content = html! {
            div class="" {
                h3 class="" { "Home" }
                table class="border" {
                    thead class="border" { tr {
                        th class="" { "Page" }
                    }}
                    tbody class="border" {
                        @for node in root.depth_first_traversal() {
                            @if node.page().is_some() {
                                tr class="border" {
                                    td class="border" {
                                        @match node.page().unwrap().page_type() {
                                            PageType::Index(_) => {
                                                a href=(diff_paths(node.path().join("index.html"), root.path()).unwrap().to_str().unwrap()) {(node.name()) }
                                            }
                                            _ => {
                                                a href=(diff_paths(node.path(), root.path()).unwrap().to_str().unwrap()) {(node.name()) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        let html = full_page(
            "Home",
            html! {
                div class="flex flex-row items-start" {
                    div class="flex sticky top-0 resize-x max-w-1/6" {
                        (left_sidebar)
                    }
                    div class="flex grow resize-x p-4 ml-4" {
                        (content)
                    }
                    div class="flex top-0 right-0 sticky" {
                        (self.render_right_sidebar())
                    }
                }
            },
            self.stylesheet_relative_to(root.path()),
        );
        std::fs::write(index_path, html.into_string())?;
        Ok(())
    }

    /// Write a page to disk at the designated path.
    pub fn write_page<P: Into<PathBuf>>(&self, page: &HTMLPage, path: P) -> anyhow::Result<()> {
        let mut path = path.into();

        let content = match page.page_type() {
            PageType::Index(doc) => {
                path = path.join("index.html");
                doc.render()
            }
            PageType::Struct(s) => s.render(),
            PageType::Task(t) => t.render(),
            PageType::Workflow(w) => w.render(),
        };

        let stylesheet =
            self.stylesheet_relative_to(path.parent().expect("path should have a parent"));
        let left_sidebar = self.render_left_sidebar(&path);

        let html = full_page(
            page.name(),
            html! {
                div class="flex flex-row items-start" {
                    div class="flex sticky top-0 resize-x max-w-1/6" {
                        (left_sidebar)
                    }
                    div class="flex grow resize-x p-4 ml-4" {
                        (content)
                    }
                    div class="flex top-0 right-0 sticky" {
                        (self.render_right_sidebar())
                    }
                }
            },
            stylesheet,
        );
        std::fs::write(path, html.into_string())?;
        Ok(())
    }
}

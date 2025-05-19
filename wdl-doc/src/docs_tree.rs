//! Implementations for a [`DocsTree`] which represents the docs directory.

use std::collections::BTreeMap;
use std::collections::HashSet;
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

/// A page header or page sub header.
#[derive(Debug)]
pub enum Header {
    /// A header in the page.
    Header(String, String),
    /// A sub header in the page.
    SubHeader(String, String),
}

/// A sorted collection of headers in a page.
#[derive(Debug, Default)]
pub struct PageHeaders {
    /// The headers of the page.
    pub headers: Vec<Header>,
}

impl PageHeaders {
    /// Push a header to the page headers.
    pub fn push(&mut self, header: Header) {
        self.headers.push(header);
    }

    /// Extend the page headers with another collection of headers.
    pub fn extend(&mut self, headers: Self) {
        self.headers.extend(headers.headers);
    }

    /// Render the page headers as HTML.
    pub fn render(&self) -> Markup {
        html!(
            @for header in &self.headers {
                @match header {
                    Header::Header(name, id) => {
                        a href=(format!("#{}", id)) class="right-sidebar__section-header" { (name) }
                    }
                    Header::SubHeader(name, id) => {
                        div class="right-sidebar__section-items" {
                            a href=(format!("#{}", id)) class="right-sidebar__section-item" { (name) }
                        }
                    }
                }
            }
        )
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
    ///
    /// Traversal order is alphabetical by node name, with the exception of the
    /// "external" node, which is always last.
    pub fn depth_first_traversal(&self) -> Vec<&Node> {
        fn recurse_depth_first<'a>(node: &'a Node, nodes: &mut Vec<&'a Node>) {
            nodes.push(node);

            for child in node.children().values() {
                recurse_depth_first(child, nodes);
            }
        }

        let mut nodes = Vec::new();
        nodes.push(self);
        for child in self.children().values().filter(|c| c.name() != "external") {
            recurse_depth_first(child, &mut nodes);
        }
        if let Some(external) = self.children().get("external") {
            recurse_depth_first(external, &mut nodes);
        }

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
    /// Create a new docs tree with a default stylesheet.
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

    /// Create a new docs tree with a custom stylesheet.
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

    /// Get the absolute path to the root directory.
    pub fn root_path(&self) -> &PathBuf {
        &self.root.path
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
    fn assets_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(&self.assets, path).unwrap()
    }

    /// Get a relative path to an asset in the assets directory (converted to a
    /// string).
    pub fn get_asset<P: AsRef<Path>>(&self, path: P, asset: &str) -> String {
        self.assets_relative_to(path)
            .join(asset)
            .to_string_lossy()
            .to_string()
    }

    /// Get a relative path to the root index page.
    pub fn root_index_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(self.root_path().join("index.html"), path).unwrap()
    }

    /// Add a page to the tree. Path is expected to be absolute.
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
    ///
    /// Can be an abolute path or a path relative to the root.
    fn get_node<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
        let root = self.root();
        let path = path.as_ref();
        let rel_path = path.strip_prefix(&root.path).unwrap_or(path);

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
    ///
    /// Can be an abolute path or a path relative to the root.
    pub fn get_page<P: AsRef<Path>>(&self, path: P) -> Option<&Rc<HTMLPage>> {
        self.get_node(path).and_then(|node| node.page())
    }

    /// Get workflows by category.
    fn get_workflows_by_category(&self) -> Vec<(String, Vec<&Node>)> {
        let mut workflows_by_category = Vec::new();
        let mut categories = HashSet::new();
        let mut nodes = Vec::new();

        for node in self.root().depth_first_traversal() {
            if let Some(page) = node.page() {
                if let PageType::Workflow(workflow) = page.page_type() {
                    if node
                        .path()
                        .strip_prefix(self.root_path())
                        .expect("path should be in the docs directory")
                        .iter()
                        .next()
                        .expect("path should have a next component")
                        .to_string_lossy()
                        == "external"
                    {
                        categories.insert("External".to_string());
                    } else if let Some(category) = workflow.category() {
                        categories.insert(category);
                    } else {
                        categories.insert("Other".to_string());
                    }
                    nodes.push(node);
                }
            }
        }
        let sorted_categories = sort_workflow_categories(categories);

        for category in sorted_categories {
            let workflows = nodes
                .iter()
                .filter(|node| {
                    let page = node.page().map(|p| p.page_type()).unwrap();
                    if let PageType::Workflow(workflow) = page {
                        if node
                            .path()
                            .strip_prefix(self.root_path())
                            .expect("path should be in the docs directory")
                            .iter()
                            .next()
                            .expect("path should have a next component")
                            .to_string_lossy()
                            == "external"
                        {
                            return category == "External";
                        } else if let Some(cat) = workflow.category() {
                            return cat == category;
                        } else {
                            return category == "Other";
                        }
                    }
                    unreachable!("Expected a workflow page");
                })
                .cloned()
                .collect::<Vec<_>>();
            workflows_by_category.push((category, workflows));
        }

        workflows_by_category
    }

    /// Render a left sidebar component in the "workflows view" mode given a
    /// path.
    fn sidebar_workflows_view(&self, destination: &Path) -> Markup {
        let base = destination.parent().unwrap();
        let workflows_by_category = self.get_workflows_by_category();
        html! {
            @for (category, workflows) in workflows_by_category {
                li class="" {
                    div class="flex items-center gap-x-1 h-6 text-slate-50" {
                        img src=(self.get_asset(base, "category-selected.svg")) class="w-4 h-4" alt="Category icon";
                        p class="" { (category) }
                    }
                    ul class="" {
                        @for node in workflows {
                            li x-data=(format!(r#"{{
                                hover: false,
                                node: {{
                                    selected: {},
                                    img: '{}',
                                }}
                            }}"#,
                            node.path() == destination,
                            self.get_asset(base, if node.path() == destination {
                                    "workflow-selected.svg"
                                } else {
                                    "workflow-unselected.svg"
                                },
                            ))) class="flex flex-row items-center gap-x-1" x-bind:class="node.selected ? 'bg-slate-800' : hover ? 'bg-slate-700' : ''" {
                                @if let Some(page) = node.page() {
                                    @match page.page_type() {
                                        PageType::Workflow(wf) => {
                                            div class="w-px h-6 mr-2 flex-none" {}
                                            div class="w-px h-6 mr-2 flex-none border rounded-none border-gray-700" {}
                                            div class="flex flex-row items-center gap-x-1" x-on:mouseenter="hover = true" x-on:mouseleave="hover = false" {
                                                img x-bind:src="node.img" class="w-4 h-4" alt="Workflow icon";
                                                p class="" x-bind:class="node.selected ? 'text-slate-50' : 'hover:text-slate-50'" { a href=(diff_paths(node.path(), base).unwrap().to_string_lossy()) { (wf.pretty_name()) } }
                                            }
                                        }
                                        _ => {
                                            p { "ERROR: Not a workflow page" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Render a left sidebar component given a path.
    pub fn render_left_sidebar<P: AsRef<Path>>(&self, path: P) -> Markup {
        let root = self.root();
        let path = path.as_ref();
        let base = path.parent().unwrap();

        let make_key = |path: &Path| -> String {
            path.strip_prefix(self.root_path())
                .expect("path should be in the docs directory")
                .to_string_lossy()
                .to_string()
                .replace("-", "_")
                .replace(".", "_")
                .replace(std::path::MAIN_SEPARATOR_STR, "_")
        };

        struct JsNode {
            /// The key of the node.
            key: String,
            /// The display name of the node.
            display_name: String,
            /// The path from the root to the node.
            path: String,
            /// The search name of the node.
            search_name: String,
            /// The icon for the node.
            img: String,
            /// The href for the node.
            href: Option<String>,
            /// Whether the node is selected.
            selected: bool,
            /// Whether the node is the current page.
            current: bool,
            /// The nest level of the node.
            nest_level: usize,
            /// The children of the node.
            children: Vec<String>,
        }

        impl JsNode {
            /// Convert the node to a JavaScript object.
            fn to_js(&self) -> String {
                format!(
                    r#"{{
                        key: '{}',
                        display_name: '{}',
                        path: '{}',
                        search_name: '{}',
                        img: '{}',
                        href: {},
                        selected: {},
                        current: {},
                        nest_level: {}
                    }}"#,
                    self.key,
                    self.display_name,
                    self.path,
                    self.search_name,
                    self.img,
                    if let Some(href) = &self.href {
                        format!("'{}'", href)
                    } else {
                        "null".to_string()
                    },
                    self.selected,
                    self.current,
                    self.nest_level
                )
            }
        }

        let all_nodes = root
            .depth_first_traversal()
            .iter()
            .skip(1) // Skip the root node
            .map(|node| {
                let key = make_key(node.path());
                let display_name = match node.page() {
                    Some(page) => page.name().to_string(),
                    None => node.name().to_string(),
                };
                let inner_path = node
                    .path()
                    .strip_prefix(self.root_path())
                    .expect("path should be in the docs directory")
                    .parent()
                    .expect("path should have a parent")
                    .to_string_lossy()
                    .to_string();
                let search_name = if node.page().is_none() {
                    // Page-less nodes should not be searchable
                    "".to_string()
                } else {
                    node.path()
                        .strip_prefix(self.root_path())
                        .expect("path should be in the docs directory")
                        .to_string_lossy()
                        .to_string()
                };
                let href = match node.page() {
                    Some(page) => match page.page_type() {
                        // TODO: revisit
                        PageType::Index(_) => Some(
                            diff_paths(node.path().join("index.html"), base)
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                        ),
                        _ => Some(
                            diff_paths(node.path(), base)
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                        ),
                    },
                    None => None,
                };
                let selected = path.starts_with(node.path());
                let current = path == node.path();
                let img = match node.page() {
                    Some(page) => match page.page_type() {
                        PageType::Task(_) => self.get_asset(
                            base,
                            if selected {
                                "task-selected.svg"
                            } else {
                                "task-unselected.svg"
                            },
                        ),
                        PageType::Struct(_) => self.get_asset(
                            base,
                            if selected {
                                "struct-selected.svg"
                            } else {
                                "struct-unselected.svg"
                            },
                        ),
                        PageType::Workflow(_) => self.get_asset(
                            base,
                            if selected {
                                "workflow-selected.svg"
                            } else {
                                "workflow-unselected.svg"
                            },
                        ),
                        PageType::Index(_) => self.get_asset(
                            base,
                            if selected {
                                "dir-selected.svg"
                            } else {
                                "dir-unselected.svg"
                            },
                        ),
                    },
                    None => self.get_asset(
                        base,
                        if selected {
                            "dir-selected.svg"
                        } else {
                            "dir-unselected.svg"
                        },
                    ),
                };
                let nest_level = node
                    .path()
                    .strip_prefix(self.root_path())
                    .expect("path should be in the docs directory")
                    .components()
                    .count();
                let children = node
                    .children()
                    .values()
                    .map(|child| make_key(child.path()))
                    .collect::<Vec<String>>();
                JsNode {
                    key,
                    display_name,
                    path: inner_path,
                    search_name: search_name.clone(),
                    img,
                    href,
                    selected,
                    current,
                    nest_level,
                    children,
                }
            })
            .collect::<Vec<JsNode>>();

        let js_dag = all_nodes
            .iter()
            .map(|node| {
                let children = node
                    .children
                    .iter()
                    .map(|child| format!("'{}'", child))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("'{}': [{}]", node.key, children)
            })
            .collect::<Vec<String>>()
            .join(", ");

        let all_nodes_true = all_nodes
            .iter()
            .map(|node| format!("'{}': true", node.key))
            .collect::<Vec<String>>()
            .join(", ");

        let data = format!(
            r#"{{
                showWorkflows: $persist(true).using(sessionStorage),
                search: $persist('').using(sessionStorage),
                chevron: '{}',
                nodes: [{}],
                get searchedNodes() {{
                    if (this.search === '') {{
                        return [];
                    }}
                    this.showWorkflows = false;
                    return this.nodes.filter(node => node.search_name.toLowerCase().includes(this.search.toLowerCase()));
                }},
                get shownNodes() {{
                    if (this.search !== '') {{
                        return [];
                    }}
                    return this.nodes.filter(node => this.showSelfCache[node.key]);
                }},
                dag: {{{}}},
                showSelfCache: $persist({{{}}}).using(sessionStorage),
                showChildrenCache: $persist({{{}}}).using(sessionStorage),
                children(key) {{
                    return this.dag[key];
                }},
                toggleChildren(key) {{
                    this.nodes.forEach(n => {{
                        if (n.key === key) {{
                            this.showChildrenCache[key] = !this.showChildrenCache[key];
                            this.children(key).forEach(child => {{
                                this.setShow(child, this.showChildrenCache[key]);
                            }});
                        }}
                    }});
                }},
                setShow(key, value) {{
                    this.nodes.forEach(n => {{
                        if (n.key === key) {{
                            this.showSelfCache[key] = value;
                            this.showChildrenCache[key] = value;
                            this.children(key).forEach(child => {{
                                this.setShow(child, value);
                            }});
                        }}
                    }});
                }},
                reset() {{
                    this.nodes.forEach(n => {{
                        this.showSelfCache[n.key] = true;
                        this.showChildrenCache[n.key] = true;
                    }});
                }}
            }}"#,
            self.get_asset(base, "chevron-down.svg"),
            all_nodes
                .iter()
                .map(|node| node.to_js())
                .collect::<Vec<String>>()
                .join(", "),
            js_dag,
            all_nodes_true,
            all_nodes_true,
        );

        html! {
            div x-data=(data) class="docs-tree__container" {
                div class="" {
                    img src=(self.get_asset(base, "sprocket-logo.svg")) class="w-2/3 flex-none sticky mb-4" alt="Sprocket logo";
                    form id="searchbar" class="flex-none items-center gap-x-2 w-9/10 h-[40px] sticky rounded-md border border-slate-700 mb-4" {
                        div class="flex flex-row items-center h-full w-full" {
                            img src=(self.get_asset(base, "search.svg")) class="flex size-6" alt="Search icon";
                            input id="searchbox" x-model="search" type="text" placeholder="Search..." class="flex h-full w-full text-slate-300 pl-2";
                            img src=(self.get_asset(base, "x-mark.svg")) class="flex size-6 hover:cursor-pointer ml-2 pr-2" alt="Clear icon" x-show="search !== ''" x-on:click="search = ''";
                        }
                    }
                    div class="flex items-center sticky gap-x-1 pr-4" {
                        div x-on:click="showWorkflows = true; search = ''" class="flex grow items-center gap-x-1 border-b hover:cursor-pointer" x-bind:class="! showWorkflows ? 'text-slate-400 hover:text-slate-300' : 'text-slate-50'" {
                            img src=(self.get_asset(base, "list-bullet-selected.svg")) class="w-4 h-4" alt="List icon";
                            p { "Workflows" }
                        }
                        div x-on:click="showWorkflows = false" class="flex grow items-center gap-x-1 border-b hover:cursor-pointer" x-bind:class="showWorkflows ? 'text-slate-400 hover:text-slate-300' : 'text-slate-50'" {
                            img src=(self.get_asset(base, "folder-selected.svg")) class="w-4 h-4" alt="List icon";
                            p { "Full Directory" }
                        }
                    }
                }
                div x-cloak class="flex-row w-full h-full rounded-md pt-2 pl-2 overflow-x-auto overflow-y-scroll" {
                    ul x-show="! showWorkflows || search != ''" class="w-max pr-3" {
                        li class="flex flex-row items-center gap-x-1 text-slate-50" {
                            img x-show="search === ''" src=(self.get_asset(base, "dir-selected.svg")) class="w-4 h-4" alt="Directory icon";
                            p x-show="search === ''" class="" { a href=(self.root_index_relative_to(base).to_string_lossy()) { (root.name()) } }
                        }
                        template x-for="node in shownNodes" {
                            li x-data="{ hover: false }" class="flex flex-row items-center gap-x-1" x-bind:class="node.current ? 'bg-slate-800' : hover ? 'bg-slate-700' : ''" {
                                template x-for="i in Array.from({ length: node.nest_level })" {
                                    div x-show="showSelfCache[node.key]" class="w-px h-6 border rounded-none border-gray-700 mr-2" {}
                                }
                                div class="flex flex-row items-center gap-x-1" x-show="showSelfCache[node.key]" x-on:mouseenter="hover = (node.href !== null)" x-on:mouseleave="hover = false" {
                                    img x-show="showSelfCache[node.key]" x-data="{ showChevron: false }" x-on:click="toggleChildren(node.key)" x-on:mouseenter="showChevron = true" x-on:mouseleave="showChevron = false" x-bind:src="showChevron && (children(node.key).length > 0) ? chevron : node.img" x-bind:class="(children(node.key).length > 0) ? 'hover:cursor-pointer' : ''" class="w-4 h-4" alt="Node icon";
                                    p x-show="showSelfCache[node.key]" class="" x-bind:class="node.selected ? 'text-slate-50' : (node.search_name === '') ? '' : 'hover:text-slate-50'" { a x-bind:href="node.href" x-text="node.display_name" {} }
                                }
                            }
                        }
                        template x-for="node in searchedNodes" {
                            li class="flex flex-col hover:bg-slate-800 border-b border-gray-600 pl-2" {
                                p class="text-xs" x-text="node.path" {}
                                div class="flex flex-row items-center gap-x-1 mb-2" {
                                    img x-bind:src="node.img" class="w-4 h-4" alt="Node icon";
                                    p class="text-slate-50" { a x-bind:href="node.href" x-text="node.display_name" {} }
                                }
                            }
                        }
                        li class="flex place-content-center" {
                            img x-show="search !== '' && searchedNodes.length === 0" src=(self.get_asset(base, "search.svg")) class="size-8" alt="Search icon";
                        }
                        li class="flex place-content-center" {
                            p x-show="search !== '' && searchedNodes.length === 0" class="" x-text="'No results found for \"' + search + '\"'" {}
                        }
                    }
                    ul x-show="showWorkflows && search === ''" class="w-max pr-3" {
                        (self.sidebar_workflows_view(path))
                    }
                    div class="w-6 h-full absolute bg-linear-to-r from-transparent to-slate-900 top-0 right-3" {}
                }
            }
        }
    }

    /// Render a right sidebar component.
    pub fn render_right_sidebar(&self, headers: PageHeaders) -> Markup {
        html! {
            div class="right-sidebar__container" {
                div class="right-sidebar__header" {
                    "ON THIS PAGE"
                }
                (headers.render())
                div class="right-sidebar__back-to-top-container" {
                    a href="#title" class="right-sidebar__back-to-top" {
                        span class="right-sidebar__back-to-top-icon" {
                            "↑"
                        }
                        span class="right-sidebar__back-to-top-text" {
                            "Back to top"
                        }
                    }
                }
            }
        }
    }

    /// Renders a page "breadcrumb" navigation component.
    pub fn render_breadcrumbs<P: AsRef<Path>>(&self, path: P) -> Markup {
        let path = path.as_ref();
        let base = path.parent().expect("path should have a parent");

        let mut current_path = path
            .strip_prefix(self.root().path())
            .expect("path should be in the docs directory");

        let mut breadcrumbs = vec![];

        let cur_page = self.get_page(path).unwrap_or(self.get_page(base).unwrap());
        breadcrumbs.push((cur_page.name(), None));

        if matches!(cur_page.page_type(), PageType::Index(_)) {
            // TODO: revisit logic to remove this hack
            current_path = current_path.parent().expect("path should have a parent");
        }

        while let Some(parent) = current_path.parent() {
            let cur_node = self.get_node(parent).expect("path should have a node");
            breadcrumbs.push((
                cur_node.page().map(|n| n.name()).unwrap_or(cur_node.name()),
                if let Some(page) = cur_node.page() {
                    match page.page_type() {
                        // TODO: revisit
                        PageType::Index(_) => {
                            Some(diff_paths(cur_node.path().join("index.html"), base).unwrap())
                        }
                        _ => Some(diff_paths(cur_node.path(), base).unwrap()),
                    }
                } else {
                    None
                },
            ));
            current_path = parent;
        }
        breadcrumbs.reverse();
        let mut breadcrumbs = breadcrumbs.into_iter();
        let first = breadcrumbs
            .next()
            .expect("should have at least one breadcrumb");
        let first = html! {
            a href=(self.root_index_relative_to(base).to_string_lossy()) { (first.0) }
        };

        html! {
            div class="" {
                (first)
                @for crumb in breadcrumbs {
                    span { " / " }
                    @if let Some(path) = crumb.1 {
                        a href=(path.to_string_lossy()) {(crumb.0)}
                    } @else {
                        span { (crumb.0) }
                    }
                }
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
        let index_path = self.root_path().join("index.html");

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
                                            // TODO: revisit
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
                div class="layout__container" {
                    div class="layout__sidebar-left" {
                        (left_sidebar)
                    }
                    div class="layout__main-center" {
                        (content)
                    }
                    div class="layout__sidebar-right" {
                        (self.render_right_sidebar(PageHeaders::default()))
                    }
                }
            },
            self.stylesheet_relative_to(self.root_path()),
        );
        std::fs::write(index_path, html.into_string())?;
        Ok(())
    }

    /// Write a page to disk at the designated path.
    pub fn write_page<P: Into<PathBuf>>(&self, page: &HTMLPage, path: P) -> anyhow::Result<()> {
        let mut path = path.into();

        let (content, headers) = match page.page_type() {
            // TODO: revisit
            PageType::Index(doc) => {
                path = path.join("index.html");
                doc.render()
            }
            PageType::Struct(s) => s.render(),
            PageType::Task(t) => t.render(),
            PageType::Workflow(w) => w.render(),
        };

        let breadcrumbs = self.render_breadcrumbs(&path);

        let stylesheet =
            self.stylesheet_relative_to(path.parent().expect("path should have a parent"));
        let left_sidebar = self.render_left_sidebar(&path);

        let html = full_page(
            page.name(),
            html! {
                div class="layout__container" {
                    div class="layout__sidebar-left" {
                        (left_sidebar)
                    }
                    div class="layout__main-center" {
                        (breadcrumbs)
                        (content)
                    }
                    div class="layout__sidebar-right" {
                        (self.render_right_sidebar(headers))
                    }
                }
            },
            stylesheet,
        );
        std::fs::write(path, html.into_string())?;
        Ok(())
    }
}

/// Sort workflow categories in a specific order.
fn sort_workflow_categories(categories: HashSet<String>) -> Vec<String> {
    let mut sorted_categories: Vec<String> = categories.into_iter().collect();
    sorted_categories.sort_by(|a, b| {
        if a == "External" {
            std::cmp::Ordering::Greater
        } else if b == "External" {
            std::cmp::Ordering::Less
        } else if a == "Other" {
            std::cmp::Ordering::Greater
        } else if b == "Other" {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });
    sorted_categories
}

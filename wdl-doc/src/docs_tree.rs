//! Implementations for a [`DocsTree`] which represents the docs directory.

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::path::absolute;
use std::rc::Rc;

use anyhow::Result;
use maud::Markup;
use maud::html;
use pathdiff::diff_paths;

use crate::Document;
use crate::Markdown;
use crate::Render;
use crate::full_page;
use crate::r#struct::Struct;
use crate::task::Task;
use crate::workflow::Workflow;
use crate::write_assets;

/// The type of a page.
#[derive(Debug)]
pub(crate) enum PageType {
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
pub(crate) struct HTMLPage {
    /// The display name of the page.
    name: String,
    /// The type of the page.
    page_type: PageType,
}

impl HTMLPage {
    /// Create a new HTML page.
    pub(crate) fn new(name: String, page_type: PageType) -> Self {
        Self { name, page_type }
    }

    /// Get the name of the page.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the type of the page.
    pub(crate) fn page_type(&self) -> &PageType {
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
    /// The path from the root to the node.
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

    /// Get the path from the root to the node.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Determine if the node is part of a path.
    ///
    /// Path can be an absolute path or a path relative to the root.
    pub fn part_of_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        self.path()
            .components()
            .all(|c| path.components().any(|p| p == c))
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

/// A builder for a [`DocsTree`] which represents the docs directory.
#[derive(Debug)]
pub struct DocsTreeBuilder {
    /// The root directory for the docs.
    root: PathBuf,
    /// The path to a Markdown file to embed in the `<root>/index.html` page.
    homepage: Option<PathBuf>,
}

impl DocsTreeBuilder {
    /// Create a new docs tree builder.
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = path_clean::clean(absolute(root.as_ref()).unwrap());
        Self {
            root,
            homepage: None,
        }
    }

    /// Set the homepage for the docs with an option.
    pub fn maybe_homepage(mut self, homepage: Option<impl Into<PathBuf>>) -> Self {
        self.homepage = homepage.map(|hp| hp.into());
        self
    }

    /// Set the homepage for the docs.
    pub fn homepage(self, homepage: impl Into<PathBuf>) -> Self {
        self.maybe_homepage(Some(homepage))
    }

    /// Build the docs tree.
    pub fn build(self) -> Result<DocsTree> {
        write_assets(&self.root)?;
        let node = Node::new(
            self.root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or("docs".to_string()),
            PathBuf::from(""),
        );
        Ok(DocsTree {
            root: node,
            path: self.root,
            homepage: self.homepage,
        })
    }
}

/// A tree representing the docs directory.
#[derive(Debug)]
pub struct DocsTree {
    /// The root of the tree.
    root: Node,
    /// The absolute path to the root directory.
    path: PathBuf,
    /// An optional path to a Markdown file to embed in the `<root>/index.html`
    /// page.
    homepage: Option<PathBuf>,
}

impl DocsTree {
    /// Get the root of the tree.
    fn root(&self) -> &Node {
        &self.root
    }

    /// Get the root of the tree as mutable.
    fn root_mut(&mut self) -> &mut Node {
        &mut self.root
    }

    /// Get the absolute path to the root directory.
    fn root_abs_path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the path to the root directory relative to a given path.
    pub fn root_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(self.root_abs_path(), path).unwrap()
    }

    /// Get the absolute path to the stylesheet.
    pub fn stylesheet(&self) -> PathBuf {
        self.root_abs_path().join("style.css")
    }

    /// Get the absolute path to the assets directory.
    pub fn assets(&self) -> PathBuf {
        self.root_abs_path().join("assets")
    }

    /// Get a relative path to the assets directory.
    fn assets_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(self.assets(), path).unwrap()
    }

    /// Get a relative path to an asset in the assets directory (converted to a
    /// string).
    fn get_asset<P: AsRef<Path>>(&self, path: P, asset: &str) -> String {
        self.assets_relative_to(path)
            .join(asset)
            .to_string_lossy()
            .to_string()
    }

    /// Get a relative path to the root index page.
    fn root_index_relative_to<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        diff_paths(self.root_abs_path().join("index.html"), path).unwrap()
    }

    /// Add a page to the tree.
    ///
    /// Path can be an absolute path or a path relative to the root.
    pub(crate) fn add_page<P: Into<PathBuf>>(&mut self, path: P, page: Rc<HTMLPage>) {
        let path = path.into();
        let rel_path = path.strip_prefix(self.root_abs_path()).unwrap_or(&path);

        let root = self.root_mut();
        let mut current_node = root;

        let mut components = rel_path.components().peekable();
        while let Some(component) = components.next() {
            let cur_name = component.as_os_str().to_str().unwrap();
            if current_node.children.contains_key(cur_name) {
                current_node = current_node.children.get_mut(cur_name).unwrap();
            } else {
                let new_path = current_node.path().join(component);
                let new_node = Node::new(cur_name.to_string(), new_path);
                current_node.children.insert(cur_name.to_string(), new_node);
                current_node = current_node.children.get_mut(cur_name).unwrap();
            }
            if let Some(next_component) = components.peek() {
                if next_component.as_os_str().to_str().unwrap() == "index.html" {
                    current_node.path = current_node.path().join("index.html");
                    break;
                }
            }
        }

        current_node.page = Some(page);
    }

    /// Get the Node associated with a path.
    ///
    /// Path can be an absolute path or a path relative to the root.
    fn get_node<P: AsRef<Path>>(&self, path: P) -> Option<&Node> {
        let root = self.root();
        let path = path.as_ref();
        let rel_path = path.strip_prefix(self.root_abs_path()).unwrap_or(path);

        let mut current_node = root;

        for component in rel_path
            .components()
            .map(|c| c.as_os_str().to_str().unwrap())
        {
            if component == "index.html" {
                return Some(current_node);
            }
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
    pub(crate) fn get_page<P: AsRef<Path>>(&self, path: P) -> Option<&Rc<HTMLPage>> {
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
    ///
    /// Destination is expected to be an absolute path.
    fn sidebar_workflows_view(&self, destination: &Path) -> Markup {
        let base = destination.parent().unwrap();
        let workflows_by_category = self.get_workflows_by_category();
        html! {
            @for (category, workflows) in workflows_by_category {
                li class="" {
                    div class="flex items-center gap-x-1 h-6 text-slate-50" {
                        img src=(self.get_asset(base, "category-selected.svg")) class="size-4" alt="Category icon";
                        p class="" { (category) }
                    }
                    ul class="" {
                        @for node in workflows {
                            li x-data=(format!(r#"{{
                                hover: false,
                                node: {{
                                    current: {},
                                    icon: '{}',
                                }}
                            }}"#,
                            self.root_abs_path().join(node.path()) == destination,
                            self.get_asset(base, if self.root_abs_path().join(node.path()) == destination {
                                    "workflow-selected.svg"
                                } else {
                                    "workflow-unselected.svg"
                                },
                            ))) class="flex flex-row items-center gap-x-1" x-bind:class="node.current ? 'bg-slate-800' : hover ? 'bg-slate-700' : ''" {
                                @if let Some(page) = node.page() {
                                    @match page.page_type() {
                                        PageType::Workflow(wf) => {
                                            div class="w-px h-6 mr-2 flex-none" {}
                                            div class="w-px h-6 mr-2 flex-none border rounded-none border-gray-700" {}
                                            div class="flex flex-row items-center gap-x-1" x-on:mouseenter="hover = true" x-on:mouseleave="hover = false" {
                                                img x-bind:src="node.icon" class="size-4" alt="Workflow icon";
                                                sprocket-tooltip content=(wf.pretty_name()) class="" x-bind:class="node.current ? 'text-slate-50' : 'hover:text-slate-50'" {
                                                    a href=(diff_paths(self.root_abs_path().join(node.path()), base).unwrap().to_string_lossy()) {
                                                        (wf.pretty_name())
                                                    }
                                                }
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
    ///
    /// Path is expected to be an absolute path.
    fn render_left_sidebar<P: AsRef<Path>>(&self, path: P) -> Markup {
        let root = self.root();
        let path = path.as_ref();
        let base = path.parent().unwrap();

        let make_key = |path: &Path| -> String {
            let path = if path.file_name().unwrap() == "index.html" {
                // Remove unnecessary index.html from the path.
                // Not needed for the key.
                path.parent().unwrap()
            } else {
                path
            };
            path.to_string_lossy()
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
            /// The parent directory of the node.
            ///
            /// This is used for displaying the path to the node in the sidebar.
            parent: String,
            /// The search name of the node.
            search_name: String,
            /// The icon for the node.
            icon: Option<String>,
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
                        parent: '{}',
                        search_name: '{}',
                        icon: {},
                        href: {},
                        selected: {},
                        current: {},
                        nest_level: {}
                    }}"#,
                    self.key,
                    self.display_name,
                    self.parent,
                    self.search_name,
                    if let Some(icon) = &self.icon {
                        format!("'{}'", icon)
                    } else {
                        "null".to_string()
                    },
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
                let parent = node
                    .path()
                    .parent()
                    .expect("path should have a parent")
                    .to_string_lossy()
                    .to_string();
                let search_name = if node.page().is_none() {
                    // Page-less nodes should not be searchable
                    "".to_string()
                } else {
                    node.path().to_string_lossy().to_string()
                };
                let href = if node.page().is_some() {
                    Some(
                        diff_paths(self.root_abs_path().join(node.path()), base)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    )
                } else {
                    None
                };
                let selected = node.part_of_path(path);
                let current = path == self.root_abs_path().join(node.path());
                let icon = match node.page() {
                    Some(page) => match page.page_type() {
                        PageType::Task(_) => Some(self.get_asset(
                            base,
                            if selected {
                                "task-selected.svg"
                            } else {
                                "task-unselected.svg"
                            },
                        )),
                        PageType::Struct(_) => Some(self.get_asset(
                            base,
                            if selected {
                                "struct-selected.svg"
                            } else {
                                "struct-unselected.svg"
                            },
                        )),
                        PageType::Workflow(_) => Some(self.get_asset(
                            base,
                            if selected {
                                "workflow-selected.svg"
                            } else {
                                "workflow-unselected.svg"
                            },
                        )),
                        PageType::Index(_) => Some(self.get_asset(
                            base,
                            if selected {
                                "wdl-dir-selected.svg"
                            } else {
                                "wdl-dir-unselected.svg"
                            },
                        )),
                    },
                    None => None,
                };
                let nest_level = node
                    .path()
                    .components()
                    .filter(|c| c.as_os_str().to_str().unwrap() != "index.html")
                    .count();
                let children = node
                    .children()
                    .values()
                    .map(|child| make_key(child.path()))
                    .collect::<Vec<String>>();
                JsNode {
                    key,
                    display_name,
                    parent,
                    search_name: search_name.clone(),
                    icon,
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
                dirOpen: '{}',
                dirClosed: '{}',
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
            self.get_asset(base, "dir-open.svg"),
            self.get_asset(base, "dir-closed.svg"),
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
            div x-data=(data) x-init="$nextTick(() => { document.querySelector('.is-scrolled-to')?.scrollIntoView(); })" class="docs-tree__container" {
                div class="sticky" {
                    img src=(self.get_asset(base, "sprocket-logo.svg")) class="w-2/3 flex-none mb-4" alt="Sprocket logo";
                    form id="searchbar" class="flex-none items-center gap-x-2 w-9/10 h-[40px] rounded-md border border-slate-700 mb-4" {
                        div class="flex flex-row items-center size-full" {
                            img src=(self.get_asset(base, "search.svg")) class="flex size-6" alt="Search icon";
                            input id="searchbox" x-model="search" type="text" placeholder="Search..." class="flex size-full text-slate-300 pl-2";
                            img src=(self.get_asset(base, "x-mark.svg")) class="flex size-6 hover:cursor-pointer ml-2 pr-2" alt="Clear icon" x-show="search !== ''" x-on:click="search = ''";
                        }
                    }
                    div class="flex items-center gap-x-1 pr-4" {
                        div x-on:click="showWorkflows = true; search = ''" class="flex grow items-center gap-x-1 border-b hover:cursor-pointer" x-bind:class="! showWorkflows ? 'text-slate-400 hover:text-slate-300' : 'text-slate-50'" {
                            img src=(self.get_asset(base, "list-bullet-selected.svg")) class="size-4" alt="List icon";
                            p { "Workflows" }
                        }
                        div x-on:click="showWorkflows = false" class="flex grow items-center gap-x-1 border-b hover:cursor-pointer" x-bind:class="showWorkflows ? 'text-slate-400 hover:text-slate-300' : 'text-slate-50'" {
                            img src=(self.get_asset(base, "folder-selected.svg")) class="size-4" alt="List icon";
                            p { "Full Directory" }
                        }
                    }
                }
                div x-cloak class="size-full rounded-md pt-2 pl-2 overflow-x-clip overflow-y-scroll" {
                    ul x-show="! showWorkflows || search != ''" class="w-max pr-3" {
                        li class="flex flex-row items-center gap-x-1 text-slate-50" {
                            img x-show="search === ''" src=(self.get_asset(base, "dir-open.svg")) class="size-4" alt="Directory icon";
                            sprocket-tooltip content=(root.name()) x-show="search === ''" {
                                a href=(self.root_index_relative_to(base).to_string_lossy()) { (root.name()) }
                            }
                        }
                        template x-for="node in shownNodes" {
                            li x-data="{ hover: false }" class="flex flex-row items-center gap-x-1" x-bind:class="node.current ? 'bg-slate-800 is-scrolled-to' : hover ? 'bg-slate-700' : ''" {
                                template x-for="i in Array.from({ length: node.nest_level })" {
                                    div x-show="showSelfCache[node.key]" class="w-px h-6 border rounded-none border-gray-700 mr-2" {}
                                }
                                div class="flex flex-row items-center gap-x-1" x-show="showSelfCache[node.key]" x-on:mouseenter="hover = (node.href !== null)" x-on:mouseleave="hover = false" {
                                    img x-show="showSelfCache[node.key]" x-data="{ showChevron: false }" x-on:click="toggleChildren(node.key)" x-on:mouseenter="showChevron = true" x-on:mouseleave="showChevron = false" x-bind:src="showChevron && (children(node.key).length > 0) ? chevron : (node.icon !== null) ? node.icon : (showChildrenCache[node.key]) ? dirOpen : dirClosed" x-bind:class="(children(node.key).length > 0) ? 'hover:cursor-pointer' : ''" class="size-4" alt="Node icon";
                                    sprocket-tooltip x-bind:content="node.display_name" x-show="showSelfCache[node.key]" class="" x-bind:class="node.selected ? 'text-slate-50' : (node.search_name === '') ? '' : 'hover:text-slate-50'" {
                                        a x-bind:href="node.href" x-text="node.display_name" {}
                                    }
                                }
                            }
                        }
                        template x-for="node in searchedNodes" {
                            li class="flex flex-col hover:bg-slate-800 border-b border-gray-600 text-slate-50 pl-2" {
                                p class="text-xs" x-text="node.parent" {}
                                div class="flex flex-row items-center gap-x-1 mb-2" {
                                    img x-bind:src="node.icon" class="size-4" alt="Node icon";
                                    sprocket-tooltip x-bind:content="node.display_name" {
                                        a x-bind:href="node.href" x-text="node.display_name" {}
                                    }
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
                }
            }
        }
    }

    /// Render a right sidebar component.
    fn render_right_sidebar(&self, headers: PageHeaders) -> Markup {
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
    ///
    /// Path is expected to be an absolute path.
    fn render_breadcrumbs<P: AsRef<Path>>(&self, path: P) -> Markup {
        let path = path.as_ref();
        let base = path.parent().expect("path should have a parent");

        let mut current_path = path
            .strip_prefix(self.root_abs_path())
            .expect("path should be in the docs directory");

        let mut breadcrumbs = vec![];

        let cur_page = self.get_page(path).expect("path should have a page");
        breadcrumbs.push((cur_page.name(), None));

        while let Some(parent) = current_path.parent() {
            let cur_node = self.get_node(parent).expect("path should have a node");
            breadcrumbs.push((
                cur_node.page().map(|n| n.name()).unwrap_or(cur_node.name()),
                if cur_node.page().is_some() {
                    Some(diff_paths(self.root_abs_path().join(cur_node.path()), base).unwrap())
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
            div class="mb-4" {
                (first)
                @for crumb in breadcrumbs {
                    span { " / " }
                    @if let Some(path) = crumb.1 {
                        a href=(path.to_string_lossy()) class="layout__breadcrumb-clickable" { (crumb.0) }
                    } @else {
                        span class="layout__breadcrumb-inactive" { (crumb.0) }
                    }
                }
            }
        }
    }

    /// Render every page in the tree.
    pub fn render_all(&self) -> Result<()> {
        let root = self.root();

        for node in root.depth_first_traversal() {
            if let Some(page) = node.page() {
                self.write_page(page.as_ref(), self.root_abs_path().join(node.path()))?;
            }
        }

        self.write_homepage()?;
        Ok(())
    }

    /// Write the homepage to disk.
    fn write_homepage(&self) -> Result<()> {
        let index_path = self.root_abs_path().join("index.html");

        let left_sidebar = self.render_left_sidebar(&index_path);
        let content = html! {
            h1 class="main__title" { "Home" }
            @if let Some(homepage) = &self.homepage {
                div class="markdown-body" {
                    (Markdown(std::fs::read_to_string(homepage)?).render())
                }
            } @else {
                div class="flex flex-col flex-grow items-center justify-center size-full gap-y-2 pt-8" {
                    img src=(self.get_asset(self.root_abs_path(), "missing-home.svg")) class="size-12" alt="Missing home icon";
                    h2 class="main__section-header" { "There's nothing to see on this page" }
                    p { "The markdown file for this page wasn't supplied." }
                }
            }
        };

        let html = full_page(
            "Home",
            html! {
                div class="layout__container layout__container--with-mobile-menu" x-data="{ open: false }" x-bind:class="open ? 'open' : ''" {
                    div class="layout__sidebar-left" {
                        (left_sidebar)
                    }
                    div class="layout__main-center" {
                        button type="button" class="layout__mobile-menu-button" x-on:click="open = !open" aria-label="Toggle menu" {
                            svg viewBox="0 0 100 80" width="40" height="40" stroke="none" fill="currentColor" {
                                rect width="100" height="15" {}
                                rect y="35" width="100" height="15" {}
                                rect y="70" width="100" height="15" {}
                            }
                        }
                        (content)
                    }
                    div class="layout__sidebar-right" {
                        (self.render_right_sidebar(PageHeaders::default()))
                    }
                }
            },
            self.root().path(),
        );
        std::fs::write(index_path, html.into_string())?;
        Ok(())
    }

    /// Write a page to disk at the designated path.
    ///
    /// Path is expected to be an absolute path.
    fn write_page<P: Into<PathBuf>>(&self, page: &HTMLPage, path: P) -> Result<()> {
        let path = path.into();
        let base = path.parent().expect("path should have a parent");

        let (content, headers) = match page.page_type() {
            PageType::Index(doc) => doc.render(),
            PageType::Struct(s) => s.render(),
            PageType::Task(t) => t.render(&self.assets_relative_to(base)),
            PageType::Workflow(w) => w.render(&self.assets_relative_to(base)),
        };

        let breadcrumbs = self.render_breadcrumbs(&path);

        let left_sidebar = self.render_left_sidebar(&path);

        let html = full_page(
            page.name(),
            html! {
                div class="layout__container layout__container--with-mobile-menu" x-data="{ open: false }" x-bind:class="open ? 'open' : ''" {
                    div class="layout__sidebar-left" {
                        (left_sidebar)
                    }
                    div class="layout__main-center" {
                        div class="layout__breadcrumbs" {
                            (breadcrumbs)
                        }
                        button type="button" class="layout__mobile-menu-button" x-on:click="open = !open" aria-label="Toggle menu" {
                            svg viewBox="0 0 100 80" width="40" height="40" stroke="none" fill="currentColor" {
                                rect width="100" height="15" {}
                                rect y="35" width="100" height="15" {}
                                rect y="70" width="100" height="15" {}
                            }
                        }
                        (content)
                    }
                    div class="layout__sidebar-right" {
                        (self.render_right_sidebar(headers))
                    }
                }
            },
            self.root_relative_to(base),
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

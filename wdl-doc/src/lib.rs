//! Library for generating HTML documentation from WDL files.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

pub mod callable;
pub mod docs_tree;
pub mod meta;
pub mod parameter;
pub mod r#struct;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
pub use callable::Callable;
pub use callable::task;
pub use callable::workflow;
pub use docs_tree::DocsTree;
use docs_tree::HTMLPage;
pub use docs_tree::Node;
use docs_tree::PageType;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::Render;
use maud::html;
use parameter::InputOutput;
use pathdiff::diff_paths;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use tokio::io::AsyncWriteExt;
use wdl_analysis::Analyzer;
use wdl_analysis::DiagnosticsConfig;
use wdl_analysis::rules;
use wdl_ast::AstToken;
use wdl_ast::SyntaxTokenExt;
use wdl_ast::VersionStatement;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::MetadataValue;

/// The directory where the generated documentation will be stored.
///
/// This directory will be created in the workspace directory.
const DOCS_DIR: &str = "docs";

/// Links to a CSS stylesheet at the given path.
struct Css<'a>(&'a str);

impl Render for Css<'_> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" type="text/css" href=(self.0);
        }
    }
}

/// A basic header with a `page_title` and link to the stylesheet.
pub(crate) fn header(page_title: &str, stylesheet: &Path) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { (page_title) }
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,100..1000;1,9..40,100..1000&display=swap" rel="stylesheet";
            (Css(stylesheet.to_str().unwrap()))
        }
    }
}

pub(crate) fn sidebar(docs_tree: &DocsTree) -> Markup {
    html! {
        div class="top-0 left-0 h-full w-1/6 dark:bg-slate-950 dark:text-white" {
            h1 class="text-2xl text-center" { "Table of Contents" }
            @for node in docs_tree.depth_first_traversal() {
                @if let Some(page) = node.page() {
                    p { a href=(page.path().to_str().unwrap()) { (page.name()) } }
                } @else {
                    p class="" { (node.name()) }
                }
            }
        }
    }
}

/// A full HTML page with a header and body.
pub(crate) fn full_page(
    page_title: &str,
    docs_tree: &DocsTree,
    stylesheet: &Path,
    body: Markup,
) -> Markup {
    html! {
        (DOCTYPE)
        html class="dark size-full" {
            (header(page_title, stylesheet))
            body class="flex dark size-full dark:bg-slate-950 dark:text-white" {
                (sidebar(docs_tree))
                (body)
           }
        }
    }
}

/// Renders a block of Markdown using `pulldown-cmark`.
pub(crate) struct Markdown<T>(T);

impl<T: AsRef<str>> Render for Markdown<T> {
    fn render(&self) -> Markup {
        // Generate raw HTML
        let mut unsafe_html = String::new();
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(self.0.as_ref(), options);
        pulldown_cmark::html::push_html(&mut unsafe_html, parser);
        // Sanitize it with ammonia
        let safe_html = ammonia::clean(&unsafe_html);
        PreEscaped(safe_html)
    }
}

/// Render an HTML Table of Contents for the home page.
pub(crate) fn home_toc(docs_tree: &DocsTree) -> Markup {
    html! {
        div class="flex flex-col items-center text-left" {
            h3 class="" { "Table of Contents" }
            table class="border" {
                thead class="border" { tr {
                    th class="" { "Page" }
                }}
                tbody class="border" {
                    @for entry in docs_tree.get_index_pages() {
                        tr class="border" {
                            td class="border" {
                                a href=(entry.path().to_str().unwrap()) { (entry.name()) }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render an HTML Table of Contents for an index page.
pub(crate) fn toc(pages: &[&HTMLPage]) -> Markup {
    html! {
        div class="flex flex-col items-center text-left" {
            h3 class="" { "Table of Contents" }
            table class="border" {
                thead class="border" { tr {
                    th class="" { "Page" }
                    th class="" { "Type" }
                    th class="" { "Description" }
                }}
                tbody class="border" {
                    @for entry in pages {
                        tr class="border" {
                            td class="border" {
                                a href=(entry.path().to_str().unwrap()) { (entry.name()) }
                            }
                            td class="border" {
                                @match &entry.page_type().as_ref().expect("should have a page type") {
                                    PageType::Struct => { "Struct" }
                                    PageType::Task(_) => { "Task" }
                                    PageType::Workflow(_) => { "Workflow" }
                                }
                            }
                            td class="border" {
                                @match &entry.page_type().as_ref().expect("should have a page type") {
                                    PageType::Struct => {}
                                    PageType::Task(desc) => { (desc) }
                                    PageType::Workflow(desc) => { (desc) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Parse the preamble comments of a document using the version statement.
fn parse_preamble_comments(version: VersionStatement) -> String {
    let comments = version
        .keyword()
        .syntax()
        .preceding_trivia()
        .map(|t| match t.kind() {
            wdl_ast::SyntaxKind::Comment => match t.to_string().strip_prefix("## ") {
                Some(comment) => comment.to_string(),
                None => "".to_string(),
            },
            wdl_ast::SyntaxKind::Whitespace => "".to_string(),
            _ => {
                panic!("Unexpected token kind: {:?}", t.kind())
            }
        })
        .collect::<Vec<_>>();
    comments.join("\n")
}

/// A WDL document.
#[derive(Debug)]
pub struct Document {
    /// The name of the document.
    name: String,
    /// The parent directory of the document.
    parent: PathBuf,
    /// The AST node for the version statement.
    ///
    /// This is used both to display the WDL version number and to fetch the
    /// preamble comments.
    version: VersionStatement,
}

impl Document {
    /// Create a new document.
    pub fn new(name: String, parent: PathBuf, version: VersionStatement) -> Self {
        Self {
            name,
            parent,
            version,
        }
    }

    /// Get the name of the document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the document as text.
    pub fn version(&self) -> String {
        self.version.version().as_str().to_string()
    }

    /// Get the preamble comments of the document.
    pub fn preamble(&self) -> Markup {
        let preamble = parse_preamble_comments(self.version.clone());
        Markdown(&preamble).render()
    }

    /// Render the document as HTML.
    pub fn render(&self, docs_tree: &DocsTree, stylesheet: &Path) -> Markup {
        let body = html! {
            h1 { (self.name()) }
            h3 { "WDL Version: " (self.version()) }
            div { (self.preamble()) }
            (toc(&docs_tree.get_node(&self.parent).unwrap().get_non_index_pages()))
        };

        full_page(self.name(), docs_tree, stylesheet, body)
    }
}

/// Generate HTML documentation for a workspace.
///
/// This function will generate HTML documentation for all WDL files in the
/// workspace directory. The generated documentation will be stored in a
/// `docs` directory within the workspace.
///
/// The contents of `css` will be written to a `style.css` file
/// in the `docs` directory.
pub async fn document_workspace(workspace: PathBuf, css: String) -> Result<PathBuf> {
    if !workspace.is_dir() {
        return Err(anyhow!("Workspace is not a directory"));
    }

    let abs_path = workspace.canonicalize()?;

    let docs_dir = abs_path.join(DOCS_DIR);
    if !docs_dir.exists() {
        std::fs::create_dir(&docs_dir)?;
    }

    let abs_css_path = docs_dir.join("style.css");
    tokio::fs::write(&abs_css_path, css).await?;

    let analyzer = Analyzer::new(DiagnosticsConfig::new(rules()), |_: (), _, _, _| async {});
    analyzer.add_directory(abs_path.clone()).await?;
    let results = analyzer.analyze(()).await?;

    let mut docs_tree = docs_tree::DocsTree::new(Node::new(DOCS_DIR.to_string(), docs_dir.clone()));

    for result in results {
        let uri = result.document().uri();
        let relative_path = match uri.to_file_path() {
            Ok(path) => match path.strip_prefix(&abs_path) {
                Ok(path) => path.to_path_buf(),
                Err(_) => PathBuf::from("external").join(path.strip_prefix("/").unwrap()),
            },
            Err(_) => PathBuf::from("external").join(uri.path().strip_prefix("/").unwrap()),
        };
        let cur_dir = docs_dir.clone().join(relative_path.with_extension(""));
        if !cur_dir.exists() {
            std::fs::create_dir_all(&cur_dir)?;
        }
        let name = cur_dir
            .file_name()
            .expect("current directory should have a file name")
            .to_string_lossy();
        let ast_doc = result.document().node();
        let version = ast_doc
            .version_statement()
            .expect("Document should have a version statement");
        let ast = ast_doc.ast().unwrap_v1();

        let rel_css_path = diff_paths(&abs_css_path, &cur_dir).unwrap();

        for item in ast.items() {
            match item {
                DocumentItem::Struct(s) => {
                    let struct_name = s.name().as_str().to_owned();
                    let struct_path = cur_dir.join(format!("{}-struct.html", struct_name));
                    docs_tree.add_path(
                        struct_path.clone(),
                        Some(HTMLPage::new(
                            struct_name.clone(),
                            struct_path.clone(),
                            Some(PageType::Struct),
                        )),
                    );
                    let mut struct_file = tokio::fs::File::create(&struct_path).await?;

                    let r#struct = r#struct::Struct::new(s.clone());
                    struct_file
                        .write_all(
                            r#struct
                                .render(&docs_tree, &rel_css_path)
                                .into_string()
                                .as_bytes(),
                        )
                        .await?;
                }
                DocumentItem::Task(t) => {
                    let task_name = t.name().as_str().to_owned();
                    let task_path = cur_dir.join(format!("{}-task.html", task_name));
                    let mut task_file = tokio::fs::File::create(&task_path).await?;

                    let parameter_meta: HashMap<String, MetadataValue> = t
                        .parameter_metadata()
                        .into_iter()
                        .flat_map(|p| p.items())
                        .map(|p| {
                            let name = p.name().as_str().to_owned();
                            let item = p.value();
                            (name, item)
                        })
                        .collect();

                    let meta: HashMap<String, MetadataValue> = t
                        .metadata()
                        .into_iter()
                        .flat_map(|m| m.items())
                        .map(|m| {
                            let name = m.name().as_str().to_owned();
                            let item = m.value();
                            (name, item)
                        })
                        .collect();

                    let output_meta: HashMap<String, MetadataValue> = meta
                        .get("outputs")
                        .cloned()
                        .into_iter()
                        .flat_map(|v| v.unwrap_object().items())
                        .map(|m| {
                            let name = m.name().as_str().to_owned();
                            let item = m.value();
                            (name, item)
                        })
                        .collect();

                    let inputs = t
                        .input()
                        .into_iter()
                        .flat_map(|i| i.declarations())
                        .map(|decl| {
                            let name = decl.name().as_str().to_owned();
                            let meta = parameter_meta.get(&name);
                            parameter::Parameter::new(
                                decl.clone(),
                                meta.cloned(),
                                InputOutput::Input,
                            )
                        })
                        .collect();

                    let outputs = t
                        .output()
                        .into_iter()
                        .flat_map(|o| o.declarations())
                        .map(|decl| {
                            let name = decl.name().as_str().to_owned();
                            let meta = output_meta.get(&name);
                            parameter::Parameter::new(
                                wdl_ast::v1::Decl::Bound(decl.clone()),
                                meta.cloned(),
                                InputOutput::Output,
                            )
                        })
                        .collect();

                    let task = task::Task::new(
                        task_name.clone(),
                        t.metadata(),
                        t.runtime(),
                        inputs,
                        outputs,
                    );

                    task_file
                        .write_all(
                            task.render(&docs_tree, &rel_css_path)
                                .into_string()
                                .as_bytes(),
                        )
                        .await?;

                    docs_tree.add_path(
                        task_path.clone(),
                        Some(HTMLPage::new(
                            task_name.clone(),
                            task_path.clone(),
                            Some(PageType::Task(task.description())),
                        )),
                    );
                }
                DocumentItem::Workflow(w) => {
                    let workflow_name = w.name().as_str().to_owned();
                    let workflow_path = cur_dir.join(format!("{}-workflow.html", workflow_name));
                    let mut workflow_file = tokio::fs::File::create(&workflow_path).await?;

                    let parameter_meta: HashMap<String, MetadataValue> = w
                        .parameter_metadata()
                        .into_iter()
                        .flat_map(|p| p.items())
                        .map(|p| {
                            let name = p.name().as_str().to_owned();
                            let item = p.value();
                            (name, item)
                        })
                        .collect();

                    let meta: HashMap<String, MetadataValue> = w
                        .metadata()
                        .into_iter()
                        .flat_map(|m| m.items())
                        .map(|m| {
                            let name = m.name().as_str().to_owned();
                            let item = m.value();
                            (name, item)
                        })
                        .collect();

                    let output_meta: HashMap<String, MetadataValue> = meta
                        .get("outputs")
                        .cloned()
                        .into_iter()
                        .flat_map(|v| v.unwrap_object().items())
                        .map(|m| {
                            let name = m.name().as_str().to_owned();
                            let item = m.value();
                            (name, item)
                        })
                        .collect();

                    let inputs = w
                        .input()
                        .into_iter()
                        .flat_map(|i| i.declarations())
                        .map(|decl| {
                            let name = decl.name().as_str().to_owned();
                            let meta = parameter_meta.get(&name);
                            parameter::Parameter::new(
                                decl.clone(),
                                meta.cloned(),
                                InputOutput::Input,
                            )
                        })
                        .collect();

                    let outputs = w
                        .output()
                        .into_iter()
                        .flat_map(|o| o.declarations())
                        .map(|decl| {
                            let name = decl.name().as_str().to_owned();
                            let meta = output_meta.get(&name);
                            parameter::Parameter::new(
                                wdl_ast::v1::Decl::Bound(decl.clone()),
                                meta.cloned(),
                                InputOutput::Output,
                            )
                        })
                        .collect();

                    let workflow = workflow::Workflow::new(
                        workflow_name.clone(),
                        w.metadata(),
                        inputs,
                        outputs,
                    );

                    workflow_file
                        .write_all(
                            workflow
                                .render(&docs_tree, &rel_css_path)
                                .into_string()
                                .as_bytes(),
                        )
                        .await?;

                    docs_tree.add_path(
                        workflow_path.clone(),
                        Some(HTMLPage::new(
                            workflow_name.clone(),
                            workflow_path.clone(),
                            Some(PageType::Workflow(workflow.description())),
                        )),
                    );
                }
                DocumentItem::Import(_) => {}
            }
        }
        let document = Document::new(name.to_string(), cur_dir.clone(), version);

        let index_path = cur_dir.join("index.html");
        docs_tree.add_path(
            index_path.clone(),
            Some(HTMLPage::new(name.to_string(), index_path.clone(), None)),
        );
        let mut index = tokio::fs::File::create(index_path.clone()).await?;

        index
            .write_all(
                document
                    .render(&docs_tree, &rel_css_path)
                    .into_string()
                    .as_bytes(),
            )
            .await?;
    }

    let homepage_path = docs_dir.join("index.html");
    let mut homepage = tokio::fs::File::create(homepage_path.clone()).await?;

    homepage
        .write_all(
            full_page(
                "Homepage",
                &docs_tree,
                &diff_paths(&abs_css_path, &docs_dir).unwrap(),
                html! {
                    h1 { "Homepage" }
                    (home_toc(&docs_tree))
                },
            )
            .into_string()
            .as_bytes(),
        )
        .await?;

    Ok(docs_dir)
}

/// Build a stylesheet for the documentation, given the path to the `themes`
/// directory.
pub fn build_stylesheet(themes_dir: &Path) -> Result<String> {
    let themes_dir = themes_dir.canonicalize()?;
    let output = std::process::Command::new("npx")
        .arg("@tailwindcss/cli")
        .arg("-i")
        .arg("src/main.css")
        .arg("-o")
        .arg("dist/style.css")
        .current_dir(&themes_dir)
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("Failed to build stylesheet"));
    }
    let css_path = themes_dir.join("dist/style.css");
    if !css_path.exists() {
        return Err(anyhow!("Failed to find stylesheet"));
    }

    Ok(std::fs::read_to_string(css_path)?)
}

#[cfg(test)]
mod tests {
    use wdl_ast::Document as AstDocument;

    use super::*;

    #[test]
    fn test_parse_preamble_comments() {
        let source = r#"
        ## This is a comment
        ## This is also a comment
        version 1.0
        workflow test {
            input {
                String name
            }
            output {
                String greeting = "Hello, ${name}!"
            }
            call say_hello as say_hello {
                input:
                    name = name
            }
        }
        "#;
        let (document, _) = AstDocument::parse(source);
        let preamble = parse_preamble_comments(document.version_statement().unwrap());
        assert_eq!(preamble, "This is a comment\nThis is also a comment");
    }
}

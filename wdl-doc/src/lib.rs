//! Library for generating HTML documentation from WDL files.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

pub(crate) mod meta;
pub(crate) mod parameter;
pub mod r#struct;
pub mod task;
pub mod workflow;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::Render;
use maud::html;
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

/// A full HTML page with a header and body.
pub(crate) fn full_page(page_title: &str, style_sheet: &Path, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html class="dark
                    size-full" {
            (header(page_title, style_sheet))
            body class="size-full
                        dark:bg-slate-950
                        dark:text-white" {
                (body)
           }
        }
    }
}

/// A basic header with a dynamic `page_title` and link to the stylesheet.
pub(crate) fn header(page_title: &str, style_sheet: &Path) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { (page_title) }
            (Css(style_sheet.to_str().unwrap()))
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,100..1000;1,9..40,100..1000&display=swap" rel="stylesheet";
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

/// A WDL document.
#[derive(Debug)]
pub struct Document {
    /// The name of the document.
    ///
    /// This is the filename of the document without the extension.
    name: String,
    /// The version of the document.
    version: VersionStatement,
}

impl Document {
    /// Create a new document.
    pub fn new(name: String, version: VersionStatement) -> Self {
        Self { name, version }
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
        let preamble = fetch_preamble_comments(self.version.clone());
        Markdown(&preamble).render()
    }

    /// Render the document as HTML.
    pub fn render(&self, parent_dir: &Path) -> Markup {
        let body = html! {
            h1 { (self.name()) }
            h3 { "WDL Version: " (self.version()) }
            div { (self.preamble()) }
        };

        full_page(self.name(), parent_dir, body)
    }
}

/// Fetch the preamble comments from a document.
fn fetch_preamble_comments(version: VersionStatement) -> String {
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

/// Generate HTML documentation for a workspace.
pub async fn document_workspace(path: PathBuf) -> Result<()> {
    if !path.is_dir() {
        return Err(anyhow!("Workspace is not a directory"));
    }

    let abs_path = std::path::absolute(&path)?;

    let docs_dir = abs_path.clone().join(DOCS_DIR);
    if !docs_dir.exists() {
        std::fs::create_dir(&docs_dir)?;
    }

    // Get the relative path to the CSS style sheet.
    // TODO: This is a hack. We should have a better way to do this.
    let css_path = PathBuf::from("/styles.css");

    let analyzer = Analyzer::new(DiagnosticsConfig::new(rules()), |_: (), _, _, _| async {});
    analyzer.add_directory(abs_path.clone()).await?;
    let results = analyzer.analyze(()).await?;

    for result in results {
        let cur_path = result
            .document()
            .uri()
            .to_file_path()
            .expect("URI should have a file path");
        let relative_path = match cur_path.strip_prefix(&abs_path) {
            Ok(path) => path,
            Err(_) => &PathBuf::from("external").join(cur_path.strip_prefix("/").unwrap()),
        };
        let cur_dir = docs_dir.join(relative_path.with_extension(""));
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

        let document = Document::new(name.to_string(), version);

        let index = cur_dir.join("index.html");
        let mut index = tokio::fs::File::create(index).await?;

        index
            .write_all(document.render(&css_path).into_string().as_bytes())
            .await?;

        for item in ast.items() {
            match item {
                DocumentItem::Struct(s) => {
                    let struct_name = s.name().as_str().to_owned();
                    let struct_file = cur_dir.join(format!("{}-struct.html", struct_name));
                    let mut struct_file = tokio::fs::File::create(struct_file).await?;

                    let r#struct = r#struct::Struct::new(s.clone());
                    struct_file
                        .write_all(r#struct.render(&css_path).into_string().as_bytes())
                        .await?;
                }
                DocumentItem::Task(t) => {
                    let task_name = t.name().as_str().to_owned();
                    let task_file = cur_dir.join(format!("{}-task.html", task_name));
                    let mut task_file = tokio::fs::File::create(task_file).await?;

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
                            parameter::Parameter::new(decl.clone(), meta.cloned())
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
                            )
                        })
                        .collect();

                    let task = task::Task::new(task_name, t.metadata(), inputs, outputs);

                    task_file
                        .write_all(task.render(&css_path).into_string().as_bytes())
                        .await?;
                }
                DocumentItem::Workflow(w) => {
                    let workflow_name = w.name().as_str().to_owned();
                    let workflow_file = cur_dir.join(format!("{}-workflow.html", workflow_name));
                    let mut workflow_file = tokio::fs::File::create(workflow_file).await?;

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
                            parameter::Parameter::new(decl.clone(), meta.cloned())
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
                            )
                        })
                        .collect();

                    let workflow =
                        workflow::Workflow::new(workflow_name, w.metadata(), inputs, outputs);

                    workflow_file
                        .write_all(workflow.render(&css_path).into_string().as_bytes())
                        .await?;
                }
                DocumentItem::Import(_) => {}
            }
        }
    }
    anyhow::Ok(())
}

#[cfg(test)]
mod tests {
    use wdl_ast::Document as AstDocument;

    use super::*;

    #[test]
    fn test_fetch_preamble_comments() {
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
        let preamble = fetch_preamble_comments(document.version_statement().unwrap());
        assert_eq!(preamble, "This is a comment\nThis is also a comment");
    }
}

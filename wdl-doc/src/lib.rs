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

use std::path::Path;
use std::path::PathBuf;
use std::path::absolute;
use std::rc::Rc;

use anyhow::Result;
use anyhow::anyhow;
pub use callable::Callable;
pub use callable::task;
pub use callable::workflow;
pub use docs_tree::DocsTree;
use docs_tree::HTMLPage;
use docs_tree::PageType;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::Render;
use maud::html;
use path_clean::clean;
use pathdiff::diff_paths;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use wdl_analysis::Analyzer;
use wdl_analysis::DiagnosticsConfig;
use wdl_analysis::rules;
use wdl_ast::AstToken;
use wdl_ast::SyntaxTokenExt;
use wdl_ast::VersionStatement;
use wdl_ast::v1::DocumentItem;

/// Write assets to the given root docs directory.
fn write_assets<P: AsRef<Path>>(dir: P) -> Result<()> {
    let dir = dir.as_ref();
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&assets_dir)?;
    std::fs::write(
        dir.join("style.css"),
        include_str!("../theme/dist/style.css"),
    )?;

    std::fs::write(
        assets_dir.join("sprocket-logo.svg"),
        include_bytes!("../theme/assets/sprocket-logo.svg"),
    )?;
    std::fs::write(
        assets_dir.join("search.svg"),
        include_bytes!("../theme/assets/search.svg"),
    )?;
    std::fs::write(
        assets_dir.join("x-mark.svg"),
        include_bytes!("../theme/assets/x-mark.svg"),
    )?;
    std::fs::write(
        assets_dir.join("chevron-down.svg"),
        include_bytes!("../theme/assets/chevron-down.svg"),
    )?;
    std::fs::write(
        assets_dir.join("category-selected.svg"),
        include_bytes!("../theme/assets/category-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("list-bullet-selected.svg"),
        include_bytes!("../theme/assets/list-bullet-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("list-bullet-unselected.svg"),
        include_bytes!("../theme/assets/list-bullet-unselected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("folder-selected.svg"),
        include_bytes!("../theme/assets/folder-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("folder-unselected.svg"),
        include_bytes!("../theme/assets/folder-unselected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("dir-selected.svg"),
        include_bytes!("../theme/assets/dir-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("dir-unselected.svg"),
        include_bytes!("../theme/assets/dir-unselected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("struct-selected.svg"),
        include_bytes!("../theme/assets/struct-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("struct-unselected.svg"),
        include_bytes!("../theme/assets/struct-unselected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("task-selected.svg"),
        include_bytes!("../theme/assets/task-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("task-unselected.svg"),
        include_bytes!("../theme/assets/task-unselected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("workflow-selected.svg"),
        include_bytes!("../theme/assets/workflow-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("workflow-unselected.svg"),
        include_bytes!("../theme/assets/workflow-unselected.svg"),
    )?;

    Ok(())
}

/// Links to a CSS stylesheet at the given path.
struct Css<'a>(&'a str);

impl Render for Css<'_> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" type="text/css" href=(self.0);
        }
    }
}

/// A basic header with a `page_title` and an optional link to the stylesheet.
pub(crate) fn header<P: AsRef<Path>>(page_title: &str, stylesheet: P) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { (page_title) }
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,100..1000;1,9..40,100..1000&display=swap" rel="stylesheet";
            script defer src="https://cdn.jsdelivr.net/npm/@alpinejs/persist@3.x.x/dist/cdn.min.js" {}
            script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js" {}
            (Css(stylesheet.as_ref().to_str().unwrap()))
        }
    }
}

/// A full HTML page.
pub(crate) fn full_page<P: AsRef<Path>>(page_title: &str, body: Markup, stylesheet: P) -> Markup {
    html! {
        (DOCTYPE)
        html class="dark" {
            (header(page_title, stylesheet))
            body class="size-full table-auto border-collapse text-base" {
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

        // Remove the outer `<p>` tag that `pulldown_cmark` wraps single lines in
        let safe_html = if safe_html.starts_with("<p>") && safe_html.ends_with("</p>\n") {
            let trimmed = &safe_html[3..safe_html.len() - 5];
            if trimmed.contains("<p>") {
                // If the trimmed string contains another `<p>` tag, it means
                // that the original string was more complicated than a single-line paragraph,
                // so we should keep the outer `<p>` tag.
                safe_html
            } else {
                trimmed.to_string()
            }
        } else {
            safe_html
        };
        PreEscaped(safe_html)
    }
}

/// Parse the preamble comments of a document using the version statement.
fn parse_preamble_comments(version: VersionStatement) -> String {
    let comments = version
        .keyword()
        .inner()
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
    /// The AST node for the version statement.
    ///
    /// This is used both to display the WDL version number and to fetch the
    /// preamble comments.
    version: VersionStatement,
    /// The pages that this document should link to.
    local_pages: Vec<(PathBuf, Rc<HTMLPage>)>,
}

impl Document {
    /// Create a new document.
    pub fn new(
        name: String,
        version: VersionStatement,
        local_pages: Vec<(PathBuf, Rc<HTMLPage>)>,
    ) -> Self {
        Self {
            name,
            version,
            local_pages,
        }
    }

    /// Get the name of the document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the document as text.
    pub fn version(&self) -> String {
        self.version.version().text().to_string()
    }

    /// Get the preamble comments of the document.
    pub fn preamble(&self) -> Markup {
        let preamble = parse_preamble_comments(self.version.clone());
        Markdown(&preamble).render()
    }

    /// Render the document as HTML.
    pub fn render(&self) -> Markup {
        html! {
            div class="flex flex-col gap-y-6" {
                h1 { (self.name()) }
                h3 { "WDL Version: " (self.version()) }
                div { (self.preamble()) }
                div class="flex flex-col gap-y-6" {
                    h2 { "Table of Contents" }
                    table class="border" {
                        thead class="border" { tr {
                            th class="" { "Page" }
                            th class="" { "Type" }
                            th class="" { "Description" }
                        }}
                        tbody class="border" {
                            @for page in &self.local_pages {
                                tr class="border" {
                                    td class="border" {
                                        a href=(page.0.to_str().unwrap()) { (page.1.name()) }
                                    }
                                    td class="border" {
                                        @match page.1.page_type() {
                                            PageType::Index(_) => { "TODO ERROR" }
                                            PageType::Struct(_) => { "Struct" }
                                            PageType::Task(_) => { "Task" }
                                            PageType::Workflow(_) => { "Workflow" }
                                        }
                                    }
                                    td class="border" {
                                        @match page.1.page_type() {
                                            PageType::Index(_) => { "TODO ERROR" }
                                            PageType::Struct(_) => { "N/A" }
                                            PageType::Task(t) => { (t.description()) }
                                            PageType::Workflow(w) => { (w.description()) }
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
}

/// Generate HTML documentation for a workspace.
///
/// This function will generate HTML documentation for all WDL files in the
/// workspace directory. The generated documentation will be stored in a
/// `docs` directory within the workspace.
pub async fn document_workspace(
    workspace: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    stylesheet: Option<impl AsRef<Path>>,
) -> Result<()> {
    let workspace_abs_path = clean(absolute(workspace.as_ref())?);
    let stylesheet = stylesheet.and_then(|p| absolute(p.as_ref()).ok());

    if !workspace_abs_path.is_dir() {
        return Err(anyhow!("Workspace is not a directory"));
    }

    let docs_dir = clean(absolute(output_dir.as_ref())?);
    if !docs_dir.exists() {
        std::fs::create_dir(&docs_dir)?;
    }

    let analyzer = Analyzer::new(DiagnosticsConfig::new(rules()), |_: (), _, _, _| async {});
    analyzer.add_directory(workspace_abs_path.clone()).await?;
    let results = analyzer.analyze(()).await?;

    let mut docs_tree = if let Some(ss) = stylesheet {
        docs_tree::DocsTree::new_with_stylesheet(&docs_dir, ss)?
    } else {
        docs_tree::DocsTree::new(&docs_dir)?
    };

    for result in results {
        let uri = result.document().uri();
        let rel_wdl_path = match uri.to_file_path() {
            Ok(path) => match path.strip_prefix(&workspace_abs_path) {
                Ok(path) => path.to_path_buf(),
                Err(_) => {
                    PathBuf::from("external").join(path.components().skip(1).collect::<PathBuf>())
                }
            },
            Err(_) => PathBuf::from("external").join(
                uri.path()
                    .strip_prefix("/")
                    .expect("URI path should start with /"),
            ),
        };
        let cur_dir = docs_dir.join(rel_wdl_path.with_extension(""));
        if !cur_dir.exists() {
            std::fs::create_dir_all(&cur_dir)?;
        }
        let ast_doc = result.document().root();
        let version = ast_doc
            .version_statement()
            .expect("document should have a version statement");
        let ast = ast_doc.ast().unwrap_v1();

        let mut local_pages = Vec::new();

        for item in ast.items() {
            match item {
                DocumentItem::Struct(s) => {
                    let name = s.name().text().to_owned();
                    let path = cur_dir.join(format!("{}-struct.html", name));

                    let r#struct = r#struct::Struct::new(s.clone());

                    let page = Rc::new(HTMLPage::new(name.clone(), PageType::Struct(r#struct)));
                    docs_tree.add_page(path.clone(), page.clone());
                    local_pages.push((diff_paths(path, &cur_dir).unwrap(), page));
                }
                DocumentItem::Task(t) => {
                    let name = t.name().text().to_owned();
                    let path = cur_dir.join(format!("{}-task.html", name));

                    let task = task::Task::new(
                        name.clone(),
                        t.metadata(),
                        t.parameter_metadata(),
                        t.input(),
                        t.output(),
                        t.runtime(),
                    );

                    let page = Rc::new(HTMLPage::new(name, PageType::Task(task)));
                    docs_tree.add_page(path.clone(), page.clone());
                    local_pages.push((diff_paths(path, &cur_dir).unwrap(), page));
                }
                DocumentItem::Workflow(w) => {
                    let name = w.name().text().to_owned();
                    let path = cur_dir.join(format!("{}-workflow.html", name));

                    let workflow = workflow::Workflow::new(
                        name.clone(),
                        w.metadata(),
                        w.parameter_metadata(),
                        w.input(),
                        w.output(),
                    );

                    let page = Rc::new(HTMLPage::new(name, PageType::Workflow(workflow)));
                    docs_tree.add_page(path.clone(), page.clone());
                    local_pages.push((diff_paths(path, &cur_dir).unwrap(), page));
                }
                DocumentItem::Import(_) => {}
            }
        }
        let name = rel_wdl_path.file_stem().unwrap().to_str().unwrap();
        let document = Document::new(name.to_string(), version, local_pages);

        let index_path = cur_dir.join("index.html");

        docs_tree.add_page(
            index_path,
            Rc::new(HTMLPage::new(name.to_string(), PageType::Index(document))),
        );
    }

    docs_tree.render_all()?;

    Ok(())
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

    #[test]
    fn test_markdown_render() {
        let source = r#"
        ## This is a paragraph.
        ##
        ## This is the start of a new paragraph.
        ## And this is the same paragraph continued.
        version 1.0
        workflow test {
            meta {
                description: "A simple description should not render with p tags"
            }
        }
        "#;
        let (document, _) = AstDocument::parse(source);
        let preamble = parse_preamble_comments(document.version_statement().unwrap());
        let markdown = Markdown(&preamble).render();
        assert_eq!(
            markdown.into_string(),
            "<p>This is a paragraph.</p>\n<p>This is the start of a new paragraph.\nAnd this is \
             the same paragraph continued.</p>\n"
        );

        let doc_item = document.ast().into_v1().unwrap().items().next().unwrap();
        let ast_workflow = doc_item.into_workflow_definition().unwrap();
        let workflow = workflow::Workflow::new(
            ast_workflow.name().text().to_string(),
            ast_workflow.metadata(),
            ast_workflow.parameter_metadata(),
            ast_workflow.input(),
            ast_workflow.output(),
        );

        let description = workflow.description();
        assert_eq!(
            description.into_string(),
            "A simple description should not render with p tags"
        );
    }
}

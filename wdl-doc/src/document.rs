//! Create HTML documentation (index pages) for WDL documents.

use std::path::PathBuf;
use std::rc::Rc;

use maud::Markup;
use maud::Render;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxTokenExt;
use wdl_ast::VersionStatement;

use crate::HTMLPage;
use crate::Markdown;
use crate::VersionBadge;
use crate::callable::Callable;
use crate::docs_tree::Header;
use crate::docs_tree::PageHeaders;
use crate::docs_tree::PageType;

/// Parse the preamble comments of a document using the version statement.
pub fn parse_preamble_comments(version: VersionStatement) -> String {
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

/// A WDL document. This is an index page that links to other HTML pages.
#[derive(Debug)]
pub(crate) struct Document {
    /// The name of the document.
    name: String,
    /// The version badge for the document.
    version: VersionBadge,
    /// The AST node for the version statement.
    ///
    /// This is used to fetch to the preamble comments.
    version_statement: VersionStatement,
    /// The pages that this document should link to.
    local_pages: Vec<(PathBuf, Rc<HTMLPage>)>,
}

impl Document {
    /// Create a new document.
    pub(crate) fn new(
        name: String,
        version: SupportedVersion,
        version_statement: VersionStatement,
        local_pages: Vec<(PathBuf, Rc<HTMLPage>)>,
    ) -> Self {
        Self {
            name,
            version: VersionBadge::new(version),
            version_statement,
            local_pages,
        }
    }

    /// Get the name of the document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the document as text.
    pub fn version(&self) -> &VersionBadge {
        &self.version
    }

    /// Get the preamble comments of the document.
    pub fn preamble(&self) -> Markup {
        let preamble = parse_preamble_comments(self.version_statement.clone());
        Markdown(&preamble).render()
    }

    /// Render the document as HTML.
    pub fn render(&self) -> (Markup, PageHeaders) {
        let markup = html! {
            div class="main__container" {
                h1 id="title" class="main__title" { (self.name()) }
                div class="main__badge-container" {
                    (self.version().render())
                }
                div id="preamble" class="main__section" {
                    div class="markdown-body" {
                        (self.preamble())
                    }
                }
                div class="main__section" {
                    h2 id="toc" class="main__section-header" { "Table of Contents" }
                    div class="main__table-outer-container" {
                        div class="main__table-inner-container" {
                            table class="main__table" {
                                thead { tr {
                                    th { "Page" }
                                    th { "Type" }
                                    th { "Description" }
                                }}
                                tbody {
                                    @for page in &self.local_pages {
                                        tr {
                                            @match page.1.page_type() {
                                                PageType::Struct(_) => {
                                                    td {
                                                        a class="text-pink-400 hover:text-pink-300 hover:underline main__toc-link" href=(page.0.to_string_lossy()) {
                                                            (page.1.name())
                                                        }
                                                    }
                                                    td { code { "struct" } }
                                                    td { "N/A" }
                                                }
                                                PageType::Task(t) => {
                                                    td {
                                                        a class="text-violet-400 hover:text-violet-300 hover:underline main__toc-link" href=(page.0.to_string_lossy()) {
                                                            (page.1.name())
                                                        }
                                                    }
                                                    td { code { "task" } }
                                                    td { (t.description(true)) }
                                                }
                                                PageType::Workflow(w) => {
                                                    td {
                                                        a class="text-emerald-400 hover:text-emerald-300 hover:underline main__toc-link" href=(page.0.to_string_lossy()) {
                                                            (page.1.name())
                                                        }
                                                    }
                                                    td { code { "workflow" } }
                                                    td { (w.description(true)) }
                                                }
                                                // Index pages should not link to other index pages.
                                                PageType::Index(_) => {
                                                    // This should never happen
                                                    td { "ERROR" }
                                                    td { "ERROR" }
                                                    td { "ERROR" }
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
        };

        let mut headers = PageHeaders::default();
        headers.push(Header::Header(
            "Preamble".to_string(),
            "preamble".to_string(),
        ));
        headers.push(Header::Header(
            "Table of Contents".to_string(),
            "toc".to_string(),
        ));

        (markup, headers)
    }
}

//! Library for generating HTML documentation from WDL files.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]
#![recursion_limit = "512"]

pub mod parameter;
pub mod r#struct;

use std::fmt::Display;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use html::content;
use html::text_content;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use tokio::io::AsyncWriteExt;
use wdl_analysis::Analyzer;
use wdl_analysis::rules;
use wdl_ast::AstToken;
use wdl_ast::Document as AstDocument;
use wdl_ast::SyntaxTokenExt;
use wdl_ast::Version;

/// A WDL document.
#[derive(Debug)]
pub struct Document {
    /// The name of the document.
    ///
    /// This is the filename of the document without the extension.
    name: String,
    /// The version of the document.
    version: Version,
    /// The Markdown preamble comments.
    preamble: String,
    /// The structs in the document.
    structs: Vec<r#struct::Struct>,
}

impl Document {
    /// Create a new document.
    pub fn new(
        name: String,
        version: Version,
        preamble: String,
        structs: Vec<r#struct::Struct>,
    ) -> Self {
        Self {
            name,
            version,
            preamble,
            structs,
        }
    }

    /// Get the name of the document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the document.
    pub fn version(&self) -> String {
        self.version.as_str().to_owned()
    }

    /// Get the preamble comments of the document.
    pub fn preamble(&self) -> &str {
        &self.preamble
    }

    /// Get the structs in the document.
    pub fn structs(&self) -> &[r#struct::Struct] {
        &self.structs
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let document_name = content::Heading1::builder()
            .text(self.name().to_owned())
            .build();
        let version = text_content::Paragraph::builder()
            .text(format!("WDL Version: {}", self.version()))
            .build();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&self.preamble, options);
        let mut preamble = String::new();
        pulldown_cmark::html::push_html(&mut preamble, parser);

        write!(f, "{}", document_name)?;
        write!(f, "{}", version)?;
        write!(f, "{}", preamble)
    }
}

/// Fetch the preamble comments from a document.
pub fn fetch_preamble_comments(document: AstDocument) -> String {
    let comments = match document.version_statement() {
        Some(version) => {
            let comments = version
                .keyword()
                .syntax()
                .preceding_trivia()
                .map(|t| match t.kind() {
                    wdl_ast::SyntaxKind::Comment => {
                        match t.to_string().strip_prefix('#').unwrap().strip_prefix("# ") {
                            Some(comment) => comment.to_string(),
                            None => "".to_string(),
                        }
                    }
                    wdl_ast::SyntaxKind::Whitespace => "".to_string(),
                    _ => {
                        panic!("Unexpected token kind: {:?}", t.kind())
                    }
                })
                .collect::<Vec<_>>();
            comments
        }
        None => {
            vec![]
        }
    }
    .join("\n");
    comments
}

/// Generate HTML documentation for a workspace.
pub async fn document_workspace(path: PathBuf) -> Result<()> {
    if !path.exists() || !path.is_dir() {
        return Err(anyhow!("The path is not a directory"));
    }

    let abs_path = std::fs::canonicalize(&path)?;

    let docs_dir = abs_path.clone().join("docs");
    if !docs_dir.exists() {
        std::fs::create_dir(&docs_dir)?;
    }

    let analyzer = Analyzer::new(rules(), |_: (), _, _, _| async {});
    analyzer.add_directory(abs_path.clone()).await?;
    let results = analyzer.analyze(()).await?;

    for result in results {
        let cur_path = PathBuf::from(result.uri().path());
        let cur_dir = docs_dir.join(cur_path.strip_prefix(&abs_path).unwrap().with_extension(""));
        if !cur_dir.exists() {
            std::fs::create_dir_all(&cur_dir)?;
        }
        let name = cur_dir.file_name().unwrap().to_str().unwrap();
        let ast_doc = result.parse_result().document().unwrap();
        let version = ast_doc.version_statement().unwrap().version();
        let preamble = fetch_preamble_comments(ast_doc.clone());
        let ast = ast_doc.ast().unwrap_v1();
        let structs = ast.structs().map(r#struct::Struct::new).collect::<Vec<_>>();

        let document = Document::new(name.to_owned(), version, preamble, structs);

        let index = cur_dir.join("index.html");
        let mut index = tokio::fs::File::create(index).await?;

        index.write_all(document.to_string().as_bytes()).await?;

        for r#struct in document.structs() {
            let struct_name = r#struct.name();
            let struct_file = cur_dir.join(format!("{}.html", struct_name));
            let mut struct_file = tokio::fs::File::create(struct_file).await?;

            struct_file
                .write_all(r#struct.to_string().as_bytes())
                .await?;
        }
    }
    anyhow::Ok(())
}

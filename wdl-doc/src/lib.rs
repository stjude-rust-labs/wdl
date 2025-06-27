//! Library for generating HTML documentation from WDL files.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

mod command_section;
mod docs_tree;
mod document;
mod meta;
mod parameter;
mod runnable;
mod r#struct;

use std::path::Path;
use std::path::PathBuf;
use std::path::absolute;
use std::rc::Rc;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
pub use command_section::CommandSectionExt;
pub use docs_tree::DocsTree;
pub use docs_tree::DocsTreeBuilder;
use docs_tree::HTMLPage;
use docs_tree::PageType;
use document::Document;
pub use document::parse_preamble_comments;
use maud::DOCTYPE;
use maud::Markup;
use maud::PreEscaped;
use maud::Render;
use maud::html;
use path_clean::clean;
use pathdiff::diff_paths;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use runnable::task;
use runnable::workflow;
use wdl_analysis::Analyzer;
use wdl_analysis::DiagnosticsConfig;
use wdl_analysis::rules;
use wdl_ast::AstToken;
use wdl_ast::SupportedVersion;
use wdl_ast::v1::DocumentItem;
use wdl_ast::version::V1;

/// Install the theme dependencies using npm.
pub fn install_theme(theme_dir: &Path) -> Result<()> {
    let theme_dir = absolute(theme_dir)?;
    if !theme_dir.exists() {
        bail!("Theme directory does not exist: {}", theme_dir.display());
    }
    let output = std::process::Command::new("npm")
        .arg("install")
        .current_dir(&theme_dir)
        .output()
        .with_context(|| {
            format!(
                "Failed to run npm install in theme directory: {}",
                theme_dir.display()
            )
        })?;
    if !output.status.success() {
        bail!(
            "failed to install theme dependencies: {stderr}",
            stderr = String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// Build the web components for the theme.
pub fn build_web_components(theme_dir: &Path) -> Result<()> {
    let theme_dir = absolute(theme_dir)?;
    let output = std::process::Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&theme_dir)
        .output()
        .with_context(|| {
            format!(
                "Failed to run npm build in theme directory: {}",
                theme_dir.display()
            )
        })?;
    if !output.status.success() {
        bail!(
            "failed to build web components: {stderr}",
            stderr = String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// Build a stylesheet for the documentation, using Tailwind CSS.
pub fn build_stylesheet(theme_dir: &Path) -> Result<()> {
    let theme_dir = absolute(theme_dir)?;
    let output = std::process::Command::new("npx")
        .arg("@tailwindcss/cli")
        .arg("-i")
        .arg("src/main.css")
        .arg("-o")
        .arg("dist/style.css")
        .current_dir(&theme_dir)
        .output()?;
    if !output.status.success() {
        bail!(
            "failed to build stylesheet: {stderr}",
            stderr = String::from_utf8_lossy(&output.stderr)
        );
    }
    let css_path = theme_dir.join("dist/style.css");
    if !css_path.exists() {
        bail!("failed to build stylesheet: no output file found");
    }

    Ok(())
}

/// Write assets to the given root docs directory.
///
/// This will create an `assets` directory in the given path and write all
/// necessary assets to it. It will also write the default `style.css` and
/// `index.js` files to the root of the directory unless a custom theme is
/// provided, in which case it will copy the `style.css` and `index.js` files
/// from the custom theme's `dist` directory.
fn write_assets<P: AsRef<Path>>(dir: P, custom_theme: Option<P>) -> Result<()> {
    let dir = dir.as_ref();
    let custom_theme = custom_theme.as_ref().map(|p| p.as_ref());
    let assets_dir = dir.join("assets");
    std::fs::create_dir_all(&assets_dir).with_context(|| {
        format!(
            "Failed to create assets directory: {}",
            assets_dir.display()
        )
    })?;

    if let Some(custom_theme) = custom_theme {
        let custom_theme = absolute(custom_theme).with_context(|| {
            format!(
                "Failed to resolve absolute path for custom theme: {}",
                custom_theme.display()
            )
        })?;
        if !custom_theme.exists() {
            bail!(
                "Custom theme directory does not exist: {}",
                custom_theme.display()
            );
        }
        std::fs::copy(
            custom_theme.join("dist").join("style.css"),
            dir.join("style.css"),
        )
        .with_context(|| {
            format!(
                "Failed to copy custom theme style.css to {}",
                dir.join("style.css").display()
            )
        })?;
        std::fs::copy(
            custom_theme.join("dist").join("index.js"),
            dir.join("index.js"),
        )
        .with_context(|| {
            format!(
                "Failed to copy custom theme index.js to {}",
                dir.join("index.js").display()
            )
        })?;
    } else {
        std::fs::write(
            dir.join("style.css"),
            include_str!("../theme/dist/style.css"),
        )
        .with_context(|| {
            format!(
                "Failed to write default style.css to {}",
                dir.join("style.css").display()
            )
        })?;
        std::fs::write(dir.join("index.js"), include_str!("../theme/dist/index.js")).with_context(
            || {
                format!(
                    "Failed to write default index.js to {}",
                    dir.join("index.js").display()
                )
            },
        )?;
    }

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
        assets_dir.join("chevron-up.svg"),
        include_bytes!("../theme/assets/chevron-up.svg"),
    )?;
    std::fs::write(
        assets_dir.join("dir-open.svg"),
        include_bytes!("../theme/assets/dir-open.svg"),
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
        assets_dir.join("wdl-dir-selected.svg"),
        include_bytes!("../theme/assets/wdl-dir-selected.svg"),
    )?;
    std::fs::write(
        assets_dir.join("wdl-dir-unselected.svg"),
        include_bytes!("../theme/assets/wdl-dir-unselected.svg"),
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
    std::fs::write(
        assets_dir.join("missing-home.svg"),
        include_bytes!("../theme/assets/missing-home.svg"),
    )?;
    std::fs::write(
        assets_dir.join("link.svg"),
        include_bytes!("../theme/assets/link.svg"),
    )?;
    std::fs::write(
        assets_dir.join("information-circle.svg"),
        include_bytes!("../theme/assets/information-circle.svg"),
    )?;

    Ok(())
}

/// HTML link to a CSS stylesheet at the given path.
struct Css<'a>(&'a str);

impl Render for Css<'_> {
    fn render(&self) -> Markup {
        html! {
            link rel="stylesheet" type="text/css" href=(self.0);
        }
    }
}

/// An HTML header with a `page_title` and all the link/script dependencies
/// expected by `wdl-doc`.
///
/// Requires a relative path to the root where `style.css` and `index.js` files
/// are expected.
pub(crate) fn header<P: AsRef<Path>>(page_title: &str, root: P) -> Markup {
    let root = root.as_ref();
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
            script defer src=(root.join("index.js").to_string_lossy()) {}
            (Css(&root.join("style.css").to_string_lossy()))
        }
    }
}

/// Returns a full HTML page, including the `DOCTYPE`, `html`, `head`, and
/// `body` tags,
pub(crate) fn full_page<P: AsRef<Path>>(page_title: &str, body: Markup, root: P) -> Markup {
    html! {
        (DOCTYPE)
        html class="dark" {
            (header(page_title, root))
            body class="body--base" {
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
        options.insert(Options::ENABLE_GFM);
        options.insert(Options::ENABLE_DEFINITION_LIST);
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

/// A version badge for a WDL document. This is used to display the WDL
/// version at the top of each documentation page.
#[derive(Debug, Clone)]
pub(crate) struct VersionBadge {
    /// The WDL version of the document.
    version: SupportedVersion,
}

impl VersionBadge {
    /// Create a new version badge.
    fn new(version: SupportedVersion) -> Self {
        Self { version }
    }

    /// Render the version badge as HTML.
    fn render(&self) -> Markup {
        let latest = match &self.version {
            SupportedVersion::V1(v) => matches!(v, V1::Two),
            _ => unreachable!("Only V1 is supported"),
        };
        let text = self.version.to_string();
        html! {
            div class="main__badge" {
                span class="main__badge-text" {
                    "WDL Version"
                }
                div class="main__badge-inner" {
                    span class="main__badge-inner-text" {
                        (text)
                    }
                }
                @if latest {
                    div class="main__badge-inner main__badge-inner-latest" {
                        span class="main__badge-inner-text" {
                            "Latest"
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
/// workspace directory. This function will overwrite any existing files which
/// conflict with the generated files, but will not delete any files that
/// are already present.
pub async fn document_workspace(
    workspace: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    homepage: Option<impl AsRef<Path>>,
    custom_theme: Option<impl AsRef<Path>>,
) -> Result<()> {
    let workspace_abs_path = clean(absolute(workspace.as_ref()).with_context(|| {
        format!(
            "Failed to resolve absolute path for workspace: {}",
            workspace.as_ref().display()
        )
    })?);
    let homepage = homepage.and_then(|p| absolute(p.as_ref()).ok());
    let custom_theme = custom_theme.and_then(|p| absolute(p.as_ref()).ok());

    if !workspace_abs_path.is_dir() {
        return Err(anyhow!("Workspace is not a directory"));
    }

    let docs_dir = clean(absolute(output_dir.as_ref()).with_context(|| {
        format!(
            "Failed to resolve absolute path for output directory: {}",
            output_dir.as_ref().display()
        )
    })?);
    if !docs_dir.exists() {
        std::fs::create_dir(&docs_dir).with_context(|| {
            format!("Failed to create output directory: {}", docs_dir.display())
        })?;
    }

    let analyzer = Analyzer::new(DiagnosticsConfig::new(rules()), |_: (), _, _, _| async {});
    analyzer
        .add_directory(workspace_abs_path.clone())
        .await
        .with_context(|| {
            format!(
                "Failed to add workspace directory to analyzer: {}",
                workspace_abs_path.display()
            )
        })?;
    let results = analyzer.analyze(()).await.with_context(|| {
        format!(
            "Failed to analyze workspace directory: {}",
            workspace_abs_path.display()
        )
    })?;

    let mut docs_tree = DocsTreeBuilder::new(docs_dir.clone())
        .maybe_homepage(homepage)
        .maybe_custom_theme(custom_theme)
        .build()
        .with_context(|| {
            format!(
                "Failed to build documentation tree for output directory: {}",
                docs_dir.display()
            )
        })?;

    for result in results {
        let uri = result.document().uri();
        // TODO: revisit these error paths
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
        let version = result
            .document()
            .version()
            .expect("document should have a supported version");
        let ast = result.document().root();
        let version_statement = ast
            .version_statement()
            .expect("document should have a version statement");
        let ast = ast.ast().unwrap_v1();

        let mut local_pages = Vec::new();

        for item in ast.items() {
            match item {
                DocumentItem::Struct(s) => {
                    let name = s.name().text().to_owned();
                    let path = cur_dir.join(format!("{name}-struct.html"));

                    // TODO: handle >=v1.2 structs
                    let r#struct = r#struct::Struct::new(s.clone(), version);

                    let page = Rc::new(HTMLPage::new(name.clone(), PageType::Struct(r#struct)));
                    docs_tree.add_page(path.clone(), page.clone());
                    local_pages
                        .push((diff_paths(path, &cur_dir).expect("should diff paths"), page));
                }
                DocumentItem::Task(t) => {
                    let name = t.name().text().to_owned();
                    let path = cur_dir.join(format!("{name}-task.html"));

                    let task = task::Task::new(
                        name.clone(),
                        version,
                        t,
                        if rel_wdl_path.starts_with("external") {
                            None
                        } else {
                            Some(rel_wdl_path.clone())
                        },
                    );

                    let page = Rc::new(HTMLPage::new(name, PageType::Task(task)));
                    docs_tree.add_page(path.clone(), page.clone());
                    local_pages
                        .push((diff_paths(path, &cur_dir).expect("should diff paths"), page));
                }
                DocumentItem::Workflow(w) => {
                    let name = w.name().text().to_owned();
                    let path = cur_dir.join(format!("{name}-workflow.html"));

                    let workflow = workflow::Workflow::new(
                        name.clone(),
                        version,
                        w,
                        if rel_wdl_path.starts_with("external") {
                            None
                        } else {
                            Some(rel_wdl_path.clone())
                        },
                    );

                    let page = Rc::new(HTMLPage::new(
                        workflow.name_override().unwrap_or(name),
                        PageType::Workflow(workflow),
                    ));
                    docs_tree.add_page(path.clone(), page.clone());
                    local_pages
                        .push((diff_paths(path, &cur_dir).expect("should diff paths"), page));
                }
                DocumentItem::Import(_) => {}
            }
        }
        let name = rel_wdl_path
            .file_stem()
            .expect("WDL file should have stem")
            .to_string_lossy();
        let document = Document::new(name.to_string(), version, version_statement, local_pages);

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
    use crate::runnable::Runnable;

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
        let preamble = parse_preamble_comments(&document.version_statement().unwrap());
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
        let preamble = parse_preamble_comments(&document.version_statement().unwrap());
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
            SupportedVersion::V1(V1::Zero),
            ast_workflow,
            None,
        );

        let description = workflow.render_description(false);
        assert_eq!(
            description.into_string(),
            "A simple description should not render with p tags"
        );
    }
}

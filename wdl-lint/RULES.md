# Rules

This table documents all implemented `wdl` lint rules implemented on the `main`
branch of the `stjude-rust-labs/wdl` repository. Note that the information may
be out of sync with released packages.

## Lint Rules

| Name                        | Tags                          | Description                                                                                         |
| :-------------------------- | :---------------------------- | :-------------------------------------------------------------------------------------------------- |
| `ElementSpacing`            | Spacing                       | Ensures proper blank space between elements                                                         |
| `CallInputSpacing`          | Clarity, Spacing, Style       | Ensures proper spacing for call inputs                                                              |
| `CommandSectionIndentation` | Clarity, Correctness, Spacing | Ensures consistent indentation (no mixed spaces/tabs) within command sections.                      |
| `CommentWhitespace`         | Spacing                       | Ensures that comments are properly spaced.                                                          |
| `ContainerUri`              | Clarity, Portability          | Ensures that the value for the `container` key in `runtime`/`requirements` sections is well-formed. |
| `DeprecatedObject`          | Deprecated                    | Ensures that the deprecated `Object` construct is not used.                                         |
| `DeprecatedPlaceholder`     | Deprecated                    | Ensures that the deprecated placeholder options construct is not used.                              |
| `MetaDescription`           | Completeness                  | Ensures the `meta` section contains a `description` key.                                            |
| `DeclarationName`           | Naming                        | Ensures declaration names do not redundantly include their type name.                               |
| `InputName`                 | Naming                        | Ensures input names are meaningful (e.g., not generic like 'input', 'in', or too short).            |
| `OutputName`                | Naming                        | Ensures output names are meaningful (e.g., not generic like 'output', 'out', or too short).         |
| `DoubleQuotes`              | Clarity, Style                | Ensures that strings are defined using double quotes.                                               |
| `EndingNewline`             | Spacing, Style                | Ensures that documents end with a single newline character.                                         |
| `ExpressionSpacing`         | Spacing                       | Ensures that expressions are properly spaced.                                                       |
| `ImportPlacement`           | Clarity, Sorting              | Ensures that imports are placed between the version statement and any document items.               |
| `ImportSorted`              | Clarity, Style                | Ensures that imports are sorted lexicographically.                                                  |
| `ImportWhitespace`          | Clarity, Spacing, Style       | Ensures that there is no extraneous whitespace between or within imports.                           |
| `ConsistentNewlines`        | Clarity, Style                | Ensures that `\n` or `\r\n` newlines are used consistently within the file.                         |
| `InputSorted`               | Clarity, Sorting, Style       | Ensures that input declarations are sorted                                                          |
| `MetaKeyValueFormatting`    | Style                         | Ensures that metadata objects and arrays are properly spaced.                                       |
| `LineWidth`                 | Clarity, Spacing, Style       | Ensures that lines do not exceed a certain width.                                                   |
| `LintDirectiveFormatted`    | Clarity, Correctness          | Ensures lint directives are correctly formatted.                                                    |
| `ParameterMetaMatched`      | Completeness, Sorting         | Ensures that inputs have a matching entry in a `parameter_meta` section.                            |
| `LintDirectiveValid`        | Clarity, Correctness          | Ensures lint directives are placed correctly to have the intended effect.                           |
| `MetaSections`              | Clarity, Completeness         | Ensures that tasks and workflows have the required `meta` and `parameter_meta` sections.            |
| `OutputSection`             | Completeness, Portability     | Ensures that tasks and workflows have an `output` section.                                          |
| `RequirementsSection`       | Completeness, Portability     | Ensures that >=v1.2 tasks have a requirements section.                                              |
| `RuntimeSection`            | Completeness, Portability     | Ensures that <v1.2 tasks have a runtime section.                                                    |
| `HereDocCommands`           | Clarity                       | Ensures that tasks use heredoc syntax in command sections.                                          |
| `MatchingOutputMeta`        | Completeness                  | Ensures that each output field is documented in the meta section under `meta.outputs`.              |
| `PascalCase`                | Clarity, Naming, Style        | Ensures that structs are defined with PascalCase names.                                             |
| `PreambleCommentPlacement`  | Clarity                       | Ensures that documents have correct comments in the preamble.                                       |
| `PreambleFormatted`         | Clarity, Spacing, Style       | Ensures that documents have correct whitespace in the preamble.                                     |
| `ConciseInput`              | Style                         | Ensures concise input assignments are used (implicit binding when available).                       |
| `ExpectedRuntimeKeys`       | Completeness, Deprecated      | Ensures that runtime sections have the appropriate keys.                                            |
| `SectionOrdering`           | Sorting, Style                | Ensures that sections within tasks and workflows are sorted.                                        |
| `ShellCheck`                | Correctness, Portability      | (BETA) Ensures that command sections are free of shellcheck diagnostics.                            |
| `SnakeCase`                 | Clarity, Naming, Style        | Ensures that tasks, workflows, and variables are defined with snake_case names.                     |
| `TodoComment`               | Completeness                  | Ensures that `TODO` statements are flagged for followup.                                            |
| `TrailingComma`             | Style                         | Ensures that lists and objects in meta have a trailing comma.                                       |
| `KnownRules`                | Clarity                       | Ensures only known rules are used in lint directives.                                               |
| `VersionStatementFormatted` | Style                         | Ensures the `version` statement is correctly formatted.                                             |
| `Whitespace`                | Spacing, Style                | Ensures that a document does not contain undesired whitespace.                                      |

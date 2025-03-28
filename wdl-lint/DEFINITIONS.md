# Definitions

This document defines key concepts and terminology used throughout the WDL linting rules.

## WDL Document Structure

### Preamble

The document preamble is defined as anything before the version declaration statement and the version declaration statement itself. Only comments and whitespace are permitted before the version declaration.

## Comment Types

### Lint Directives

Lint directives are special comments that begin with `#@ except:` followed by a comma-delimited list of rule IDs. These comments are used to disable specific lint rules for a section of the document. When a lint directive is encountered in the preamble, it will disable the specified rules for the entire document. For example:

```wdl
#@ except: LineWidth, CommentWhitespace
```

Lint directives are expected to be full line comments (i.e., they should not have any whitespace before the comment). If lint directives are present, they should be at the absolute beginning of the document. Multiple lint directives are permitted, but they should not be interleaved with preamble comments or blank lines.

### Preamble Comments

Preamble comments are special comments that begin with double pound signs (`##`). These comments are used for documentation that doesn't fit within any of the WDL-defined documentation elements (i.e., `meta` and `parameter_meta` sections). They may provide context for a collection of tasks or structs, or they may provide a high-level overview of the workflow. For example:

```wdl
## This workflow performs RNA-seq analysis
## It aligns reads and quantifies gene expression
```

A space should follow the double pound sign if there is any text within the preamble comment. "Empty" preamble comments are permitted and should not have any whitespace following the `##`. Comments beginning with 3 or more pound signs before the version declaration are not permitted.

All preamble comments should be in a single block without blank lines. Following this block, there should always be a blank line before the version statement.

### Regular Comments

Regular comments begin with a single pound sign (`#`) and can appear anywhere in the document except in the preamble. These are used for inline documentation and explanations.

## Metadata Sections

### Meta Section

The `meta` section provides metadata about a workflow, task, or other WDL element. This section can include a description of what the element does, author information, and other relevant details.

### Parameter Meta Section

The `parameter_meta` section provides metadata specifically about input parameters. It should include descriptions of each input parameter, including its purpose, expected format, and any constraints.

## Runtime Requirements

### Runtime Section

The `runtime` section specifies the computational resources and environment needed to execute a task. This might include memory requirements, CPU allocation, Docker container information, and other execution environment details.

### Requirements Section

In WDL version 1.2 and later, the `requirements` section is used to specify additional requirements for task execution that are not directly related to computational resources, such as network access or specialized hardware needs.

## Naming Conventions

### Snake Case

Snake case is a naming convention where words are written in lowercase with underscores between them (e.g., `my_variable_name`). This is the recommended convention for tasks, workflows, and variables in WDL.

### Pascal Case

Pascal case is a naming convention where words are written without spaces and each word starts with an uppercase letter (e.g., `MyStructName`). This is the recommended convention for struct definitions in WDL. 
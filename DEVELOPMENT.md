# FAQs

## How do I set up Rust?

[The official Rust docs guide](https://www.rust-lang.org/tools/install).

## What IDE should I use?

Most of this team uses VScode with the `rust-analyzer` extension but that preference is not hardcoded anywhere. Feel free to use any IDE you want!

## What's a good first issue?

We will try to keep a handful of [issues](https://github.com/stjude-rust-labs/wdl/issues) marked `good first issue` open and ready for new contributors.

## I don't want to write code, can I still contribute?

Sure!

You can always open a [discussion](https://github.com/stjude-rust-labs/wdl/discussions/categories/rule-proposals) with a propsal for a new lint rule. Or contribute to any open discussions.

We also appreciate feedback on our documentation. Feel free to look over any of our `*.md` files and note any issues you find. You can also explore our lint rule documentation by [installing `sprocket`](https://stjude-rust-labs.github.io/sprocket/installation.html) and reading the output of `sprocket explain`. (n.b.: we hope to replace `sprocket explain` with a website where each rule will have a dedicated page, but that has not been realized yet)

## What's the difference between `error`, `warning`, and `note`?

- an `error` is emitted when the source WDL is incorrect or invalid in some way
- a `warning` is emitted when the source WDL is confusing, problematic, error-prone, etc. but not invalid or incorrect
- a `note` is emitted in all other cases and is mostly used for issues of style or conformity

## What is gauntlet?

[Gauntlet](https://github.com/stjude-rust-labs/wdl/tree/main/gauntlet) is the main driver of our CI. Take a look at the file [`Gauntlet.toml`](https://github.com/stjude-rust-labs/wdl/blob/main/Gauntlet.toml). The entries at the top are all GitHub repositories of WDL code. The remaining entries are diagnostics emitted while analyzing those repositories. These should remain relatively static between PRs, and any change in emitted diagnostics should be reviewed carefully.

In order to turn the Gauntlet CI green, run `cargo run --release --bin gauntlet -- --refresh`. The `--refresh` flag will save any changes to the `Gauntlet.toml` file. This should then be committed and included in your PR.

## What is arena?

Arena is the alternate run mode of `gauntlet`. [`Arena.toml`](https://github.com/stjude-rust-labs/wdl/blob/main/Arena.toml) is very similar to `Gauntlet.toml`, except it has fewer repository entries and instead of analysis diagnostics it contains only lint diagnostics (which are not included in `Gauntlet.toml`).

In order to turn the Arena CI green, run `cargo run --release --bin gauntlet -- --arena --refresh`. The `--refresh` flag (in conjunction with the `--arena` flag) will save any changes to the `Arena.toml` file. This should then be committed and included in your PR.

## The CI has turned red. How do I make it green again?

There are a handful of reasons the CI may have turned red. Try the following fixes:

- `cargo +nightly fmt` to format your Rust code
- `cargo clippy --all-features` and then fix any warnings emitted
- `BLESS=1 cargo test --all-features` to "bless" any test changes
    - Please review any changes this causes to make sure they seem right!
- `cargo run --release --bin gauntlet -- --refresh`
    - see the `What is gauntlet?` question for more information
- `cargo run --release --bin gauntlet -- --refresh --arena`
    - see the `What is arena?` question for more information
- `rustup update` to update your local toolchains

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

* Added analysis support for the WDL 1.2 `env` declaration modifier ([#296](https://github.com/stjude-rust-labs/wdl/pull/296)).
* Fixed missing diagnostic for unknown local name when using the abbreviated
  syntax for specifying a call input ([#292](https://github.com/stjude-rust-labs/wdl/pull/292))
* Added functions for getting type information of task requirements and hints ([#241](https://github.com/stjude-rust-labs/wdl/pull/241)).
* Exposed information about workflow calls from an analyzed document ([#239](https://github.com/stjude-rust-labs/wdl/pull/239)).
* Added formatting to the analyzer ([#247](https://github.com/stjude-rust-labs/wdl/pull/247)).

### Changed

* Entry nodes in a workflow evaluation graph now contain information about the
  corresponding exit node. ([#292](https://github.com/stjude-rust-labs/wdl/pull/292))
* Removed `Types` collection from `wdl-analysis` to simplify the API ([#277](https://github.com/stjude-rust-labs/wdl/pull/277)).
* Changed the `new` and `new_with_validator` methods of `Analyzer` to take the
  diagnostics configuration rather than a rule iterator ([#274](https://github.com/stjude-rust-labs/wdl/pull/274)).
* Refactored the `AnalysisResult` and `Document` types to move properties of
  the former into the latter; this will assist in evaluation of documents in
  that the `Document` alone can be passed into evaluation ([#265](https://github.com/stjude-rust-labs/wdl/pull/265)).
* Removed the "optional type" constraint for the `select_first`, `select_all`,
  and `defined` functions; instead, these functions now accepted non-optional
  types and analysis emits a warning when the functions are called with
  non-optional types ([#258](https://github.com/stjude-rust-labs/wdl/pull/258)).
* The "required primitive type" constraint has been removed as every place the
  constraint was used should allow for optional primitive types as well;
  consequently, the AnyPrimitiveTypeConstraint was renamed to simply
  `PrimitiveTypeConstraint` ([#257](https://github.com/stjude-rust-labs/wdl/pull/257)).
* The common type calculation now favors the "left-hand side" of the
  calculation rather than the right, making it more intuitive to use. For
  example, a calculation of `File | String` is now `File` rather than
  `String` ([#257](https://github.com/stjude-rust-labs/wdl/pull/257)).
* Refactored function call binding information to aid with call evaluation in
  `wdl-engine` ([#251](https://github.com/stjude-rust-labs/wdl/pull/251)).
* Made diagnostic creation functions public ([#249](https://github.com/stjude-rust-labs/wdl/pull/249)).
* Refactored expression type evaluator to provide context via a trait ([#249](https://github.com/stjude-rust-labs/wdl/pull/249)).
* Removed `PartialEq`, `Eq`, and `Hash` from WDL-type-related types ([#249](https://github.com/stjude-rust-labs/wdl/pull/249)).

### Fixed

* Fixed an issue where imported structs weren't always checked correctly for
  type equivalence with local structs ([#265](https://github.com/stjude-rust-labs/wdl/pull/265)).
* Common type calculation now supports discovering common types between the
  compound types containing Union and None as inner types, e.g.
  `Array[String] | Array[None] -> Array[String?]` ([#257](https://github.com/stjude-rust-labs/wdl/pull/257)).
* Static analysis of expressions within object literal members now takes place ([#254](https://github.com/stjude-rust-labs/wdl/pull/254)).
* Certain standard library functions with an existing constraint on generic
  parameters that take structs are further constrained to take structs
  containing only primitive members ([#254](https://github.com/stjude-rust-labs/wdl/pull/254)).
* Fixed signatures and minimum required versions for certain standard library
  functions ([#254](https://github.com/stjude-rust-labs/wdl/pull/254)).

## 0.5.0 - 10-22-2024

### Changed

* Refactored the `DocumentScope` API to simply `Document` and exposed more
  information about tasks and workflows such as their inputs and outputs ([#232](https://github.com/stjude-rust-labs/wdl/pull/232)).
* Switched to `rustls-tls` for TLS implementation rather than relying on
  OpenSSL for Linux builds ([#228](https://github.com/stjude-rust-labs/wdl/pull/228)).

## 0.4.0 - 10-16-2024

### Added

* Implemented `UnusedImport`, `UnusedInput`, `UnusedDeclaration`, and
  `UnusedCall` analysis warnings ([#211](https://github.com/stjude-rust-labs/wdl/pull/211))
* Implemented static analysis for workflows ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).

### Fixed

* Allow coercion of `Array[T]` to `Array[T]+` unless from an empty array
  literal ([#213](https://github.com/stjude-rust-labs/wdl/pull/213)).
* Improved type calculations in function calls and when determining common
  types in certain expressions ([#209](https://github.com/stjude-rust-labs/wdl/pull/209)).
* Treat a coercion to `T?` for a function argument of type `T` as a preference
  over any other coercion ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Fix the signature of `select_first` such that it is monomorphic ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Only consider signatures in overload resolution that have sufficient
  arguments ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Allow coercion from `File` and `Directory` to `String` ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Allow non-empty array literals to coerce to either empty or non-empty ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Fix element type calculations for `Array` and `Map` so that `[a, b]` and
  `{"a": a, "b": b }` successfully calculates when `a` is coercible to `b` ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Fix `if` expression type calculation such that `if (x) then a else b` works
  when `a` is coercible to `b` ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Ensure that only equality/inequality expressions are supported on `File` and
  `Directory` now that there is a coercion to `String` ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).
* Allow index expressions on `Map` ([#199](https://github.com/stjude-rust-labs/wdl/pull/199)).

## 0.3.0 - 09-16-2024

### Added

* Implemented type checking in task runtime, requirements, and hints sections
  ([#170](https://github.com/stjude-rust-labs/wdl/pull/170)).
* Add support for the `task` variable in WDL 1.2 ([#168](https://github.com/stjude-rust-labs/wdl/pull/168)).
* Full type checking support in task definitions ([#163](https://github.com/stjude-rust-labs/wdl/pull/163)).

### Changed

* Use `tracing` events instead of the `log` crate ([#172](https://github.com/stjude-rust-labs/wdl/pull/172))
* Refactored crate layout ([#163](https://github.com/stjude-rust-labs/wdl/pull/163)).

### Fixed

* Fixed definition of `basename` and `size` functions to accept `String` ([#163](https://github.com/stjude-rust-labs/wdl/pull/163)).

## 0.2.0 - 08-22-2024

### Added

* Implemented type checking of struct definitions ([#160](https://github.com/stjude-rust-labs/wdl/pull/160)).
* Implemented a type system and representation of the WDL standard library for
  future type checking support ([#156](https://github.com/stjude-rust-labs/wdl/pull/156)).
* Specified the MSRV for the crate ([#144](https://github.com/stjude-rust-labs/wdl/pull/144)).

### Changed

* Refactored `Analyzer` API to support change notifications ([#146](https://github.com/stjude-rust-labs/wdl/pull/146)).
* Replaced `AnalysisEngine` with `Analyzer` ([#143](https://github.com/stjude-rust-labs/wdl/pull/143)).

## 0.1.0 - 07-17-2024

### Added

* Added the `wdl-analysis` crate for analyzing WDL documents ([#110](https://github.com/stjude-rust-labs/wdl/pull/110)).

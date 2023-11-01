use pest::consumes_to;
use pest::fails_with;
use pest::parses_to;

use crate::Parser as WdlParser;
use crate::Rule;

#[test]
fn it_fails_to_parse_an_empty_remainder() {
    fails_with! {
        parser: WdlParser,
        input: "",
        rule: Rule::remainder,
        positives: vec![Rule::remainder],
        negatives: vec![],
        pos: 0
    }
}

#[test]
fn it_fails_to_parse_a_value_that_is_not_remainder() {
    fails_with! {
        parser: WdlParser,
        input: "*",
        rule: Rule::remainder,
        positives: vec![Rule::remainder],
        negatives: vec![],
        pos: 0
    }
}

#[test]
fn it_successfully_parses_remainder() {
    parses_to! {
        parser: WdlParser,
        input: "%",
        rule: Rule::remainder,
        tokens: [remainder(0, 1)]
    }
}

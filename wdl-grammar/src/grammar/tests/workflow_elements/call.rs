use pest::consumes_to;
use pest::fails_with;
use pest::parses_to;

use crate::Parser as WdlParser;
use crate::Rule;

#[test]
fn it_fails_to_parse_an_empty_call() {
    fails_with! {
        parser: WdlParser,
        input: "",
        rule: Rule::workflow_call,
        positives: vec![Rule::workflow_call],
        negatives: vec![],
        pos: 0
    }
}

#[test]
fn it_successfully_parses_plain_call() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 13)]
    }
}

#[test]
fn it_successfully_parses_call_with_empty_body() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task{}",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 15)]
    }
}

#[test]
fn it_successfully_parses_call_with_implicitly_declared_input() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task{input:a}",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 21)]
    }
}

#[test]
fn it_successfully_parses_call_with_multiple_inputs() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task{input:a,b=b,c=z}",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 30)]
    }
}

#[test]
fn it_successfully_parses_call_with_as() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task as different_task",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 31)]
    }
}

#[test]
fn it_successfully_parses_call_with_after() {
    parses_to! {
        parser: WdlParser,
        input: "call imported_doc.my_task after different_task",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 47)]
    }
}

#[test]
fn it_successfully_parses_call_with_all_options() {
    parses_to! {
        parser: WdlParser,
        input: "call imported_doc.my_task as their_task after different_task { input: a, b = b, c=z }",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 86)]
    }
}

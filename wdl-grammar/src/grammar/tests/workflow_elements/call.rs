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
        tokens: [workflow_call(0, 12, [WHITESPACE(4, 5), identifier(5, 12)])]
    }
}

#[test]
fn it_successfully_parses_call_with_empty_body() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task{}",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 14,
            [WHITESPACE(4, 5), identifier(5, 12), workflow_call_body(12, 14)]
        )]
    }
}

#[test]
fn it_successfully_parses_call_with_implicitly_declared_input() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task{input:a}",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 21, [
            WHITESPACE(4, 5),
            identifier(5, 12),
            workflow_call_body(12, 21, [
                workflow_call_input(19, 20, [identifier(19, 20)])
            ])
        ])]
    }
}

#[test]
fn it_successfully_parses_call_with_multiple_inputs() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task{input:a,b=b,c=z}",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 29, [
            WHITESPACE(4, 5),
            identifier(5, 12),
            workflow_call_body(12, 29, [
                workflow_call_input(19, 20, [identifier(19, 20)]),
                workflow_call_input(21, 24, [identifier(21, 22), identifier(23, 24)]),
                workflow_call_input(25, 28, [identifier(25, 26), identifier(27, 28)])
            ])
        ])]
    }
}

#[test]
fn it_successfully_parses_call_with_as() {
    parses_to! {
        parser: WdlParser,
        input: "call my_task as different_task",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 30, [
            WHITESPACE(4, 5),
            identifier(5, 12),
            WHITESPACE(12, 13),
            workflow_call_as(13, 30, [
                WHITESPACE(15, 16),
                identifier(16, 30)
            ])
        ])]
    }
}

#[test]
fn it_successfully_parses_call_with_after() {
    parses_to! {
        parser: WdlParser,
        input: "call imported_doc.my_task after different_task",
        rule: Rule::workflow_call,
        tokens: [workflow_call(0, 46, [
            WHITESPACE(4, 5),
            qualified_identifier(5, 25, [
                identifier(5, 17),
                identifier(18, 25)
            ]),
            WHITESPACE(25, 26),
            workflow_call_after(26, 46, [
                WHITESPACE(31, 32),
                identifier(32, 46)
            ])
        ])]
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

use pest::consumes_to;
use pest::fails_with;
use pest::parses_to;

use crate::Parser as WdlParser;
use crate::Rule;

#[test]
fn it_fails_to_parse_an_empty_scatter() {
    fails_with! {
        parser: WdlParser,
        input: "",
        rule: Rule::workflow_scatter,
        positives: vec![Rule::workflow_scatter],
        negatives: vec![],
        pos: 0
    }
}

#[test]
fn it_successfully_parses_scatter_without_spaces() {
    parses_to! {
        parser: WdlParser,
        input: "scatter (i in range(10)){call my_task}",
        rule: Rule::workflow_scatter,
        tokens: [workflow_scatter(0, 38, [
            WHITESPACE(7, 8, [INDENT(7, 8, [SPACE(7, 8)])]),
            workflow_scatter_iteration_stmnt(8, 24, [
                identifier(9, 10),
                WHITESPACE(10, 11, [INDENT(10, 11, [SPACE(10, 11)])]),
                WHITESPACE(13, 14, [INDENT(13, 14, [SPACE(13, 14)])]),
                expression(14, 23, [
                    identifier(14, 19),
                    apply(19, 23, [
                        expression(20, 22, [
                            integer(20, 22, [
                                integer_decimal(20, 22)
                            ])
                        ])
                    ])
                ]),
            ]),
            workflow_execution_stmnt(25, 37, [
                workflow_call(25, 37, [
                    WHITESPACE(29, 30, [INDENT(29, 30, [SPACE(29, 30)])]),
                    identifier(30, 37)
                ])
            ])
        ])]
    }
}

#[test]
fn it_successfully_parses_scatter_with_spaces() {
    parses_to! {
        parser: WdlParser,
        input: "scatter ( i in range( 10 ) ) { call my_task }",
        rule: Rule::workflow_scatter,
        tokens: [workflow_scatter(0, 45, [
            WHITESPACE(7, 8, [INDENT(7, 8, [SPACE(7, 8)])]),
            workflow_scatter_iteration_stmnt(8, 28, [
                WHITESPACE(9, 10, [INDENT(9, 10, [SPACE(9, 10)])]),
                identifier(10, 11),
                WHITESPACE(11, 12, [INDENT(11, 12, [SPACE(11, 12)])]),
                WHITESPACE(14, 15, [INDENT(14, 15, [SPACE(14, 15)])]),
                expression(15, 26, [
                    identifier(15, 20),
                    apply(20, 26, [
                        WHITESPACE(21, 22, [INDENT(21, 22, [SPACE(21, 22)])]),
                        expression(22, 24, [
                            integer(22, 24, [
                                integer_decimal(22, 24)
                            ])
                        ]),
                        WHITESPACE(24, 25, [INDENT(24, 25, [SPACE(24, 25)])]),
                    ])
                ]),
                WHITESPACE(26, 27, [INDENT(26, 27, [SPACE(26, 27)])]),
            ]),
            WHITESPACE(28, 29, [INDENT(28, 29, [SPACE(28, 29)])]),
            WHITESPACE(30, 31, [INDENT(30, 31, [SPACE(30, 31)])]),
            workflow_execution_stmnt(31, 43, [
                workflow_call(31, 43, [
                    WHITESPACE(35, 36, [INDENT(35, 36, [SPACE(35, 36)])]),
                    identifier(36, 43)
                ])
            ]),
            WHITESPACE(43, 44, [INDENT(43, 44, [SPACE(43, 44)])]),
        ])]
    }
}

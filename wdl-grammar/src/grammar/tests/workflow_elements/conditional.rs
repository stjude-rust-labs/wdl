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
        rule: Rule::workflow_conditional,
        positives: vec![Rule::workflow_conditional],
        negatives: vec![],
        pos: 0
    }
}

#[test]
fn it_successfully_parses_conditional_without_space() {
    parses_to! {
        parser: WdlParser,
        input: "if(true){Int a=10}",
        rule: Rule::workflow_conditional,
        tokens: [workflow_conditional(0, 18, [
            expression(3, 7, [
                boolean(3, 7)
            ]),
            workflow_execution_stmnt(9, 17, [
                workflow_private_declarations(9, 17, [
                    bound_declaration(9, 17, [
                        int_type(9, 12),
                        WHITESPACE(12, 13, [SPACE(12, 13)]),
                        identifier(13, 14),
                        expression(15, 17, [
                            integer(15, 17, [
                                integer_decimal(15, 17)
                            ])
                        ])
                    ])
                ]),
            ]),
        ])]
    }
}

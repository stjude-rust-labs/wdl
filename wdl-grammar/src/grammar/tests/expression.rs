use pest::consumes_to;
use pest::parses_to;

use crate::Parser as WdlParser;
use crate::Rule;

mod core;
mod infix;
mod prefix;
mod suffix;

#[test]
fn it_parses_an_extremely_complicated_expression() {
    parses_to! {
        parser: WdlParser,
        input: "if
    if true == false && 2 != 1 then
        (
            object {a: true}.a ||
            !(true, false)[0]
        )
    else
        -struct {b: 10}.b
then
    [0, 1, 2, 3e10][if true then 2 else 1] ||
    [0, 0432, 0xF2, -3e+10](zulu)
else
    false
",
        rule: Rule::expression,
        tokens: [expression(0, 257, [
          r#if(0, 257, [
            // ``
            WHITESPACE(2, 3, [
              // ``
              NEWLINE(2, 3),
            ]),
            WHITESPACE(3, 4, [
              SPACE(3, 4),
            ]),
            WHITESPACE(4, 5, [
              SPACE(4, 5),
            ]),
            WHITESPACE(5, 6, [
              SPACE(5, 6),
            ]),
            WHITESPACE(6, 7, [
              SPACE(6, 7),
            ]),
            expression(7, 157, [
              r#if(7, 157, [
                WHITESPACE(9, 10, [
                  SPACE(9, 10),
                ]),
                expression(10, 33, [
                  boolean(10, 14),
                  WHITESPACE(14, 15, [
                    SPACE(14, 15),
                  ]),
                  eq(15, 17),
                  WHITESPACE(17, 18, [
                    SPACE(17, 18),
                  ]),
                  boolean(18, 23),
                  WHITESPACE(23, 24, [
                    SPACE(23, 24),
                  ]),
                  and(24, 26),
                  WHITESPACE(26, 27, [
                    SPACE(26, 27),
                  ]),
                  integer(27, 28, [
                    integer_decimal(27, 28),
                  ]),
                  WHITESPACE(28, 29, [
                    SPACE(28, 29),
                  ]),
                  neq(29, 31),
                  WHITESPACE(31, 32, [
                    SPACE(31, 32),
                  ]),
                  integer(32, 33, [
                    integer_decimal(32, 33),
                  ]),
                ]),
                WHITESPACE(33, 34, [
                  SPACE(33, 34),
                ]),
                // ``
                WHITESPACE(38, 39, [
                  // ``
                  NEWLINE(38, 39),
                ]),
                WHITESPACE(39, 40, [
                  SPACE(39, 40),
                ]),
                WHITESPACE(40, 41, [
                  SPACE(40, 41),
                ]),
                WHITESPACE(41, 42, [
                  SPACE(41, 42),
                ]),
                WHITESPACE(42, 43, [
                  SPACE(42, 43),
                ]),
                WHITESPACE(43, 44, [
                  SPACE(43, 44),
                ]),
                WHITESPACE(44, 45, [
                  SPACE(44, 45),
                ]),
                WHITESPACE(45, 46, [
                  SPACE(45, 46),
                ]),
                WHITESPACE(46, 47, [
                  SPACE(46, 47),
                ]),
                expression(47, 122, [
                  group(47, 122, [
                    // ``
                    WHITESPACE(48, 49, [
                      // ``
                      NEWLINE(48, 49),
                    ]),
                    WHITESPACE(49, 50, [
                      SPACE(49, 50),
                    ]),
                    WHITESPACE(50, 51, [
                      SPACE(50, 51),
                    ]),
                    WHITESPACE(51, 52, [
                      SPACE(51, 52),
                    ]),
                    WHITESPACE(52, 53, [
                      SPACE(52, 53),
                    ]),
                    WHITESPACE(53, 54, [
                      SPACE(53, 54),
                    ]),
                    WHITESPACE(54, 55, [
                      SPACE(54, 55),
                    ]),
                    WHITESPACE(55, 56, [
                      SPACE(55, 56),
                    ]),
                    WHITESPACE(56, 57, [
                      SPACE(56, 57),
                    ]),
                    WHITESPACE(57, 58, [
                      SPACE(57, 58),
                    ]),
                    WHITESPACE(58, 59, [
                      SPACE(58, 59),
                    ]),
                    WHITESPACE(59, 60, [
                      SPACE(59, 60),
                    ]),
                    WHITESPACE(60, 61, [
                      SPACE(60, 61),
                    ]),
                    expression(61, 112, [
                      object_literal(61, 77, [
                        WHITESPACE(67, 68, [
                          SPACE(67, 68),
                        ]),
                        identifier_based_kv_pair(69, 76, [
                          identifier_based_kv_key(69, 70, [
                            identifier(69, 70),
                          ]),
                          WHITESPACE(71, 72, [
                            SPACE(71, 72),
                          ]),
                          kv_value(72, 76, [
                            expression(72, 76, [
                              boolean(72, 76),
                            ]),
                          ]),
                        ]),
                      ]),
                      member(77, 79, [
                        identifier(78, 79),
                      ]),
                      WHITESPACE(79, 80, [
                        SPACE(79, 80),
                      ]),
                      or(80, 82),
                      // ``
                      WHITESPACE(82, 83, [
                        // ``
                        NEWLINE(82, 83),
                      ]),
                      WHITESPACE(83, 84, [
                        SPACE(83, 84),
                      ]),
                      WHITESPACE(84, 85, [
                        SPACE(84, 85),
                      ]),
                      WHITESPACE(85, 86, [
                        SPACE(85, 86),
                      ]),
                      WHITESPACE(86, 87, [
                        SPACE(86, 87),
                      ]),
                      WHITESPACE(87, 88, [
                        SPACE(87, 88),
                      ]),
                      WHITESPACE(88, 89, [
                        SPACE(88, 89),
                      ]),
                      WHITESPACE(89, 90, [
                        SPACE(89, 90),
                      ]),
                      WHITESPACE(90, 91, [
                        SPACE(90, 91),
                      ]),
                      WHITESPACE(91, 92, [
                        SPACE(91, 92),
                      ]),
                      WHITESPACE(92, 93, [
                        SPACE(92, 93),
                      ]),
                      WHITESPACE(93, 94, [
                        SPACE(93, 94),
                      ]),
                      WHITESPACE(94, 95, [
                        SPACE(94, 95),
                      ]),
                      negation(95, 96),
                      pair_literal(96, 109, [
                        expression(97, 101, [
                          boolean(97, 101),
                        ]),
                        WHITESPACE(102, 103, [
                          SPACE(102, 103),
                        ]),
                        expression(103, 108, [
                          boolean(103, 108),
                        ]),
                      ]),
                      index(109, 112, [
                        expression(110, 111, [
                          integer(110, 111, [
                            integer_decimal(110, 111),
                          ]),
                        ]),
                      ]),
                    ]),
                    // ``
                    WHITESPACE(112, 113, [
                      // ``
                      NEWLINE(112, 113),
                    ]),
                    WHITESPACE(113, 114, [
                      SPACE(113, 114),
                    ]),
                    WHITESPACE(114, 115, [
                      SPACE(114, 115),
                    ]),
                    WHITESPACE(115, 116, [
                      SPACE(115, 116),
                    ]),
                    WHITESPACE(116, 117, [
                      SPACE(116, 117),
                    ]),
                    WHITESPACE(117, 118, [
                      SPACE(117, 118),
                    ]),
                    WHITESPACE(118, 119, [
                      SPACE(118, 119),
                    ]),
                    WHITESPACE(119, 120, [
                      SPACE(119, 120),
                    ]),
                    WHITESPACE(120, 121, [
                      SPACE(120, 121),
                    ]),
                  ]),
                ]),
                // ``
                WHITESPACE(122, 123, [
                  // ``
                  NEWLINE(122, 123),
                ]),
                WHITESPACE(123, 124, [
                  SPACE(123, 124),
                ]),
                WHITESPACE(124, 125, [
                  SPACE(124, 125),
                ]),
                WHITESPACE(125, 126, [
                  SPACE(125, 126),
                ]),
                WHITESPACE(126, 127, [
                  SPACE(126, 127),
                ]),
                // ``
                WHITESPACE(131, 132, [
                  // ``
                  NEWLINE(131, 132),
                ]),
                WHITESPACE(132, 133, [
                  SPACE(132, 133),
                ]),
                WHITESPACE(133, 134, [
                  SPACE(133, 134),
                ]),
                WHITESPACE(134, 135, [
                  SPACE(134, 135),
                ]),
                WHITESPACE(135, 136, [
                  SPACE(135, 136),
                ]),
                WHITESPACE(136, 137, [
                  SPACE(136, 137),
                ]),
                WHITESPACE(137, 138, [
                  SPACE(137, 138),
                ]),
                WHITESPACE(138, 139, [
                  SPACE(138, 139),
                ]),
                WHITESPACE(139, 140, [
                  SPACE(139, 140),
                ]),
                expression(140, 157, [
                  unary_signed(140, 141),
                  struct_literal(141, 155, [
                    identifier(141, 147),
                    WHITESPACE(147, 148, [
                      SPACE(147, 148),
                    ]),
                    identifier_based_kv_pair(149, 154, [
                      identifier_based_kv_key(149, 150, [
                        identifier(149, 150),
                      ]),
                      WHITESPACE(151, 152, [
                        SPACE(151, 152),
                      ]),
                      kv_value(152, 154, [
                        expression(152, 154, [
                          integer(152, 154, [
                            integer_decimal(152, 154),
                          ]),
                        ]),
                      ]),
                    ]),
                  ]),
                  member(155, 157, [
                    identifier(156, 157),
                  ]),
                ]),
              ]),
            ]),
            // ``
            WHITESPACE(157, 158, [
              // ``
              NEWLINE(157, 158),
            ]),
            // ``
            WHITESPACE(162, 163, [
              // ``
              NEWLINE(162, 163),
            ]),
            WHITESPACE(163, 164, [
              SPACE(163, 164),
            ]),
            WHITESPACE(164, 165, [
              SPACE(164, 165),
            ]),
            WHITESPACE(165, 166, [
              SPACE(165, 166),
            ]),
            WHITESPACE(166, 167, [
              SPACE(166, 167),
            ]),
            expression(167, 242, [
              array_literal(167, 182, [
                expression(168, 169, [
                  integer(168, 169, [
                    integer_decimal(168, 169),
                  ]),
                ]),
                WHITESPACE(170, 171, [
                  SPACE(170, 171),
                ]),
                expression(171, 172, [
                  integer(171, 172, [
                    integer_decimal(171, 172),
                  ]),
                ]),
                WHITESPACE(173, 174, [
                  SPACE(173, 174),
                ]),
                expression(174, 175, [
                  integer(174, 175, [
                    integer_decimal(174, 175),
                  ]),
                ]),
                WHITESPACE(176, 177, [
                  SPACE(176, 177),
                ]),
                expression(177, 181, [
                  float(177, 181, [
                    float_simple(177, 181),
                  ]),
                ]),
              ]),
              index(182, 205, [
                expression(183, 204, [
                  r#if(183, 204, [
                    WHITESPACE(185, 186, [
                      SPACE(185, 186),
                    ]),
                    expression(186, 190, [
                      boolean(186, 190),
                    ]),
                    WHITESPACE(190, 191, [
                      SPACE(190, 191),
                    ]),
                    WHITESPACE(195, 196, [
                      SPACE(195, 196),
                    ]),
                    expression(196, 197, [
                      integer(196, 197, [
                        integer_decimal(196, 197),
                      ]),
                    ]),
                    WHITESPACE(197, 198, [
                      SPACE(197, 198),
                    ]),
                    WHITESPACE(202, 203, [
                      SPACE(202, 203),
                    ]),
                    expression(203, 204, [
                      integer(203, 204, [
                        integer_decimal(203, 204),
                      ]),
                    ]),
                  ]),
                ]),
              ]),
              WHITESPACE(205, 206, [
                SPACE(205, 206),
              ]),
              or(206, 208),
              // ``
              WHITESPACE(208, 209, [
                // ``
                NEWLINE(208, 209),
              ]),
              WHITESPACE(209, 210, [
                SPACE(209, 210),
              ]),
              WHITESPACE(210, 211, [
                SPACE(210, 211),
              ]),
              WHITESPACE(211, 212, [
                SPACE(211, 212),
              ]),
              WHITESPACE(212, 213, [
                SPACE(212, 213),
              ]),
              array_literal(213, 236, [
                expression(214, 215, [
                  integer(214, 215, [
                    integer_decimal(214, 215),
                  ]),
                ]),
                WHITESPACE(216, 217, [
                  SPACE(216, 217),
                ]),
                expression(217, 221, [
                  integer(217, 221, [
                    integer_octal(217, 221),
                  ]),
                ]),
                WHITESPACE(222, 223, [
                  SPACE(222, 223),
                ]),
                expression(223, 227, [
                  integer(223, 227, [
                    integer_hex(223, 227),
                  ]),
                ]),
                WHITESPACE(228, 229, [
                  SPACE(228, 229),
                ]),
                expression(229, 235, [
                  unary_signed(229, 230),
                  float(230, 235, [
                    float_simple(230, 235),
                  ]),
                ]),
              ]),
              apply(236, 242, [
                expression(237, 241, [
                  identifier(237, 241),
                ]),
              ]),
            ]),
            // ``
            WHITESPACE(242, 243, [
              // ``
              NEWLINE(242, 243),
            ]),
            // ``
            WHITESPACE(247, 248, [
              // ``
              NEWLINE(247, 248),
            ]),
            WHITESPACE(248, 249, [
              SPACE(248, 249),
            ]),
            WHITESPACE(249, 250, [
              SPACE(249, 250),
            ]),
            WHITESPACE(250, 251, [
              SPACE(250, 251),
            ]),
            WHITESPACE(251, 252, [
              SPACE(251, 252),
            ]),
            expression(252, 257, [
              boolean(252, 257),
            ]),
          ]),
        ])
        ]
    }
}

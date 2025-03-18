#[test]
fn test_mixed_indentation_in_document_v1_0() {
    let lint = CommandSectionMixedIndentationRule;

    // Mixed indentation in the entire document → Warning for WDL v1.0
    let doc = Document::parse("task { \t input { String name  } }").unwrap();
    let mut diagnostics = Diagnostics::default();

    lint.document(&mut diagnostics, VisitReason::Enter, &doc, SupportedVersion::V1_0);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "mixed indentation (warning)");
}

#[test]
fn test_mixed_indentation_in_document_v1_2() {
    let lint = CommandSectionMixedIndentationRule;

    // Mixed indentation in the entire document → Note for WDL v1.2
    let doc = Document::parse("workflow { \t input { String name  } }").unwrap();
    let mut diagnostics = Diagnostics::default();

    lint.document(&mut diagnostics, VisitReason::Enter, &doc, SupportedVersion::V1_2);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "mixed indentation (note)");
}

#[test]
fn test_mixed_indentation_in_command_section_v1_0() {
    let lint = CommandSectionMixedIndentationRule;

    // Mixed indentation inside command block → Warning for WDL v1.0
    let doc = Document::parse("task { command { \t echo 'Hello'  } }").unwrap();
    let mut diagnostics = Diagnostics::default();

    lint.document(&mut diagnostics, VisitReason::Enter, &doc, SupportedVersion::V1_0);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "mixed indentation (warning)");
}

#[test]
fn test_mixed_indentation_in_command_section_v1_2() {
    let lint = CommandSectionMixedIndentationRule;

    // Mixed indentation inside command block → Warning for WDL v1.2 (inside commands always warning)
    let doc = Document::parse("task { command { \t echo 'Hello'  } }").unwrap();
    let mut diagnostics = Diagnostics::default();

    lint.document(&mut diagnostics, VisitReason::Enter, &doc, SupportedVersion::V1_2);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].message, "mixed indentation (warning)");
}

#[test]
fn test_no_mixed_indentation() {
    let lint = CommandSectionMixedIndentationRule;

    // No mixed indentation → No diagnostics
    let doc = Document::parse("task { command { echo 'Hello' } }").unwrap();
    let mut diagnostics = Diagnostics::default();

    lint.document(&mut diagnostics, VisitReason::Enter, &doc, SupportedVersion::V1_2);

    assert_eq!(diagnostics.len(), 0);
}

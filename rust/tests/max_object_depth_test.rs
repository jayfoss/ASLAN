use aslan::ASLANParser;
use aslan::ASLANParserSettings;
use serde_json::json;

// =============================================================================
// Current behavior - demonstrating the issue
// =============================================================================

#[test]
fn collapse_enabled_nested_objects_with_inline_content_work_correctly() {
    // Case 1: Works fine with collapse enabled - content on same line
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        ..Default::default()
    });
    let result = parser.parse(
        "[asland_edit1][aslano]\n\
         [asland_text]New text\n\
         [aslano]\n\
         \n\
         [asland_edit2][aslano]\n\
         [asland_text]New text\n\
         [aslano]"
    );
    assert_eq!(result, json!({
        "_default": null,
        "edit1": {
            "text": "New text\n"
        },
        "edit2": {
            "text": "New text\n"
        }
    }));
}

#[test]
fn collapse_enabled_empty_field_followed_by_second_edit_gets_truncated() {
    // Case 2: The problematic case - empty text field causes edit2 to get truncated (BUG)
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        ..Default::default()
    });
    let result = parser.parse(
        "[asland_edit1][aslano]\n\
         [asland_text]\n\
         [aslano]\n\
         \n\
         [asland_edit2][aslano]\n\
         [asland_text]This will get ignored\n\
         [aslano]"
    );
    // This demonstrates the BUG: edit2 gets nested inside edit1.text
    // because whitespace collapse makes get_object_safe_latest_result() return false
    assert_eq!(result, json!({
        "_default": null,
        "edit1": {
            "text": {
                "edit2": {
                    "text": "This will get ignored\n"
                }
            }
        }
    }));
}

#[test]
fn collapse_disabled_newline_before_aslano_causes_object_creation_to_fail() {
    // Case 3: Potential issue with collapse disabled if model puts o on new line (BUG)
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: false,
        ..Default::default()
    });
    let result = parser.parse(
        "[asland_edit1]\n\
         [aslano]\n\
         [asland_text]Some text\n\
         [aslano]"
    );
    // The newline becomes content for edit1, so [aslano] closes instead of opening
    assert_eq!(result, json!({
        "_default": null,
        "edit1": "\n",
        "text": "Some text\n"
    }));
}

#[test]
fn collapse_disabled_empty_field_followed_by_second_edit_works() {
    // This case works with collapse disabled because the newline provides content
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: false,
        ..Default::default()
    });
    let result = parser.parse(
        "[asland_edit1][aslano]\n\
         [asland_text]\n\
         [aslano]\n\
         \n\
         [asland_edit2][aslano]\n\
         [asland_text]This should not get ignored\n\
         [aslano]"
    );
    assert_eq!(result, json!({
        "_default": null,
        "edit1": {
            "text": "\n"
        },
        "edit2": {
            "text": "This should not get ignored\n"
        }
    }));
}

// =============================================================================
// maxObjectDepth feature - expected behavior
// =============================================================================
// With max_object_depth: Some(1), the o operator should be binary:
// - At object depth 0: [aslano] always creates a new object (push to stack)
// - At object depth 1: [aslano] always closes the current object (pop from stack)
// This eliminates ambiguity from whitespace content

#[test]
fn max_object_depth_1_empty_field_should_not_cause_truncation() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(1),
        ..Default::default()
    });
    let result = parser.parse(
        "[asland_edit1][aslano]\n\
         [asland_text]\n\
         [aslano]\n\
         \n\
         [asland_edit2][aslano]\n\
         [asland_text]This should NOT get ignored\n\
         [aslano]"
    );
    // With max_object_depth: Some(1):
    // - [aslano] after edit1: depth 0 -> creates object (depth becomes 1)
    // - [aslano] after text: depth 1 -> closes object (depth becomes 0)
    // - [aslano] after edit2: depth 0 -> creates object (depth becomes 1)
    // - [aslano] after text: depth 1 -> closes object (depth becomes 0)
    assert_eq!(result, json!({
        "_default": null,
        "edit1": {
            "text": "\n"
        },
        "edit2": {
            "text": "This should NOT get ignored\n"
        }
    }));
}

#[test]
fn max_object_depth_1_newline_before_aslano_follows_existing_logic_at_depth_0() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: false,
        max_object_depth: Some(1),
        ..Default::default()
    });
    let result = parser.parse(
        "[asland_edit1]\n\
         [aslano]\n\
         [asland_text]Some text\n\
         [aslano]"
    );
    // With max_object_depth: Some(1) and collapse_object_start_whitespace: false:
    // At depth 0 < max_depth, existing create/close logic still applies
    // The newline becomes content for edit1, so existing logic sees non-empty
    // and tries to close (which is noop at root). Object is NOT created.
    // NOTE: This is expected - max_object_depth only limits depth, doesn't change
    // create/close logic at depth < max_depth. To fix this case, use
    // collapse_object_start_whitespace: true instead.
    assert_eq!(result, json!({
        "_default": null,
        "edit1": "\n",
        "text": "Some text\n"
    }));
}

#[test]
fn max_object_depth_1_inline_version_same_structure_as_multiline() {
    let mut parser_inline = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(1),
        ..Default::default()
    });
    let result_inline = parser_inline.parse(
        "[asland_edit1][aslano][asland_text]content[aslano][asland_edit2][aslano][asland_text]more content[aslano]"
    );

    let mut parser_multiline = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(1),
        ..Default::default()
    });
    let result_multiline = parser_multiline.parse(
        "[asland_edit1][aslano]\n\
         [asland_text]content\n\
         [aslano]\n\
         \n\
         [asland_edit2][aslano]\n\
         [asland_text]more content\n\
         [aslano]"
    );

    // Both should have the same structure (ignoring whitespace in content)
    assert_eq!(result_inline, json!({
        "_default": null,
        "edit1": {
            "text": "content"
        },
        "edit2": {
            "text": "more content"
        }
    }));

    assert_eq!(result_multiline, json!({
        "_default": null,
        "edit1": {
            "text": "content\n"
        },
        "edit2": {
            "text": "more content\n"
        }
    }));
}

#[test]
fn max_object_depth_1_second_aslano_should_close_instead_of_nesting() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(1),
        ..Default::default()
    });

    // Attempting to nest more than 1 level deep
    let result = parser.parse(
        "[asland_a][aslano][asland_b][aslano][asland_c]value[aslano][aslano]"
    );

    // With max_object_depth: Some(1):
    // - [aslano] after a: depth 0, 'a' is empty -> creates object for a (depth becomes 1)
    // - [asland_b]: sets current key to 'b' inside 'a', but no content written yet
    // - [aslano] after b: depth 1 >= max_depth -> always close (depth becomes 0)
    // - [asland_c]: sets key to 'c' at top level
    // - value: c = 'value'
    // - [aslano] after c/value: depth 0, follows normal logic (close noop at root)
    // - final [aslano]: same, noop
    assert_eq!(result, json!({
        "_default": null,
        "a": {},
        "c": "value"
    }));
}

#[test]
fn max_object_depth_1_works_with_arrays_inside_objects() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(1),
        ..Default::default()
    });

    let result = parser.parse(
        "[asland_edit][aslano][asland_items][aslana][asland]item1[asland]item2[aslano]"
    );

    // Array nesting should still work, only object depth is limited
    assert_eq!(result, json!({
        "_default": null,
        "edit": {
            "items": ["item1", "item2"]
        }
    }));
}

#[test]
fn max_object_depth_0_no_objects_allowed() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(0),
        ..Default::default()
    });

    let result = parser.parse(
        "[asland_edit][aslano][asland_text]content[aslano]"
    );

    // With max_object_depth: Some(0), [aslano] always tries to close (depth 0 >= 0)
    // At depth 0, closing is a no-op (nothing to pop from stack)
    // So [aslano] is effectively ignored
    assert_eq!(result, json!({
        "_default": null,
        "text": "content"
    }));
}

#[test]
fn max_object_depth_2_allows_two_levels_of_nesting() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        max_object_depth: Some(2),
        ..Default::default()
    });

    let result = parser.parse(
        "[asland_a][aslano][asland_b][aslano][asland_c]value[aslano][aslano]"
    );

    // With max_object_depth: Some(2):
    // - [aslano] after a: depth 0 < 2, 'a' is empty -> creates object (depth 1)
    // - [aslano] after b: depth 1 < 2, 'b' is empty -> creates nested object (depth 2)
    // - [aslano] after c/value: depth 2 >= 2 -> closes (depth 1)
    // - final [aslano]: depth 1 < 2, but at this point we've returned to parent with non-empty content
    //   Following normal logic, this should close (depth 0)
    assert_eq!(result, json!({
        "_default": null,
        "a": {
            "b": {
                "c": "value"
            }
        }
    }));
}

#[test]
fn max_object_depth_none_default_behavior_unlimited_depth() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        // max_object_depth not set - should behave as before (None means unlimited)
        ..Default::default()
    });

    let result = parser.parse(
        "[asland_a][aslano][asland_b][aslano][asland_c][aslano][asland_d]value"
    );

    // Without max_object_depth, deep nesting should work as before
    assert_eq!(result, json!({
        "_default": null,
        "a": {
            "b": {
                "c": {
                    "d": "value"
                }
            }
        }
    }));
}

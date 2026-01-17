use aslan::ASLANParser;
use aslan::ASLANParserSettings;
use serde_json::json;

#[test]
fn parses_simple_string_with_no_defaults() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World!");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_leading_content() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("This is still valid.[asland_hi]Hello [asland_lo]World!");
    assert_eq!(result, json!({
        "_default": "This is still valid.",
        "hi": "Hello ",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_default_append_behavior() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_hi]Hello");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello Hello",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_keep_first_key_behavior() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi:f]Hello [asland_lo]World![asland_hi]Hello");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_keep_last_key_behavior() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi:l]Hello [asland_lo]World![asland_hi]Hello");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_keep_first_key_behavior_ignore_duplicate_behavior_redefinition() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi:f]Hello [asland_lo]World![asland_hi:a]Hello[asland_hi:l]Test");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_keep_first_key_behavior_on_non_first_key_treated_like_default_append() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_hi:f]Hello[asland_hi:l]Test");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello HelloTest",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_keep_last_key_behavior_on_non_first_key_treated_like_default_append() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_hi:l]Hello[asland_hi:f]Test");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello HelloTest",
        "lo": "World!"
    }));
}

#[test]
fn parses_simple_string_with_append_separator() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        append_separator: " ".to_string(),
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello[asland_lo]World![asland_hi]Hello[asland_hi]World!");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello Hello World!",
        "lo": "World!"
    }));
}

use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_empty_string() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("");
    assert_eq!(result, json!({
        "_default": ""
    }));
}

#[test]
fn parses_string_starting_with_delimiter() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_test]test");
    assert_eq!(result, json!({
        "_default": null,
        "test": "test"
    }));
}

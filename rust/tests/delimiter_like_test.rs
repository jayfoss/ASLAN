use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_parts_and_delimiter_like_items() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_test][aslanp]This is an element with [Your name].[aslanp]This is the second part.[aslanp]This is the third part.");
    assert_eq!(result, json!({
        "_default": null,
        "test": [
            "This is an element with [Your name].",
            "This is the second part.",
            "This is the third part."
        ]
    }));
}

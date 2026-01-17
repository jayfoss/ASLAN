use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_string_with_escape_delimiter() {
    let mut parser = ASLANParser::new();
    let result = parser.parse(r#"[asland_example_code][aslane_CODE_BLOCK]function greet(name) {
  console.log(`Hello, ${name}!`);
  [asland_this_is_not_parsed]This is treated as a regular string[asland_neither_is_this]So is this
}[aslane_CODE_BLOCK]"#);
    
    assert_eq!(result, json!({
        "_default": null,
        "example_code": "function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a regular string[asland_neither_is_this]So is this\n}"
    }));
}

#[test]
fn parses_string_with_escape_delimiter_ignore_non_matching_close_escape_delimiter() {
    let mut parser = ASLANParser::new();
    let result = parser.parse(r#"[asland_example_code][aslane_CODE_BLOCK]function greet(name) {
  console.log(`Hello, ${name}!`);
  [asland_this_is_not_parsed]This is treated as a[aslane_other] regular string[asland_neither_is_this]So is this
}[aslane_CODE_BLOCK]"#);
    
    assert_eq!(result, json!({
        "_default": null,
        "example_code": "function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a[aslane_other] regular string[asland_neither_is_this]So is this\n}"
    }));
}

#[test]
fn parses_string_with_escape_delimiter_continues_parsing_after_escape_closed() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_example][aslane_test][asland_this_is_not_parsed]This is treated as a regular string[aslane_test][asland_this_is_parsed]My value");
    
    assert_eq!(result, json!({
        "_default": null,
        "example": "[asland_this_is_not_parsed]This is treated as a regular string",
        "this_is_parsed": "My value"
    }));
}

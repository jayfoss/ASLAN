use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_parts() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_formatted_text][aslanp]This is the first part.[aslanp]This is the second part.[aslanp]This is the third part.");
    assert_eq!(result, json!({
        "_default": null,
        "formatted_text": [
            "This is the first part.",
            "This is the second part.",
            "This is the third part."
        ]
    }));
}

#[test]
fn parses_simple_string_with_parts_and_instructions() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.");
    assert_eq!(result, json!({
        "_default": null,
        "styled_text": [
            "This is bold and red text.",
            "This is italic and underlined text.",
            "This is large monospace text."
        ]
    }));
}

#[test]
fn parses_simple_string_with_parts_and_text_before_first_part() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_formatted_text]This is some preamble.[aslanp]This is the first part.[aslanp]This is the second part.[aslanp]This is the third part.");
    assert_eq!(result, json!({
        "_default": null,
        "formatted_text": [
            "This is some preamble.",
            "This is the first part.",
            "This is the second part.",
            "This is the third part."
        ]
    }));
}

use aslan::ASLANParser;
use aslan::ASLANParserSettings;
use serde_json::json;

#[test]
fn parses_simple_string_with_object_and_no_go_strict_start() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_start: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!");
    assert_eq!(result, json!({
        "_default": ""
    }));
}

#[test]
fn parses_simple_string_with_object_and_go_strict_start() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_start: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [aslang][asland_lo]World![asland_foo][aslano][asland_bar]Baz!");
    assert_eq!(result, json!({
        "_default": null,
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        }
    }));
}

#[test]
fn parses_simple_string_with_object_and_go_strict_start_disabled() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_start: false,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [aslang][asland_lo]World![asland_foo][aslano][asland_bar]Baz!");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        }
    }));
}

#[test]
fn parses_simple_string_with_object_and_go_strict_go_escaped_go() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_start: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![aslane_test][aslang][aslane_test][asland_foo][aslano][asland_bar]Baz!");
    assert_eq!(result, json!({
        "_default": "[asland_foo][aslano][asland_bar]Baz!"
    }));
}

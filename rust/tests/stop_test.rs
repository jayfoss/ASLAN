use aslan::ASLANParser;
use aslan::ASLANParserSettings;
use serde_json::json;

#[test]
fn parses_simple_string_with_object_and_no_stop_strict_end() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_end: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!");
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
fn parses_simple_string_with_object_and_stop_strict_end() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_end: true,
        multi_aslan_output: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![aslans][asland_foo][aslano][asland_bar]Baz!");
    assert_eq!(result, json!([
        {
            "_default": null,
            "hi": "Hello ",
            "lo": "World!"
        },
        {
            "_default": null,
            "foo": {
                "bar": "Baz!"
            }
        }
    ]));
}

#[test]
fn parses_simple_string_with_object_and_stop_strict_end_disabled() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_end: false,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [aslans][asland_lo]World![asland_foo][aslano][asland_bar]Baz!");
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
fn parses_simple_string_with_object_and_stop_strict_end_escaped_stop() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        strict_end: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![aslane_test][aslans][aslane_test][asland_foo][aslano][asland_bar]Baz!");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World![aslans]",
        "foo": {
            "bar": "Baz!"
        }
    }));
}

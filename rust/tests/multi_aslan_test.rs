use aslan::ASLANParser;
use aslan::ASLANParserSettings;
use serde_json::json;

#[test]
fn parses_simple_multi_aslan_string() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        multi_aslan_output: true,
        strict_start: true,
        strict_end: true,
        ..Default::default()
    });
    let result = parser.parse("[aslang][asland_hi]Hello [asland_lo]World![aslans][aslang][asland_foo][aslano][asland_bar]Baz![aslans][aslang]Starting again[aslang]And again[aslang][asland_further]This should also work[aslang]And this");
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
        },
        {
            "_default": "Starting again"
        },
        {
            "_default": "And again"
        },
        {
            "_default": null,
            "further": "This should also work"
        },
        {
            "_default": "And this"
        }
    ]));
}

#[test]
fn parses_multi_aslan_string_with_no_content() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        multi_aslan_output: true,
        strict_start: true,
        strict_end: true,
        ..Default::default()
    });
    let result = parser.parse("[aslang][aslang]");
    assert_eq!(result, json!([
        {
            "_default": ""
        },
        {
            "_default": ""
        }
    ]));
}

#[test]
fn parses_multi_aslan_string_with_content_between_stop_and_go_delimiters() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        multi_aslan_output: true,
        strict_start: true,
        strict_end: true,
        ..Default::default()
    });
    let result = parser.parse("[aslang]This is a test[aslans]this should be ignored[aslang]but not this");
    assert_eq!(result, json!([
        {
            "_default": "This is a test"
        },
        {
            "_default": "but not this"
        }
    ]));
}

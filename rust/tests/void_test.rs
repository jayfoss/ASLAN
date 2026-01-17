use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_object_and_void_after_other_content() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslanv]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": null
        }
    }));
}

#[test]
fn parses_simple_string_with_object_and_void_before_other_content() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar][aslanv]Baz!");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": null
        }
    }));
}

#[test]
fn parses_simple_string_with_object_and_void_in_a_later_duplicate_key_field() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![asland_bar][aslanv]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": null
        }
    }));
}

#[test]
fn parses_simple_string_with_object_and_void_in_a_previous_duplicate_key_field() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar][aslanv][asland_x]hi[asland_bar]oops");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": null,
            "x": "hi"
        }
    }));
}

use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_object() {
    let mut parser = ASLANParser::new();
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
fn parses_string_with_object_and_comment_between_asland_and_aslano() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][asland_bar]Baz!");
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
fn parses_more_complex_string_with_object() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y]you are reading spec[asland_z]and it continues here");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        },
        "x": {
            "y": "you are reading spec",
            "z": "and it continues here"
        }
    }));
}

#[test]
fn parses_more_complex_string_with_object_and_neighbor_aslano() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][aslano][asland_y]you are reading spec[asland_z]and it continues here");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        },
        "x": {},
        "y": "you are reading spec",
        "z": "and it continues here"
    }));
}

#[test]
fn parses_more_complex_string_with_object_and_neighbor_aslano_into_outer_scope() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][aslano][aslano][asland_y]you are reading spec[asland_z]and it continues here");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        },
        "x": {},
        "y": "you are reading spec",
        "z": "and it continues here"
    }));
}

#[test]
fn parses_more_complex_string_with_nested_objects() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y][aslano][asland_z]and it continues here");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        },
        "x": {
            "y": {
                "z": "and it continues here"
            }
        }
    }));
}

#[test]
fn parses_simple_string_with_multiple_objects_same_key_should_override() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi][aslano][asland_x]foo[aslano][asland_hi][aslano][asland_y]bar[aslano]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": {
            "y": "bar"
        }
    }));
}

#[test]
fn parses_simple_string_with_object_then_string_same_key_should_not_override_object() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi][aslano][asland_x]foo[aslano][asland_hi]not overriding");
    assert_eq!(result, json!({
        "_default": null,
        "hi": {
            "x": "foo"
        }
    }));
}

#[test]
fn parses_simple_string_with_string_then_object_same_key_should_override_string_with_object() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]test[asland_hi][aslano][asland_y]bar[aslano]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": {
            "y": "bar"
        }
    }));
}

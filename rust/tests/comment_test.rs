use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_object_comments_ignored() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![aslanc]This is a comment[asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz!");
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
fn parses_more_complex_string_with_object_comments_ignored() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![aslanc]This is a comment[asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here");
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
fn parses_more_complex_string_with_object_and_neighbor_aslano_comments_ignored() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here");
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
fn parses_more_complex_string_with_object_and_neighbor_aslano_into_outer_scope_comments_ignored() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here");
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
fn parses_more_complex_string_with_nested_objects_and_arrays_comments_ignored() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[asland_y][aslano][aslanc]This is a comment[asland_z]and it continues here[aslana][aslanc]This is a comment[asland_a][aslanc]This is a comment[aslano]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": "Hello ",
        "lo": "World!",
        "foo": {
            "bar": "Baz!"
        },
        "x": {
            "a": {},
            "y": {
                "z": "and it continues here"
            }
        }
    }));
}

use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_array() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_fruits][aslana][asland]Apple[asland]Banana[asland]Cherry");
    assert_eq!(result, json!({
        "_default": null,
        "fruits": ["Apple", "Banana", "Cherry"]
    }));
}

#[test]
fn parses_simple_string_with_array_indices() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_custom_array][aslana][asland_2]Third item[asland_0]First item[asland_1]Second item");
    assert_eq!(result, json!({
        "_default": null,
        "custom_array": ["First item", "Second item", "Third item"]
    }));
}

#[test]
fn parses_simple_string_with_mixed_array_indices() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D");
    assert_eq!(result, json!({
        "_default": null,
        "mixed_array": ["B", null, "A", "C", "D"]
    }));
}

#[test]
fn parses_simple_string_with_more_mixed_array_indices() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland_3]F[asland]G");
    assert_eq!(result, json!({
        "_default": null,
        "mixed_array": ["B", null, "A", "CF", "D", "E", "G"]
    }));
}

#[test]
fn parses_simple_string_with_more_mixed_array_indices_and_sibling_object() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland_3]F[asland]G[aslana][asland_other]Not in array");
    assert_eq!(result, json!({
        "_default": null,
        "mixed_array": ["B", null, "A", "CF", "D", "E", "G"],
        "other": "Not in array"
    }));
}

#[test]
fn parses_simple_string_with_nested_arrays() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland][aslana][asland]hi[asland]lo[aslana][asland]G");
    assert_eq!(result, json!({
        "_default": null,
        "mixed_array": ["B", null, "A", "C", "D", "E", ["hi", "lo"], "G"]
    }));
}

#[test]
fn parses_simple_string_with_multiple_arrays_same_key_should_override() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi][aslana][asland]foo[aslana][asland_hi][aslana][asland]bar[aslana]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": ["bar"]
    }));
}

#[test]
fn parses_simple_string_with_array_then_string_same_key_should_not_override_array() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi][aslana][asland]foo[aslana][asland_hi]not overriding");
    assert_eq!(result, json!({
        "_default": null,
        "hi": ["foo"]
    }));
}

#[test]
fn parses_simple_string_with_string_then_array_same_key_should_override_string_with_array() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi]test[asland_hi][aslana][asland]bar[aslana]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": ["bar"]
    }));
}

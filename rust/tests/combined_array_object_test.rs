use aslan::ASLANParser;
use serde_json::json;

#[test]
fn parses_simple_string_with_object_in_array() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_fruits][aslana][asland]Apple[asland]Banana[asland][aslano][asland_x]hi[asland_y]lo");
    assert_eq!(result, json!({
        "_default": null,
        "fruits": ["Apple", "Banana", {"x": "hi", "y": "lo"}]
    }));
}

#[test]
fn parses_simple_string_with_array_in_object() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_fruits][aslano][asland_best]Apple[asland_worst]Durian[asland_others][aslana][asland]Banana[asland]Pear[aslana][asland_next]Plum");
    assert_eq!(result, json!({
        "_default": null,
        "fruits": {
            "best": "Apple",
            "worst": "Durian",
            "others": ["Banana", "Pear"],
            "next": "Plum"
        }
    }));
}

#[test]
fn parses_simple_string_with_object_then_array_same_key_should_override_object() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi][aslano][asland_x]foo[aslano][asland_hi][aslana][asland]bar[aslana]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": ["bar"]
    }));
}

#[test]
fn parses_simple_string_with_array_then_object_same_key_should_override_array() {
    let mut parser = ASLANParser::new();
    let result = parser.parse("[asland_hi][aslana][asland]foo[aslana][asland_hi][aslano][asland_x]bar[aslano]");
    assert_eq!(result, json!({
        "_default": null,
        "hi": {"x": "bar"}
    }));
}

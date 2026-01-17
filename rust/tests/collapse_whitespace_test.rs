use aslan::ASLANParser;
use aslan::ASLANParserSettings;
use serde_json::json;

#[test]
fn parses_array_of_objects_without_whitespace_no_collapse_mode() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: false,
        ..Default::default()
    });
    let result = parser.parse("[asland_array][aslana][asland][aslano][asland_el1]Value 1[asland_el2]Value 2[aslano][asland][aslano][asland_el3]Value 3[asland_el4]Value 4");
    assert_eq!(result, json!({
        "_default": null,
        "array": [
            {
                "el1": "Value 1",
                "el2": "Value 2"
            },
            {
                "el3": "Value 3",
                "el4": "Value 4"
            }
        ]
    }));
}

#[test]
fn parses_array_of_objects_with_whitespace_collapse_mode() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        collapse_object_start_whitespace: true,
        ..Default::default()
    });
    let result = parser.parse("[asland_array]
      [aslana]
      [asland]
      [aslano]
      [asland_el1]Value 1[asland_el2]Value 2[aslano]
      [asland]
      [aslano]
      [asland_el3]Value 3[asland_el4]Value 4");
    assert_eq!(result, json!({
        "_default": null,
        "array": [
            {
                "el1": "Value 1",
                "el2": "Value 2"
            },
            {
                "el3": "Value 3",
                "el4": "Value 4"
            }
        ]
    }));
}

#[test]
fn parses_array_of_objects_default_field_name() {
    let mut parser = ASLANParser::with_settings(ASLANParserSettings {
        default_field_name: "_modified".to_string(),
        ..Default::default()
    });
    let result = parser.parse("[asland_hi]Hello [asland_lo]World!");
    assert_eq!(result, json!({
        "_modified": null,
        "hi": "Hello ",
        "lo": "World!"
    }));
}

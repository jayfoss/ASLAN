use aslan::{ASLANInstruction, ASLANParser};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn parses_simple_string_with_instructions() {
    let mut parser = ASLANParser::new();
    let content_events: Rc<RefCell<Vec<ASLANInstruction>>> = Rc::new(RefCell::new(Vec::new()));
    let end_events: Rc<RefCell<Vec<ASLANInstruction>>> = Rc::new(RefCell::new(Vec::new()));

    let content_events_clone = content_events.clone();
    parser.add_content_listener(move |event| {
        content_events_clone.borrow_mut().push(event.clone());
    });

    let end_events_clone = end_events.clone();
    parser.add_end_listener(move |event| {
        end_events_clone.borrow_mut().push(event.clone());
    });

    let result = parser.parse(
        "[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.",
    );

    assert_eq!(
        result,
        json!({
            "_default": null,
            "styled_text": [
                "This is bold and red text.",
                "This is italic and underlined text.",
                "This is large monospace text."
            ]
        })
    );

    // Verify we got content events
    let content = content_events.borrow();
    assert!(content.len() > 0, "Should have content events");

    // Verify we got end events
    let end = end_events.borrow();
    assert!(end.len() > 0, "Should have end events");

    // Check first content event instruction
    let first_event = &content[0];
    assert_eq!(first_event.instruction, "bold");
    assert_eq!(first_event.field_name, "styled_text");
}

#[test]
fn parses_simple_string_with_instructions_and_data_with_l_arg() {
    let mut parser = ASLANParser::new();
    let result = parser.parse(
        "[asland_styled_text:l][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.",
    );

    assert_eq!(
        result,
        json!({
            "_default": null,
            "styled_text": ["This is italic and blue text."]
        })
    );
}

#[test]
fn parses_simple_string_with_instructions_and_data_with_f_arg() {
    let mut parser = ASLANParser::new();
    let result = parser.parse(
        "[asland_styled_text:f][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.",
    );

    assert_eq!(
        result,
        json!({
            "_default": null,
            "styled_text": ["This is bold and red text."]
        })
    );
}

#[test]
fn parses_simple_string_with_instructions_no_part() {
    let mut parser = ASLANParser::new();
    let result = parser.parse(
        "[asland_test][aslana][asland][aslano][asland_styled_text][aslani_bold][aslani_color:red]This is bold and red text.[asland_next]Next item",
    );

    assert_eq!(
        result,
        json!({
            "_default": null,
            "test": [
                {
                    "next": "Next item",
                    "styled_text": "This is bold and red text."
                }
            ]
        })
    );
}

#[test]
fn event_listener_removal_works() {
    let mut parser = ASLANParser::new();
    let event_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

    let event_count_clone = event_count.clone();
    let key = parser.add_content_listener(move |_| {
        *event_count_clone.borrow_mut() += 1;
    });

    parser.parse_next("[asland_test][aslani_x]a");
    let count_before = *event_count.borrow();

    parser.remove_content_listener(&key);
    parser.parse_next("[aslani_y]b");
    let count_after = *event_count.borrow();

    // After removal, no new events should be counted
    assert_eq!(count_before, count_after);
}

#[test]
fn clear_event_listeners_works() {
    let mut parser = ASLANParser::new();
    let event_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

    let event_count_clone = event_count.clone();
    parser.add_content_listener(move |_| {
        *event_count_clone.borrow_mut() += 1;
    });

    parser.parse_next("[asland_test][aslani_x]a");
    let count_before = *event_count.borrow();

    parser.clear_event_listeners();
    parser.parse_next("[aslani_y]b");
    let count_after = *event_count.borrow();

    assert_eq!(count_before, count_after);
}

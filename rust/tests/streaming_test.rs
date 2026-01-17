use aslan::ASLANParser;
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn parse_next_streams_incrementally() {
    let mut parser = ASLANParser::new();
    
    // Parse in chunks to simulate streaming
    parser.parse_next("[asland_");
    parser.parse_next("greeting]");
    parser.parse_next("Hello, ");
    parser.parse_next("World!");
    parser.parse_next("[asland_name]");
    parser.parse_next("Alice");
    
    parser.close();
    
    let result = parser.get_result();
    assert_eq!(result, json!({
        "_default": null,
        "greeting": "Hello, World!",
        "name": "Alice"
    }));
}

#[test]
fn parse_next_emits_events_during_streaming() {
    let mut parser = ASLANParser::new();
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    
    let events_clone = events.clone();
    parser.add_content_listener(move |event| {
        events_clone.borrow_mut().push(format!(
            "content:{}:{}:{}",
            event.field_name,
            event.instruction,
            event.content
        ));
    });
    
    // Stream input character by character for the instruction part
    parser.parse_next("[asland_msg][aslani_bold]");
    
    // At this point, instruction is registered but no content yet
    let events_so_far = events.borrow().len();
    
    // Now stream content
    parser.parse_next("H");
    parser.parse_next("e");
    parser.parse_next("l");
    parser.parse_next("l");
    parser.parse_next("o");
    
    parser.close();
    
    // We should have received content events as characters were added
    let final_events = events.borrow();
    assert!(final_events.len() > events_so_far, "Should have emitted events during streaming");
    
    // Check the final result
    let result = parser.get_result();
    assert_eq!(result, json!({
        "_default": null,
        "msg": "Hello"
    }));
}

#[test]
fn parse_next_handles_split_delimiters() {
    let mut parser = ASLANParser::new();
    
    // Split a delimiter across multiple parse_next calls
    parser.parse_next("[asl");
    parser.parse_next("and_");
    parser.parse_next("test]");
    parser.parse_next("value");
    
    parser.close();
    
    let result = parser.get_result();
    assert_eq!(result, json!({
        "_default": null,
        "test": "value"
    }));
}

#[test]
fn parse_next_handles_nested_structures_incrementally() {
    let mut parser = ASLANParser::new();
    
    // Build a nested structure piece by piece
    parser.parse_next("[asland_user]");
    parser.parse_next("[aslano]");
    parser.parse_next("[asland_name]");
    parser.parse_next("Bob");
    parser.parse_next("[asland_age]");
    parser.parse_next("30");
    parser.parse_next("[aslano]");
    parser.parse_next("[asland_active]");
    parser.parse_next("true");
    
    parser.close();
    
    let result = parser.get_result();
    assert_eq!(result, json!({
        "_default": null,
        "user": {
            "name": "Bob",
            "age": "30"
        },
        "active": "true"
    }));
}

#[test]
fn parse_next_can_get_intermediate_results() {
    let mut parser = ASLANParser::new();
    
    // Parse first field
    parser.parse_next("[asland_first]one");
    
    // Check intermediate result
    let intermediate = parser.get_result();
    assert_eq!(intermediate, json!({
        "_default": null,
        "first": "one"
    }));
    
    // Parse second field
    parser.parse_next("[asland_second]two");
    parser.close();
    
    let final_result = parser.get_result();
    assert_eq!(final_result, json!({
        "_default": null,
        "first": "one",
        "second": "two"
    }));
}

#[test]
fn parse_next_with_end_events_shows_completed_fields() {
    let mut parser = ASLANParser::new();
    let completed_fields: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    
    let fields_clone = completed_fields.clone();
    parser.add_end_listener(move |event| {
        fields_clone.borrow_mut().push(format!(
            "{}={}",
            event.field_name,
            event.content
        ));
    });
    
    // Stream and watch for completion events
    parser.parse_next("[asland_a][aslani_x]alpha");
    parser.parse_next("[asland_b][aslani_y]beta");
    parser.parse_next("[asland_c][aslani_z]gamma");
    parser.close();
    
    let fields = completed_fields.borrow();
    
    // End events should have been emitted as each field completed
    assert!(fields.iter().any(|f| f.contains("a=alpha")));
    assert!(fields.iter().any(|f| f.contains("b=beta")));
    assert!(fields.iter().any(|f| f.contains("c=gamma")));
}

#[test]
fn parse_next_simulates_llm_token_streaming() {
    let mut parser = ASLANParser::new();
    let content_updates: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    
    let updates_clone = content_updates.clone();
    parser.add_content_listener(move |event| {
        updates_clone.borrow_mut().push(event.content.clone());
    });
    
    // Simulate tokens coming from an LLM
    let tokens = vec![
        "[asland_",
        "response]",
        "[aslani_",
        "markdown]",
        "Here ",
        "is ",
        "the ",
        "answer",
        ".",
    ];
    
    for token in tokens {
        parser.parse_next(token);
    }
    
    parser.close();
    
    let result = parser.get_result();
    assert_eq!(result, json!({
        "_default": null,
        "response": "Here is the answer."
    }));
    
    // Verify we got progressive content updates
    let updates = content_updates.borrow();
    assert!(updates.len() >= 1, "Should have received content updates");
}

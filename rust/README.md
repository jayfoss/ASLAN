# ASLAN Parser for Rust

A streaming parser for ASLAN (ASLAN Sufficiently Lax AI Notation) - an LLM stream compatible structured data format.

[![Crates.io](https://img.shields.io/crates/v/aslan.svg)](https://crates.io/crates/aslan)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

See the [full specification](https://github.com/jayfoss/ASLAN) in the main repository.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aslan = "0.1.0"
```

## Usage

### Basic Parsing

```rust
use aslan::ASLANParser;

let mut parser = ASLANParser::new();
let result = parser.parse("[asland_greeting]Hello [asland_name]World!");

// Result is a serde_json::Value:
// {
//   "_default": null,
//   "greeting": "Hello ",
//   "name": "World!"
// }
```

### Streaming with `parse_next`

ASLAN is designed for streaming LLM outputs. Use `parse_next` to feed tokens incrementally:

```rust
use aslan::ASLANParser;

let mut parser = ASLANParser::new();

// Simulate streaming tokens from an LLM
parser.parse_next("[asland_");
parser.parse_next("response]");
parser.parse_next("Here is ");
parser.parse_next("the answer.");

parser.close();
let result = parser.get_result();
```

### Event Listeners

Subscribe to events for real-time processing:

```rust
use aslan::ASLANParser;

let mut parser = ASLANParser::new();

// Listen for content updates
parser.add_content_listener(|event| {
    println!("Field: {}, Content: {}", event.field_name, event.content);
});

// Listen for field completions
parser.add_end_listener(|event| {
    println!("Completed: {} = {}", event.field_name, event.content);
});

parser.parse("[asland_msg]Hello!");
```

### Nested Objects and Arrays

```rust
use aslan::ASLANParser;

let mut parser = ASLANParser::new();

// Objects
let result = parser.parse(
    "[asland_user][aslano][asland_name]Alice[asland_age]30"
);
// { "user": { "name": "Alice", "age": "30" } }

// Arrays
let mut parser = ASLANParser::new();
let result = parser.parse(
    "[asland_items][aslana][asland]Apple[asland]Banana[asland]Cherry"
);
// { "items": ["Apple", "Banana", "Cherry"] }
```

### Parser Settings

```rust
use aslan::{ASLANParser, ASLANParserSettings};

let mut parser = ASLANParser::with_settings(ASLANParserSettings {
    strict_start: true,                      // Require [aslang] to start parsing
    strict_end: true,                        // Stop parsing at [aslans]
    collapse_object_start_whitespace: true,  // Collapse whitespace at object/array starts
    max_object_depth: Some(1),               // Limit object nesting to 1 level
    ..Default::default()
});
```

### Limiting Object Depth

The `max_object_depth` setting makes the `[aslano]` delimiter deterministic based on nesting level:

```rust
use aslan::{ASLANParser, ASLANParserSettings};

// With max_object_depth: Some(1), [aslano] is binary:
// - At depth 0: always opens an object
// - At depth 1: always closes the object
let mut parser = ASLANParser::with_settings(ASLANParserSettings {
    max_object_depth: Some(1),
    ..Default::default()
});

let result = parser.parse(
    "[asland_edit][aslano][asland_text]content[aslano]"
);
// { "_default": null, "edit": { "text": "content" } }
```

This is useful for LLM outputs with known structure depth, eliminating ambiguity from whitespace or empty content.

## Features

- **Streaming-first**: Parse character-by-character, perfect for LLM token streams
- **Fault-tolerant**: Designed to handle malformed input gracefully
- **Event-driven**: Subscribe to content, end, and end_data events
- **Nested structures**: Full support for objects, arrays, and mixed nesting
- **Instructions**: Emit metadata events without affecting the data structure
- **Comments & Escapes**: Skip content or escape ASLAN delimiters

## ASLAN Delimiters

| Suffix | Name | Description |
|--------|------|-------------|
| `d` | Data | Define a field: `[asland_fieldname]` |
| `o` | Object | Start/end object: `[aslano]` |
| `a` | Array | Start/end array: `[aslana]` |
| `i` | Instruction | Emit event: `[aslani_name:arg1:arg2]` |
| `p` | Part | Split into array: `[aslanp]` |
| `c` | Comment | Ignore until next delimiter: `[aslanc]` |
| `e` | Escape | Escape content: `[aslane_ID]...[aslane_ID]` |
| `v` | Void | Null value: `[aslanv]` |
| `g` | Go | Start ASLAN (with strict_start): `[aslang]` |
| `s` | Stop | Stop ASLAN (with strict_end): `[aslans]` |

## License

MIT

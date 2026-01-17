use crate::recent_items::RecentItems;
use crate::utils::generate_random_idempotency_key;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

/// Delimiter types in ASLAN
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ASLANDelimiterType {
    Data,
    Object,
    Instruction,
    Array,
    Comment,
    Escape,
    Part,
    Void,
    Go,
    Stop,
}

/// Duplicate key behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASLANDuplicateKeyBehavior {
    Append,
    KeepFirst,
    KeepLast,
}

/// Data insertion type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ASLANDataInsertionType {
    Default,
    Append,
    KeepFirst,
    KeepLast,
}

/// Parser states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ASLANParserState {
    GoDelimiter,
    StopDelimiter,
    Start,
    MaybeDelimiter,
    Delimiter,
    ReservedDelimiter,
    Object,
    Array,
    Comment,
    Escape,
    CommentDelimiter,
    EscapeDelimiter,
    EscapeDelimiterName,
    InstructionDelimiter,
    InstructionDelimiterName,
    InstructionDelimiterArgs,
    DataDelimiter,
    DataDelimiterName,
    DataDelimiterArgs,
    ObjectDelimiter,
    ArrayDelimiter,
    VoidDelimiter,
    PartDelimiter,
    Data,
    Go,
    Stop,
    Locked,
}

/// Delimiter data being parsed
#[derive(Debug, Clone)]
struct ASLANDelimiterData {
    prefix: Option<String>,
    suffix: Option<ASLANDelimiterType>,
    content: Option<String>,
    args: Vec<String>,
}

/// Key type - either string or numeric index
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ASLANKey {
    String(String),
    Index(i64),
}

impl ASLANKey {
    fn as_string(&self) -> String {
        match self {
            ASLANKey::String(s) => s.clone(),
            ASLANKey::Index(i) => i.to_string(),
        }
    }
}

/// A registered instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASLANRegisteredInstruction {
    pub key: String,
    pub name: String,
    pub index: usize,
    pub args: Vec<String>,
    pub part_index: usize,
}

/// Instruction event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASLANInstruction {
    pub content: String,
    pub part_index: usize,
    pub field_name: String,
    pub path: Vec<String>,
    pub structure: Value,
    pub instruction: String,
    pub args: Vec<String>,
    pub index: usize,
    pub multi_aslan_index: usize,
    pub tag: String,
}

/// Content part with instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASLANContentPart {
    pub value: String,
    pub part_index: usize,
    pub instructions: Vec<ASLANInstructionInfo>,
}

/// Instruction info for end_data events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASLANInstructionInfo {
    pub name: String,
    pub args: Vec<String>,
    pub index: usize,
}

/// End data instruction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASLANEndDataInstruction {
    pub content: Vec<ASLANContentPart>,
    pub field_name: String,
    pub path: Vec<String>,
    pub structure: Value,
    pub multi_aslan_index: usize,
    pub tag: String,
}

/// Parser stack frame
#[derive(Debug, Clone)]
struct ASLANParserStateStackFrame {
    inner_result: Value,
    data_insertion_types: HashMap<String, ASLANDataInsertionType>,
    data_insertion_locks: HashMap<String, bool>,
    current_key: ASLANKey,
    min_array_index: i64,
    void_fields: HashMap<String, bool>,
    already_seen_duplicate_keys: HashMap<String, bool>,
    implicit_arrays: HashMap<String, bool>,
    registered_instructions: Vec<ASLANRegisteredInstruction>,
}

/// Parser settings
#[derive(Debug, Clone)]
pub struct ASLANParserSettings {
    pub prefix: String,
    pub default_field_name: String,
    pub strict_start: bool,
    pub strict_end: bool,
    pub emittable_events: EmittableEvents,
    pub multi_aslan_output: bool,
    pub collapse_object_start_whitespace: bool,
    pub append_separator: String,
}

#[derive(Debug, Clone)]
pub struct EmittableEvents {
    pub content: bool,
    pub end: bool,
    pub end_data: bool,
}

impl Default for EmittableEvents {
    fn default() -> Self {
        Self {
            content: true,
            end: true,
            end_data: true,
        }
    }
}

impl Default for ASLANParserSettings {
    fn default() -> Self {
        Self {
            prefix: "aslan".to_string(),
            default_field_name: "_default".to_string(),
            strict_start: false,
            strict_end: false,
            emittable_events: EmittableEvents::default(),
            multi_aslan_output: false,
            collapse_object_start_whitespace: true,
            append_separator: String::new(),
        }
    }
}

/// Event handler types
pub type ContentEventHandler = Box<dyn FnMut(&ASLANInstruction)>;
pub type EndEventHandler = Box<dyn FnMut(&ASLANInstruction)>;
pub type EndDataEventHandler = Box<dyn FnMut(&ASLANEndDataInstruction)>;

/// Event listeners
struct ASLANEventListeners {
    content: Vec<(String, ContentEventHandler)>,
    end: Vec<(String, EndEventHandler)>,
    end_data: Vec<(String, EndDataEventHandler)>,
}

impl Default for ASLANEventListeners {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            end: Vec::new(),
            end_data: Vec::new(),
        }
    }
}

/// The ASLAN Parser
pub struct ASLANParser {
    state: ASLANParserState,
    stack: Vec<ASLANParserStateStackFrame>,
    current_delimiter: Option<ASLANDelimiterData>,
    current_value: String,
    delimiter_buffer: String,
    delimiter_open_substring: String,
    recent_delimiters: RecentItems<ASLANDelimiterType>,
    current_escape_delimiter: Option<String>,
    parsing_locked: bool,
    parser_settings: ASLANParserSettings,
    multi_aslan_results: Vec<Value>,
    did_stop: bool,
    event_listeners: ASLANEventListeners,
    listener_idempotency_keys: HashSet<String>,
}

impl ASLANParser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self::with_settings(ASLANParserSettings::default())
    }

    /// Create a new parser with custom settings
    pub fn with_settings(settings: ASLANParserSettings) -> Self {
        let delimiter_open_substring = format!("[{}", settings.prefix);
        let default_field_name = settings.default_field_name.clone();
        let strict_start = settings.strict_start;

        let initial_result = json!({ default_field_name.clone(): "" });

        let initial_frame = ASLANParserStateStackFrame {
            inner_result: initial_result.clone(),
            data_insertion_types: {
                let mut map = HashMap::new();
                map.insert(default_field_name.clone(), ASLANDataInsertionType::Default);
                map
            },
            data_insertion_locks: {
                let mut map = HashMap::new();
                map.insert(default_field_name.clone(), false);
                map
            },
            current_key: ASLANKey::String(default_field_name),
            min_array_index: 0,
            void_fields: HashMap::new(),
            already_seen_duplicate_keys: HashMap::new(),
            implicit_arrays: HashMap::new(),
            registered_instructions: Vec::new(),
        };

        let initial_state = if strict_start {
            ASLANParserState::Locked
        } else {
            ASLANParserState::Start
        };

        let parser = Self {
            state: initial_state,
            stack: vec![initial_frame],
            current_delimiter: None,
            current_value: String::new(),
            delimiter_buffer: String::new(),
            delimiter_open_substring,
            recent_delimiters: RecentItems::default(),
            current_escape_delimiter: None,
            parsing_locked: strict_start,
            parser_settings: settings,
            multi_aslan_results: vec![initial_result],
            did_stop: true,
            event_listeners: ASLANEventListeners::default(),
            listener_idempotency_keys: HashSet::new(),
        };

        parser
    }

    /// Parse a complete input string and return the result
    pub fn parse(&mut self, input: &str) -> Value {
        for ch in input.chars() {
            self.handle_next_char(ch);
        }
        self.close();

        if self.parser_settings.multi_aslan_output {
            Value::Array(self.multi_aslan_results.clone())
        } else {
            self.stack[0].inner_result.clone()
        }
    }

    /// Parse input incrementally (streaming)
    pub fn parse_next(&mut self, input: &str) {
        for ch in input.chars() {
            self.handle_next_char(ch);
        }
    }

    /// Close the parser and finalize results
    pub fn close(&mut self) {
        self.emit_end_events_if_required();
        self.emit_end_data_events_if_required();
        self.store_current_value();
        self.sync_stack_to_root();
    }

    /// Get the current result
    pub fn get_result(&self) -> Value {
        self.stack[0].inner_result.clone()
    }

    /// Get all multi-aslan results
    pub fn get_results(&self) -> Vec<Value> {
        self.multi_aslan_results.clone()
    }

    /// Reset the parser state
    pub fn reset(&mut self) {
        let default_field_name = self.parser_settings.default_field_name.clone();
        let initial_result = json!({ default_field_name.clone(): "" });

        self.stack = vec![ASLANParserStateStackFrame {
            inner_result: initial_result,
            data_insertion_types: {
                let mut map = HashMap::new();
                map.insert(default_field_name.clone(), ASLANDataInsertionType::Default);
                map
            },
            data_insertion_locks: {
                let mut map = HashMap::new();
                map.insert(default_field_name.clone(), false);
                map
            },
            current_key: ASLANKey::String(default_field_name),
            min_array_index: 0,
            void_fields: HashMap::new(),
            already_seen_duplicate_keys: HashMap::new(),
            implicit_arrays: HashMap::new(),
            registered_instructions: Vec::new(),
        }];
    }

    /// Add an event listener for content events
    pub fn add_content_listener<F>(&mut self, callback: F) -> String
    where
        F: FnMut(&ASLANInstruction) + 'static,
    {
        let key = generate_random_idempotency_key();
        self.add_content_listener_with_key(key.clone(), callback);
        key
    }

    /// Add an event listener with a specific idempotency key
    pub fn add_content_listener_with_key<F>(&mut self, key: String, callback: F)
    where
        F: FnMut(&ASLANInstruction) + 'static,
    {
        if self.listener_idempotency_keys.contains(&key) {
            return;
        }
        self.listener_idempotency_keys.insert(key.clone());
        self.event_listeners.content.push((key, Box::new(callback)));
    }

    /// Add an event listener for end events
    pub fn add_end_listener<F>(&mut self, callback: F) -> String
    where
        F: FnMut(&ASLANInstruction) + 'static,
    {
        let key = generate_random_idempotency_key();
        self.add_end_listener_with_key(key.clone(), callback);
        key
    }

    /// Add an end event listener with a specific idempotency key
    pub fn add_end_listener_with_key<F>(&mut self, key: String, callback: F)
    where
        F: FnMut(&ASLANInstruction) + 'static,
    {
        if self.listener_idempotency_keys.contains(&key) {
            return;
        }
        self.listener_idempotency_keys.insert(key.clone());
        self.event_listeners.end.push((key, Box::new(callback)));
    }

    /// Add an event listener for end_data events
    pub fn add_end_data_listener<F>(&mut self, callback: F) -> String
    where
        F: FnMut(&ASLANEndDataInstruction) + 'static,
    {
        let key = generate_random_idempotency_key();
        self.add_end_data_listener_with_key(key.clone(), callback);
        key
    }

    /// Add an end_data event listener with a specific idempotency key
    pub fn add_end_data_listener_with_key<F>(&mut self, key: String, callback: F)
    where
        F: FnMut(&ASLANEndDataInstruction) + 'static,
    {
        if self.listener_idempotency_keys.contains(&key) {
            return;
        }
        self.listener_idempotency_keys.insert(key.clone());
        self.event_listeners.end_data.push((key, Box::new(callback)));
    }

    /// Remove a content event listener by key
    pub fn remove_content_listener(&mut self, key: &str) {
        self.listener_idempotency_keys.remove(key);
        self.event_listeners.content.retain(|(k, _)| k != key);
    }

    /// Remove an end event listener by key
    pub fn remove_end_listener(&mut self, key: &str) {
        self.listener_idempotency_keys.remove(key);
        self.event_listeners.end.retain(|(k, _)| k != key);
    }

    /// Remove an end_data event listener by key
    pub fn remove_end_data_listener(&mut self, key: &str) {
        self.listener_idempotency_keys.remove(key);
        self.event_listeners.end_data.retain(|(k, _)| k != key);
    }

    /// Clear all event listeners
    pub fn clear_event_listeners(&mut self) {
        self.listener_idempotency_keys.clear();
        self.event_listeners = ASLANEventListeners::default();
    }

    // Private helper methods

    fn get_current_key(&self) -> &ASLANKey {
        &self.stack.last().unwrap().current_key
    }

    fn get_current_key_string(&self) -> String {
        self.get_current_key().as_string()
    }

    fn set_current_key(&mut self, key: ASLANKey) {
        self.stack.last_mut().unwrap().current_key = key;
    }

    fn get_min_array_index(&self) -> i64 {
        self.stack.last().unwrap().min_array_index
    }

    fn set_min_array_index(&mut self, index: i64) {
        self.stack.last_mut().unwrap().min_array_index = index;
    }

    fn get_latest_result(&self) -> &Value {
        &self.stack.last().unwrap().inner_result
    }

    fn get_latest_result_mut(&mut self) -> &mut Value {
        &mut self.stack.last_mut().unwrap().inner_result
    }

    fn get_2nd_most_recent_material_delimiter(&self) -> Option<&ASLANDelimiterType> {
        let mut excluded = HashSet::new();
        excluded.insert(ASLANDelimiterType::Comment);
        excluded.insert(ASLANDelimiterType::Escape);
        self.recent_delimiters.get_nth_most_recent_not_in(2, &excluded)
    }

    fn exit_delimiter_into_data(&mut self, ch: char) {
        self.current_value.push_str(&self.delimiter_buffer);
        self.current_value.push(ch);
        self.delimiter_buffer.clear();
        self.current_delimiter = None;
        self.state = ASLANParserState::Data;
    }

    fn handle_next_char(&mut self, ch: char) {
        match self.state {
            ASLANParserState::GoDelimiter => self.handle_go_delimiter(ch),
            ASLANParserState::StopDelimiter => self.handle_stop_delimiter(ch),
            ASLANParserState::Go => self.handle_go(ch),
            ASLANParserState::Stop => self.handle_stop(ch),
            ASLANParserState::Start => self.handle_start(ch),
            ASLANParserState::MaybeDelimiter => self.handle_maybe_delimiter(ch),
            ASLANParserState::Delimiter => self.handle_delimiter(ch),
            ASLANParserState::ReservedDelimiter => self.handle_reserved_delimiter(ch),
            ASLANParserState::Object => self.handle_object(ch),
            ASLANParserState::Array => self.handle_array(ch),
            ASLANParserState::Comment => self.handle_comment(ch),
            ASLANParserState::Escape => self.handle_escape(ch),
            ASLANParserState::InstructionDelimiter => self.handle_instruction_delimiter(ch),
            ASLANParserState::InstructionDelimiterName => self.handle_instruction_delimiter_name(ch),
            ASLANParserState::InstructionDelimiterArgs => self.handle_instruction_delimiter_args(ch),
            ASLANParserState::DataDelimiter => self.handle_data_delimiter(ch),
            ASLANParserState::DataDelimiterName => self.handle_data_delimiter_name(ch),
            ASLANParserState::DataDelimiterArgs => self.handle_data_delimiter_args(ch),
            ASLANParserState::ObjectDelimiter => self.handle_object_delimiter(ch),
            ASLANParserState::ArrayDelimiter => self.handle_array_delimiter(ch),
            ASLANParserState::VoidDelimiter => self.handle_void_delimiter(ch),
            ASLANParserState::CommentDelimiter => self.handle_comment_delimiter(ch),
            ASLANParserState::EscapeDelimiter => self.handle_escape_delimiter(ch),
            ASLANParserState::EscapeDelimiterName => self.handle_escape_delimiter_name(ch),
            ASLANParserState::PartDelimiter => self.handle_part_delimiter(ch),
            ASLANParserState::Data => self.handle_data(ch),
            ASLANParserState::Locked => self.handle_locked(ch),
        }
    }

    fn handle_locked(&mut self, ch: char) {
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer = ch.to_string();
        }
    }

    fn handle_go_delimiter(&mut self, ch: char) {
        if ch == ']' {
            // Spec: Go delimiters have no <CONTENT> or args
            // VALID GO DELIMITER
            self.state = ASLANParserState::Go;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            self.parsing_locked = false;
            if self.parser_settings.strict_start && !self.did_stop {
                self.close();
                self.reset();
                self.multi_aslan_results.push(self.stack[0].inner_result.clone());
            }
            self.did_stop = false;
            return;
        }
        // Spec: Go delimiters have no <CONTENT> or args
        // INVALID GO DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_stop_delimiter(&mut self, ch: char) {
        if ch == ']' {
            // Spec: Stop delimiters have no <CONTENT> or args
            // VALID STOP DELIMITER
            self.state = ASLANParserState::Stop;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            if self.parser_settings.strict_end {
                if self.parser_settings.strict_start {
                    self.parsing_locked = true;
                }
                self.close();
                self.reset();
                self.multi_aslan_results.push(self.stack[0].inner_result.clone());
                self.state = ASLANParserState::Start;
                self.did_stop = true;
            }
            return;
        }
        // Spec: Stop delimiters have no <CONTENT> or args
        // INVALID STOP DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_start(&mut self, ch: char) {
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
        } else {
            self.state = ASLANParserState::Data;
            self.current_value.push(ch);
        }
    }

    fn handle_maybe_delimiter(&mut self, ch: char) {
        if self.delimiter_buffer.len() > self.delimiter_open_substring.len() {
            self.state = ASLANParserState::Data;
            self.current_value.push(ch);
            return;
        }
        let expected_char = self.delimiter_open_substring.chars().nth(self.delimiter_buffer.len());
        if expected_char == Some(ch) {
            self.delimiter_buffer.push(ch);
            if self.delimiter_buffer == self.delimiter_open_substring {
                self.state = ASLANParserState::Delimiter;
            }
            return;
        }
        self.exit_delimiter_into_data(ch);
    }

    fn handle_delimiter(&mut self, ch: char) {
        if self.parsing_locked && ch != 'g' && !self.parser_settings.strict_start {
            self.state = ASLANParserState::Locked;
            return;
        }
        self.current_delimiter = Some(ASLANDelimiterData {
            prefix: Some(self.parser_settings.prefix.clone()),
            suffix: None,
            content: None,
            args: Vec::new(),
        });

        match ch {
            'd' => {
                self.state = ASLANParserState::DataDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Data);
                self.delimiter_buffer.push(ch);
            }
            'o' => {
                self.state = ASLANParserState::ObjectDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Object);
                self.delimiter_buffer.push(ch);
            }
            'i' => {
                self.state = ASLANParserState::InstructionDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Instruction);
                self.delimiter_buffer.push(ch);
            }
            'a' => {
                self.state = ASLANParserState::ArrayDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Array);
                self.delimiter_buffer.push(ch);
            }
            'c' => {
                self.state = ASLANParserState::CommentDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Comment);
                self.delimiter_buffer.push(ch);
            }
            'e' => {
                self.state = ASLANParserState::EscapeDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Escape);
                self.delimiter_buffer.push(ch);
            }
            'p' => {
                self.state = ASLANParserState::PartDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Part);
                self.delimiter_buffer.push(ch);
            }
            'v' => {
                self.state = ASLANParserState::VoidDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Void);
                self.delimiter_buffer.push(ch);
            }
            'g' => {
                self.state = ASLANParserState::GoDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Go);
                self.delimiter_buffer.push(ch);
            }
            's' => {
                self.state = ASLANParserState::StopDelimiter;
                self.current_delimiter.as_mut().unwrap().suffix = Some(ASLANDelimiterType::Stop);
                self.delimiter_buffer.push(ch);
            }
            _ => {
                if ch.is_ascii_alphanumeric() {
                    self.state = ASLANParserState::ReservedDelimiter;
                    self.delimiter_buffer.push(ch);
                    return;
                }
                self.exit_delimiter_into_data(ch);
                return;
            }
        }

        if let Some(ref delimiter) = self.current_delimiter {
            if let Some(suffix) = delimiter.suffix {
                self.recent_delimiters.add(suffix);
            }
        }
    }

    fn handle_reserved_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch != ']' {
            // Spec: Reserved delimiters contain no <CONTENT> or args
            // INVALID RESERVED DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        self.delimiter_buffer.clear();
        self.state = ASLANParserState::Data;
        self.current_value.clear();
    }

    fn handle_object_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            // Spec: Object delimiters have no <CONTENT> or args
            // VALID OBJECT DELIMITER
            self.state = ASLANParserState::Object;
            self.delimiter_buffer.clear();
            let second_most_recent = self.get_2nd_most_recent_material_delimiter().copied();
            
            if self.get_object_safe_latest_result() || second_most_recent != Some(ASLANDelimiterType::Data) {
                let current_key = self.get_current_key_string();
                let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
                
                if !is_object || second_most_recent != Some(ASLANDelimiterType::Data) {
                    let already_seen = self.stack.last().unwrap().already_seen_duplicate_keys.get(&current_key).copied().unwrap_or(false);
                    if already_seen {
                        self.stack.last_mut().unwrap().already_seen_duplicate_keys.insert(current_key, false);
                        self.create_new_object();
                        return;
                    }
                    if self.stack.len() > 1 {
                        self.emit_end_events_if_required();
                        self.emit_end_data_events_if_required();
                        self.sync_stack_to_root();
                        self.stack.pop();
                    }
                } else {
                    self.create_new_object();
                }
                return;
            }
            self.create_new_object();
            return;
        }
        // Spec: Object delimiters have no <CONTENT> or args
        // INVALID OBJECT DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn get_object_safe_latest_result(&self) -> bool {
        let current_key = self.get_current_key_string();
        if let Some(value) = self.get_value_at_key(&current_key) {
            if let Some(s) = value.as_str() {
                if self.parser_settings.collapse_object_start_whitespace {
                    return !s.trim().is_empty();
                }
                return !s.is_empty();
            }
            return !value.is_null();
        }
        false
    }

    fn get_value_at_key(&self, key: &str) -> Option<&Value> {
        let latest = self.get_latest_result();
        if let Some(obj) = latest.as_object() {
            obj.get(key)
        } else if let Some(arr) = latest.as_array() {
            key.parse::<usize>().ok().and_then(|i| arr.get(i))
        } else {
            None
        }
    }

    fn create_new_object(&mut self) {
        self.current_value.clear();
        let current_key = self.get_current_key_string();
        
        // Set the value to an empty object
        let latest = self.get_latest_result_mut();
        if let Some(obj) = latest.as_object_mut() {
            obj.insert(current_key.clone(), json!({}));
        } else if let Some(arr) = latest.as_array_mut() {
            if let Ok(idx) = current_key.parse::<usize>() {
                while arr.len() <= idx {
                    arr.push(Value::Null);
                }
                arr[idx] = json!({});
            }
        }

        // Get the new inner result reference
        let new_inner = self.get_value_at_key(&current_key).cloned().unwrap_or(json!({}));
        
        self.stack.push(ASLANParserStateStackFrame {
            inner_result: new_inner,
            data_insertion_types: HashMap::new(),
            data_insertion_locks: HashMap::new(),
            current_key: ASLANKey::String(self.parser_settings.default_field_name.clone()),
            min_array_index: 0,
            void_fields: HashMap::new(),
            already_seen_duplicate_keys: HashMap::new(),
            implicit_arrays: HashMap::new(),
            registered_instructions: Vec::new(),
        });
    }

    fn handle_instruction_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            // Spec: Instruction delimiters must contain <CONTENT>
            // INVALID INSTRUCTION DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        if ch == '_' {
            // Spec: Instruction delimiters must contain <CONTENT>
            self.state = ASLANParserState::InstructionDelimiterName;
            self.delimiter_buffer.push(ch);
            self.current_delimiter.as_mut().unwrap().content = Some(String::new());
            self.current_value.clear();
            return;
        }
        // Spec: Instruction delimiters must contain <CONTENT>
        // INVALID INSTRUCTION DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_instruction_delimiter_name(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        
        let content = self.current_delimiter.as_ref().unwrap().content.clone().unwrap_or_default();
        
        if !content.is_empty() && ch == ':' {
            if content.ends_with('_') {
                // Spec: Delimiter <CONTENT> may not end with an underscore.
                // INVALID INSTRUCTION DELIMITER
                return self.exit_delimiter_into_data(ch);
            }
            // Spec: Instructions may have arguments.
            self.state = ASLANParserState::InstructionDelimiterArgs;
            self.current_delimiter.as_mut().unwrap().args = vec![String::new()];
            self.current_value.clear();
            self.delimiter_buffer.push(ch);
            return;
        }
        if ch == '_' && content.is_empty() {
            // Spec: Delimiter <CONTENT> may not start with an underscore.
            // INVALID INSTRUCTION DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            if content.ends_with('_') {
                // Spec: Delimiter <CONTENT> may not end with an underscore.
                // INVALID INSTRUCTION DELIMITER
                return self.exit_delimiter_into_data(ch);
            }
            // Spec: Instruction delimiter of the form [<PREFIX>i_<CONTENT>]
            // VALID INSTRUCTION DELIMITER
            let (index, part_index) = self.get_instruction_indices();
            self.state = ASLANParserState::Data;
            
            let current_key = self.get_current_key_string();
            let already_seen = self.stack.last().unwrap().already_seen_duplicate_keys.get(&current_key).copied().unwrap_or(false);
            let is_keep_first = self.stack.last().unwrap().data_insertion_types.get(&current_key) == Some(&ASLANDataInsertionType::KeepFirst);
            
            if !already_seen || !is_keep_first {
                let args = self.current_delimiter.as_ref().unwrap().args.clone();
                let content = self.current_delimiter.as_ref().unwrap().content.clone().unwrap_or_default();
                self.register_instruction(ASLANRegisteredInstruction {
                    name: content,
                    index,
                    args,
                    key: current_key.clone(),
                    part_index,
                });
                
                let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
                if !is_object {
                    self.emit_content_events_for_primitive();
                }
                let is_implicit_array = self.stack.last().unwrap().implicit_arrays.get(&current_key).copied().unwrap_or(false);
                if is_implicit_array {
                    self.emit_content_events_for_implicit_array();
                }
            }
            self.delimiter_buffer.clear();
            self.current_value.clear();
            return;
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            // Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
            // INVALID INSTRUCTION DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        self.current_delimiter.as_mut().unwrap().content.as_mut().unwrap().push(ch);
        self.delimiter_buffer.push(ch);
    }

    fn handle_instruction_delimiter_args(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == ']' {
            // Spec: Instruction delimiter of the form [<PREFIX>i_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
            // VALID INSTRUCTION DELIMITER
            let (index, part_index) = self.get_instruction_indices();
            self.state = ASLANParserState::Data;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            
            let current_key = self.get_current_key_string();
            let already_seen = self.stack.last().unwrap().already_seen_duplicate_keys.get(&current_key).copied().unwrap_or(false);
            let is_keep_first = self.stack.last().unwrap().data_insertion_types.get(&current_key) == Some(&ASLANDataInsertionType::KeepFirst);
            
            if !already_seen || !is_keep_first {
                let args = self.current_delimiter.as_ref().unwrap().args.clone();
                let content = self.current_delimiter.as_ref().unwrap().content.clone().unwrap_or_default();
                self.register_instruction(ASLANRegisteredInstruction {
                    name: content,
                    index,
                    args,
                    key: current_key.clone(),
                    part_index,
                });
                
                let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
                if !is_object {
                    self.emit_content_events_for_primitive();
                }
                let is_implicit_array = self.stack.last().unwrap().implicit_arrays.get(&current_key).copied().unwrap_or(false);
                if is_implicit_array {
                    self.emit_content_events_for_implicit_array();
                }
            }
            return;
        }
        if ch == ':' {
            // Start a new arg
            self.delimiter_buffer.push(ch);
            self.current_delimiter.as_mut().unwrap().args.push(String::new());
            return;
        }
        // Add to the current arg
        let args = &mut self.current_delimiter.as_mut().unwrap().args;
        if let Some(last) = args.last_mut() {
            last.push(ch);
        }
        self.delimiter_buffer.push(ch);
    }

    fn get_instruction_indices(&self) -> (usize, usize) {
        let current_key = self.get_current_key_string();
        let value = self.get_value_at_key(&current_key);
        
        if let Some(arr) = value.and_then(|v| v.as_array()) {
            let index = arr.last().and_then(|v| v.as_str()).map(|s| s.len()).unwrap_or(0);
            let part_index = arr.len().saturating_sub(1);
            (index, part_index)
        } else if let Some(s) = value.and_then(|v| v.as_str()) {
            (s.len(), 0)
        } else {
            (0, 0)
        }
    }

    fn register_instruction(&mut self, instruction: ASLANRegisteredInstruction) {
        self.stack.last_mut().unwrap().registered_instructions.push(instruction);
    }

    fn handle_data_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            let is_array = self.get_latest_result().is_array();
            if is_array {
                // Spec: Data delimiters can have no <CONTENT> or args if the current result is an array.
                // VALID DATA DELIMITER
                self.state = ASLANParserState::Data;
                self.delimiter_buffer.clear();
                self.current_value.clear();
                self.emit_end_events_if_required();
                self.emit_end_data_events_if_required();
                self.next_key();
                return;
            }
            // Spec: Data delimiters must contain <CONTENT> if the current result is not an array.
            // INVALID DATA DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        if ch == '_' {
            self.state = ASLANParserState::DataDelimiterName;
            self.delimiter_buffer.push(ch);
            self.current_delimiter.as_mut().unwrap().content = Some(String::new());
            self.current_value.clear();
            return;
        }
        // Spec: Data delimiters must be valid of the form [<PREFIX>d_<CONTENT>] or [<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
        // INVALID DATA DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_data_delimiter_name(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        
        let content = self.current_delimiter.as_ref().unwrap().content.clone().unwrap_or_default();
        
        if !content.is_empty() && ch == ':' {
            if content.ends_with('_') {
                // Spec: Delimiter <CONTENT> may not end with an underscore.
                // INVALID DATA DELIMITER
                return self.exit_delimiter_into_data(ch);
            }
            // Spec: Data may have arguments.
            self.state = ASLANParserState::DataDelimiterArgs;
            self.current_delimiter.as_mut().unwrap().args = vec![String::new()];
            self.current_value.clear();
            self.delimiter_buffer.push(ch);
            self.emit_end_events_if_required();
            self.emit_end_data_events_if_required();
            self.next_key();
            return;
        }
        if ch == '_' && content.is_empty() {
            // Spec: Delimiter <CONTENT> may not start with an underscore.
            // INVALID DATA DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            if content.ends_with('_') {
                // Spec: Delimiter <CONTENT> may not end with an underscore.
                // INVALID DATA DELIMITER
                return self.exit_delimiter_into_data(ch);
            }
            // Spec: Data delimiter of the form [<PREFIX>d_<CONTENT>]
            // VALID DATA DELIMITER
            self.state = ASLANParserState::Data;
            self.emit_end_events_if_required();
            self.emit_end_data_events_if_required();
            self.next_key();
            self.delimiter_buffer.clear();
            self.set_data_insertion_type(ASLANDataInsertionType::Default);
            
            let current_key = self.get_current_key_string();
            let already_seen = self.stack.last().unwrap().already_seen_duplicate_keys.get(&current_key).copied().unwrap_or(false);
            let value_exists = self.get_value_at_key(&current_key).is_some();
            let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
            
            if already_seen && value_exists && !is_object {
                self.current_value = self.parser_settings.append_separator.clone();
                self.store_current_value();
            }
            self.current_value.clear();
            return;
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            // Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
            // INVALID DATA DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        self.current_delimiter.as_mut().unwrap().content.as_mut().unwrap().push(ch);
        self.delimiter_buffer.push(ch);
    }

    fn handle_data_delimiter_args(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == ']' {
            // Spec: Data delimiter of the form [<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
            // VALID DATA DELIMITER
            self.state = ASLANParserState::Data;
            self.delimiter_buffer.clear();
            
            let arg = self.current_delimiter.as_ref().and_then(|d| d.args.first()).cloned().unwrap_or_default();
            match arg.as_str() {
                "a" => self.set_data_insertion_type(ASLANDataInsertionType::Append),
                "f" => self.set_data_insertion_type(ASLANDataInsertionType::KeepFirst),
                "l" => self.set_data_insertion_type(ASLANDataInsertionType::KeepLast),
                _ => self.set_data_insertion_type(ASLANDataInsertionType::Default),
            }
            self.emit_end_events_if_required();
            self.emit_end_data_events_if_required();
            
            let current_key = self.get_current_key_string();
            let already_seen = self.stack.last().unwrap().already_seen_duplicate_keys.get(&current_key).copied().unwrap_or(false);
            let insertion_type = self.stack.last().unwrap().data_insertion_types.get(&current_key).copied();
            let value_exists = self.get_value_at_key(&current_key).is_some();
            let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
            
            if already_seen 
                && (insertion_type == Some(ASLANDataInsertionType::Append) || insertion_type == Some(ASLANDataInsertionType::Default))
                && value_exists 
                && !is_object 
            {
                self.current_value = self.parser_settings.append_separator.clone();
                self.store_current_value();
            }
            self.current_value.clear();
            return;
        }
        if ch == ':' {
            // Start a new arg
            self.delimiter_buffer.push(ch);
            self.current_delimiter.as_mut().unwrap().args.push(String::new());
            return;
        }
        // Add to the current arg
        let args = &mut self.current_delimiter.as_mut().unwrap().args;
        if let Some(last) = args.last_mut() {
            last.push(ch);
        }
        self.delimiter_buffer.push(ch);
    }

    fn handle_array_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            // Spec: Array delimiters have no <CONTENT> or args
            // VALID ARRAY DELIMITER
            self.state = ASLANParserState::Array;
            self.delimiter_buffer.clear();
            let second_most_recent = self.get_2nd_most_recent_material_delimiter().copied();
            
            if self.get_object_safe_latest_result() || second_most_recent != Some(ASLANDelimiterType::Data) {
                let current_key = self.get_current_key_string();
                let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
                
                if !is_object || second_most_recent != Some(ASLANDelimiterType::Data) {
                    let already_seen = self.stack.last().unwrap().already_seen_duplicate_keys.get(&current_key).copied().unwrap_or(false);
                    if already_seen {
                        self.stack.last_mut().unwrap().already_seen_duplicate_keys.insert(current_key, false);
                        self.create_new_array();
                        return;
                    }
                    if self.stack.len() > 1 {
                        self.emit_end_events_if_required();
                        self.emit_end_data_events_if_required();
                        self.sync_stack_to_root();
                        self.stack.pop();
                    }
                } else {
                    self.create_new_array();
                }
                return;
            }
            self.create_new_array();
            return;
        }
        // Spec: Array delimiters have no <CONTENT> or args
        // INVALID ARRAY DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn create_new_array(&mut self) {
        self.current_value.clear();
        let current_key = self.get_current_key_string();
        
        // Set the value to an empty array
        let latest = self.get_latest_result_mut();
        if let Some(obj) = latest.as_object_mut() {
            obj.insert(current_key.clone(), json!([]));
        } else if let Some(arr) = latest.as_array_mut() {
            if let Ok(idx) = current_key.parse::<usize>() {
                while arr.len() <= idx {
                    arr.push(Value::Null);
                }
                arr[idx] = json!([]);
            }
        }

        // Get the new inner result reference
        let new_inner = self.get_value_at_key(&current_key).cloned().unwrap_or(json!([]));
        
        self.stack.push(ASLANParserStateStackFrame {
            inner_result: new_inner,
            data_insertion_types: HashMap::new(),
            data_insertion_locks: HashMap::new(),
            current_key: ASLANKey::Index(-1),
            min_array_index: 0,
            void_fields: HashMap::new(),
            already_seen_duplicate_keys: HashMap::new(),
            implicit_arrays: HashMap::new(),
            registered_instructions: Vec::new(),
        });
    }

    fn handle_void_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            // Spec: Void delimiters have no <CONTENT> or args
            // VALID VOID DELIMITER
            self.state = ASLANParserState::Data;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            let current_key = self.get_current_key_string();
            self.stack.last_mut().unwrap().void_fields.insert(current_key, true);
            return;
        }
        // Spec: Void delimiters have no <CONTENT> or args
        // INVALID VOID DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_comment_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            // Spec: Comment delimiters have no <CONTENT> or args
            // VALID COMMENT DELIMITER
            self.state = ASLANParserState::Comment;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            return;
        }
        // Spec: Comment delimiters have no <CONTENT> or args
        // INVALID COMMENT DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_escape_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == ']' {
            // Spec: Escape delimiters must contain <CONTENT>
            // INVALID ESCAPE DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        if ch == '_' {
            self.state = ASLANParserState::EscapeDelimiterName;
            self.delimiter_buffer.push(ch);
            self.current_delimiter.as_mut().unwrap().content = Some(String::new());
            self.current_value.clear();
            return;
        }
        // Spec: Escape delimiters must be valid of the form [<PREFIX>e_<CONTENT>]
        // INVALID ESCAPE DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_escape_delimiter_name(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        
        let content = self.current_delimiter.as_ref().unwrap().content.clone().unwrap_or_default();
        
        if ch == '_' && content.is_empty() {
            // Spec: Delimiter <CONTENT> may not start with an underscore.
            // INVALID ESCAPE DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            if content.ends_with('_') {
                // Spec: Delimiter <CONTENT> may not end with an underscore.
                // INVALID ESCAPE DELIMITER
                return self.exit_delimiter_into_data(ch);
            }
            // Spec: Escape delimiter of the form [<PREFIX>e_<CONTENT>]
            // VALID ESCAPE DELIMITER
            self.state = ASLANParserState::Escape;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            
            if self.current_escape_delimiter.is_none() {
                self.current_escape_delimiter = Some(content);
            } else if self.current_escape_delimiter.as_ref() != Some(&content) {
                // Make sure we write out the escape delimiter with different content since the escape hasn't closed
                let prefix = self.current_delimiter.as_ref().unwrap().prefix.clone().unwrap_or_default();
                let escape_content = self.current_delimiter.as_ref().unwrap().content.clone().unwrap_or_default();
                self.current_value = format!("[{}e_{}", prefix, escape_content);
                self.store_current_value();
                // Spec: Escape delimiters must be the same for the entire string.
                // INVALID ESCAPE DELIMITER
                return self.exit_delimiter_into_data(ch);
            } else {
                self.current_escape_delimiter = None;
                self.state = ASLANParserState::Data;
                self.delimiter_buffer.clear();
                self.current_value.clear();
            }
            return;
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            // Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
            // INVALID ESCAPE DELIMITER
            return self.exit_delimiter_into_data(ch);
        }
        self.current_delimiter.as_mut().unwrap().content.as_mut().unwrap().push(ch);
        self.delimiter_buffer.push(ch);
    }

    fn handle_part_delimiter(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if self.current_escape_delimiter.is_some() {
            return self.exit_delimiter_into_data(ch);
        }
        if ch == ']' {
            // Spec: Part delimiters have no <CONTENT> or args
            // VALID PART DELIMITER
            let current_key = self.get_current_key_string();
            let is_locked = self.stack.last().unwrap().data_insertion_locks.get(&current_key).copied().unwrap_or(false);
            
            if !is_locked {
                let value = self.get_value_at_key(&current_key).cloned();
                // Check if value is "falsy" like in JavaScript (None, Null, or empty string)
                let is_falsy = match &value {
                    None => true,
                    Some(Value::Null) => true,
                    Some(Value::String(s)) if s.is_empty() => true,
                    _ => false,
                };
                
                if is_falsy {
                    self.stack.last_mut().unwrap().implicit_arrays.insert(current_key.clone(), true);
                    self.set_value_at_key(&current_key, json!([""]));
                } else if let Some(Value::String(s)) = value {
                    self.stack.last_mut().unwrap().implicit_arrays.insert(current_key.clone(), true);
                    self.set_value_at_key(&current_key, json!([s, ""]));
                } else if let Some(Value::Array(_)) = value {
                    self.emit_end_events_if_required();
                    self.append_to_array_at_key(&current_key, json!(""));
                }
            }
            self.state = ASLANParserState::Data;
            self.delimiter_buffer.clear();
            self.current_value.clear();
            self.next_key();
            return;
        }
        // Spec: Part delimiters have no <CONTENT> or args
        // INVALID PART DELIMITER
        self.exit_delimiter_into_data(ch);
    }

    fn handle_go(&mut self, ch: char) {
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
            return;
        }
        self.exit_delimiter_into_data(ch);
    }

    fn handle_stop(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
            return;
        }
        self.append_to_current_value(ch);
    }

    fn handle_object(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
            return;
        }
        self.append_to_current_value(ch);
    }

    fn handle_array(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
            return;
        }
        self.append_to_current_value(ch);
    }

    fn handle_comment(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
        }
    }

    fn handle_escape(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
        }
        self.append_to_current_value(ch);
        self.store_current_value();
        self.state = ASLANParserState::Data;
        self.delimiter_buffer.clear();
        self.current_value.clear();
    }

    fn handle_data(&mut self, ch: char) {
        if self.parsing_locked {
            self.state = ASLANParserState::Locked;
            return;
        }
        if ch == '[' {
            self.state = ASLANParserState::MaybeDelimiter;
            self.delimiter_buffer.push(ch);
            return;
        }
        self.append_to_current_value(ch);
        self.store_current_value();
    }

    fn append_to_current_value(&mut self, ch: char) {
        self.current_value.push(ch);
    }

    fn store_current_value(&mut self) {
        let current_key = self.get_current_key_string();
        
        if self.stack.last().unwrap().void_fields.get(&current_key).copied().unwrap_or(false) {
            self.current_value.clear();
            self.set_value_at_key(&current_key, Value::Null);
            return;
        }

        if !self.current_value.is_empty() {
            let is_locked = self.stack.last().unwrap().data_insertion_locks.get(&current_key).copied().unwrap_or(false);
            let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
            let is_implicit_array = self.stack.last().unwrap().implicit_arrays.get(&current_key).copied().unwrap_or(false);
            
            if !is_locked && !is_object {
                // Append to string value
                let current = self.get_value_at_key(&current_key).and_then(|v| v.as_str()).unwrap_or("").to_string();
                self.set_value_at_key(&current_key, Value::String(current + &self.current_value));
                self.emit_content_events_for_primitive();
            }
            
            if !is_locked && is_implicit_array {
                // Append to last element of implicit array
                if let Some(arr) = self.get_value_at_key(&current_key).and_then(|v| v.as_array()).cloned() {
                    if let Some(last) = arr.last() {
                        let last_str = last.as_str().unwrap_or("");
                        let new_value = format!("{}{}", last_str, self.current_value);
                        let mut new_arr = arr.clone();
                        if let Some(last_elem) = new_arr.last_mut() {
                            *last_elem = Value::String(new_value);
                        }
                        self.set_value_at_key(&current_key, Value::Array(new_arr));
                    }
                }
                self.emit_content_events_for_implicit_array();
            }
            self.current_value.clear();
        }
    }

    fn set_value_at_key(&mut self, key: &str, value: Value) {
        // Update in both the latest result and propagate up the stack
        let frame = self.stack.last_mut().unwrap();
        if let Some(obj) = frame.inner_result.as_object_mut() {
            obj.insert(key.to_string(), value.clone());
        } else if let Some(arr) = frame.inner_result.as_array_mut() {
            if let Ok(idx) = key.parse::<usize>() {
                while arr.len() <= idx {
                    arr.push(Value::Null);
                }
                arr[idx] = value.clone();
            }
        }
        
        // Propagate changes up the stack
        self.sync_stack_to_root();
    }

    fn append_to_array_at_key(&mut self, key: &str, value: Value) {
        let frame = self.stack.last_mut().unwrap();
        if let Some(obj) = frame.inner_result.as_object_mut() {
            if let Some(arr) = obj.get_mut(key).and_then(|v| v.as_array_mut()) {
                arr.push(value);
            }
        } else if let Some(outer_arr) = frame.inner_result.as_array_mut() {
            if let Ok(idx) = key.parse::<usize>() {
                if let Some(arr) = outer_arr.get_mut(idx).and_then(|v| v.as_array_mut()) {
                    arr.push(value);
                }
            }
        }
        
        // Propagate changes up the stack
        self.sync_stack_to_root();
    }

    fn sync_stack_to_root(&mut self) {
        // Propagate changes from innermost frame back to root
        for i in (1..self.stack.len()).rev() {
            let inner_result = self.stack[i].inner_result.clone();
            let parent_key = if i > 0 {
                self.stack[i - 1].current_key.as_string()
            } else {
                continue;
            };
            
            if let Some(obj) = self.stack[i - 1].inner_result.as_object_mut() {
                obj.insert(parent_key, inner_result);
            } else if let Some(arr) = self.stack[i - 1].inner_result.as_array_mut() {
                if let Ok(idx) = parent_key.parse::<usize>() {
                    while arr.len() <= idx {
                        arr.push(Value::Null);
                    }
                    arr[idx] = inner_result;
                }
            }
        }
        
        // Update multi_aslan_results
        if !self.multi_aslan_results.is_empty() {
            let last_idx = self.multi_aslan_results.len() - 1;
            self.multi_aslan_results[last_idx] = self.stack[0].inner_result.clone();
        }
    }

    fn set_data_insertion_type(&mut self, insertion_type: ASLANDataInsertionType) {
        let current_key = self.get_current_key_string();
        let frame = self.stack.last_mut().unwrap();
        
        // Spec: Data insertion type can only be set once for a given key in an object/array.
        if frame.data_insertion_types.contains_key(&current_key) {
            // Check existing type and act accordingly
            match frame.data_insertion_types.get(&current_key) {
                Some(ASLANDataInsertionType::KeepLast) => {
                    // Clear the value
                    if let Some(obj) = frame.inner_result.as_object_mut() {
                        obj.insert(current_key.clone(), Value::String(String::new()));
                    }
                    // Clear instructions for this key
                    frame.registered_instructions.retain(|i| i.key != current_key);
                }
                Some(ASLANDataInsertionType::KeepFirst) => {
                    frame.data_insertion_locks.insert(current_key, true);
                }
                _ => {}
            }
            return;
        }
        frame.data_insertion_types.insert(current_key, insertion_type);
    }

    fn next_key(&mut self) {
        let is_array = self.get_latest_result().is_array();
        
        if is_array {
            if let Some(content) = self.current_delimiter.as_ref().and_then(|d| d.content.as_ref()) {
                // Explicit index
                if let Ok(new_index) = content.parse::<i64>() {
                    self.set_current_key(ASLANKey::Index(new_index));
                    let min = self.get_min_array_index();
                    self.set_min_array_index(min.max(new_index + 1));
                } else {
                    let min = self.get_min_array_index();
                    self.set_current_key(ASLANKey::Index(min));
                    self.set_min_array_index(min + 1);
                }
            } else {
                // Implicit index
                let min = self.get_min_array_index();
                self.set_current_key(ASLANKey::Index(min));
                self.set_min_array_index(min + 1);
            }
        } else {
            // Object
            if let Some(content) = self.current_delimiter.as_ref().and_then(|d| d.content.clone()) {
                let frame = self.stack.last_mut().unwrap();
                
                // Check if default field exists and is empty, set to null
                if let Some(obj) = frame.inner_result.as_object_mut() {
                    let default_field = self.parser_settings.default_field_name.clone();
                    if let Some(val) = obj.get(&default_field) {
                        if val.as_str() == Some("") {
                            obj.insert(default_field, Value::Null);
                        }
                    }
                }
                
                // Check if we've already seen this key
                if frame.inner_result.as_object().and_then(|o| o.get(&content)).is_some() {
                    frame.already_seen_duplicate_keys.insert(content.clone(), true);
                }
                
                self.set_current_key(ASLANKey::String(content));
            }
        }
    }

    fn get_current_path(&self) -> Vec<String> {
        let mut path = Vec::new();
        for frame in &self.stack {
            match &frame.current_key {
                ASLANKey::String(s) if s != &self.parser_settings.default_field_name => {
                    path.push(s.clone());
                }
                ASLANKey::Index(i) => {
                    path.push(i.to_string());
                }
                _ => {}
            }
        }
        path
    }

    fn emit_end_events_if_required(&mut self) {
        if !self.parser_settings.emittable_events.end {
            return;
        }
        let current_key = self.get_current_key_string();
        let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
        if !is_object {
            self.emit_content_events_for_primitive_with_tag("end");
        }
        let is_implicit_array = self.stack.last().unwrap().implicit_arrays.get(&current_key).copied().unwrap_or(false);
        if is_implicit_array {
            self.emit_content_events_for_implicit_array_with_tag("end");
        }
    }

    fn emit_end_data_events_if_required(&mut self) {
        if !self.parser_settings.emittable_events.end_data {
            return;
        }
        let current_key = self.get_current_key_string();
        let is_object = self.get_value_at_key(&current_key).map(|v| v.is_object() || v.is_array()).unwrap_or(false);
        let is_implicit_array = self.stack.last().unwrap().implicit_arrays.get(&current_key).copied().unwrap_or(false);
        
        if !is_object && !is_implicit_array {
            let value = self.get_value_at_key(&current_key).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let instructions: Vec<ASLANInstructionInfo> = self.stack.last().unwrap().registered_instructions
                .iter()
                .filter(|i| i.key == current_key)
                .map(|i| ASLANInstructionInfo {
                    name: i.name.clone(),
                    args: i.args.clone(),
                    index: i.index,
                })
                .collect();
            
            let content = vec![ASLANContentPart {
                value,
                part_index: 0,
                instructions,
            }];
            
            self.emit_end_data_event(content, &current_key);
        } else if is_implicit_array {
            let arr = self.get_value_at_key(&current_key).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut instructions_by_part: HashMap<usize, Vec<ASLANInstructionInfo>> = HashMap::new();
            
            for instruction in &self.stack.last().unwrap().registered_instructions {
                if instruction.key == current_key {
                    instructions_by_part
                        .entry(instruction.part_index)
                        .or_default()
                        .push(ASLANInstructionInfo {
                            name: instruction.name.clone(),
                            args: instruction.args.clone(),
                            index: instruction.index,
                        });
                }
            }
            
            let content: Vec<ASLANContentPart> = arr.iter().enumerate().map(|(i, v)| {
                ASLANContentPart {
                    value: v.as_str().unwrap_or("").to_string(),
                    part_index: i,
                    instructions: instructions_by_part.get(&i).cloned().unwrap_or_default(),
                }
            }).collect();
            
            self.emit_end_data_event(content, &current_key);
        }
    }

    fn emit_content_events_for_primitive(&mut self) {
        self.emit_content_events_for_primitive_with_tag("content");
    }

    fn emit_content_events_for_primitive_with_tag(&mut self, tag: &str) {
        if tag == "content" && !self.parser_settings.emittable_events.content {
            return;
        }
        if tag == "end" && !self.parser_settings.emittable_events.end {
            return;
        }
        
        let current_key = self.get_current_key_string();
        let value = self.get_value_at_key(&current_key).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let path = self.get_current_path();
        let structure = self.get_result();
        let multi_aslan_index = self.multi_aslan_results.len().saturating_sub(1);
        
        let instructions: Vec<_> = self.stack.last().unwrap().registered_instructions
            .iter()
            .filter(|i| i.key == current_key && i.part_index == 0)
            .cloned()
            .collect();
        
        for instruction in instructions {
            let event = ASLANInstruction {
                content: value.clone(),
                part_index: 0,
                field_name: current_key.clone(),
                path: path.clone(),
                structure: structure.clone(),
                instruction: instruction.name,
                args: instruction.args,
                index: instruction.index,
                multi_aslan_index,
                tag: tag.to_string(),
            };
            
            if tag == "content" {
                for (_, handler) in &mut self.event_listeners.content {
                    handler(&event);
                }
            } else {
                for (_, handler) in &mut self.event_listeners.end {
                    handler(&event);
                }
            }
        }
    }

    fn emit_content_events_for_implicit_array(&mut self) {
        self.emit_content_events_for_implicit_array_with_tag("content");
    }

    fn emit_content_events_for_implicit_array_with_tag(&mut self, tag: &str) {
        if tag == "content" && !self.parser_settings.emittable_events.content {
            return;
        }
        if tag == "end" && !self.parser_settings.emittable_events.end {
            return;
        }
        
        let current_key = self.get_current_key_string();
        let arr = self.get_value_at_key(&current_key).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let part_index = arr.len().saturating_sub(1);
        let value = arr.last().and_then(|v| v.as_str()).unwrap_or("").to_string();
        let path = self.get_current_path();
        let structure = self.get_result();
        let multi_aslan_index = self.multi_aslan_results.len().saturating_sub(1);
        
        let instructions: Vec<_> = self.stack.last().unwrap().registered_instructions
            .iter()
            .filter(|i| i.key == current_key && i.part_index == part_index)
            .cloned()
            .collect();
        
        for instruction in instructions {
            let event = ASLANInstruction {
                content: value.clone(),
                part_index,
                field_name: current_key.clone(),
                path: path.clone(),
                structure: structure.clone(),
                instruction: instruction.name,
                args: instruction.args,
                index: instruction.index,
                multi_aslan_index,
                tag: tag.to_string(),
            };
            
            if tag == "content" {
                for (_, handler) in &mut self.event_listeners.content {
                    handler(&event);
                }
            } else {
                for (_, handler) in &mut self.event_listeners.end {
                    handler(&event);
                }
            }
        }
    }

    fn emit_end_data_event(&mut self, content: Vec<ASLANContentPart>, field_name: &str) {
        if !self.parser_settings.emittable_events.end_data {
            return;
        }
        
        let path = self.get_current_path();
        let structure = self.get_result();
        let multi_aslan_index = self.multi_aslan_results.len().saturating_sub(1);
        
        let event = ASLANEndDataInstruction {
            content,
            field_name: field_name.to_string(),
            path,
            structure,
            multi_aslan_index,
            tag: "end_data".to_string(),
        };
        
        for (_, handler) in &mut self.event_listeners.end_data {
            handler(&event);
        }
    }
}

impl Default for ASLANParser {
    fn default() -> Self {
        Self::new()
    }
}

# ASLAN: ASLAN Sufficiently Lax AI Notation
An LLM stream compatible structured data format that lets you safely output, parse and display rich content

ASLAN gives you the power of your favorite data format, parsable with a simple state machine based parser, without worrying that content you send to your users will be completely unrenderable.

## Why not JSON, XML, YAML, TOML, Markdown, [Insert favorite format here]?
Most other data formats heavily rely on special characters which often appear in user generated content, and aren't always reliably escaped in LLM output especially in long context windows.

Additionally, traditional structured formats are often strict and even good streaming parsers cannot fully compensate.

At worst, nothing renders, so you'll regularly end up having to show the user the last good state. When returning structured data to a UI and only rendering some fields, you can end up having several instances in a stream where the end user sees parts of the underlying data structure e.g. a JSON field that hasn't yet had a closing quote may be incorrectly inferred by a streaming parser to be part of the previous field's content.

- JSON breaks on quotes & braces. Common parsers such as Python's `json` module are bad at handling control characters such as new lines. If using handlebars, template variables or f-Strings in your prompts, you're in for a bad day. Despite prompt engineering tricks, LLMs are still chatty and sometimes output content outside of your JSON which can be handled in the simple case, but not always.
- LLMs can be a bit unreliable at generating XML, closing tags often get omitted. Text interspersed with XML isn't valid. Unclosed XML isn't valid: streaming parsers do hack around this as they do for JSON.
- YAML and TOML are messy with too many special or control characters to escape
- Markdown is tricky to structure and has a lot of special characters to escape

***This is a working spec document at the moment and is subject to change based on incoming feedback***

## Specification
ASLAN files SHOULD have the extension `.llm` or `.aslan`.

ASLAN data consists of plaintext strings with a series of special tokens. To avoid confusion with LLM tokens, this spec uses the term 'ASLAN delimiter(s)', or just 'delimiter(s)' instead of 'token' from now on.

### 1. Prefix
ASLAN delimiters start with a customizable prefix `<PREFIX>`. All implementations MUST provide the default prefixes `llm` and `aslan`.

The delimiter prefix acts as a namespace to ensure external data also using ASLAN within a stream is ignored by the parser.

Prefixes MUST only contain alphanumeric characters but to maximize readability and minimize LLM generation mistakes, it is RECOMMENDED to use only lower case characters and numbers.

To minimize token usage in LLM calls, it is RECOMMENDED that prefixes be <= 7 characters.

### 2. The root
All ASLAN content is a child or subchild of the root. Strings with no delimiters are valid ASLAN and considered the only child of the root.

`The quick brown fox jumps over the lazy dog` is valid ASLAN.

The root is a pseudo element that never appears in the output.

#### 2.1 Default field
Strings with no delimiters or `field-scope` delimiters only are inside an implicit `data` scope and will always be inserted into the `_default` field in the root, unless the `_default` field has been renamed.

Implementations MUST provide a way to rename the `_default` field. In other words, application developers MUST be able to instruct the parser to output content that would go in the `_default` field into a field of a different name.

### 3. Delimiter format
Delimiters MUST have the format `[<PREFIX><SUFFIX>_<CONTENT>]` or `[<PREFIX><SUFFIX>]` depending on the suffix. Delimiters may have optional arguments of the form `[<PREFIX><SUFFIX>_<CONTENT>:<ARG0>:<ARG1>]`, with or without content depending on the suffix.

Optional arguments are designed to allow an application developer to extend or modify the behavior of the parser at runtime.

### 4. Delimiter content
Delimiter content in `<CONTENT>` MUST only consist of alphanumeric characters and underscores, and may not start or end with an underscore.

In most cases, the content is the name of a data field or instruction but there are some special cases which will be discussed later in this spec.

### 5. Delimiter suffixes
The `<SUFFIX>` in a delimiter MUST be a single character from the following list `d`, `o`, `i`, `a`, `c`, `e`, `p`, `v`.
All other alphanumeric characters both upper and lower case are reserved for future use. Parsers MUST ignore delimiters with unimplemented suffixes and not add them to any generated data structure.

#### 5.1 The `d` suffix
The `d` suffix denotes `data` and can be thought of as similar to a field in JSON. `data` delimiters are `block-scope`.

#### 5.2 The `o` suffix
The `o` suffix denotes an `object` and can be thought of as similar to a JSON object. `object` delimiters are `block-scope`.

#### 5.3 The `i` suffix
The `i` suffix denotes an `instruction` to the parser in a `data` context. These are used to modify the handling of content in the field. `instruction` delimiters are `field-scope`, but affect `part-rules`.

#### 5.4 The `a` suffix
The `a` suffix denotes an `array` and can be thought of as similar to a JSON array. `array` delimiters are `block-scope`.

#### 5.5 The `c` suffix
The `c` suffix denotes a `comment`. Subsequent content will be ignored by the parser until a valid ASLAN delimiter with the known current prefix is reached. `comment` delimiters are `field-scope`.

#### 5.6 The `e` suffix
The `e` suffix denotes an `escape`. Subsequent content will be ignored by the parser until a corresponding `e` suffix delimiter is reached. `escape` delimiters are `field-scope`.

#### 5.7 The `p` suffix
The `p` suffix denotes a `part`. This creates a split point in the data content, essentially turning what would be a string into an array of part strings, without needing to use indices or be in an `array` state. `part` delimiters are `field-scope`.

#### 5.8 The `v` suffix
The `v` suffix denotes a `void`. This is equivalent to `null` in most languages. `void` delimiters are `field-scope`.

#### 5.9 The `g` suffix
The `g` suffix denotes a `go` and is optionally used to mark the start of an ASLAN string for multi-ASLAN support, or increased content safety.

#### 5.10 The `s` suffix
The `s` suffix denotes a `stop` and is optionally used to mark the end of an ASLAN string for multi-ASLAN support, or increased content safety.

### 6. Rules for `data`
The `data` delimiter is the most common way of creating structured data in ASLAN. It MUST adhere to the syntax `[<PREFIX>d]`, `[<PREFIX>d_<CONTENT>]` (or `[<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:...]` when using args) where `<CONTENT>` will become the name of the field.

In an `object` scope, the `<CONTENT>` is required and each time a `data` delimiter is reached, a new field is added to the current `object` scope. Subsequent characters are appended to the added field until any of `data`, `object`, `comment` or EOF are reached.

In an `array` scope, the `<CONTENT>` is optional and each time a `data` delimiter is reached, a new value at index `<CONTENT>` (or the next available index if not specified) is added to the current `array` scope. Subsequent characters are appended to the added field until any of `data`, `array`, `comment` or EOF are reached.

The next `data` field encountered will be added to the current `object` scope as before.

A `data` field can be empty by immediately following it with another `data` delimiter, or a `comment` delimiter with no subsequent `field-scope` delimiters.

Of course, a `data` delimiter is not necessary, since strings with no delimiters are valid ASLAN. Implementations MUST fill a `_default` field with undelimited (or only `field-scope` delimited) content, unless the default field has been renamed (See 2.1).

By default, duplicate ASLAN fields in the same block scope have their values merged via concatenation if they are primitives. Duplicate ASLAN fields in the same block scope with non-primitive values such as `object` or `array` MUST always take the last value output for the field.

The `data` delimiter has a single extension which species how duplicate instances of the key in the same block scope behave for primitives. `<ARG0>` may contain the value `a`, `f` or `l` for `append` (the default behavior), `first` (to take the first occurrence of the key in the block scope), `last` (to take the last occurrence of the key in the block scope) respectively. Implementations may track this however they wish.

The duplicate behavior definition MUST be applied to the first duplicate `data` delimiter in a block scope. If there are multiple duplicate behavior definitions for the same key in a block, all after the first are ignored.

If the current `data` block scope has a string value, when the `data` block scope ends, either due to another `data` delimiter or due to the end of the `data` field (both auto-closing and non auto-closing via an `object` or `array` close), an event MUST be emitted as below:
1. an array containing objects for every string `part` where each object contains:
  - the value of the string `part` (this does not include any `instruction` delimiters)
  - the index of the `part` in the `data`
  - an array of objects for every `instruction` in the `part` containing:
    - the `<CONTENT>` value of the `instruction`
    - all args in the `instruction`, in order, or an empty array if none exist
    - the index of the `instruction` delimiter within the `part` (note that for the purposes of this, `instruction` delimiters are treated as having length 1, that is `ABC[aslani_ins]DEF[aslani_ins2]G` would put 'D' at index 4 and 'G' at index 8)
2. the `data` field name (or index) it is in
3. a path to the field within the overall ASLAN data structure e.g. `["address", "line1"]`
4. the overall ASLAN structure
5. an `instruction tag` containing an enum value that is `END DATA`

Parser implementations MUST provide a way for application developers to disable the emission of events with `END DATA` `instruction tag`s.

Whitespace including new lines is always preserved - ASLAN supports multiline strings out of the box.

#### 6.1 Example `data` usage
1. The string `[asland_hi]Hello [asland_lo]World!` is equivalent to the JSON:

```json
{
  "_default": null,
  "hi": "Hello ",
  "lo": "World!"
}
```

2. The string `This is still valid.[asland_hi]Hello [asland_lo]World!` is equivalent to the JSON:

```json
{
  "_default": "This is still valid.",
  "hi": "Hello ",
  "lo": "World!"
}
```

3. The string `[asland_hi]Hello [asland_lo]World![asland_hi]Hello` is equivalent to the JSON:

```json
{
  "_default": null,
  "hi": "Hello Hello",
  "lo": "World!"
}
```

### 7. Rules for `object`s
`object` delimiters MUST adhere to the syntax `[<PREFIX>o]`. Almost all ASLAN fields are considered strings by default. The root is considered an implicit `object`. If we wrote out explict delimiters for the root it would be equivalent to `[<PREFIX>d_root][<PREFIX>o]` which would be the JSON object `{}`.

By convention, we assume the named field for the root is the underlying variable containing the ASLAN structure, equivalent to this:
```typescript
const root = {};
```

`object` delimiters immediately after `data` delimiters will start a new nested block scope on the corresponding field. Every block in ASLAN is self-closing, but it is possible to close a block early with another `object` delimiter not immediately after a `data` delimiter to get the desired nesting behavior. Comments count as length zero and do not affect the delimiter adjacency rules: it is valid to have a `data` `comment` `object` set of delimiters and the `comment` will be ignored by the parser as usual.

Each closing `object` delimiter will shift the parser into the parent block scope, unless the parser is already in the root block scope or in an `array` block, in which case all extraneous `object` delimiters will be ignored.

Every `data` delimiter inside an `object` block will create a field in that `object` with the key name being the `<CONTENT>` of the `data` delimiter.

#### 7.1 Example `object` usage
1. The string `[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslan_bar]Baz!` and `[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][aslan_bar]Baz!` are equivalent to the JSON:

```json
{
  "_default": null,
  "hi": "Hello ",
  "lo": "World!",
  "foo": {
    "bar": "Baz!"
  }
}
```

2. The string `[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslan_bar]Baz![aslano][asland_x][aslano][aslan_y]you are reading spec[aslan_z]and it continues here` is equivalent to the JSON:

```json
{
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
}
```

### 8. Rules for `instruction`s
`instruction` delimiters MUST adhere to the syntax `[<PREFIX>i_<CONTENT>]` (or `[<PREFIX>i_<CONTENT>:<ARG0>:<ARG1>:...]` when using args) where `<CONTENT>` is the name of `instruction` to run.

`instruction`s operate on the `part` within which they are found. A string field with no explicit `part` delimiters is considered a single `part`.

`instruction`s do not affect the structure of the data or split string content. They are emitted as events for the parser to handle separately from the data structure.

When the parser encounters an `instruction`, it MUST emit an event containing:
1. the value of the `part` (this does not include any `instruction` delimiters)
2. the index of the `part` in the `data` field
3. the `data` field name (or index) it is in
4. a path to the field within the overall ASLAN data structure e.g. `["address", "line1"]`
5. the overall ASLAN structure
6. the `instruction` `<CONTENT>` value
7. all args in the `instruction`, in order, or an empty array if none exist
8. the index of the `instruction` delimiter within the `part` (note that for the purposes of this, `instruction` delimiters are treated as having length 1, that is `ABC[aslani_ins]DEF[aslani_ins2]G` would put 'D' at index 4 and 'G' at index 8)
9. an `instruction tag` containing an enum value that is either `CONTENT` or `END` (this is `CONTENT`) when the instruction is first encountered

On every subsequent change to the content of the `part` containing the `instruction` (remember that `comment`s are ignored as they are not content), an additional event MUST be emitted as above, with the `CONTENT` `instruction tag`.

When the `part` ends, either due to another `part` delimiter or due to the end of the `data` field (both auto-closing and non auto-closing via an `object` or `array` close), an additional event MUST be emitted as above, with the `END` `instruction tag`. This gives an application developer the option to only run an `instruction` at the end of the `part` or on every change.

Every `part` may have multiple `instruction`s and each `instruction` MUST be run in the order it appears.

ASLAN parser implementations MUST provide API hooks that allow an application developer to listen to `instruction` events.

Parser implementations MUST provide a way for application developers to disable the emission of events with `CONTENT` and `END` `instruction tag`s.

`instruction`s apply to the entire `part` in which they appear.

#### 8.1 Example `instruction` usage
1. The ASLAN string

```aslan
[asland_article][aslano]
[asland_title]The Future of AI
[asland_content]
[aslanp][aslani_heading:1]Introduction
[aslanp]Artificial Intelligence has come a long way in recent years.[aslani_highlight] From machine learning to neural networks, AI is revolutionizing various industries.[aslani_citation:1]
[aslanp][aslani_heading:2]Key Areas of AI Development
[aslanp][aslani_list]Natural Language Processing
[aslanp][aslani_list]Computer Vision
[aslanp][aslani_list]Robotics
[aslanp][aslani_heading:2]Challenges and Ethical Considerations
[aslanp]As AI continues to advance, we must address important ethical questions.[aslani_emphasis] Balancing progress with responsibility is crucial for the future of AI.[aslani_citation:2]
[asland_author]Dr. Jane Smith
[asland_date]2024-09-08
```

is equivalent to the JSON:

```json
{
  "article": {
    "title": "The Future of AI\n",
    "content": [
      "Introduction\n",
      "Artificial Intelligence has come a long way in recent years. From machine learning to neural networks, AI is revolutionizing various industries.\n",
      "Key Areas of AI Development\n",
      "Natural Language Processing\n",
      "Computer Vision\n",
      "Robotics\n",
      "Challenges and Ethical Considerations\n",
      "As AI continues to advance, we must address important ethical questions. Balancing progress with responsibility is crucial for the future of AI.\n"
    ],
    "author": "Dr. Jane Smith\n",
    "date": "2024-09-08\n"
  }
}
```

### 9. Rules for `array`s
`array` delimiters MUST adhere to the syntax `[<PREFIX>a]`. `array` delimiters immediately after `data` delimiters will start a new nested array block scope on the corresponding field. `array` blocks are self-closing, but it is possible to close a block early with another `array` delimiter not immediately after a `data` delimiter to get the desired nesting behavior. Comments count as length zero and do not affect the delimiter adjacency rules: it is valid to have a `data` `comment` `array` set of delimiters and the `comment` will be ignored by the parser as usual.

Each closing `array` delimiter will shift the parser into the parent block scope, unless the parser is already in the root block scope or in an `object` block, in which case all extraneous `array` delimiters will be ignored.

Every `data` delimiter inside an `array` block will create a field in index in the `array` with the index being the `<CONTENT>` of the `data` delimiter if it exists and is a valid integer, or the next available index if not.

#### 9.1 Example `array` usage
1. The string `[asland_fruits][aslana][asland]Apple[asland]Banana[asland]Cherry` is equivalent to the JSON:

```json
{
  "_default": null,
  "fruits": [
    "Apple",
    "Banana",
    "Cherry"
  ]
}
```

2. The string `[asland_custom_array][aslana][asland_2]Third item[asland_0]First item[asland_1]Second item` is equivalent to the JSON:

```json
{
  "_default": null,
  "custom_array":[
    "First item",
    "Second item",
    "Third item"
  ]
}
```

It is RECOMMENDED that `data` inside an `array` all either use valid `<CONTENT>` or not, allowing for auto-incrementing indices starting from 0. However, parser implementations MUST be able to gracefully handle mixed missing/valid/invalid `<CONTENT>` in an `array` block scope, even if that means leaving holes in the `array`.

### 10. Rules for `comment`s
`comment` delimiters MUST adhere to the syntax `[<PREFIX>c]`. `comment` delimiters indicate the start of a comment and can be placed anywhere. The comment ends when the parser encounters any ASLAN delimiter (with the current prefix - it is perfectly valid to use ASLAN delimiters with a different prefix in a comment and this will have no impact on the parsed output). 

All `comment`s are ignored by the parser and will not be output into the parsed data structure. Furthermore, for the purposes of other delimiter rules, `comment`s can be thought of as having 0 length, that is they do not affect other rules.

### 11. Rules for `escape`s
`escape` delimiters MUST adhere to the syntax `[<PREFIX>e_<CONTENT>]` where `<CONTENT>` is any alphanumeric string. `escape` allows subsequent characters to be treated as regular strings, allowing content that would otherwise be considered ASLAN delimiters with the current prefix to be safely used as field content. This is particularly useful when passing ASLAN from external sources into an LLM where you might have it accidentally output content that would be treated as part of the data structure otherwise.

The `escape` ends when the parser encounters another `escape` delimiter with the current prefix and identical `<CONTENT>`.

`<CONTENT>` is RECOMMENDED to be sufficiently long and/or unique that it doesn't appear in the content the application developer wishes to escape.

#### 10.1 Example `escape` usage
1. The ASLAN string

```aslan
[asland_example_code]
[aslane_CODE_BLOCK]
function greet(name) {
  console.log(`Hello, ${name}!`);
  [asland_this_is_not_parsed]This is treated as a regular string
}
[aslane_CODE_BLOCK]
```

is equivalent to the JSON:

```json
{
  "example_code": "function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a regular string\n}"
}
```

### 12. Rules for `part`s
`part` delimiters MUST adhere to the syntax `[<PREFIX>p]`. `part`s split a string at the `part` and convert it into an implicit array of partial strings.

This is an alternative, more LLM friendly, way of outputting content that may be individually styled by the end system compared to the `array` delimiter. However, unlike the `array` delimiter, `part` doesn't allow custom indices. For the purposes of `instruction`s, `part`s create their own internal `part-rules`.

`part` split a string field into an array of strings. Any `instruction`s within `part`s do not appear in the final data structure but are emitted as separate events.

#### 12.1 Example `part` usage
1. The string `[asland_formatted_text][aslanp]This is the first part.[aslanp]This is the second part.[aslanp]This is the third part.` is equivalent to the JSON:
```json
{
  "_default": null,
  "formatted_text": [
    "This is the first part.",
    "This is the second part.",
    "This is the third part."
  ]
}
```

2. The string `[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.` is equivalent to the JSON:
```json
{
  "_default": null,
  "styled_text": [
    "This is bold and red text.",
    "This is italic and underlined text.",
    "This is large monospace text."
  ]
}
```

### 13. Rules for `void`s
`void` delimiters MUST adhere to the syntax `[<PREFIX>v]` which is equivalent to a `null` in JSON. Any `data` field can have a `void` delimiter to set its value to `null`.

Any `data` field can have more than one `void` delimiter but all `void` delimiters after the first will be ignored.

`void` delimiters have a higher priority than all other string content in a `field-scope`, meaning if a `void` occurs before or after other string content, the field will be treated as `void`. Other delimiters in the field will be ignored, including `escape`s and `comment`s (although `comment`s are ignored anyway). When a `void` occurs after other string content in a field, the field is treated as `void` and any content the parser has already stored is overridden. All content after a `void` is always ignored.

#### 13.1 Example `void` usage
1. The string `[asland_hi]Hello [asland_lo]World![asland_fi][aslanv]` is equivalent to the JSON:

```json
{
  "_default": null,
  "hi": "Hello",
  "lo": "World!",
  "fi": null
}
```

### 14. Rules for `go`s
`go` delimiters MUST adhere to the syntax `[<PREFIX>g]`. If the parser has the `strictStart` flag enabled, it must only start trying to parse content as ASLAN after the `go` delimiter.

If the `strictStart` flag is disabled, all content in the stream will be parsed as ASLAN and any `go` delimiter will be ignored.

Multiple `go` delimiters are allowed in a stream but this creates multi-ASLAN i.e. multiple ASLAN objects will be output from a single stream. This allows for ASLAN to be used in complex outputs such as multi-stage LLM calls with tool calls.

Each time a `go` delimiter is encountered, the current state of the result object MUST be added to the output array and all other parser state MUST be reset.

The `go` delimiter is strictly optional and for a single ASLAN result is unnecessary. However, it may still be useful if the application developer doesn't want LLM preamble to end up in the result object (although the preferred way to handle this is to just ignore the `_default` field application code and explicitly start your first desired field with `[<PREFIX>d]`).

ASLAN parsers MUST implement the `strictStart` flag and have it default to `false`.

#### 14.1 Example `go` usage
1. The string `Here is some some valid ASLAN I have created for you: [asland_hi]Hello [asland_lo]World![asland_fi][aslanv]` with `strictStart` set to `true` is equivalent to:

```json
{
  "_default": "Here is some some valid ASLAN I have created for you: ",
  "hi": "Hello",
  "lo": "World!",
  "fi": null
}
```

2. The string `Here is some some valid ASLAN I have created for you: [aslang][asland_hi]Hello [asland_lo]World![asland_fi][aslanv]` with `strictStart` set to `true` is equivalent to:

```json
{
  "_default": null,
  "hi": "Hello",
  "lo": "World!",
  "fi": null
}
```

3. The string `Here is some some valid ASLAN I have created for you: [aslang][asland_hi]Hello [asland_lo]World![asland_fi][aslanv][aslang]Here is some more content` with `strictStart` set to `true` is equivalent to:

```json
[
  {
    "_default": null,
    "hi": "Hello",
    "lo": "World!",
    "fi": null
  },
  {
    "_default": "Here is some more content",
  }
]
```

Note that an ASLAN parser, whether it has encountered multi-ASLAN or single-ASLAN MUST always output the final result of a parsed stream as an array of 1 or more ASLAN objects. For brevity in this spec, we only show the wrapper array when there are multiple elements.

### 15. Rules for `stop`s
`stop` delimiters MUST adhere to the syntax `[<PREFIX>s]`. If the parser has the `strictEnd` flag enabled, it must stop start trying to parse content as ASLAN after the `stop` delimiter.

If the `strictEnd` flag is disabled, all content in the stream will be parsed as ASLAN and any `stop` delimiter will be ignored.

The `stop` delimiter is strictly optional and is designed purely as a safe way to avoid LLM epilogue being included in the result. It is unnecessary to use the `stop` delimiter in conjunction with the `go` delimiter if you are certain there will be no epilogue since the `go` delimiter starts a new ASLAN object by itself.

### 16. Error handling
ASLAN is designed specifically for IO in non-deterministic LLM based systems. As such, it aims to be permissive and forgiving. Where other data notation parsers may throw errors, ASLAN is designed to be able to ignore the issue and recover. An example of this is how ASLAN deals with duplicate `void`s or `void`s mixed with string content by simply ignoring the duplicates or additional string content.

### 17. Special cases
Empty strings in both the `_default` field and in any `data` field MUST always be treated as the string "" by parser implementations, never `void`.

If any field is declared in the root scope, then the `_default` field is `void`.

The empty string is equivalent to the JSON:
```json
{
  "_default": ""
}
```

### 18. Auto-closing behavior

ASLAN implements automatic closing of dangling block scopes (`object`s and `array`s) at the end of a stream. This means that explicit closing delimiters (`[aslano]` for `objects` and `[aslana]` for `arrays`) are not necessary for the outermost unclosed structure at the end of a stream. Adding closing delimiters on dangling scopes MUST be permitted but strictly optional.

#### 18.1 Example of auto-closing

```aslan
[asland_person][aslano]
[asland_name]John Doe
[asland_age]30
[asland_hobbies][aslana]
[asland]Reading
[asland]Hiking
[aslana]
[asland_address][aslano]
[asland_street]123 Main St
[asland_city]Anytown
```

In this example, only the outermost `person` `object` doesn't need to be explicitly closed. The `hobbies` `array` must be closed explicitly to differentiate it from a potential object with numeric string keys. The `address` `object` is left unclosed as it's the last nested structure within `person`.

#### 18.2 Best practices

- It is RECOMMENDED to explicitly close all non-dangling structures to ensure clear intent and prevent ambiguity. Only rely on auto-closing for the outermost structure if it's left unclosed at the end of the stream.
- Since `instruction`s apply to whole `part`s, it is RECOMMENDED for application developers to use them in conjunction with `part` delimiters when styling specific substrings, or using them to apply an operation such as adding a citation after a piece of text. However, `instruction` events do contain indices so an application developer can manually use multiple styles or operations via `instruction`s over a `part` if they need finer control and don't want the benefits of the ASLAN parser handling this.

### 19. Implementation specifics
- It is RECOMMENDED to parse ASLAN strings one character at a time
- Parsers MUST never look ahead as this will break the assumption of streaming compatibility
- Parsers MUST provide the option to buffer delimiters (i.e. when a `[` character arrives, wait for more characters until either it is recognized as an ASLAN delimiter, or recognized as part of regular string content) to ensure that application developers can directly display ASLAN to an end user without partially streamed delimiters being rendered. This is especially true for `instruction` & `part` delimiters which regularly appear in string content. It is strongly RECOMMENDED that buffering delimiters is the default option for parsers.

## Discussion
### Why doesn't ASLAN support `numbers` and `booleans`?
The short answer is it's probably not that useful. If you need a number or a boolean for a specific field, you as an application developer can parse the field with your favorite transform library. If someone makes a compelling use case argument, I'll add it.

### Why don't you throw errors?
The aim is to be as permissive as possible because LLM reliability can be hard so minor output hiccups are to be expected. We'd all rather get some correct data than no data at all.

### What are some disadvantages?
ASLAN can be larger than other formats such as JSON, especially for arrays. However, when dealing with complex or external content it is much more reliable and can often be smaller when your use case involves a few structured fields and a lot of content.

Since ASLAN is so forgiving you are perfectly able to mix other data formats into an ASLAN stream e.g. encode arrays as JSON or CSV in an ASLAN field (though you will have to manually parse the string yourself).

# ASLAN Sufficiently Lax AI Notation
An LLM stream compatible structured data format that lets you safely output, parse and display rich content

ASLAN gives you the power of your favorite data format, parsable with a simple state machine based parser, without worrying that content you send to your users will be completely unrenderable.

## Why not JSON, XML, YAML, TOML, Markdown, [Insert favorite format here]?
Most other data formats heavily rely on special characters which often appear in user generated content, and aren't always reliably escaped in LLM output especially in long context windows.

Additionally, traditional structured formats are often strict and even good streaming parsers cannot fully compensate.

At worst, nothing renders, so often you'll end up having to show the user the last good state. When returning structured data to a UI and only rendering some fields, you can end up having several instances in a stream where the end user sees parts of the underlying data structure e.g. a JSON field that hasn't yet had a closing quote may be incorrectly inferred by a streaming parser to be part of the previous field's content.

- JSON breaks on quotes & braces. Common parsers such as Python's ```json``` module are bad at handling control characters such as new lines. If using handlebars, template variables or f-Strings in your prompts, you're in for a bad day. Despite prompt engineering tricks, LLMs are still chatty and sometimes output content outside of your JSON which can be handled in the simple case, but not always.
- LLMs can be a bit unreliable at generating XML, closing tags often get omitted. Text interspersed with XML isn't valid. Unclosed XML isn't valid: streaming parsers do hack around this as they do for JSON.
- YAML and TOML are messy with too many special or control characters to escape
- Markdown is tricky to structure and has a lot of special characters to escape

## Specification
ASLAN files SHOULD have the extension ```.llm``` or ```.aslan```.

ASLAN data consists of plaintext strings with a series of special tokens. To avoid confusion with LLM tokens, this spec uses the term 'ASLAN delimiter(s)', or just 'delimiter(s)' instead of 'token' from now on.

### 1. Prefix
ASLAN delimiters start with a customizable prefix ```<PREFIX>```. All implementations MUST provide the default prefixes ```llm``` and ```aslan```.

The delimiter prefix acts as a namespace to ensure external data also using ASLAN within a stream is ignored by the parser.

Prefixes MUST only contain alphanumeric characters but to maximize readability and minimize LLM generation mistakes, it is RECOMMENDED to use only lower case characters and numbers.

To minimize token usage in LLM calls, it is RECOMMENDED that prefixes be <= 7 characters.

### 2. The root
All ASLAN content is a child or subchild of the root. Strings with no delimiters are valid ASLAN and considered the only child of the root.

```The quick brown fox jumps over the lazy dog``` is valid ASLAN.

### 3. Delimiter format
Delimiters MUST have the format ```[<PREFIX><SUFFIX>_<CONTENT>]``` with optional arguments of the form ```[<PREFIX><SUFFIX>_<CONTENT>:<ARG0>:<ARG1>]```. 
Optional arguments are designed to allow a system developer to extend or modify the behavior of the parser at runtime.

### 4. Delimiter content
Delimiter content in ```<CONTENT>``` MUST only consist of alphanumeric characters and underscores, and may not start or end with an underscore.

In most cases, the content is the name of a data field or instruction but there are some special cases which will be discussed later in this spec.

### 5. Delimiter suffix
The ```<SUFFIX>``` in a delimiter MUST be a single character from the following list ```i```, ```d```, ```o```, ```a```, ```c```, ```e```.

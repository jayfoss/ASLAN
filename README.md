# ASLAN Sufficiently Lax AI Notation
An LLM stream compatible structured data format that lets you safely output, parse and display rich content

ASLAN gives you the power of your favorite data format, parsable with a simple state machine based parser, without worrying that content you send to your users will be completely unrenderable.

## Why not JSON, XML, YAML, TOML, Markdown, [Insert favorite format here]?
Most other data formats heavily rely on special characters which often appear in user generated content, and aren't always reliably escaped in LLM output especially in long context windows.
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

### 2. The root
All ASLAN content is a child or subchild of the root. Strings with no delimiters are valid ASLAN and considered the only child of the root.

```The quick brown fox jumps over the lazy dog``` is valid ASLAN.

### 3. Delimiter format
Delimiters MUST have the format ```[<PREFIX><SUFFIX>_<CONTENT>]``` with optional arguments of the form ```[<PREFIX><SUFFIX>_<CONTENT>:<ARG0>:<ARG1>]```. 
Optional arguments are designed to allow a system developer to extend or modify the behavior of the parser at runtime.

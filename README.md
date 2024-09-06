# ASLAN Sufficiently Lax AI Notation
An LLM stream compatible structured data format that lets you safely output, parse and display rich content

## Why not JSON, XML, YAML, TOML, [Insert favorite format here]?
Most other data formats heavily rely on special characters which often appear in user generated content, and aren't always reliably escaped in LLM output especially in long context windows.
- JSON breaks on quotes & braces. Common parsers such as Python's ```json``` module are bad at handling control characters such as new lines. If using handlebars, template variables or f-Strings in your prompts, you're in for a bad day. Despite prompt engineering tricks, LLMs are still chatty and sometimes output content outside of your JSON which can be handled in the simple case, but not always.
- LLMs can be a bit unreliable at generating XML, closing tags often get omitted. Text interspersed with XML isn't valid. Unclosed XML isn't valid: streaming parsers do hack around this as they do for JSON.
- YAML and TOML are messy with too many special or control characters to escape


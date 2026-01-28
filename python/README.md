# ASLAN Parser for Python

A streaming parser for ASLAN (ASLAN Sufficiently Lax AI Notation) - an LLM stream compatible structured data format.

[![PyPI](https://img.shields.io/pypi/v/aslang.svg)](https://pypi.org/project/aslang/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

See the [full specification](https://github.com/jayfoss/ASLAN) in the main repository.

## Installation

```bash
pip install aslang
```

Or with Poetry:

```bash
poetry add aslang
```

## Usage

### Basic Parsing

```python
from aslan import ASLANParser

parser = ASLANParser()
result = parser.parse('[asland_greeting]Hello [asland_name]World!')

# Result:
# {
#   "_default": None,
#   "greeting": "Hello ",
#   "name": "World!"
# }
```

### Streaming with `parse_next`

ASLAN is designed for streaming LLM outputs. Use `parse_next` to feed tokens incrementally:

```python
from aslan import ASLANParser

parser = ASLANParser()

# Simulate streaming tokens from an LLM
parser.parse_next('[asland_')
parser.parse_next('response]')
parser.parse_next('Here is ')
parser.parse_next('the answer.')

parser.close()
result = parser.get_result()
```

### Nested Objects and Arrays

```python
from aslan import ASLANParser

# Objects
parser = ASLANParser()
result = parser.parse('[asland_user][aslano][asland_name]Alice[asland_age]30')
# { "user": { "name": "Alice", "age": "30" } }

# Arrays
parser = ASLANParser()
result = parser.parse('[asland_items][aslana][asland]Apple[asland]Banana[asland]Cherry')
# { "items": ["Apple", "Banana", "Cherry"] }
```

### Parser Settings

```python
from aslan import ASLANParser, create_default_parser_settings

settings = create_default_parser_settings()
settings['strictStart'] = True                    # Require [aslang] to start parsing
settings['strictEnd'] = True                      # Stop parsing at [aslans]
settings['collapseObjectStartWhitespace'] = True  # Collapse whitespace at object/array starts
settings['maxObjectDepth'] = 1                    # Limit object nesting to 1 level

parser = ASLANParser(settings)
```

### Limiting Object Depth

The `maxObjectDepth` setting makes the `[aslano]` delimiter deterministic based on nesting level:

```python
from aslan import ASLANParser, create_default_parser_settings

# With maxObjectDepth = 1, [aslano] is binary:
# - At depth 0: always opens an object
# - At depth 1: always closes the object
settings = create_default_parser_settings()
settings['maxObjectDepth'] = 1

parser = ASLANParser(settings)
result = parser.parse('[asland_edit][aslano][asland_text]content[aslano]')
# { "_default": None, "edit": { "text": "content" } }
```

This is useful for LLM outputs with known structure depth, eliminating ambiguity from whitespace or empty content.

## Features

- **Streaming-first**: Parse character-by-character, perfect for LLM token streams
- **Fault-tolerant**: Designed to handle malformed input gracefully
- **Event-driven**: Subscribe to content, end, and end_data events
- **Nested structures**: Full support for objects, arrays, and mixed nesting
- **Instructions**: Emit metadata events without affecting the data structure
- **Comments & Escapes**: Skip content or escape ASLAN delimiters

## License

MIT

# A typescript reference parser for ASLAN

## Installation
npm
```bash
npm install aslang
```

pnpm
```bash
pnpm add aslang
```

## Usage

You can parse a fixed ASLAN string
```typescript
import { ASLANParser } from 'aslang';

const parser = new ASLANParser();
const result = parser.parse('[asland_hi]Hello [asland_lo]World!');
console.log(result);
```

will output
```typescript
{
  _default: null,
  hi: 'Hello ',
  lo: 'World!',
}
```

You can parse an ASLAN string as part of a stream
```typescript
import { ASLANParser } from 'aslang';

const parser = new ASLANParser();

myStream.on('data', (data: string) => {
  parser.parseNext(data);

  console.log(parser.getResult()); //Will print the current JS representation of the parsed ASLAN string
});

myStream.on('end', () => {
  parser.close(); //Close the parser. Important to finalize the parsing process otherwise the result will be incomplete.
  console.log(parser.getResult()); //Will print the final JS representation of the parsed ASLAN string (or the latest ASLAN object when using multi-ASLAN strings)
  console.log(parser.getResults()); //Will print the final JS representation of the parsed ASLAN string, as a multi-ASLAN array
});
```

## Parser Settings

You can customize parser behavior with settings:

```typescript
import { ASLANParser, createDefaultParserSettings } from 'aslang';

const parser = new ASLANParser({
  ...createDefaultParserSettings(),
  strictStart: true,                    // Require [aslang] to start parsing
  strictEnd: true,                      // Stop parsing at [aslans]
  collapseObjectStartWhitespace: true,  // Collapse whitespace at object/array starts
  maxObjectDepth: 1,                    // Limit object nesting to 1 level
});
```

### Limiting Object Depth

The `maxObjectDepth` setting makes the `[aslano]` delimiter deterministic based on nesting level:

```typescript
import { ASLANParser, createDefaultParserSettings } from 'aslang';

// With maxObjectDepth: 1, [aslano] is binary:
// - At depth 0: always opens an object
// - At depth 1: always closes the object
const parser = new ASLANParser({
  ...createDefaultParserSettings(),
  maxObjectDepth: 1,
});

const result = parser.parse('[asland_edit][aslano][asland_text]content[aslano]');
// { _default: null, edit: { text: 'content' } }
```

This is useful for LLM outputs with known structure depth, eliminating ambiguity from whitespace or empty content.

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

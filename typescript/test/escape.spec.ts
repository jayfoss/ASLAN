import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Escape', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses string with escape delimiter', () => {
    const result = parser.parse(
      `[asland_example_code][aslane_CODE_BLOCK]function greet(name) {
  console.log(\`Hello, \${name}!\`);
  [asland_this_is_not_parsed]This is treated as a regular string[asland_neither_is_this]So is this
}[aslane_CODE_BLOCK]`,
    );

    expect(result).toEqual({
      _default: null,
      example_code:
        'function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a regular string[asland_neither_is_this]So is this\n}',
    });
  });

  test('parses string with escape delimiter, ignore non matching close escape delimiter', () => {
    const result = parser.parse(
      `[asland_example_code][aslane_CODE_BLOCK]function greet(name) {
  console.log(\`Hello, \${name}!\`);
  [asland_this_is_not_parsed]This is treated as a[aslane_other] regular string[asland_neither_is_this]So is this
}[aslane_CODE_BLOCK]`,
    );

    expect(result).toEqual({
      _default: null,
      example_code:
        'function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a[aslane_other] regular string[asland_neither_is_this]So is this\n}',
    });
  });

  test('parses string with escape delimiter, continues parsing after escape closed', () => {
    const result = parser.parse(
      `[asland_example][aslane_test][asland_this_is_not_parsed]This is treated as a regular string[aslane_test][asland_this_is_parsed]My value`,
    );

    expect(result).toEqual({
      _default: null,
      example: '[asland_this_is_not_parsed]This is treated as a regular string',
      this_is_parsed: 'My value',
    });
  });
});

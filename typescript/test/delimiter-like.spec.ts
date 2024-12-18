import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Delimiter-like', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with parts & delimiter-like items', () => {
    const result = parser.parse(
      '[asland_test][aslanp]This is an element with [Your name].[aslanp]This is the second part.[aslanp]This is the third part.',
    );
    expect(result).toEqual({
      _default: null,
      test: [
        'This is an element with [Your name].',
        'This is the second part.',
        'This is the third part.',
      ],
    });
  });
});

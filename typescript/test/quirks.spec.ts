import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Quirks', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses empty string', () => {
    const result = parser.parse('');
    expect(result).toEqual({
      _default: '',
    });
  });

  test('parses string starting with delimiter', () => {
    const result = parser.parse('[asland_test]test');
    expect(result).toEqual({
      _default: null,
      test: 'test',
    });
  });
});

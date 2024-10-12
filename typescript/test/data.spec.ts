import { ASLANParser } from "../src/aslan-parser";

describe('ASLANParser', () => {
  let parser: ASLANParser;
  
  beforeEach(() => {
    parser = new ASLANParser();
  });
  
  /**
     * SPEC 6.1:1
   */
  test('parses simple string with no defaults', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World!');
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
    });
  });

  test('parses simple string with leading content', () => {
    const result = parser.parse('This is still valid.[asland_hi]Hello [asland_lo]World!');
    expect(result).toEqual({
      _default: 'This is still valid.',
      hi: 'Hello ',
      lo: 'World!',
    });
  });

  test('parses simple string with default append behavior', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_hi]Hello');
    expect(result).toEqual({
      _default: null,
      hi: 'Hello Hello',
      lo: 'World!',
    });
  });
});

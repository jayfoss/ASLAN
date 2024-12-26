import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Data', () => {
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
    const result = parser.parse(
      'This is still valid.[asland_hi]Hello [asland_lo]World!',
    );
    expect(result).toEqual({
      _default: 'This is still valid.',
      hi: 'Hello ',
      lo: 'World!',
    });
  });

  test('parses simple string with default append behavior', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_hi]Hello',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello Hello',
      lo: 'World!',
    });
  });

  test('parses simple string with keep first key behavior', () => {
    const result = parser.parse(
      '[asland_hi:f]Hello [asland_lo]World![asland_hi]Hello',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
    });
  });

  test('parses simple string with keep last key behavior', () => {
    const result = parser.parse(
      '[asland_hi:l]Hello [asland_lo]World![asland_hi]Hello',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello',
      lo: 'World!',
    });
  });

  test('parses simple string with keep first key behavior, ignore duplicate behavior redefinition', () => {
    const result = parser.parse(
      '[asland_hi:f]Hello [asland_lo]World![asland_hi:a]Hello[asland_hi:l]Test',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
    });
  });

  test('parses simple string with keep first key behavior on non first key, treated like default append', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_hi:f]Hello[asland_hi:l]Test',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello HelloTest',
      lo: 'World!',
    });
  });

  test('parses simple string with keep last key behavior on non first key, treated like default append', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_hi:l]Hello[asland_hi:f]Test',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello HelloTest',
      lo: 'World!',
    });
  });

  test('parses simple string with append separator', () => {
    const parser = new ASLANParser({ appendSeparator: ' ' });
    const result = parser.parse(
      '[asland_hi]Hello[asland_lo]World![asland_hi]Hello[asland_hi]World!',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello Hello World!',
      lo: 'World!',
    });
  });
});

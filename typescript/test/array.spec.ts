import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Array', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with array', () => {
    const result = parser.parse('[asland_fruits][aslana][asland]Apple[asland]Banana[asland]Cherry');
    expect(result).toEqual({
      _default: null,
      fruits: ['Apple', 'Banana', 'Cherry'],
    });
  });

  test('parses simple string with array indices', () => {
    const result = parser.parse(
      '[asland_custom_array][aslana][asland_2]Third item[asland_0]First item[asland_1]Second item',
    );
    expect(result).toEqual({
      _default: null,
      custom_array: ['First item', 'Second item', 'Third item'],
    });
  });

  test('parses simple string with mixed array indices', () => {
    const result = parser.parse(
      '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D',
    );
    expect(result).toEqual({
      _default: null,
      mixed_array: ['B', undefined, 'A', 'C', 'D'],
    });
  });

  test('parses simple string with more mixed array indices', () => {
    const result = parser.parse(
      '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland_3]F[asland]G',
    );
    expect(result).toEqual({
      _default: null,
      mixed_array: ['B', undefined, 'A', 'CF', 'D', 'E', 'G'],
    });
  });

  test('parses simple string with more mixed array indices and sibling object', () => {
    const result = parser.parse(
      '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland_3]F[asland]G[aslana][asland_other]Not in array',
    );
    expect(result).toEqual({
      _default: null,
      mixed_array: ['B', undefined, 'A', 'CF', 'D', 'E', 'G'],
      other: 'Not in array',
    });
  });

  test('parses simple string with nested arrays', () => {
    const result = parser.parse(
      '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland][aslana][asland]hi[asland]lo[aslana][asland]G',
    );
    expect(result).toEqual({
      _default: null,
      mixed_array: ['B', undefined, 'A', 'C', 'D', 'E', ['hi', 'lo'], 'G'],
    });
  });

  test('parses simple string with multiple arrays, same key, should override', () => {
    const result = parser.parse(
      '[asland_hi][aslana][asland]foo[aslana][asland_hi][aslana][asland]bar[aslana]',
    );
    expect(result).toEqual({
      _default: null,
      hi: ['bar']
    });
  });

  test('parses simple string with array then string, same key, should not override array', () => {
    const result = parser.parse(
      '[asland_hi][aslana][asland]foo[aslana][asland_hi]not overriding',
    );
    expect(result).toEqual({
      _default: null,
      hi: ['foo']
    });
  });

  test('parses simple string with string then array, same key, should override string with array', () => {
    const result = parser.parse(
      '[asland_hi]test[asland_hi][aslana][asland]bar[aslana]',
    );
    expect(result).toEqual({
      _default: null,
      hi: ['bar'],
    });
  });
});

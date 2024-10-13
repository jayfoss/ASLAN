import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Void', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with object and void after other content', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslanv]',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: null,
      },
    });
  });

  test('parses simple string with object and void before other content', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar][aslanv]Baz!',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: null,
      },
    });
  });

  test('parses simple string with object and void in a later duplicate key field', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![asland_bar][aslanv]',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: null,
      },
    });
  });

  test('parses simple string with object and void in a previous duplicate key field', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar][aslanv][asland_x]hi[asland_bar]oops',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: null,
        x: 'hi',
      },
    });
  });
});

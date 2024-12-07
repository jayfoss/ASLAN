import { ASLANParser } from "../src/aslan-parser";

describe('ASLANParser Stop', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser({
      strictEnd: true
    });
  });

  test('parses simple string with object and no stop, strict end', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
    });
  });

  test('parses simple string with object and stop, strict end', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![aslans][asland_foo][aslano][asland_bar]Baz!',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
    });
  });

  test('parses simple string with object and stop, strict end disabled', () => {
    const parser = new ASLANParser({
      strictEnd: false
    });
    const result = parser.parse(
      '[asland_hi]Hello [aslans][asland_lo]World![asland_foo][aslano][asland_bar]Baz!',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
    });
  });
});

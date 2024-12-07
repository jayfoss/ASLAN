import { ASLANParser } from "../src/aslan-parser";

describe('ASLANParser Go', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser({
      strictStart: true
    });
  });

  test('parses simple string with object and no go, strict start', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!',
    );
    expect(result).toEqual({
      _default: null,
    });
  });

  test('parses simple string with object and go, strict start', () => {
    const result = parser.parse(
      '[asland_hi]Hello [aslang][asland_lo]World![asland_foo][aslano][asland_bar]Baz!',
    );
    expect(result).toEqual({
      _default: null,
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
    });
  });

  test('parses simple string with object and go, strict start disabled', () => {
    const parser = new ASLANParser({
      strictStart: false
    });
    const result = parser.parse(
      '[asland_hi]Hello [aslang][asland_lo]World![asland_foo][aslano][asland_bar]Baz!',
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

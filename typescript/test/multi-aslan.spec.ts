import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Multi ASLAN', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser({
      multiAslanOutput: true,
      strictStart: true,
      strictEnd: true,
    });
  });

  test('parses simple multi ASLAN string', () => {
    const result = parser.parse(
      '[aslang][asland_hi]Hello [asland_lo]World![aslans][aslang][asland_foo][aslano][asland_bar]Baz![aslans][aslang]Starting again[aslang]And again[aslang][asland_further]This should also work[aslang]And this',
    );

    expect(result).toEqual([
      {
        _default: null,
        hi: 'Hello ',
        lo: 'World!',
      },
      {
        _default: null,
        foo: {
          bar: 'Baz!',
        },
      },
      {
        _default: 'Starting again',
      },
      {
        _default: 'And again',
      },
      {
        _default: null,
        further: 'This should also work',
      },
      {
        _default: 'And this',
      },
    ]);
  });

  test('parses multi ASLAN string with no content', () => {
    const result = parser.parse('[aslang][aslang]');
    expect(result).toEqual([
      {
        _default: null,
      },
      {
        _default: null,
      },
    ]);
  });

  test('parses multi ASLAN string with content between stop and go delimiters', () => {
    const result = parser.parse(
      '[aslang]This is a test[aslans]this should be ignored[aslang]but not this',
    );
    expect(result).toEqual([
      {
        _default: 'This is a test',
      },
      {
        _default: 'but not this',
      },
    ]);
  });
});

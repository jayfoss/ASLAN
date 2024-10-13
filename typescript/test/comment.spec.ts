import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Comment', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with object, comments ignored', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![aslanc]This is a comment[asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz!',
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

  test('parses more complex string with object, comments ignored', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![aslanc]This is a comment[asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
      x: {
        y: 'you are reading spec',
        z: 'and it continues here',
      },
    });
  });

  test('parses more complex string with object and neighbor [aslano], comments ignored', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
      x: {},
      y: 'you are reading spec',
      z: 'and it continues here',
    });
  });

  test('parses more complex string with object and neighbor [aslano], into outer scope, comments ignored', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
      x: {},
      y: 'you are reading spec',
      z: 'and it continues here',
    });
  });

  test('parses more complex string with nested objects and arrays, comments ignored', () => {
    const result = parser.parse(
      '[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[asland_y][aslano][aslanc]This is a comment[asland_z]and it continues here[aslana][aslanc]This is a comment[asland_a][aslanc]This is a comment[aslano]',
    );
    expect(result).toEqual({
      _default: null,
      hi: 'Hello ',
      lo: 'World!',
      foo: {
        bar: 'Baz!',
      },
      x: {
        a: {},
        y: {
          z: 'and it continues here',
        },
      },
    });
  });
});

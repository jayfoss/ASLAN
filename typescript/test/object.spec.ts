import { ASLANParser } from "../src/aslan-parser";

describe('ASLANParser Object', () => {
  let parser: ASLANParser;
  
  beforeEach(() => {
    parser = new ASLANParser();
  });
  
  test('parses simple string with object', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!');
    expect(result).toEqual({
      "_default": null,
      "hi": "Hello ",
      "lo": "World!",
      "foo": {
        "bar": "Baz!"
      }
    });
  });

  test('parses string with object and comment between [asland_...] and [aslano]', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][asland_bar]Baz!');
    expect(result).toEqual({
      "_default": null,
      "hi": "Hello ",
      "lo": "World!",
      "foo": {
        "bar": "Baz!"
      }
    });
  });

  test('parses more complex string with object', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y]you are reading spec[asland_z]and it continues here');
    expect(result).toEqual({
      "_default": null,
      "hi": "Hello ",
      "lo": "World!",
      "foo": {
        "bar": "Baz!"
      },
      "x": {
        "y": "you are reading spec",
        "z": "and it continues here"
      }
    });
  });

  test('parses more complex string with object and neighbor [aslano]', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][aslano][asland_y]you are reading spec[asland_z]and it continues here');
    expect(result).toEqual({
      "_default": null,
      "hi": "Hello ",
      "lo": "World!",
      "foo": {
        "bar": "Baz!"
      },
      "x": {},
      "y": "you are reading spec",
      "z": "and it continues here"
    });
  });
  
  test('parses more complex string with object and neighbor [aslano], into outer scope', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][aslano][aslano][asland_y]you are reading spec[asland_z]and it continues here');
    expect(result).toEqual({
      "_default": null,
      "hi": "Hello ",
      "lo": "World!",
      "foo": {
        "bar": "Baz!"
      },
      "x": {},
      "y": "you are reading spec",
      "z": "and it continues here"
    });
  });

  test('parses more complex string with nested objects', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y][aslano][asland_z]and it continues here');
    expect(result).toEqual({
      "_default": null,
      "hi": "Hello ",
      "lo": "World!",
      "foo": {
        "bar": "Baz!"
      },
      "x": {
        "y": {
          "z": "and it continues here"
        }
      }
    });
  });
});

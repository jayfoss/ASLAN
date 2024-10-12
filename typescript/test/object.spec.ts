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

  test('parses more complex string with object', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y]you are reading spec[asland_z]and it continues here');
    console.log(result);
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
});

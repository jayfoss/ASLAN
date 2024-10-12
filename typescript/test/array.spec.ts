import { ASLANParser } from "../src/aslan-parser";

describe('ASLANParser Array', () => {
  let parser: ASLANParser;
  
  beforeEach(() => {
    parser = new ASLANParser();
  });
  
  test('parses simple string with array', () => {
    const result = parser.parse('[asland_fruits][aslana][asland]Apple[asland]Banana[asland]Cherry');
    console.log(result);
    expect(result).toEqual({
      "_default": null,
      "fruits": [
        "Apple",
        "Banana",
        "Cherry"
      ]
    });
  });

  test('parses simple string with array indices', () => {
    const result = parser.parse('[asland_custom_array][aslana][asland_2]Third item[asland_0]First item[asland_1]Second item');
    expect(result).toEqual({
      "_default": null,
      "custom_array": [
        "First item",
        "Second item",
        "Third item"
      ]
    });
  });

  test('parses simple string with mixed array indices', () => {
    const result = parser.parse('[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D');
    expect(result).toEqual({
      "_default": null,
      "mixed_array": [
        "B",
        undefined,
        "A",
        "C",
        "D"
      ]
    });
  });
});

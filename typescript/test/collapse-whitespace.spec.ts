import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Collapse Whitespace', () => {
  test('parses array of objects without whitespace, no collapse mode', () => {
    const parser = new ASLANParser({
      collapseObjectStartWhitespace: false,
    });
    const result = parser.parse(`[asland_array][aslana][asland][aslano][asland_el1]Value 1[asland_el2]Value 2[aslano][asland][aslano][asland_el3]Value 3[asland_el4]Value 4`);
    expect(result).toEqual({
      _default: null,
      array: [
        {
          el1: 'Value 1',
          el2: 'Value 2',
        },
        {
          el3: 'Value 3',
          el4: 'Value 4',
        },
      ],
    });
  });

  test('parses array of objects with whitespace, collapse mode', () => {
    const parser = new ASLANParser({
      collapseObjectStartWhitespace: true,
    });
    const result = parser.parse(`[asland_array]
      [aslana]
      [asland]
      [aslano]
      [asland_el1]Value 1[asland_el2]Value 2[aslano]
      [asland]
      [aslano]
      [asland_el3]Value 3[asland_el4]Value 4`);
    expect(result).toEqual({
      _default: null,
      array: [
        {
          el1: 'Value 1',
          el2: 'Value 2',
        },
        {
          el3: 'Value 3',
          el4: 'Value 4',
        },
      ],
    });
  });
});

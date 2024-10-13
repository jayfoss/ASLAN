import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Combined Array and Object', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with object in array', () => {
    const result = parser.parse('[asland_fruits][aslana][asland]Apple[asland]Banana[asland][aslano][asland_x]hi[asland_y]lo');
    expect(result).toEqual({
      _default: null,
      fruits: ['Apple', 'Banana', { x: 'hi', y: 'lo' }],
    });
  });

  test('parses simple string with array in object', () => {
    const result = parser.parse('[asland_fruits][aslano][asland_best]Apple[asland_worst]Durian[asland_others][aslana][asland]Banana[asland]Pear[aslana][asland_next]Plum');
    expect(result).toEqual({
      _default: null,
      fruits: { best: 'Apple', worst: 'Durian', others: ['Banana', 'Pear'], next: 'Plum' },
    });
  });
});
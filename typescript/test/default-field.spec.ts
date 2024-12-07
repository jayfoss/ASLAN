import { ASLANParser } from "../src/aslan-parser";

describe('ASLANParser Default Field', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser({ defaultFieldName: '_modified' });
  });

  test('parses simple string with renamed default field', () => {
    const result = parser.parse('[asland_hi]Hello [asland_lo]World!');

    expect(result).toEqual({
      _modified: null,
      hi: 'Hello ',
      lo: 'World!',
    });
  });
});

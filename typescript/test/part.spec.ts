import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Part', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with parts', () => {
    const result = parser.parse(
      '[asland_formatted_text][aslanp]This is the first part.[aslanp]This is the second part.[aslanp]This is the third part.',
    );
    expect(result).toEqual({
      _default: null,
      formatted_text: [
        'This is the first part.',
        'This is the second part.',
        'This is the third part.',
      ],
    });
  });

  test('parses simple string with parts & instructions', () => {
    const result = parser.parse(
      '[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.',
    );
    expect(result).toEqual({
      _default: null,
      styled_text: [
        'This is bold and red text.',
        'This is italic and underlined text.',
        'This is large monospace text.',
      ],
    });
  });
});

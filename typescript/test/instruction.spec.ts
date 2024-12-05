import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser Instruction', () => {
  let parser: ASLANParser;

  beforeEach(() => {
    parser = new ASLANParser();
  });

  test('parses simple string with instructions', () => {
    const result = parser.parse(
      '[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.',
    );
    expect(result).toEqual({
      "_default": null,
      "styled_text": [
        "This is bold and red text.",
        "This is italic and underlined text.",
        "This is large monospace text."
      ]
    });
  });

  test('parses simple string with instructions and data with l arg', () => {
    const result = parser.parse(
      '[asland_styled_text:l][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.',
    );
    expect(result).toEqual({
      "_default": null,
      "styled_text": [
        "This is italic and blue text.",
      ]
    });
  });

  test('parses simple string with instructions and data with f arg', () => {
    const result = parser.parse(
      '[asland_styled_text:f][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.',
    );
    expect(result).toEqual({
      "_default": null,
      "styled_text": [
        "This is bold and red text.",
      ]
    });
  });

  test('parses simple string with instructions no part', () => {
    parser.addEventListener('content', (event) => {
      console.log(event);
    });
    const result = parser.parse(
      '[asland_test][aslana][asland][aslano][asland_styled_text][aslani_bold][aslani_color:red]This is bold and red text.',
    );
    expect(result).toEqual({
      "_default": null,
      "styled_text": "This is bold and red text.",
    });
  });

});

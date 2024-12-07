import {
  ASLANEndDataInstruction,
  ASLANInstruction,
  ASLANParser,
} from '../src/aslan-parser';
import { deepCopy } from '../src/utils';

describe('ASLANParser Instruction', () => {
  let parser: ASLANParser;
  let contentEvents: ASLANInstruction[];
  let endEvents: ASLANInstruction[];
  let endDataEvents: ASLANEndDataInstruction[];

  beforeEach(() => {
    parser = new ASLANParser();
    contentEvents = [];
    parser.addEventListener('content', (event) => {
      contentEvents.push(deepCopy(event) as ASLANInstruction);
    });
    endEvents = [];
    parser.addEventListener('end', (event) => {
      endEvents.push(deepCopy(event) as ASLANInstruction);
    });
    endDataEvents = [];
    parser.addEventListener('end_data', (event) => {
      endDataEvents.push(deepCopy(event) as ASLANEndDataInstruction);
    });
  });

  test('parses simple string with instructions', () => {
    const result = parser.parse(
      '[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.',
    );

    expect(contentEvents).toMatchSnapshot('instruction-events-content');
    expect(endEvents).toMatchSnapshot('instruction-events-end');
    expect(endDataEvents).toMatchSnapshot('instruction-events-end-data');
    expect(result).toEqual({
      _default: null,
      styled_text: [
        'This is bold and red text.',
        'This is italic and underlined text.',
        'This is large monospace text.',
      ],
    });
  });

  test('parses simple string with instructions and data with l arg', () => {
    const result = parser.parse(
      '[asland_styled_text:l][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.',
    );
    expect(contentEvents).toMatchSnapshot('instruction-events-content');
    expect(endEvents).toMatchSnapshot('instruction-events-end');
    expect(endDataEvents).toMatchSnapshot('instruction-events-end-data');
    expect(result).toEqual({
      _default: null,
      styled_text: ['This is italic and blue text.'],
    });
  });

  test('parses simple string with instructions and data with f arg', () => {
    const result = parser.parse(
      '[asland_styled_text:f][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.',
    );
    expect(contentEvents).toMatchSnapshot('instruction-events-content');
    expect(endEvents).toMatchSnapshot('instruction-events-end');
    expect(endDataEvents).toMatchSnapshot('instruction-events-end-data');
    expect(result).toEqual({
      _default: null,
      styled_text: ['This is bold and red text.'],
    });
  });

  test('parses simple string with instructions no part', () => {
    const result = parser.parse(
      '[asland_test][aslana][asland][aslano][asland_styled_text][aslani_bold][aslani_color:red]This is bold and red text.[asland_next]Next item',
    );
    expect(contentEvents).toMatchSnapshot('instruction-events-content');
    expect(endEvents).toMatchSnapshot('instruction-events-end');
    expect(endDataEvents).toMatchSnapshot('instruction-events-end-data');
    expect(result).toEqual({
      _default: null,
      test: [
        {
          next: 'Next item',
          styled_text: 'This is bold and red text.',
        },
      ],
    });
  });
});
